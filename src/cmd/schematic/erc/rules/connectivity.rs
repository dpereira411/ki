use std::collections::{BTreeMap, BTreeSet};

use crate::schematic::render::{
    pins_are_stacked, ParsedSchema, PhysicalGroup, PinNode, ResolvedNet,
};

use super::super::connectivity::{
    bus_members_for_name, bus_name_for_entry, bus_segment_for_entry,
    format_bus_item_description, pin_library_is_power, power_kind_for_pin, net_name_for_bus_entry,
};
use super::super::format::{
    duplicate_pin_fallback_net_name,
};
use super::super::geom::{is_on_connection_grid, segment_anchor_mm, segments_touch};
use super::super::items::{bus_item, label_item, pin_item, point_item, segment_item};
use super::super::project::{project_rule_severity, RuleSeverityMap};
use super::super::{is_generated_power_label, is_helper_power_symbol, PendingViolation, Severity};

const CONNECTION_GRID_MM: f64 = 1.27;

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
            let embedded = schema.embedded_symbols.get(&symbol.lib_id)?;
            if embedded.duplicate_pin_numbers_are_jumpers {
                return None;
            }

            let mut pins_by_number = schema
                .pin_nodes
                .iter()
                .filter(|pin| pin.reference == symbol.reference)
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

    let net_names = nets
        .iter()
        .flat_map(|net| {
            net.nodes.iter().map(move |node| {
                (
                    (node.reference.clone(), node.unit, node.pin.clone()),
                    net.name.clone(),
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
        .filter_map(|((_reference, pin_number), mut pins)| {
            pins.sort_by(|a, b| a.unit.cmp(&b.unit).then_with(|| a.order.cmp(&b.order)));
            pins.dedup_by(|a, b| a.unit == b.unit);

            let first = pins.first().copied()?;
            let first_net = net_names
                .get(&(first.reference.clone(), first.unit, first.pin.clone()))?
                .clone();

            let second = pins.iter().copied().find(|pin| {
                net_names
                    .get(&(pin.reference.clone(), pin.unit, pin.pin.clone()))
                    .is_some_and(|name| name != &first_net)
            })?;
            let second_net = net_names
                .get(&(second.reference.clone(), second.unit, second.pin.clone()))?
                .clone();

            Some(PendingViolation::new(
                severity,
                "different_unit_net",
                format!(
                    "Pin {} is connected to both {} and {}",
                    pin_number, first_net, second_net
                ),
                vec![
                    pin_item(first),
                    pin_item(second),
                ],
            ))
        })
        .collect()
}

pub(crate) fn bus_to_net_conflict_violations(
    schema: &ParsedSchema,
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "bus_to_net_conflict",
        Severity::Error,
    ) else {
        return Vec::new();
    };

    schema
        .buses
        .iter()
        .filter_map(|bus| {
            let wire = schema.wires.iter().find(|wire| segments_touch(bus, wire))?;
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
        .collect()
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
            let bus_members = bus_members_for_name(&bus_name);
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
                    point_item(format_bus_item_description(&bus_segment), entry.bus_point),
                ],
            ))
        })
        .collect()
}

pub(crate) fn power_pin_not_driven_violations(
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
) -> Vec<PendingViolation> {
    nets.iter()
        .flat_map(|net| {
            let has_power_driver = net.nodes.iter().any(|node| {
                matches!(
                    node.pin_type.as_deref(),
                    Some("power_out") | Some("output") | Some("bidirectional")
                )
            });

            let non_helper_power_inputs = net
                .nodes
                .iter()
                .filter(|node| node.pin_type.as_deref() == Some("power_in"))
                .filter(|node| !is_helper_power_symbol(node))
                .collect::<Vec<_>>();

            if has_power_driver {
                return Vec::new();
            }

            if !non_helper_power_inputs.is_empty() {
                return non_helper_power_inputs
                    .into_iter()
                    .map(|pin| {
                        PendingViolation::single(
                            Severity::Error,
                            "power_pin_not_driven",
                            "Input Power pin not driven by any Output Power pins",
                            pin_item(pin),
                        )
                    })
                    .collect::<Vec<_>>();
            }

            net.nodes
                .iter()
                .filter(|node| node.pin_type.as_deref() == Some("power_in"))
                .filter(|node| is_helper_power_symbol(node))
                .filter(|node| power_kind_for_pin(node, schema) == Some("global"))
                .take(1)
                .map(|pin| {
                    PendingViolation::single(
                        Severity::Error,
                        "power_pin_not_driven",
                        "Input Power pin not driven by any Output Power pins",
                        pin_item(pin),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(crate) fn pin_not_connected_violations(
    schema: &ParsedSchema,
    physical_groups: &[PhysicalGroup],
    nets: &[ResolvedNet],
) -> Vec<PendingViolation> {
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

    physical_groups
        .iter()
        .filter(|group| !group.no_connect)
        .filter_map(|group| {
            let pins = &group.nodes;
            let mut has_other_connections = group
                .labels
                .iter()
                .any(|label| !is_generated_power_label(label, schema));

            if !has_other_connections && pins.len() > 1 && !pins_are_stacked(pins) {
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

            (!has_other_connections
                && pin.pin_type.as_deref() != Some("no_connect")
                && pin.pin_type.as_deref() != Some("not_connected"))
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
                .collect::<Vec<_>>();
            if drivers.len() < 2 {
                return None;
            }

            if drivers.iter().all(|label| label.label_type == "hierarchical_label") {
                drivers.sort_by(|left, right| {
                    left.point
                        .x
                        .cmp(&right.point.x)
                        .then_with(|| left.point.y.cmp(&right.point.y))
                });
            }

            let primary = drivers[0];
            let secondary = drivers
                .iter()
                .copied()
                .find(|label| label.text != primary.text)?;

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
                    primary.text, secondary.text, primary.text
                ),
                vec![label_item(primary), label_item(secondary)],
            ))
        })
        .collect()
}

pub(crate) fn endpoint_off_grid_violations(schema: &ParsedSchema) -> Vec<PendingViolation> {
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
            || (is_on_connection_grid(pin.point.x, CONNECTION_GRID_MM)
                && is_on_connection_grid(pin.point.y, CONNECTION_GRID_MM))
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
        if is_on_connection_grid(segment.a.x, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.a.y, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.b.x, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.b.y, CONNECTION_GRID_MM)
        {
            return None;
        }

        let (x_mm, y_mm) = segment_anchor_mm(segment);
        Some(PendingViolation::single(
            Severity::Warning,
            "endpoint_off_grid",
            "Symbol pin or wire end off connection grid",
            segment_item(segment, x_mm, y_mm),
        ))
    }));

    out.extend(schema.buses.iter().filter_map(|segment| {
        if is_on_connection_grid(segment.a.x, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.a.y, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.b.x, CONNECTION_GRID_MM)
            && is_on_connection_grid(segment.b.y, CONNECTION_GRID_MM)
        {
            return None;
        }

        let (x_mm, y_mm) = segment_anchor_mm(segment);
        Some(PendingViolation::single(
            Severity::Warning,
            "endpoint_off_grid",
            "Symbol pin or wire end off connection grid",
            bus_item(segment, x_mm, y_mm),
        ))
    }));

    out.extend(schema.bus_entries.iter().flat_map(|entry| {
        [entry.bus_point, entry.wire_point]
            .into_iter()
            .filter(|point| {
                !is_on_connection_grid(point.x, CONNECTION_GRID_MM)
                    || !is_on_connection_grid(point.y, CONNECTION_GRID_MM)
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
