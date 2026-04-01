use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use kiutils_sexpr::{parse_one, Node};

use crate::error::KiError;
use crate::extract::sym_lib::ProjectSymbolLibraryIndex;
use crate::schematic::render::{
    LabelInfo, ParsedSchema, PhysicalGroup, PinNode, PlacedSymbol, Point,
    ResolvedNet, Segment,
};

use super::super::connectivity::{
    bus_members_for_name_with_aliases, connected_bus_segments,
    connected_pin_like_count_for_label,
    connected_pins_for_no_connect,
    connected_wire_segments, unique_no_connect_pins_across_nets,
    dangling_segment_endpoint_count, format_segment_item_description, is_dangling_label,
    is_dangling_logical_label, is_dangling_no_connect, label_has_no_connect,
    label_has_no_connect_across_nets,
    looks_like_bus_name,
    symbol_for_pin, wire_component_has_driver, wire_only_components,
};
use super::super::format::{
    format_label_item_description, format_pin_item_description, format_pin_type_name,
    format_symbol_item_description, unit_suffix,
};
use super::super::hierarchy::FootprintLibraryIndex;
use crate::cmd::schematic::erc::hierarchy::sheet_refs;
use super::super::geom::{point_on_segment, same_segment, segment_anchor_mm, segment_length_mm};
use super::super::pin_conflict::{
    order_pin_conflict_description, order_pin_conflict_items, reduced_pin_conflicts,
    PinConflictLevel,
};
use super::super::project::{
    project_rule_severity, NetclassAssignmentMap, RuleSeverityMap,
};
use super::super::rules::connectivity::{
    bus_to_net_conflict_violations, different_unit_net_violations, duplicate_pin_violations,
    endpoint_off_grid_violations, multiple_net_names_violations, net_not_bus_member_violations,
    pin_is_no_connect_type, pin_not_connected_violations, power_pin_not_driven_violations,
};
use super::super::rules::symbol::{
    build_footprint_link_issues, build_lib_symbol_issue, lib_symbol_mismatch_violations,
    undefined_netclass_flag_violation,
};
use super::super::sexpr::{child_items, head_of, nth_atom_f64, nth_atom_string};
use super::super::text::{parse_erc_assertion, property_contains_unresolved_variable, resembles_invalid_stacked_pin};
use super::super::{is_generated_power_label, is_helper_power_symbol, PendingItem, PendingViolation, Severity};

fn label_exports_through_sheet_bus(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    if label.label_type != "label" || looks_like_bus_name(&label.text) {
        return false;
    }

    let connected_segments = connected_wire_segments(label.point, schema);
    if connected_segments.is_empty() {
        return false;
    }

    let touches_bus_entry = schema.bus_entries.iter().any(|entry| {
        connected_segments
            .iter()
            .any(|segment| point_on_segment(entry.wire_point, segment))
    });

    if !touches_bus_entry {
        return false;
    }

    let touches_exporting_sheet_bus = schema.bus_entries.iter().any(|entry| {
        connected_segments
            .iter()
            .any(|segment| point_on_segment(entry.wire_point, segment))
            && {
                let bus_segments = connected_bus_segments(entry.bus_point, schema);
                !bus_segments.is_empty()
                    && schema.sheet_pins.iter().any(|sheet_pin| {
                        bus_segments
                            .iter()
                            .any(|segment| point_on_segment(*sheet_pin, segment))
                    })
            }
    });

    touches_exporting_sheet_bus && schema.labels.iter().any(|other| {
        other.label_type == "label"
            && looks_like_bus_name(&other.text)
            && bus_members_for_name_with_aliases(&other.text, schema).iter().any(|member| {
                member == &label.text || member == &format!("/{}", label.text)
            })
    })
}

fn root_label_isolated(
    label: &LabelInfo,
    _schema: &ParsedSchema,
    logical_nets: &[ResolvedNet],
    root_sheet_pin_names: &BTreeMap<Point, BTreeSet<String>>,
) -> bool {
    let Some(net) = logical_nets.iter().find(|net| {
        net.labels.iter().any(|other| {
            other.point == label.point
                && other.label_type == label.label_type
                && other.text == label.text
        })
    }) else {
        return false;
    };

    if net.labels.iter().any(|other| {
        other.point != label.point && looks_like_bus_name(&other.text)
    }) {
        return false;
    }

    let all_pins = logical_nets
        .iter()
        .filter(|other| other.name == net.name)
        .map(|other| other.nodes.len())
        .sum::<usize>();

    let has_no_connect = logical_nets
        .iter()
        .filter(|other| other.name == net.name)
        .any(|other| other.no_connect);

    if has_no_connect {
        return false;
    }

    let connected_sheet_pin_names = connected_root_sheet_pin_names(
        label,
        _schema,
        root_sheet_pin_names,
    );

    if !connected_sheet_pin_names.is_empty() {
        if label.label_type != "hierarchical_label" {
            return false;
        }

        return !connected_sheet_pin_names.contains(&label.text)
            && connected_pin_like_count_for_label(label, _schema) == 1;
    }

    if all_pins == 1 {
        return true;
    }

    label.label_type == "hierarchical_label"
        && all_pins == 0
        && connected_pin_like_count_for_label(label, _schema) == 1
}

fn connected_root_sheet_pin_names(
    label: &LabelInfo,
    schema: &ParsedSchema,
    root_sheet_pin_names: &BTreeMap<Point, BTreeSet<String>>,
) -> BTreeSet<String> {
    let segments = connected_wire_segments(label.point, schema);
    if segments.is_empty() {
        return root_sheet_pin_names
            .get(&label.point)
            .cloned()
            .unwrap_or_default();
    }

    root_sheet_pin_names
        .iter()
        .filter(|(point, _)| segments.iter().any(|segment| point_on_segment(**point, segment)))
        .flat_map(|(_, names)| names.iter().cloned())
        .collect()
}

pub(crate) fn collect_root_violations(
    input: &Path,
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
    physical_groups: &[PhysicalGroup],
    resolved_symbol_libs: &ProjectSymbolLibraryIndex,
    available_footprint_libs: &FootprintLibraryIndex,
    defined_netclasses: &HashSet<String>,
    project_netclass_assignments: &NetclassAssignmentMap,
    parameterized_netclasses: &HashSet<String>,
    project_rule_severities: &RuleSeverityMap,
    connection_grid_mm: f64,
    root_attached_points: &[Point],
    descendant_global_labels: &BTreeSet<String>,
) -> Result<Vec<PendingViolation>, KiError> {
    let root_sheet_pin_names = sheet_refs(input, None)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|sheet| {
            sheet
                .pin_refs
                .into_iter()
                .map(|pin| (pin.point, pin.name))
                .collect::<Vec<_>>()
        })
        .fold(BTreeMap::<Point, BTreeSet<String>>::new(), |mut acc, (point, name)| {
            acc.entry(point).or_default().insert(name);
            acc
        });

    let mut pending = power_pin_not_driven_violations(schema, nets);

    pending.extend(duplicate_sheet_name_violations(input, project_rule_severities)?);
    pending.extend(bus_to_net_conflict_violations(
        schema,
        nets,
        project_rule_severities,
    ));
    pending.extend(net_not_bus_member_violations(schema, project_rule_severities));
    pending.extend(multiple_net_names_violations(
        schema,
        nets,
        project_rule_severities,
    ));

    pending.extend(nets.iter().filter_map(|net| {
        if net.no_connect {
            return None;
        }

        if net
            .nodes
            .iter()
            .any(|node| node.pin_type.as_deref() == Some("power_in"))
        {
            return None;
        }

        let has_signal_driver = net.nodes.iter().any(|node| {
            matches!(
                node.pin_type.as_deref(),
                Some("output") | Some("bidirectional") | Some("passive") | Some("power_out")
            )
        });

        if has_signal_driver {
            return None;
        }

        let first_input = net
            .nodes
            .iter()
            .filter(|node| node.pin_type.as_deref() == Some("input"))
            .find(|node| !is_helper_power_symbol(node))?;

        Some(PendingViolation {
            severity: Severity::Error,
            description: "Input pin not driven by any Output pins".to_string(),
            violation_type: "pin_not_driven".to_string(),
            items: vec![PendingItem::from_point(
                format_pin_item_description(first_input),
                first_input.point,
            )],
        })
    }));

    pending.extend(nets.iter().flat_map(|net| {
        let pins = net.nodes.iter().collect::<Vec<_>>();

        reduced_pin_conflicts(&pins)
            .into_iter()
            .filter_map(|(first, second, level)| {
                let (primary, other) = order_pin_conflict_items(first, second);
                let (description_first, description_second) =
                    order_pin_conflict_description(first, second);

                let default_severity = match level {
                    PinConflictLevel::Warning => Severity::Warning,
                    PinConflictLevel::Error => Severity::Error,
                };
                let severity = project_rule_severity(
                    project_rule_severities,
                    "pin_to_pin",
                    default_severity,
                )?;

                Some(PendingViolation {
                    severity,
                    description: format!(
                        "Pins of type {} and {} are connected",
                        format_pin_type_name(description_first.pin_type.as_deref()),
                        format_pin_type_name(description_second.pin_type.as_deref())
                    ),
                    violation_type: "pin_to_pin".to_string(),
                    items: vec![
                        PendingItem::from_point(format_pin_item_description(primary), primary.point),
                        PendingItem::from_point(format_pin_item_description(other), other.point),
                    ],
                })
            })
            .collect::<Vec<_>>()
    }));

    let net_names_by_point = nets
        .iter()
        .flat_map(|net| {
            net.nodes
                .iter()
                .map(move |node| (node.point, net.name.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    for symbol in &schema.symbols {
        let symbol_power_pins = schema
            .pin_nodes
            .iter()
            .filter(|pin| pin.reference == symbol.reference)
            .filter(|pin| matches!(pin.pin_type.as_deref(), Some("power_in") | Some("power_out")))
            .collect::<Vec<_>>();

        let has_ground_net = symbol_power_pins.iter().any(|pin| {
            net_names_by_point
                .get(&pin.point)
                .is_some_and(|name| name.to_ascii_uppercase().contains("GND"))
        });

        if !has_ground_net {
            continue;
        }

        let Some(severity) = project_rule_severity(
            project_rule_severities,
            "ground_pin_not_ground",
            Severity::Warning,
        ) else {
            continue;
        };

        for pin in symbol_power_pins {
            if is_helper_power_symbol(pin) {
                continue;
            }

            let pin_name = pin.pin_function.as_deref().unwrap_or_default();
            let pin_name_is_ground = pin_name.to_ascii_uppercase().contains("GND");
            let net_is_ground = net_names_by_point
                .get(&pin.point)
                .is_some_and(|name| name.to_ascii_uppercase().contains("GND"));

            if !pin_name_is_ground || net_is_ground {
                continue;
            }

            pending.push(PendingViolation {
                severity,
                description: format!("Pin {} not connected to ground net", pin_name),
                violation_type: "ground_pin_not_ground".to_string(),
                items: vec![PendingItem::from_point(
                    format_pin_item_description(pin),
                    pin.point,
                )],
            });
        }
    }

    pending.extend(pin_not_connected_violations(
        schema,
        physical_groups,
        nets,
        project_rule_severities,
        project_netclass_assignments,
        parameterized_netclasses,
    ));

    pending.extend(schema.symbols.iter().filter_map(|symbol| {
        build_lib_symbol_issue(symbol, resolved_symbol_libs).map(|description| PendingViolation {
            severity: Severity::Warning,
            description,
            violation_type: "lib_symbol_issues".to_string(),
            items: vec![PendingItem::from_point(
                format_symbol_item_description(symbol),
                symbol.at,
            )],
        })
    }));

    pending.extend(lib_symbol_mismatch_violations(
        schema,
        resolved_symbol_libs,
        project_rule_severities,
    ));
    pending.extend(duplicate_pin_violations(schema, nets, project_rule_severities));
    pending.extend(different_unit_net_violations(schema, nets, project_rule_severities));

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| property.name == "Netclass")
            .filter(|property| !property.value.is_empty())
            .filter(|property| !defined_netclasses.contains(&property.value))
            .map(|property| PendingViolation {
                severity: Severity::Error,
                description: format!("Netclass {} is not defined", property.value),
                violation_type: "undefined_netclass".to_string(),
                items: vec![PendingItem::from_point(
                    format_symbol_item_description(symbol),
                    symbol.at,
                )],
            })
            .collect::<Vec<_>>()
    }));

    pending.extend(
        schema
            .netclass_flags
            .iter()
            .filter(|flag| !flag.netclass.is_empty())
            .filter(|flag| !defined_netclasses.contains(&flag.netclass))
            .map(undefined_netclass_flag_violation),
    );

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter_map(|property| parse_erc_assertion(&property.value).map(|assertion| (property, assertion)))
            .map(|(property, assertion)| PendingViolation {
                severity: assertion.severity,
                description: assertion.message,
                violation_type: assertion.violation_type.to_string(),
                items: vec![PendingItem {
                    description: format!("Field {} (empty)", property.name),
                    x_mm: property.x.unwrap_or(symbol.at.x as f64 / 10_000.0),
                    y_mm: property.y.unwrap_or(symbol.at.y as f64 / 10_000.0),
                }],
            })
            .collect::<Vec<_>>()
    }));

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| !property.name.starts_with("Sim."))
            .filter(|property| {
                property_contains_unresolved_variable(&property.value, &symbol.properties)
            })
            .map(|_| PendingViolation {
                severity: Severity::Error,
                description: "Unresolved text variable".to_string(),
                violation_type: "unresolved_variable".to_string(),
                items: vec![PendingItem::from_point(
                    format_symbol_item_description(symbol),
                    symbol.at,
                )],
            })
            .collect::<Vec<_>>()
    }));

    append_multi_unit_violations(&mut pending, schema);

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        build_footprint_link_issues(
            symbol,
            schema,
            resolved_symbol_libs,
            available_footprint_libs,
        )
        .into_iter()
        .map(|description| PendingViolation {
            severity: Severity::Warning,
            description,
            violation_type: "footprint_link_issues".to_string(),
            items: vec![PendingItem::from_point(
                format_symbol_item_description(symbol),
                symbol.at,
            )],
        })
        .collect::<Vec<_>>()
    }));

    append_misc_root_violations(
        &mut pending,
        schema,
        root_attached_points,
        &root_sheet_pin_names,
        project_rule_severities,
        project_netclass_assignments,
        parameterized_netclasses,
        connection_grid_mm,
        descendant_global_labels,
    );

    Ok(pending)
}

fn append_multi_unit_violations(pending: &mut Vec<PendingViolation>, schema: &ParsedSchema) {
    let mut symbols_by_reference = BTreeMap::<String, Vec<&PlacedSymbol>>::new();
    for symbol in &schema.symbols {
        symbols_by_reference
            .entry(symbol.reference.clone())
            .or_default()
            .push(symbol);
    }

    for symbols in symbols_by_reference.into_values() {
        let representative = symbols[0];
        let Some(embedded) = schema.embedded_symbols.get(&representative.lib_id) else {
            continue;
        };

        if embedded.unit_count <= 1 {
            continue;
        }

        let present_units = symbols.iter().map(|symbol| symbol.unit).collect::<Vec<_>>();
        let missing_units = (1..=embedded.unit_count)
            .filter(|unit| !present_units.contains(unit))
            .collect::<Vec<_>>();
        let format_missing_units = |units: &[i32]| {
            format_embedded_units_list(units, &embedded.unit_names)
        };

        let mut footprint_symbols = symbols
            .iter()
            .filter_map(|symbol| symbol.footprint.as_ref().map(|footprint| (*symbol, footprint)))
            .collect::<Vec<_>>();
        footprint_symbols.sort_by(|(lhs_symbol, lhs_fp), (rhs_symbol, rhs_fp)| {
            lhs_fp
                .cmp(rhs_fp)
                .then_with(|| lhs_symbol.unit.cmp(&rhs_symbol.unit))
        });

        if let Some((first_symbol, first_footprint)) = footprint_symbols.first().copied() {
            for (other_symbol, other_footprint) in footprint_symbols.iter().skip(1).copied() {
                if other_footprint == first_footprint {
                    continue;
                }

                pending.push(PendingViolation {
                    severity: Severity::Error,
                    description: format!(
                        "Different footprints assigned to {} and {}",
                        format!("{}{}", first_symbol.reference, unit_suffix(first_symbol.unit)),
                        format!("{}{}", other_symbol.reference, unit_suffix(other_symbol.unit))
                    ),
                    violation_type: "different_unit_footprint".to_string(),
                    items: vec![
                        PendingItem::from_point(
                            format_symbol_item_description(first_symbol),
                            first_symbol.at,
                        ),
                        PendingItem::from_point(
                            format_symbol_item_description(other_symbol),
                            other_symbol.at,
                        ),
                    ],
                });
                break;
            }
        }

        if missing_units.is_empty() {
            continue;
        }

        pending.push(PendingViolation {
            severity: Severity::Warning,
            description: format!(
                "Symbol {} has unplaced units {}",
                representative.reference,
                format_missing_units(&missing_units)
            ),
            violation_type: "missing_unit".to_string(),
            items: vec![PendingItem::from_point(
                format_symbol_item_description(representative),
                representative.at,
            )],
        });

        let mut missing_input_units = Vec::new();
        let mut missing_bidi_units = Vec::new();
        let mut missing_power_units = Vec::new();

        for unit in &missing_units {
            for pin in embedded.pins.iter().filter(|pin| pin.unit == *unit) {
                match pin.electrical_type.as_deref() {
                    Some("input") if !missing_input_units.contains(unit) => missing_input_units.push(*unit),
                    Some("bidirectional") if !missing_bidi_units.contains(unit) => missing_bidi_units.push(*unit),
                    Some("power_in") if !missing_power_units.contains(unit) => missing_power_units.push(*unit),
                    _ => {}
                }
            }
        }

        for (violation_type, description, severity, units) in [
            ("missing_input_pin", "input pins", Severity::Warning, missing_input_units.as_slice()),
            ("missing_bidi_pin", "bidirectional pins", Severity::Warning, missing_bidi_units.as_slice()),
            ("missing_power_pin", "input power pins", Severity::Error, missing_power_units.as_slice()),
        ] {
            if units.is_empty() {
                continue;
            }

            pending.push(PendingViolation {
                severity,
                description: format!(
                    "Symbol {} has {} in units {} that are not placed",
                    representative.reference,
                    description,
                    format_missing_units(units)
                ),
                violation_type: violation_type.to_string(),
                items: vec![PendingItem::from_point(
                    format_symbol_item_description(representative),
                    representative.at,
                )],
            });
        }
    }
}

fn format_embedded_units_list(units: &[i32], unit_names: &BTreeMap<i32, String>) -> String {
    let mut names = Vec::new();

    for (idx, unit) in units.iter().enumerate() {
        if idx == 3 {
            names.push(".".to_string());
            break;
        }

        names.push(
            unit_names
                .get(unit)
                .cloned()
                .unwrap_or_else(|| unit_suffix(*unit)),
        );
    }

    let names = names.join(", ");
    format!("[ {names} ]")
}

fn append_misc_root_violations(
    pending: &mut Vec<PendingViolation>,
    schema: &ParsedSchema,
    root_attached_points: &[Point],
    root_sheet_pin_names: &BTreeMap<Point, BTreeSet<String>>,
    project_rule_severities: &RuleSeverityMap,
    project_netclass_assignments: &NetclassAssignmentMap,
    parameterized_netclasses: &HashSet<String>,
    connection_grid_mm: f64,
    descendant_global_labels: &BTreeSet<String>,
) {
    let logical_nets = crate::schematic::render::resolve_nets(schema);
    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| property.name.trim() != property.name)
            .map(|property| PendingViolation {
                severity: Severity::Warning,
                description: format!(
                    "Field name has leading or trailing whitespace: '{}'",
                    property.name
                ),
                violation_type: "field_name_whitespace".to_string(),
                items: vec![
                    PendingItem::from_point(format_symbol_item_description(symbol), symbol.at),
                    PendingItem {
                        description: format!("Field {} '{}'", property.name, property.value),
                        x_mm: property.x.unwrap_or(symbol.at.x as f64 / 10_000.0),
                        y_mm: property.y.unwrap_or(symbol.at.y as f64 / 10_000.0),
                    },
                ],
            })
            .collect::<Vec<_>>()
    }));

    let single_global_label_severity = match project_rule_severities
        .get("single_global_label")
        .map(String::as_str)
    {
        Some("error") => Some(Severity::Error),
        Some("warning") => Some(Severity::Warning),
        Some("exclude") | Some("exclusion") => Some(Severity::Exclusion),
        Some("ignore") | None => None,
        Some(_) => None,
    };

    if let Some(single_global_label_severity) = single_global_label_severity {
        let mut global_labels = BTreeMap::<String, Vec<&LabelInfo>>::new();
        for label in &schema.labels {
            if label.label_type == "global_label" && !is_generated_power_label(label, schema) {
                global_labels.entry(label.text.clone()).or_default().push(label);
            }
        }

        pending.extend(global_labels.into_iter().filter_map(|(text, labels)| {
            if labels.len() != 1 {
                return None;
            }
            if schema
                .labels
                .iter()
                .any(|label| label.label_type == "label" && label.text == text)
            {
                return None;
            }
            let label = labels[0];
            Some(PendingViolation {
                severity: single_global_label_severity,
                description: "Global label only appears once in the schematic".to_string(),
                violation_type: "single_global_label".to_string(),
                items: vec![PendingItem {
                    description: format!("Global Label '{text}'"),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            })
        }));
    }

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| !is_generated_power_label(label, schema))
            .filter(|label| !looks_like_bus_name(&label.text))
            .filter(|label| {
                label.label_type != "global_label"
                    || !descendant_global_labels.contains(&label.text)
            })
            .filter(|label| {
                if let Some(net) = logical_nets.iter().find(|net| {
                    net.labels.iter().any(|other| {
                        other.point == label.point
                            && other.label_type == label.label_type
                            && other.text == label.text
                    })
                }) {
                    if root_label_isolated(label, schema, &logical_nets, &root_sheet_pin_names) {
                        return false;
                    }

                    let all_pins = logical_nets
                        .iter()
                        .filter(|other| other.name == net.name)
                        .map(|other| other.nodes.len())
                        .sum::<usize>();

                    return all_pins == 0
                        && !global_label_has_same_text_local_on_same_wire(label, schema);
                }

                is_dangling_logical_label(label, schema, &logical_nets)
            })
            .map(|label| PendingViolation {
                severity: Severity::Error,
                description: "Label not connected".to_string(),
                violation_type: "label_dangling".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    pending.extend(
        schema
            .netclass_flags
            .iter()
            .filter(|flag| is_dangling_netclass_flag(flag, schema))
            .map(|flag| PendingViolation::single(
                Severity::Error,
                "label_dangling",
                "Label not connected",
                PendingItem::new(
                    format!("Directive Label [Net Class {}]", flag.netclass),
                    flag.x,
                    flag.y,
                ),
            )),
    );

    pending.extend(
        schema
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
                label.label_type != "global_label"
                    || !descendant_global_labels.contains(&label.text)
            })
            .filter(|label| !label_has_no_connect(label, schema))
            .filter(|label| !label_has_no_connect_across_nets(label, &logical_nets))
            .filter(|label| !looks_like_bus_name(&label.text))
            .filter(|label| !label_exports_through_sheet_bus(label, schema))
            .filter(|label| {
                root_label_isolated(label, schema, &logical_nets, &root_sheet_pin_names)
            })
            .map(|label| PendingViolation {
                severity: Severity::Warning,
                description: "Label connected to only one pin".to_string(),
                violation_type: "isolated_pin_label".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    pending.extend(
        schema
            .bus_entries
            .iter()
            .filter(|entry| {
                let touches_wire = |point| {
                    schema
                        .wires
                        .iter()
                        .any(|segment| point_on_segment(point, segment))
                };

                !touches_wire(entry.bus_point)
                    && !touches_wire(entry.wire_point)
            })
            .flat_map(|entry| {
                [
                    PendingViolation {
                        severity: Severity::Warning,
                        description: "Unconnected wire to bus entry".to_string(),
                        violation_type: "unconnected_wire_endpoint".to_string(),
                        items: vec![PendingItem::from_point("Bus to wire entry", entry.bus_point)],
                    },
                    PendingViolation {
                        severity: Severity::Error,
                        description: "Wires not connected to anything".to_string(),
                        violation_type: "wire_dangling".to_string(),
                        items: vec![PendingItem::from_point("Bus to wire entry", entry.bus_point)],
                    },
                ]
            }),
    );

    pending.extend(
        logical_nets
            .iter()
            .flat_map(|net| {
                if root_hierarchical_label_should_follow_project_netclass(
                    net,
                    project_netclass_assignments,
                    parameterized_netclasses,
                ) {
                    return Vec::new();
                }

                net.labels.iter()
                    .filter(|label| {
                        label.label_type == "hierarchical_label" && !is_generated_power_label(label, schema)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .map(|label| PendingViolation {
                severity: Severity::Error,
                description: format!(
                    "Hierarchical label '{}' in root sheet cannot be connected to non-existent parent sheet",
                    label.text
                ),
                violation_type: "pin_not_connected".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(&label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    let mut labels_by_lower = BTreeMap::<String, Vec<&LabelInfo>>::new();
    for label in &schema.labels {
        if matches!(label.label_type.as_str(), "label" | "hierarchical_label")
            || (label.label_type == "global_label" && !is_generated_power_label(label, schema))
        {
            labels_by_lower
                .entry(label.text.to_lowercase())
                .or_default()
                .push(label);
        }
    }

    pending.extend(labels_by_lower.into_values().flat_map(|labels| {
        let mut pairings = Vec::new();
        for (idx, label) in labels.iter().enumerate() {
            for other in labels.iter().skip(idx + 1) {
                if label.text == other.text {
                    continue;
                }
                pairings.push(PendingViolation {
                    severity: Severity::Warning,
                    description: "Labels are similar (lower/upper case difference only)".to_string(),
                    violation_type: "similar_labels".to_string(),
                    items: vec![
                        PendingItem { description: format!("Label '{}'", label.text), x_mm: label.x, y_mm: label.y },
                        PendingItem { description: format!("Label '{}'", other.text), x_mm: other.x, y_mm: other.y },
                    ],
                });
            }
        }
        pairings
    }));

    let helper_power_pins = schema
        .pin_nodes
        .iter()
        .filter(|pin| pin.pin_type.as_deref() == Some("power_in"))
        .filter(|pin| is_helper_power_symbol(pin))
        .collect::<Vec<_>>();

    let mut power_names_by_lower = BTreeMap::<String, Vec<&PinNode>>::new();
    for pin in &helper_power_pins {
        let Some(symbol) = symbol_for_pin(pin, schema) else { continue; };
        let Some(value) = symbol.value.as_deref() else { continue; };
        power_names_by_lower.entry(value.to_lowercase()).or_default().push(*pin);
    }

    pending.extend(power_names_by_lower.into_values().flat_map(|pins| {
        let mut pairings = Vec::new();
        for (idx, pin) in pins.iter().enumerate() {
            let Some(symbol) = symbol_for_pin(pin, schema) else { continue; };
            let Some(value) = symbol.value.as_deref() else { continue; };
            for other in pins.iter().skip(idx + 1) {
                let Some(other_symbol) = symbol_for_pin(other, schema) else { continue; };
                let Some(other_value) = other_symbol.value.as_deref() else { continue; };
                if value == other_value {
                    continue;
                }
                pairings.push(PendingViolation {
                    severity: Severity::Warning,
                    description: "Power pins are similar (lower/upper case difference only)".to_string(),
                    violation_type: "similar_power".to_string(),
                    items: vec![
                        PendingItem::from_point(format_pin_item_description(pin), pin.point),
                        PendingItem::from_point(format_pin_item_description(other), other.point),
                    ],
                });
            }
        }
        pairings
    }));

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .flat_map(|label| {
                schema
                    .pin_nodes
                    .iter()
                    .filter(|pin| pin.pin_type.as_deref() == Some("power_in"))
                    .filter(|pin| is_helper_power_symbol(pin))
                    .filter_map(|pin| {
                        let symbol = schema.symbols.iter().find(|symbol| symbol.reference == pin.reference)?;
                        let symbol_value = symbol.value.as_deref()?;
                        if symbol_value.to_lowercase() != label.text.to_lowercase() || symbol_value == label.text {
                            return None;
                        }
                        Some(PendingViolation {
                            severity: Severity::Warning,
                            description: "Power pin and label are similar (lower/upper case difference only)".to_string(),
                            violation_type: "similar_label_and_power".to_string(),
                            items: vec![
                                PendingItem { description: format!("Label '{}'", label.text), x_mm: label.x, y_mm: label.y },
                                PendingItem::from_point(format_pin_item_description(pin), pin.point),
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            }),
    );

    pending.extend(schema.no_connects.iter().filter_map(|point| {
        if is_dangling_no_connect(*point, schema) {
            return Some(PendingViolation {
                severity: Severity::Warning,
                description: "Unconnected \"no connection\" flag".to_string(),
                violation_type: "no_connect_dangling".to_string(),
                items: vec![PendingItem::from_point("No Connect", *point)],
            });
        }

        let unique_connected_pins =
            unique_no_connect_pins_across_nets(*point, schema, &logical_nets);
        if unique_connected_pins.len() <= 1 {
            return None;
        }

        let mut direct_connected_pins = Vec::new();
        for candidate in connected_pins_for_no_connect(*point, schema) {
            if direct_connected_pins.iter().any(|other: &&PinNode| {
                other.reference == candidate.reference && other.point == candidate.point
            }) {
                continue;
            }

            direct_connected_pins.push(candidate);
        }

        let primary_pin = if unique_connected_pins.len() > direct_connected_pins.len() {
            unique_connected_pins.iter().max_by_key(|pin| {
                let dx = pin.point.x - point.x;
                let dy = pin.point.y - point.y;
                dx * dx + dy * dy
            })
        } else {
            direct_connected_pins
                .iter()
                .find(|pin| pin.point == *point)
                .copied()
                .or_else(|| {
                    direct_connected_pins.iter().max_by_key(|pin| {
                        let dx = pin.point.x - point.x;
                        let dy = pin.point.y - point.y;
                        dx * dx + dy * dy
                    }).copied()
                })
        };

        Some(if let Some(primary_pin) = primary_pin {
            PendingViolation {
                severity: Severity::Warning,
                description: "A pin with a \"no connection\" flag is connected".to_string(),
                violation_type: "no_connect_connected".to_string(),
                items: vec![
                    PendingItem::from_point(format_pin_item_description(primary_pin), primary_pin.point),
                    PendingItem::from_point("No Connect", *point),
                ],
            }
        } else {
            PendingViolation {
                severity: Severity::Warning,
                description: "A pin with a \"no connection\" flag is connected".to_string(),
                violation_type: "no_connect_connected".to_string(),
                items: vec![PendingItem::from_point("No Connect", *point)],
            }
        })
    }));

    pending.extend(schema.pin_nodes.iter().filter_map(|pin| {
        if !pin_is_no_connect_type(pin.pin_type.as_deref()) {
            return None;
        }

        let wire = schema
            .wires
            .iter()
            .find(|segment| segment.a == pin.point || segment.b == pin.point)?;

        Some(PendingViolation {
            severity: Severity::Warning,
            description: "Pin with 'no connection' type is connected".to_string(),
            violation_type: "no_connect_connected".to_string(),
            items: vec![
                PendingItem::from_point(format_segment_item_description(wire), wire.a),
                PendingItem::from_point(format_pin_item_description(pin), pin.point),
            ],
        })
    }));

    pending.extend(endpoint_off_grid_violations(schema, connection_grid_mm));

    pending.extend(
        schema
            .pin_nodes
            .iter()
            .filter(|pin| resembles_invalid_stacked_pin(&pin.pin))
            .map(|pin| PendingViolation {
                severity: Severity::Warning,
                description: "Pin name resembles stacked pin".to_string(),
                violation_type: "stacked_pin_name".to_string(),
                items: vec![PendingItem::from_point(format_pin_item_description(pin), pin.point)],
            }),
    );

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter_map(|label| {
                let overlapping = schema
                    .wires
                    .iter()
                    .filter(|segment| point_on_segment(label.point, segment) && label.point != segment.a && label.point != segment.b)
                    .collect::<Vec<_>>();

                if overlapping.len() <= 1 {
                    return None;
                }

                Some(PendingViolation {
                    severity: Severity::Warning,
                    description: format!("Label connects more than one wire at {}, {}", label.point.x, label.point.y),
                    violation_type: "label_multiple_wires".to_string(),
                    items: vec![
                        PendingItem { description: format!("Label '{}'", label.text), x_mm: label.x, y_mm: label.y },
                        PendingItem {
                            description: format_segment_item_description(overlapping[0]),
                            x_mm: segment_anchor_mm(overlapping[0]).0,
                            y_mm: segment_anchor_mm(overlapping[0]).1,
                        },
                    ],
                })
            }),
    );

    let mut connection_map = BTreeMap::<Point, Vec<&Segment>>::new();
    for segment in &schema.wires {
        connection_map.entry(segment.a).or_default().push(segment);
        connection_map.entry(segment.b).or_default().push(segment);
    }

    pending.extend(connection_map.into_iter().filter_map(|(point, segments)| {
        if segments.len() < 4 || schema.junctions.contains(&point) {
            return None;
        }
        let horizontal = segments.iter().find(|segment| segment.a.y == segment.b.y).copied()?;
        let vertical = segments.iter().find(|segment| segment.a.x == segment.b.x).copied()?;
        let horizontal_pos = if horizontal.a.x <= horizontal.b.x { horizontal.a } else { horizontal.b };
        Some(PendingViolation {
            severity: Severity::Warning,
            description: format!("Four items connected at {}, {}", point.x, point.y),
            violation_type: "four_way_junction".to_string(),
            items: vec![
                PendingItem {
                    description: format!(
                        "Horizontal Wire, length {:.4} mm",
                        segment_length_mm(horizontal) / 100.0
                    ),
                    x_mm: horizontal_pos.x as f64 / 10_000.0,
                    y_mm: horizontal_pos.y as f64 / 10_000.0,
                },
                PendingItem {
                    description: format!(
                        "Vertical Wire, length {:.4} mm",
                        segment_length_mm(vertical) / 100.0
                    ),
                    x_mm: point.x as f64 / 10_000.0,
                    y_mm: point.y as f64 / 10_000.0,
                },
            ],
        })
    }));

    pending.extend(
        schema
            .wires
            .iter()
            .filter_map(|segment| {
                let dangling_endpoints = dangling_segment_endpoint_count(segment, schema, root_attached_points);
                if dangling_endpoints == 0 {
                    return None;
                }
                let (x_mm, y_mm) = segment_anchor_mm(segment);
                Some(
                    (0..dangling_endpoints)
                        .map(|_| PendingViolation {
                            severity: Severity::Warning,
                            description: "Unconnected wire endpoint".to_string(),
                            violation_type: "unconnected_wire_endpoint".to_string(),
                            items: vec![PendingItem {
                                description: format_segment_item_description(segment),
                                x_mm,
                                y_mm,
                            }],
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten(),
    );

    pending.extend(
        wire_only_components(schema)
            .into_iter()
            .filter_map(|component| {
                if wire_component_has_driver(&component, schema, root_attached_points) {
                    return None;
                }
                let ordered_segments = schema
                    .wires
                    .iter()
                    .filter(|candidate| component.iter().any(|segment| same_segment(candidate, segment)))
                    .collect::<Vec<_>>();
                let items = ordered_segments
                    .into_iter()
                    .take(2)
                    .map(|segment| {
                        let (x_mm, y_mm) = segment_anchor_mm(segment);
                        PendingItem {
                            description: format_segment_item_description(segment),
                            x_mm,
                            y_mm,
                        }
                    })
                    .collect::<Vec<_>>();
                Some(PendingViolation {
                    severity: Severity::Error,
                    description: "Wires not connected to anything".to_string(),
                    violation_type: "wire_dangling".to_string(),
                    items,
                })
            }),
    );
}

fn root_hierarchical_label_should_follow_project_netclass(
    net: &ResolvedNet,
    project_netclass_assignments: &NetclassAssignmentMap,
    parameterized_netclasses: &HashSet<String>,
) -> bool {
    let _ = parameterized_netclasses;
    let assignments = project_netclass_assignments
        .get(&net.name)
        .or_else(|| net.name.rsplit('/').next().and_then(|leaf| project_netclass_assignments.get(leaf)));
    let Some(assignments) = assignments else {
        return false;
    };

    assignments.len() == 1
}

fn is_dangling_netclass_flag(
    flag: &crate::schematic::render::NetclassFlagInfo,
    schema: &ParsedSchema,
) -> bool {
    if schema
        .rule_area_borders
        .iter()
        .any(|border| point_on_segment(flag.point, border))
        || schema
            .buses
            .iter()
            .any(|segment| point_on_segment(flag.point, segment))
    {
        return false;
    }

    use crate::schematic::render::LabelInfo;

    let pseudo = LabelInfo {
        label_type: "directive_label".to_string(),
        text: flag.netclass.clone(),
        raw_text: flag.netclass.clone(),
        point: flag.point,
        x: flag.x,
        y: flag.y,
    };

    is_dangling_label(&pseudo, schema)
}

fn duplicate_sheet_name_violations(
    schematic_path: &Path,
    project_rule_severities: &RuleSeverityMap,
) -> Result<Vec<PendingViolation>, KiError> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "duplicate_sheet_names",
        Severity::Error,
    ) else {
        return Ok(Vec::new());
    };

    let text = std::fs::read_to_string(schematic_path)
        .map_err(|err| KiError::Message(format!("Failed to load schematic: {err}")))?;
    let cst = parse_one(&text)
        .map_err(|err| KiError::Message(format!("Failed to load schematic: {err}")))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    let mut sheets_by_name = BTreeMap::<String, Vec<(f64, f64)>>::new();

    for item in items.iter().skip(1) {
        if head_of(item) != Some("sheet") {
            continue;
        }

        let name = child_items(item)
            .iter()
            .find(|child| {
                head_of(child) == Some("property")
                    && nth_atom_string(child, 1).as_deref() == Some("Sheetname")
            })
            .and_then(|property| nth_atom_string(property, 2))
            .filter(|value| !value.is_empty());
        let at = child_items(item)
            .iter()
            .find(|child| head_of(child) == Some("at"))
            .map(|at| {
                (
                    nth_atom_f64(at, 1).unwrap_or(0.0),
                    nth_atom_f64(at, 2).unwrap_or(0.0),
                )
            });

        let (Some(name), Some((x, y))) = (name, at) else {
            continue;
        };

        sheets_by_name.entry(name).or_default().push((x, y));
    }

    Ok(sheets_by_name
        .into_iter()
        .filter(|(_, sheets)| sheets.len() > 1)
        .map(|(name, sheets)| {
            PendingViolation::new(
                severity,
                "duplicate_sheet_names",
                "Duplicate sheet names within a given sheet",
                sheets
                    .into_iter()
                    .map(|(x, y)| PendingItem::new(format!("Hierarchical Sheet '{name}'"), x, y))
                    .collect(),
            )
        })
        .collect())
}

fn global_label_has_same_text_local_on_same_wire(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    if label.label_type != "global_label" {
        return false;
    }

    let segments = super::super::connectivity::connected_wire_segments(label.point, schema);

    if segments.is_empty() {
        return schema
            .labels
            .iter()
            .any(|other| other.label_type == "label" && other.text == label.text);
    }

    schema.labels.iter().any(|other| {
        other.label_type == "label"
            && other.text == label.text
            && segments
                .iter()
                .any(|segment| point_on_segment(other.point, segment))
    })
}
