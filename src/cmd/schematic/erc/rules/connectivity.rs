use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::schematic::render::{
    embedded_symbol_for, pins_are_stacked, ParsedSchema, PhysicalGroup, PinNode, Point,
    ResolvedNet,
};

use super::super::connectivity::{
    bus_members_for_name_with_aliases, bus_name_for_entry, bus_segment_for_entry,
    connected_bus_segments, connected_wire_segments, format_bus_item_description,
    looks_like_bus_name, net_name_for_bus_entry, pin_library_is_power, power_kind_for_pin,
    segment_key,
};
use super::super::format::{
    duplicate_pin_fallback_net_name,
};
use super::super::geom::{
    is_on_connection_grid, point_on_segment, segment_anchor_mm, segments_touch,
};
use super::super::items::{bus_item, label_item, pin_item, point_item, segment_item};
use super::super::project::{
    project_rule_severity, NetclassAssignmentMap, RuleSeverityMap,
};
use super::super::{is_generated_power_label, is_helper_power_symbol, PendingViolation, Severity};

#[derive(Clone)]
struct MultipleNetNameDriver {
    name: String,
    priority: i32,
    item: super::super::PendingItem,
    point_key: (i64, i64),
}

pub(crate) fn pin_is_no_connect_type(pin_type: Option<&str>) -> bool {
    matches!(pin_type, Some("no_connect") | Some("not_connected") | Some("unconnected"))
}

pub(crate) fn duplicate_pin_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) =
        project_rule_severity(project_rule_severities, "duplicate_pins", Severity::Error)
    else {
        return Vec::new();
    };

    let net_names_by_point = nets
        .iter()
        .flat_map(|net| {
            net.nodes
                .iter()
                .map(move |node| (node.point, net.name.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    schema
        .symbols
        .iter()
        .filter_map(|symbol| {
            let embedded = embedded_symbol_for(symbol, &schema.embedded_symbols)?;
            if embedded.duplicate_pin_numbers_are_jumpers {
                return None;
            }

            let mut pins_by_number = schema
                .pin_nodes
                .iter()
                .filter(|pin| pin.reference == symbol.reference && pin.unit == symbol.unit)
                .fold(BTreeMap::<String, Vec<&PinNode>>::new(), |mut acc, pin| {
                    acc.entry(pin.pin.clone()).or_default().push(pin);
                    acc
                });

            for pins in pins_by_number.values_mut() {
                pins.sort_by(|a, b| {
                    a.point
                        .cmp(&b.point)
                        .then_with(|| a.pin_function.cmp(&b.pin_function))
                        .then_with(|| a.order.cmp(&b.order))
                });
                pins.dedup_by(|a, b| a.point == b.point && a.pin_function == b.pin_function);
            }

            if pins_by_number.values().flatten().all(|pin| is_helper_power_symbol(pin)) {
                return None;
            }

            let (pin_number, conflicting_pins) = pins_by_number.into_iter().find(|(_, pins)| {
                if pins.len() < 2 {
                    return false;
                }

                let mut names = pins
                    .iter()
                    .map(|pin| {
                        net_names_by_point
                            .get(&pin.point)
                            .cloned()
                            .unwrap_or_else(|| {
                                duplicate_pin_fallback_net_name(
                                    pin,
                                    schema.no_connects.contains(&pin.point),
                                )
                            })
                    })
                    .collect::<Vec<_>>();
                names.sort();
                names.dedup();
                names.len() > 1
            })?;

            let first = conflicting_pins[0];
            let second = conflicting_pins[1];
            let first_net = net_names_by_point
                .get(&first.point)
                .cloned()
                .unwrap_or_else(|| {
                    duplicate_pin_fallback_net_name(first, schema.no_connects.contains(&first.point))
                });
            let second_net = net_names_by_point
                .get(&second.point)
                .cloned()
                .unwrap_or_else(|| {
                    duplicate_pin_fallback_net_name(second, schema.no_connects.contains(&second.point))
                });

            Some(PendingViolation::new(
                severity,
                "duplicate_pins",
                format!(
                    "Pin {pin_number} on symbol '{}' is connected to different nets: {} and {}",
                    symbol.reference, first_net, second_net
                ),
                vec![
                    pin_item(first),
                    pin_item(second),
                ],
            ))
        })
        .collect()
}

pub(crate) fn different_unit_net_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "different_unit_net",
        Severity::Error,
    ) else {
        return Vec::new();
    };

    let net_by_pin = nets
        .iter()
        .flat_map(|net| {
            net.nodes.iter().map(move |node| {
                (
                    (node.reference.clone(), node.unit, node.pin.clone()),
                    net,
                )
            })
        })
        .collect::<BTreeMap<_, _>>();

    schema
        .pin_nodes
        .iter()
        .filter(|pin| {
            schema
                .symbols
                .iter()
                .filter(|symbol| symbol.reference == pin.reference)
                .map(|symbol| symbol.unit)
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        })
        .fold(
            BTreeMap::<(String, String), Vec<&PinNode>>::new(),
            |mut grouped, pin| {
                grouped
                    .entry((pin.reference.clone(), pin.pin.clone()))
                    .or_default()
                    .push(pin);
                grouped
            },
        )
        .into_iter()
        .flat_map(|((_reference, pin_number), mut pins)| {
            pins.sort_by(|a, b| a.unit.cmp(&b.unit).then_with(|| a.order.cmp(&b.order)));
            pins.dedup_by(|a, b| a.unit == b.unit);

            let Some(anchor) = pins.last().copied() else {
                return Vec::new();
            };
            let Some(anchor_net) =
                net_by_pin.get(&(anchor.reference.clone(), anchor.unit, anchor.pin.clone()))
            else {
                return Vec::new();
            };

            let anchor_name = unnamed_single_pin_net_name(anchor, anchor_net, schema);

            pins.into_iter()
                .filter(|pin| pin.unit != anchor.unit)
                .filter_map(|pin| {
                    let net = net_by_pin.get(&(pin.reference.clone(), pin.unit, pin.pin.clone()))?;
                    let net_name = unnamed_single_pin_net_name(pin, net, schema);

                    if net_name == anchor_name {
                        return None;
                    }

                    Some(PendingViolation::new(
                        severity,
                        "different_unit_net",
                        format!(
                            "Pin {} is connected to both {} and {}",
                            pin_number, net_name, anchor_name
                        ),
                        vec![pin_item(pin), pin_item(anchor)],
                    ))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn unnamed_single_pin_net_name(pin: &PinNode, net: &ResolvedNet, schema: &ParsedSchema) -> String {
    if net.labels.is_empty()
        && net.nodes.len() == 1
        && net.segments.is_empty()
        && !schema.no_connects.contains(&pin.point)
    {
        return duplicate_pin_fallback_net_name(pin, true);
    }

    net.name.clone()
}

pub(crate) fn bus_to_net_conflict_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "bus_to_net_conflict",
        Severity::Error,
    ) else {
        return Vec::new();
    };

    let net_segment_conflicts = nets
        .iter()
        .filter_map(|net| mixed_bus_net_segment_conflict(net, schema, severity))
        .collect::<Vec<_>>();

    let mut pending = schema
        .buses
        .iter()
        .filter_map(|bus| {
            let wire = schema.wires.iter().find(|wire| segments_touch(bus, wire))?;
            if bus_wire_touch_is_via_bus_entry(bus, wire, schema) {
                return None;
            }
            if net_segment_conflicts.iter().any(|conflict| {
                conflict.wire_segments.contains(&segment_key(wire))
                    && conflict.bus_segments.contains(&segment_key(bus))
            }) {
                return None;
            }
            let (wire_x_mm, wire_y_mm) = segment_anchor_mm(wire);
            let (bus_x_mm, bus_y_mm) = segment_anchor_mm(bus);

            Some(PendingViolation::new(
                severity,
                "bus_to_net_conflict",
                "Invalid connection between bus and net items",
                vec![
                    segment_item(wire, wire_x_mm, wire_y_mm),
                    bus_item(bus, bus_x_mm, bus_y_mm),
                ],
            ))
        })
        .collect::<Vec<_>>();

    pending.extend(net_segment_conflicts.into_iter().map(|conflict| conflict.violation));

    pending.extend(nets.iter().filter_map(|net| {
        let bus_label = net.labels.iter().find(|label| {
            matches!(
                label.label_type.as_str(),
                "label" | "global_label" | "hierarchical_label"
            ) && looks_like_bus_name(&label.text)
        })?;
        let net_label = net.labels.iter().find(|label| {
            matches!(
                label.label_type.as_str(),
                "label" | "global_label" | "hierarchical_label"
            ) && !looks_like_bus_name(&label.text)
        })?;

        Some(PendingViolation::new(
            severity,
            "bus_to_net_conflict",
            "Invalid connection between bus and net items",
            vec![label_item(net_label), label_item(bus_label)],
        ))
    }));

    pending
}

struct MixedBusNetSegmentConflict {
    violation: PendingViolation,
    wire_segments: BTreeSet<(Point, Point)>,
    bus_segments: BTreeSet<(Point, Point)>,
}

fn mixed_bus_net_segment_conflict(
    net: &ResolvedNet,
    schema: &ParsedSchema,
    severity: Severity,
) -> Option<MixedBusNetSegmentConflict> {
    let wire_segments = net
        .nodes
        .iter()
        .filter(|pin| schema.wires.iter().any(|segment| point_on_segment(pin.point, segment)))
        .flat_map(|pin| connected_wire_segments(pin.point, schema))
        .chain(net.labels.iter().flat_map(|label| {
            schema
                .wires
                .iter()
                .any(|segment| point_on_segment(label.point, segment))
                .then(|| connected_wire_segments(label.point, schema))
                .into_iter()
                .flatten()
        }))
        .collect::<Vec<_>>();

    let bus_segments = net
        .nodes
        .iter()
        .filter(|pin| schema.buses.iter().any(|segment| point_on_segment(pin.point, segment)))
        .flat_map(|pin| connected_bus_segments(pin.point, schema))
        .chain(net.labels.iter().flat_map(|label| {
            schema
                .buses
                .iter()
                .any(|segment| point_on_segment(label.point, segment))
                .then(|| connected_bus_segments(label.point, schema))
                .into_iter()
                .flatten()
        }))
        .collect::<Vec<_>>();

    let wire = schema
        .wires
        .iter()
        .find(|candidate| {
            wire_segments
                .iter()
                .any(|segment| segment_key(segment) == segment_key(candidate))
        })
        ?;
    let bus = schema
        .buses
        .iter()
        .find(|candidate| {
            bus_segments
                .iter()
                .any(|segment| segment_key(segment) == segment_key(candidate))
        })
        .cloned()?;

    Some(MixedBusNetSegmentConflict {
        violation: PendingViolation::new(
            severity,
            "bus_to_net_conflict",
            "Invalid connection between bus and net items",
            vec![
                segment_item(&wire, segment_anchor_mm(&wire).0, segment_anchor_mm(&wire).1),
                bus_item(&bus, segment_anchor_mm(&bus).0, segment_anchor_mm(&bus).1),
            ],
        ),
        wire_segments: wire_segments.iter().map(segment_key).collect(),
        bus_segments: bus_segments.iter().map(segment_key).collect(),
    })
}

fn bus_wire_touch_is_via_bus_entry(bus: &crate::schematic::render::Segment, wire: &crate::schematic::render::Segment, schema: &ParsedSchema) -> bool {
    let touch_points = [bus.a, bus.b, wire.a, wire.b]
        .into_iter()
        .filter(|point| super::super::geom::point_on_segment(*point, bus) && super::super::geom::point_on_segment(*point, wire))
        .collect::<BTreeSet<_>>();

    !touch_points.is_empty()
        && schema.bus_entries.iter().any(|entry| {
            touch_points.iter().any(|point| *point == entry.bus_point || *point == entry.wire_point)
        })
}

pub(crate) fn net_not_bus_member_violations(
    schema: &ParsedSchema,
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "net_not_bus_member",
        Severity::Warning,
    ) else {
        return Vec::new();
    };

    schema
        .bus_entries
        .iter()
        .filter_map(|entry| {
            let bus_segment = bus_segment_for_entry(entry, schema)?;
            let bus_name = bus_name_for_entry(entry, schema)?;
            let net_name = net_name_for_bus_entry(entry, schema)?;
            let bus_members = bus_members_for_name_with_aliases(&bus_name, schema);
            let normalized_net_name = net_name.trim_start_matches('/').to_string();

            if bus_members.contains(&net_name) || bus_members.contains(&normalized_net_name) {
                return None;
            }

            Some(PendingViolation::new(
                severity,
                "net_not_bus_member",
                format!(
                    "Net {} is graphically connected to bus {} but is not a member of that bus",
                    net_name, bus_name
                ),
                vec![
                    point_item("Bus to wire entry", entry.bus_point),
                    point_item(format_bus_item_description(&bus_segment), bus_segment.a),
                ],
            ))
        })
        .collect()
}

pub(crate) fn power_pin_not_driven_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
) -> Vec<PendingViolation> {
    power_pin_not_driven_violations_with_global_drivers(schema, nets, &BTreeSet::new())
}

pub(crate) fn power_pin_not_driven_violations_with_global_drivers(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    externally_driven_global_nets: &BTreeSet<String>,
) -> Vec<PendingViolation> {
    nets.iter()
        .flat_map(|net| {
            let is_power_net = net
                .nodes
                .iter()
                .any(|node| node.pin_type.as_deref() == Some("power_in"));
            if !is_power_net {
                return Vec::new();
            }

            let mut has_other_connections = !net.labels.is_empty() || !net.segments.is_empty();

            if !has_other_connections {
                let pins = net.nodes.iter().collect::<Vec<_>>();

                if pins.len() > 1 && !pins_are_stacked(&net.nodes) {
                    has_other_connections = true;
                }
            }

            if !has_other_connections {
                return Vec::new();
            }

            let has_local_power_driver = net.nodes.iter().any(|node| {
                node.pin_type.as_deref() == Some("power_out")
            });
            let has_external_global_power_driver = externally_driven_global_nets.contains(&net.name)
                && net
                    .labels
                    .iter()
                    .any(|label| label.label_type == "global_label");
            let pins_needing_drivers = net
                .nodes
                .iter()
                .filter(|node| {
                    matches!(node.pin_type.as_deref(), Some("input") | Some("power_in"))
                })
                .collect::<Vec<_>>();
            let visible_non_helper_pins = pins_needing_drivers
                .iter()
                .copied()
                .filter(|node| !is_helper_power_symbol(node) && !node.hidden)
                .collect::<Vec<_>>();
            let non_helper_pins = pins_needing_drivers
                .iter()
                .copied()
                .filter(|node| !is_helper_power_symbol(node))
                .collect::<Vec<_>>();

            if has_local_power_driver || has_external_global_power_driver {
                return Vec::new();
            }

            if let Some(pin) = visible_non_helper_pins
                .first()
                .copied()
                .or_else(|| non_helper_pins.first().copied())
            {
                return vec![PendingViolation::single(
                    Severity::Error,
                    "power_pin_not_driven",
                    "Input Power pin not driven by any Output Power pins",
                    pin_item(pin),
                )];
            }

            let helper_power_inputs = net.nodes
                .iter()
                .filter(|node| node.pin_type.as_deref() == Some("power_in"))
                .filter(|node| is_helper_power_symbol(node))
                .filter(|node| power_kind_for_pin(node, schema) == Some("global"))
                .collect::<Vec<_>>();

            let selected_helper = if helper_power_inputs
                .iter()
                .map(|pin| pin.reference.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                == 1
            {
                helper_power_inputs.last().copied()
            } else {
                helper_power_inputs.first().copied()
            };

            selected_helper
                .map(|pin| {
                    PendingViolation::single(
                        Severity::Error,
                        "power_pin_not_driven",
                        "Input Power pin not driven by any Output Power pins",
                        pin_item(pin),
                    )
                })
                .into_iter()
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(crate) fn pin_not_connected_violations(
    schema: &ParsedSchema,
    physical_groups: &[PhysicalGroup],
    nets: &[ResolvedNet],
    project_netclass_assignments: &NetclassAssignmentMap,
    parameterized_netclasses: &HashSet<String>,
) -> Vec<PendingViolation> {
    let conflicting_helper_power_pin_keys = nets
        .iter()
        .flat_map(|net| {
            conflicting_helper_power_pins_on_net(net, schema)
                .map(|pin| (pin.order, pin.reference.as_str().to_string(), pin.pin.as_str().to_string()))
        })
        .collect::<BTreeSet<_>>();
    let global_label_cache = schema
        .labels
        .iter()
        .filter(|label| label.label_type == "global_label")
        .filter(|label| !is_generated_power_label(label, schema))
        .map(|label| label.text.clone())
        .collect::<BTreeSet<_>>();
    let local_label_cache = schema
        .labels
        .iter()
        .filter(|label| {
            matches!(label.label_type.as_str(), "label" | "hierarchical_label")
                && !is_generated_power_label(label, schema)
        })
        .map(|label| label.text.trim_start_matches('/').to_string())
        .collect::<BTreeSet<_>>();
    let net_name_by_pin = nets
        .iter()
        .flat_map(|net| {
            net.nodes.iter().map(|pin| {
                (
                    (pin.order, pin.reference.as_str(), pin.pin.as_str()),
                    net.name.clone(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let net_has_user_label = nets
        .iter()
        .map(|net| {
            (
                net.name.as_str(),
                net.labels.iter().any(|label| !is_generated_power_label(label, schema)),
            )
        })
        .collect::<HashMap<_, _>>();
    let net_min_reference = nets
        .iter()
        .filter_map(|net| {
            let min_reference = net.nodes.iter().map(|node| node.reference.as_str()).min()?;
            Some((net.name.as_str(), min_reference))
        })
        .collect::<HashMap<_, _>>();

    physical_groups
        .iter()
        .filter(|group| !group.no_connect)
        .filter_map(|group| {
            let pins = &group.nodes;
            let mut has_other_connections = group
                .labels
                .iter()
                .any(|label| !is_generated_power_label(label, schema));

            if !has_other_connections
                && pins.iter().all(|pin| is_helper_power_symbol(pin))
                && helper_power_group_has_graphical_attachment(group, schema)
            {
                has_other_connections = true;
            }

            if !has_other_connections {
                has_other_connections = schema.sheet_pins.iter().any(|sheet_pin| {
                    group.segments
                        .iter()
                        .any(|segment| point_on_segment(*sheet_pin, segment))
                });
            }

            if !has_other_connections
                && pins.len() > 1
                && !pins_are_stacked(pins)
                && !pins.iter().all(|pin| pin_library_is_power(pin, schema))
            {
                has_other_connections = true;
            }

            let mut pin = pins.first()?;

            for test_pin in pins {
                if test_pin.pin_type.as_deref() == Some("power_in")
                    && !is_helper_power_symbol(test_pin)
                {
                    pin = test_pin;
                    break;
                }
            }

            if !has_other_connections
                && !is_helper_power_symbol(pin)
                && !pin_library_is_power(pin, schema)
            {
                let net_name = net_name_by_pin
                    .get(&(pin.order, pin.reference.as_str(), pin.pin.as_str()))
                    .cloned()
                    .unwrap_or_else(|| duplicate_pin_fallback_net_name(pin, false));
                let local_name = net_name.trim_start_matches('/').to_string();

                if global_label_cache.contains(&net_name) || local_label_cache.contains(&local_name)
                {
                    has_other_connections = true;
                }
            }

            if !has_other_connections
                && pin_not_connected_should_follow_single_assigned_sibling_netclass(
                    pin,
                    schema,
                    &net_name_by_pin,
                    &net_has_user_label,
                    &net_min_reference,
                    project_netclass_assignments,
                    parameterized_netclasses,
                )
            {
                has_other_connections = true;
            }

            if !has_other_connections
                && conflicting_helper_power_pin_keys.contains(&(
                    pin.order,
                    pin.reference.as_str().to_string(),
                    pin.pin.as_str().to_string(),
                ))
            {
                has_other_connections = true;
            }

            (!has_other_connections
                && !pin_is_no_connect_type(pin.pin_type.as_deref()))
            .then(|| {
                PendingViolation::single(
                    Severity::Error,
                    "pin_not_connected",
                    "Pin not connected",
                    pin_item(pin),
                )
            })
        })
        .collect()
}

fn helper_power_group_has_graphical_attachment(
    group: &PhysicalGroup,
    schema: &ParsedSchema,
) -> bool {
    !group.segments.is_empty()
        || group.nodes.iter().any(|pin| {
            schema
                .buses
                .iter()
                .any(|segment| point_on_segment(pin.point, segment))
        })
}

fn pin_not_connected_should_follow_single_assigned_sibling_netclass(
    pin: &PinNode,
    schema: &ParsedSchema,
    net_name_by_pin: &BTreeMap<(usize, &str, &str), String>,
    net_has_user_label: &HashMap<&str, bool>,
    net_min_reference: &HashMap<&str, &str>,
    project_netclass_assignments: &NetclassAssignmentMap,
    parameterized_netclasses: &HashSet<String>,
) -> bool {
    schema
        .pin_nodes
        .iter()
        .filter(|candidate| candidate.reference == pin.reference)
        .filter(|candidate| !(candidate.order == pin.order && candidate.pin == pin.pin))
        .filter_map(|candidate| {
            let net_name = net_name_by_pin
                .get(&(candidate.order, candidate.reference.as_str(), candidate.pin.as_str()))?;
            net_has_user_label
                .get(net_name.as_str())
                .copied()
                .filter(|has_label| *has_label)?;
            let assignments = project_netclass_assignments.get(net_name)?;
            if net_min_reference.get(net_name.as_str()).copied() != Some(pin.reference.as_str()) {
                return None;
            }
            (assignments.len() == 1 && parameterized_netclasses.contains(&assignments[0]))
                .then_some(())
        })
        .next()
        .is_some()
}

pub(crate) fn multiple_net_names_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    nets.iter()
        .filter_map(|net| {
            let net_points = net
                .labels
                .iter()
                .map(|label| (label.point, label.label_type.as_str(), label.text.as_str()))
                .collect::<BTreeSet<_>>();

            let mut drivers = schema
                .labels
                .iter()
                .filter(|label| {
                    matches!(
                        label.label_type.as_str(),
                        "label" | "global_label" | "hierarchical_label"
                    )
                })
                .filter(|label| !is_generated_power_label(label, schema))
                .filter(|label| {
                    net_points
                        .contains(&(label.point, label.label_type.as_str(), label.text.as_str()))
                })
                .map(|label| MultipleNetNameDriver {
                    name: label.text.clone(),
                    priority: multiple_net_name_driver_priority(label.label_type.as_str()),
                    item: label_item(label),
                    point_key: (label.point.x, label.point.y),
                })
                .collect::<Vec<_>>();

            let local_texts = drivers
                .iter()
                .filter(|driver| driver.priority == multiple_net_name_driver_priority("label"))
                .map(|driver| driver.name.clone())
                .collect::<BTreeSet<_>>();
            let existing_driver_keys = drivers
                .iter()
                .map(|driver| (driver.point_key, driver.name.clone()))
                .collect::<BTreeSet<_>>();

            drivers.extend(schema.labels.iter().filter_map(|label| {
                (label.label_type == "global_label"
                    && local_texts.contains(label.text.as_str())
                    && !existing_driver_keys.contains(&((label.point.x, label.point.y), label.text.clone())))
                .then(|| MultipleNetNameDriver {
                    name: label.text.clone(),
                    priority: multiple_net_name_driver_priority(label.label_type.as_str()),
                    item: label_item(label),
                    point_key: (label.point.x, label.point.y),
                })
            }));

            drivers.extend(conflicting_helper_power_pins_on_net(net, schema).map(|pin| {
                MultipleNetNameDriver {
                    name: pin.pin_function.clone().unwrap_or_default(),
                    priority: multiple_net_name_driver_priority("power_pin"),
                    item: pin_item(pin),
                    point_key: (pin.point.x, pin.point.y),
                }
            }));

            if drivers.len() < 2 {
                return None;
            }

            let primary = drivers.iter().min_by(|left, right| {
                left.priority
                    .cmp(&right.priority)
                    .then_with(|| left.name.cmp(&right.name))
                    .then_with(|| left.point_key.cmp(&right.point_key))
            })?;
            let secondary = drivers
                .iter()
                .find(|driver| driver.name != primary.name)?;

            let Some(severity) = project_rule_severity(
                project_rule_severities,
                "multiple_net_names",
                Severity::Warning,
            ) else {
                return None;
            };

            Some(PendingViolation::new(
                severity,
                "multiple_net_names",
                format!(
                    "Both {} and {} are attached to the same items; {} will be used in the netlist",
                    primary.name, secondary.name, primary.name
                ),
                vec![primary.item.clone(), secondary.item.clone()],
            ))
        })
        .collect()
}

fn multiple_net_name_driver_priority(label_type: &str) -> i32 {
    match label_type {
        "global_label" => 0,
        "label" => 1,
        "hierarchical_label" => 2,
        "power_pin" => 3,
        _ => 3,
    }
}

fn conflicting_helper_power_pins_on_net<'a>(
    net: &'a ResolvedNet,
    schema: &'a ParsedSchema,
) -> impl Iterator<Item = &'a PinNode> {
    let user_labels = net
        .labels
        .iter()
        .filter(|label| !is_generated_power_label(label, schema))
        .map(|label| label.text.as_str())
        .collect::<BTreeSet<_>>();

    net.nodes.iter().filter(move |pin| {
        is_helper_power_symbol(pin)
            && pin.pin_type.as_deref() == Some("power_in")
            && pin
                .pin_function
                .as_deref()
                .is_some_and(|name| user_labels.iter().any(|label| *label != name))
    })
}

pub(crate) fn endpoint_off_grid_violations(
    schema: &ParsedSchema,
    connection_grid_mm: f64,
) -> Vec<PendingViolation> {
    let mut out = Vec::new();

    let mut symbol_pins = schema.pin_nodes.iter().collect::<Vec<_>>();
    symbol_pins.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.pin.cmp(&b.pin))
    });

    let mut current_ref: Option<&str> = None;
    let mut emitted_for_symbol = false;

    for pin in symbol_pins {
        if current_ref != Some(pin.reference.as_str()) {
            current_ref = Some(pin.reference.as_str());
            emitted_for_symbol = false;
        }

        if emitted_for_symbol
            || pin.pin_type.as_deref() == Some("no_connect")
            || (is_on_connection_grid(pin.point.x, connection_grid_mm)
                && is_on_connection_grid(pin.point.y, connection_grid_mm))
        {
            continue;
        }

        out.push(PendingViolation::single(
            Severity::Warning,
            "endpoint_off_grid",
            "Symbol pin or wire end off connection grid",
            pin_item(pin),
        ));

        emitted_for_symbol = true;
    }

    out.extend(schema.wires.iter().filter_map(|segment| {
        let anchor = if !is_on_connection_grid(segment.a.x, connection_grid_mm)
            || !is_on_connection_grid(segment.a.y, connection_grid_mm)
        {
            Some(segment.a)
        } else if !is_on_connection_grid(segment.b.x, connection_grid_mm)
            || !is_on_connection_grid(segment.b.y, connection_grid_mm)
        {
            Some(segment.b)
        } else {
            None
        }?;

        Some(PendingViolation::single(
            Severity::Warning,
            "endpoint_off_grid",
            "Symbol pin or wire end off connection grid",
            segment_item(segment, anchor.x as f64 / 10_000.0, anchor.y as f64 / 10_000.0),
        ))
    }));

    out.extend(schema.buses.iter().filter_map(|segment| {
        let anchor = if !is_on_connection_grid(segment.a.x, connection_grid_mm)
            || !is_on_connection_grid(segment.a.y, connection_grid_mm)
        {
            Some(segment.a)
        } else if !is_on_connection_grid(segment.b.x, connection_grid_mm)
            || !is_on_connection_grid(segment.b.y, connection_grid_mm)
        {
            Some(segment.b)
        } else {
            None
        }?;

        Some(PendingViolation::single(
            Severity::Warning,
            "endpoint_off_grid",
            "Symbol pin or wire end off connection grid",
            bus_item(segment, anchor.x as f64 / 10_000.0, anchor.y as f64 / 10_000.0),
        ))
    }));

    out.extend(schema.bus_entries.iter().flat_map(|entry| {
        [entry.bus_point, entry.wire_point]
            .into_iter()
            .filter(|point| {
                !is_on_connection_grid(point.x, connection_grid_mm)
                    || !is_on_connection_grid(point.y, connection_grid_mm)
            })
            .map(|point| {
                PendingViolation::single(
                    Severity::Warning,
                    "endpoint_off_grid",
                    "Symbol pin or wire end off connection grid",
                    point_item("Bus to wire entry", point),
                )
            })
            .collect::<Vec<_>>()
    }));

    out
}
