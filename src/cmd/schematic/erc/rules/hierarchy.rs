use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::extract::sym_lib::ProjectSymbolLibraryIndex;
use crate::schematic::render::{
    parse_schema, projected_reference_for_symbol_suffix, resolve_nets, resolve_physical_groups,
};
use super::super::connectivity::{
    bus_members_for_name_with_aliases, bus_segment_for_entry, connected_bus_segments,
    connected_pin_like_count_for_label, connected_wire_segments, dangling_segment_endpoint_count,
    display_label_name, is_dangling_logical_label, is_dangling_sheet_pin, label_has_no_connect,
    looks_like_bus_name, segment_key,
};
use super::super::format::{format_pin_item_description, format_pin_type_name};
use super::super::geom::{point_on_segment, point_on_segment_local, segment_anchor_mm};
use super::super::hierarchy::{
    apply_sheet_text_vars, load_project_footprint_libraries, load_project_symbol_libraries,
    sheet_refs, SheetRef,
};
use super::super::items::{bus_item, label_item, point_item, segment_item};
use super::super::pin_conflict::{
    order_pin_conflict_description, order_pin_conflict_items, pin_conflict_level,
    reduced_pin_conflicts, PinConflictLevel,
};
use super::super::project::{
    load_project_connection_grid_mm, load_project_netclass_assignments,
    load_project_parameterized_netclasses, project_rule_severity, RuleSeverityMap,
};
use super::super::report::ViolationMap;
use super::connectivity::{
    endpoint_off_grid_violations, multiple_net_names_violations, net_not_bus_member_violations,
    pin_not_connected_violations, power_pin_not_driven_violations_with_global_drivers,
};
use super::symbol::{
    build_footprint_link_issues, build_lib_symbol_issue, lib_symbol_mismatch_violations,
};
use super::super::{PendingViolation, Severity};
use crate::error::KiError;
use crate::cmd::schematic::erc::is_generated_power_label;

type ChildScreenViolationMap = BTreeMap<String, (String, Vec<PendingViolation>)>;

fn dedup_pending_violations(violations: &mut Vec<PendingViolation>) {
    let mut seen = BTreeSet::new();
    violations.retain(|violation| {
        seen.insert((
            violation.severity.as_str().to_string(),
            violation.violation_type.clone(),
            violation.description.clone(),
            violation
                .items
                .iter()
                .map(|item| {
                    (
                        item.description.clone(),
                        item.x_mm.to_bits(),
                        item.y_mm.to_bits(),
                    )
                })
                .collect::<Vec<_>>(),
        ))
    });

    let mut chosen_net_not_bus_member = BTreeMap::<
        (String, String, String),
        PendingViolation,
    >::new();

    for violation in violations
        .iter()
        .filter(|violation| violation.violation_type == "net_not_bus_member")
    {
        let key = (
            violation.severity.as_str().to_string(),
            violation.violation_type.clone(),
            violation.description.clone(),
        );

        chosen_net_not_bus_member
            .entry(key)
            .and_modify(|existing| {
                let existing_second = existing.items.get(1).map(|item| item.description.as_str());
                let candidate_second = violation.items.get(1).map(|item| item.description.as_str());
                let candidate_is_better = match (existing_second, candidate_second) {
                    (Some(existing_desc), Some(candidate_desc))
                        if existing_desc.contains("Wire") && candidate_desc.contains("Bus") =>
                    {
                        true
                    }
                    (Some(existing_desc), Some(candidate_desc))
                        if existing_desc.contains("Bus") && candidate_desc.contains("Wire") =>
                    {
                        false
                    }
                    _ => violation
                        .items
                        .iter()
                        .map(|item| {
                            (
                                item.description.clone(),
                                item.x_mm.to_bits(),
                                item.y_mm.to_bits(),
                            )
                        })
                        .collect::<Vec<_>>()
                        < existing
                            .items
                            .iter()
                            .map(|item| {
                                (
                                    item.description.clone(),
                                    item.x_mm.to_bits(),
                                    item.y_mm.to_bits(),
                                )
                            })
                            .collect::<Vec<_>>(),
                };

                if candidate_is_better {
                    *existing = violation.clone();
                }
            })
            .or_insert_with(|| violation.clone());
    }

    violations.retain(|violation| {
        if violation.violation_type != "net_not_bus_member" {
            return true;
        }

        let key = (
            violation.severity.as_str().to_string(),
            violation.violation_type.clone(),
            violation.description.clone(),
        );

        chosen_net_not_bus_member.get(&key).is_some_and(|chosen| {
            chosen.items.len() == violation.items.len()
                && chosen
                    .items
                    .iter()
                    .zip(&violation.items)
                    .all(|(left, right)| {
                        left.description == right.description
                            && left.x_mm == right.x_mm
                            && left.y_mm == right.y_mm
                    })
        })
    });
}

fn extend_unique_violations(
    target: &mut Vec<PendingViolation>,
    violations: impl IntoIterator<Item = PendingViolation>,
) {
    for violation in violations {
        let already_present = target.iter().any(|existing| {
            existing.violation_type == violation.violation_type
                && existing.severity == violation.severity
                && existing.description == violation.description
                && existing.items.len() == violation.items.len()
                && existing
                    .items
                    .iter()
                    .zip(&violation.items)
                    .all(|(left, right)| {
                        left.description == right.description
                            && left.x_mm == right.x_mm
                            && left.y_mm == right.y_mm
                    })
        });

        if !already_present {
            target.push(violation);
        }
    }
}

fn is_repeated_hierarchical_multiple_net_names(violation: &PendingViolation) -> bool {
    violation.violation_type == "multiple_net_names"
        && !violation.items.is_empty()
        && violation
            .items
            .iter()
            .all(|item| item.description.starts_with("Hierarchical Label '"))
}

fn helper_power_input_label(violation: &PendingViolation) -> Option<String> {
    if violation.violation_type != "power_pin_not_driven" {
        return None;
    }

    let item = violation.items.first()?;
    if !item.description.starts_with("Symbol #PWR") {
        return None;
    }

    let (_, rest) = item.description.split_once('[')?;
    let (label, _) = rest.split_once(',')?;
    Some(label.trim().to_string())
}

fn helper_power_symbol_label(violation: &PendingViolation) -> Option<String> {
    let item = violation.items.first()?;
    if !item.description.starts_with("Symbol #PWR") {
        return None;
    }

    let (_, rest) = item.description.split_once('[')?;
    let label = rest
        .split_once(',')
        .map(|(label, _)| label)
        .or_else(|| rest.split_once(']').map(|(label, _)| label))?;
    Some(label.trim().to_string())
}

fn child_sheet_path(current_sheet_path: &str, sheet: &SheetRef) -> String {
    if current_sheet_path == "/" {
        sheet.path.clone()
    } else {
        format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
    }
}

fn repeated_child_pin_to_pin_violations(
    root_dir: &Path,
    current_sheet_path: &str,
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet_refs: &[SheetRef],
    project_rule_severities: &RuleSeverityMap,
) -> Result<ViolationMap, KiError> {
    let mut by_file = BTreeMap::<&str, Vec<&SheetRef>>::new();
    for sheet in sheet_refs {
        by_file.entry(sheet.file.as_str()).or_default().push(sheet);
    }

    let mut out = ViolationMap::new();

    for mut siblings in by_file.into_values() {
        if siblings.len() <= 1 {
            continue;
        }
        siblings.sort_by(|left, right| {
            left.instance_path
                .cmp(&right.instance_path)
                .then_with(|| left.path.cmp(&right.path))
        });

        for (left_index, left_sheet) in siblings.iter().enumerate() {
            let left_child_path = root_dir.join(&left_sheet.file);
            let mut left_schema =
                parse_schema(&left_child_path.to_string_lossy(), Some(&left_sheet.instance_path))
                    .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
            apply_sheet_text_vars(&mut left_schema, left_sheet);
            let left_nets = resolve_nets(&left_schema);

            for right_sheet in siblings.iter().skip(left_index + 1) {
                let right_child_path = root_dir.join(&right_sheet.file);
                let mut right_schema = parse_schema(
                    &right_child_path.to_string_lossy(),
                    Some(&right_sheet.instance_path),
                )
                .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
                apply_sheet_text_vars(&mut right_schema, right_sheet);
                let right_nets = resolve_nets(&right_schema);

                for left_pin in &left_sheet.pin_refs {
                    for right_pin in &right_sheet.pin_refs {
                        if left_pin.name != right_pin.name || left_pin.point == right_pin.point {
                            continue;
                        }

                        let connected_segments =
                            connected_wire_segments(left_pin.point, parent_schema);
                        if connected_segments.is_empty()
                            || !connected_segments
                                .iter()
                                .any(|segment| point_on_segment(right_pin.point, segment))
                        {
                            continue;
                        }

                        let Some(left_net) = left_nets.iter().find(|net| {
                            net.labels.iter().any(|label| {
                                label.label_type == "hierarchical_label"
                                    && label.text == left_pin.name
                            })
                        }) else {
                            continue;
                        };

                        let Some(right_net) = right_nets.iter().find(|net| {
                            net.labels.iter().any(|label| {
                                label.label_type == "hierarchical_label"
                                    && label.text == right_pin.name
                            })
                        }) else {
                            continue;
                        };

                        let Some(left_node) = left_net.nodes.first() else {
                            continue;
                        };
                        let Some(right_node) = right_net.nodes.first() else {
                            continue;
                        };

                        let Some(level) = pin_conflict_level(left_node, right_node) else {
                            continue;
                        };

                        let report_path = child_sheet_path(current_sheet_path, left_sheet);
                        let mut reported_node = if current_sheet_path == "/" {
                            right_node.clone()
                        } else {
                            left_node.clone()
                        };

                        if current_sheet_path != "/" {
                            if let Some(suffix) = left_sheet.instance_path.rsplit('/').next() {
                                if let Some(reference) = projected_reference_for_symbol_suffix(
                                    &left_schema,
                                    &left_node.reference,
                                    suffix,
                                ) {
                                    reported_node.reference = reference.clone();
                                    reported_node.reference_with_unit = reference;
                                }
                            }
                        }

                        let item = point_item(
                            format_pin_item_description(&reported_node),
                            reported_node.point,
                        );
                        let Some(severity) = project_rule_severity(
                            project_rule_severities,
                            "pin_to_pin",
                            match level {
                                PinConflictLevel::Warning => Severity::Warning,
                                PinConflictLevel::Error => Severity::Error,
                            },
                        ) else {
                            continue;
                        };
                        extend_unique_violations(
                            out.entry(report_path).or_default(),
                            [PendingViolation {
                                severity,
                                description: format!(
                                    "Pins of type {} and {} are connected",
                                    format_pin_type_name(left_node.pin_type.as_deref()),
                                    format_pin_type_name(right_node.pin_type.as_deref())
                                ),
                                violation_type: "pin_to_pin".to_string(),
                                items: vec![item.clone(), item],
                            }],
                        );
                    }
                }
            }
        }
    }

    Ok(out)
}

fn prefix_descendant_net_name(net_name: &str, child_sheet_path: &str) -> String {
    if child_sheet_path == "/" {
        return net_name.to_string();
    }

    let sheet_prefix = child_sheet_path.trim_end_matches('/');

    if net_name.starts_with('/') {
        format!("{sheet_prefix}{net_name}")
    } else {
        format!("{sheet_prefix}/{net_name}")
    }
}

fn remap_descendant_bus_name(
    bus_name: &str,
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet: &SheetRef,
) -> String {
    let trimmed = bus_name.trim_start_matches('/');
    let Some(group_suffix_start) = trimmed.find('{') else {
        return bus_name.to_string();
    };
    let group_suffix = &trimmed[group_suffix_start..];

    let pin_ref = sheet
        .pin_refs
        .iter()
        .find(|pin| pin.name.ends_with(group_suffix))
        .or_else(|| sheet.pin_refs.iter().find(|pin| looks_like_bus_name(&pin.name)));
    let Some(pin_ref) = pin_ref else {
        return bus_name.to_string();
    };

    let segments = connected_bus_segments(pin_ref.point, parent_schema);
    parent_schema
        .labels
        .iter()
        .filter(|label| looks_like_bus_name(&label.text))
        .filter(|label| {
            segments
                .iter()
                .any(|segment| point_on_segment_local(label.point, segment))
        })
        .map(display_label_name)
        .min()
        .unwrap_or_else(|| bus_name.to_string())
}

fn rewrite_descendant_net_not_bus_member_violation(
    mut violation: PendingViolation,
    child_sheet_path: &str,
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet: &SheetRef,
) -> PendingViolation {
    if violation.violation_type != "net_not_bus_member" {
        return violation;
    }

    let prefix = "Net ";
    let infix = " is graphically connected to bus ";
    let suffix = " but is not a member of that bus";

    if let Some(rest) = violation.description.strip_prefix(prefix) {
        if let Some((net_name, rest)) = rest.split_once(infix) {
            if let Some(bus_name) = rest.strip_suffix(suffix) {
                let prefixed_net_name = prefix_descendant_net_name(net_name, child_sheet_path);
                let remapped_bus_name = remap_descendant_bus_name(bus_name, parent_schema, sheet);
                violation.description = format!(
                    "{prefix}{prefixed_net_name}{infix}{remapped_bus_name}{suffix}"
                );
            }
        }
    }

    violation
}

fn synthetic_leaf_prefixed_bus_member_conflicts(
    child_schema: &crate::schematic::render::ParsedSchema,
    child_sheet_path: &str,
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet: &SheetRef,
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "net_not_bus_member",
        Severity::Warning,
    ) else {
        return Vec::new();
    };

    let Some(sheet_pin) = sheet.pin_refs.iter().find(|pin| looks_like_bus_name(&pin.name)) else {
        return Vec::new();
    };
    let Some((pin_prefix, _)) = sheet_pin.name.trim_start_matches('/').split_once('{') else {
        return Vec::new();
    };
    if pin_prefix.is_empty() {
        return Vec::new();
    }

    child_schema
        .labels
        .iter()
        .filter(|label| label.label_type == "label")
        .filter(|label| !looks_like_bus_name(&label.text))
        .filter(|label| label.text.starts_with(&format!("{pin_prefix}.")))
        .filter_map(|label| {
            let connected_segments = connected_wire_segments(label.point, child_schema);
            let entry = child_schema.bus_entries.iter().find(|entry| {
                connected_segments
                    .iter()
                    .any(|segment| point_on_segment(entry.wire_point, segment))
            })?;
            let bus_segment = bus_segment_for_entry(entry, child_schema)?;
            let parent_bus_name = remap_descendant_bus_name(
                &format!("/{}", sheet_pin.name.trim_start_matches('/')),
                parent_schema,
                sheet,
            );
            let prefixed_net_name =
                prefix_descendant_net_name(&format!("/{}", label.text), child_sheet_path);

            if bus_members_for_name_with_aliases(&parent_bus_name, parent_schema)
                .contains(&prefixed_net_name)
            {
                return None;
            }

            let (x_mm, y_mm) = segment_anchor_mm(&bus_segment);
            Some(PendingViolation {
                severity,
                description: format!(
                    "Net {prefixed_net_name} is graphically connected to bus {parent_bus_name} but is not a member of that bus"
                ),
                violation_type: "net_not_bus_member".to_string(),
                items: vec![
                    point_item("Bus to wire entry", entry.bus_point),
                    bus_item(&bus_segment, x_mm, y_mm),
                ],
            })
        })
        .collect()
}

fn label_is_hierarchical_bus_member_stub(
    label: &crate::schematic::render::LabelInfo,
    schema: &crate::schematic::render::ParsedSchema,
    parent_has_multiple_bus_sheets: bool,
) -> bool {
    if !parent_has_multiple_bus_sheets {
        return false;
    }

    if label.label_type != "label" {
        return false;
    }

    let connected_segments = connected_wire_segments(label.point, schema);
    let touches_bus_entry = !connected_segments.is_empty()
        && schema.bus_entries.iter().any(|entry| {
            connected_segments
                .iter()
                .any(|segment| point_on_segment(entry.wire_point, segment))
        });

    if !touches_bus_entry {
        return false;
    }

    schema
        .labels
        .iter()
        .filter(|other| other.label_type == "hierarchical_label")
        .filter(|other| looks_like_bus_name(&other.text))
        .any(|other| {
            let member_name = format!("/{}", label.text);
            let members = bus_members_for_name_with_aliases(&other.text, schema);
            members.contains(&member_name) || members.contains(&label.text)
        })
}

fn label_is_bus_member_stub(
    label: &crate::schematic::render::LabelInfo,
    schema: &crate::schematic::render::ParsedSchema,
) -> bool {
    if label.label_type != "label" {
        return false;
    }

    let connected_segments = connected_wire_segments(label.point, schema);
    let touches_bus_entry = !connected_segments.is_empty()
        && schema.bus_entries.iter().any(|entry| {
            connected_segments
                .iter()
                .any(|segment| point_on_segment(entry.wire_point, segment))
        });

    if !touches_bus_entry {
        return false;
    }

    schema
        .labels
        .iter()
        .filter(|other| other.label_type == "hierarchical_label")
        .filter(|other| looks_like_bus_name(&other.text))
        .any(|other| {
            let member_name = format!("/{}", label.text);
            let members = bus_members_for_name_with_aliases(&other.text, schema);
            members.contains(&member_name) || members.contains(&label.text)
        })
}

fn logical_label_net_exports_to_parent(
    label: &crate::schematic::render::LabelInfo,
    schema: &crate::schematic::render::ParsedSchema,
    nets: &[crate::schematic::render::ResolvedNet],
    sheet: &SheetRef,
    allow_bus_member_export: bool,
) -> bool {
    if label.label_type == "label" {
        return nets
            .iter()
            .find(|net| {
                net.labels.iter().any(|other| {
                    other.point == label.point
                        && other.label_type == label.label_type
                        && other.text == label.text
                })
            })
            .is_some_and(|net| {
                net.labels.iter().any(|other| {
                    other.label_type == "hierarchical_label" && sheet.pins.contains(&other.text)
                })
            })
            || (allow_bus_member_export && label_is_bus_member_stub(label, schema));
    }

    if !allow_bus_member_export
        || label.label_type != "hierarchical_label"
        || !sheet.pins.contains(&label.text)
    {
        return false;
    }

    nets.iter()
        .find(|net| {
            net.labels.iter().any(|other| {
                other.point == label.point
                    && other.label_type == label.label_type
                    && other.text == label.text
            })
        })
        .is_some_and(|net| {
            net.labels
                .iter()
                .filter(|other| other.label_type == "label")
                .any(|other| label_is_bus_member_stub(other, schema))
        })
}

fn child_has_prefixed_bus_alias_descendants(child_path: &Path, sheet: &SheetRef) -> bool {
    sheet_refs(child_path, Some(&sheet.instance_path))
        .map(|refs| refs.iter().any(SheetRef::uses_prefixed_bus_alias_pins))
        .unwrap_or(false)
}

fn parent_has_expanded_bus_member_labels_for_sheet(
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet: &SheetRef,
) -> bool {
    sheet.pin_refs.iter().any(|pin| {
        let segments = connected_bus_segments(pin.point, parent_schema);
        if segments.is_empty() {
            return false;
        }

        parent_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .any(|member_label| {
                parent_schema
                    .labels
                    .iter()
                    .filter(|label| looks_like_bus_name(&label.text))
                    .filter(|label| {
                        segments
                            .iter()
                            .any(|segment| point_on_segment_local(label.point, segment))
                    })
                    .any(|bus_label| {
                        let members = bus_members_for_name_with_aliases(&bus_label.text, parent_schema);
                        let prefix_match = bus_label
                            .text
                            .split_once('{')
                            .map(|(prefix, _)| format!("{prefix}."))
                            .filter(|prefix| !prefix.is_empty())
                            .is_some_and(|prefix| member_label.text.starts_with(&prefix));
                        members.contains(&member_label.text)
                            || members.contains(&format!("/{}", member_label.text))
                            || prefix_match
                    })
            })
    })
}

fn should_project_prefixed_bus_alias_descendant_conflicts(
    child_path: &Path,
    parent_schema: &crate::schematic::render::ParsedSchema,
    sheet: &SheetRef,
) -> bool {
    child_has_prefixed_bus_alias_descendants(child_path, sheet)
        || parent_has_expanded_bus_member_labels_for_sheet(parent_schema, sheet)
}

pub(crate) fn hierarchical_sheet_violations(
    schematic_path: &Path,
    project_rule_severities: &RuleSeverityMap,
) -> Result<ViolationMap, KiError> {
    let mut out = ViolationMap::new();
    let mut root_screen_violations = ViolationMap::new();
    collect_hierarchical_sheet_violations(
        schematic_path,
        "/",
        None,
        project_rule_severities,
        &mut root_screen_violations,
        &mut out,
    )?;
    out.entry("/".to_string())
        .or_default()
        .extend(root_screen_violations.into_values().flatten());
    if let Some(root) = out.get_mut("/") {
        dedup_pending_violations(root);
    }
    Ok(out)
}

pub(crate) fn descendant_sheet_violations(
    schematic_path: &Path,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &RuleSeverityMap,
) -> Result<ViolationMap, KiError> {
    let mut out = ViolationMap::new();
    let mut root_screen_violations = ViolationMap::new();
    let mut child_screen_violations = ChildScreenViolationMap::new();
    let global_power_drivers = subtree_global_power_driver_nets(schematic_path, None)?;
    collect_descendant_sheet_violations(
        schematic_path,
        "/",
        None,
        symbol_libs,
        project_rule_severities,
        &global_power_drivers,
        &mut root_screen_violations,
        &mut child_screen_violations,
    )?;
    out.entry("/".to_string())
        .or_default()
        .extend(root_screen_violations.into_values().flatten());
    if let Some(root) = out.get_mut("/") {
        dedup_pending_violations(root);
    }
    for (sheet_path, violations) in child_screen_violations.into_values() {
        let entry = out.entry(sheet_path).or_default();
        entry.extend(violations);
        dedup_pending_violations(entry);
    }
    Ok(out)
}

pub(crate) fn same_local_global_label_hierarchy_violations(
    schematic_path: &Path,
    project_rule_severities: &RuleSeverityMap,
) -> Result<ViolationMap, KiError> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "same_local_global_label",
        Severity::Warning,
    ) else {
        return Ok(ViolationMap::new());
    };

    #[derive(Clone)]
    struct LabelOccurrence {
        text: String,
        x_mm: f64,
        y_mm: f64,
        sheet_path: String,
    }

    fn collect(
        schematic_path: &Path,
        current_sheet_path: &str,
        current_instance_path: Option<&str>,
        local_labels: &mut BTreeMap<String, LabelOccurrence>,
        global_labels: &mut BTreeMap<String, LabelOccurrence>,
    ) -> Result<(), KiError> {
        let mut schema =
            parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path)
                .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;

        if let Some(instance_path) = current_instance_path {
            if let Some(root_dir) = schematic_path.parent() {
                let parent_path = root_dir.parent().unwrap_or(root_dir);
                if let Ok(parent_refs) = sheet_refs(parent_path, Some(instance_path)) {
                    if let Some(sheet) = parent_refs.into_iter().next() {
                        apply_sheet_text_vars(&mut schema, &sheet);
                    }
                }
            }
        }

        for label in &schema.labels {
            if label.label_type == "label" {
                local_labels.entry(label.text.clone()).or_insert(LabelOccurrence {
                    text: label.text.clone(),
                    x_mm: label.x,
                    y_mm: label.y,
                    sheet_path: current_sheet_path.to_string(),
                });
            } else if label.label_type == "global_label"
                && !crate::cmd::schematic::erc::is_generated_power_label(label, &schema)
            {
                global_labels.entry(label.text.clone()).or_insert(LabelOccurrence {
                    text: label.text.clone(),
                    x_mm: label.x,
                    y_mm: label.y,
                    sheet_path: current_sheet_path.to_string(),
                });
            }
        }

        let Some(root_dir) = schematic_path.parent() else {
            return Ok(());
        };

        for sheet in sheet_refs(schematic_path, current_instance_path)? {
            let child_path = root_dir.join(&sheet.file);
            if !child_path.exists() {
                continue;
            }

            let child_sheet_path = if current_sheet_path == "/" {
                sheet.path.clone()
            } else {
                format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
            };

            let mut child_schema =
                parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
                    .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
            apply_sheet_text_vars(&mut child_schema, &sheet);

            for label in &child_schema.labels {
                if label.label_type == "label" {
                    local_labels.entry(label.text.clone()).or_insert(LabelOccurrence {
                        text: label.text.clone(),
                        x_mm: label.x,
                        y_mm: label.y,
                        sheet_path: child_sheet_path.clone(),
                    });
                } else if label.label_type == "global_label"
                    && !crate::cmd::schematic::erc::is_generated_power_label(label, &child_schema)
                {
                    global_labels.entry(label.text.clone()).or_insert(LabelOccurrence {
                        text: label.text.clone(),
                        x_mm: label.x,
                        y_mm: label.y,
                        sheet_path: child_sheet_path.clone(),
                    });
                }
            }

            collect(
                &child_path,
                &child_sheet_path,
                Some(&sheet.instance_path),
                local_labels,
                global_labels,
            )?;
        }

        Ok(())
    }

    let mut local_labels = BTreeMap::<String, LabelOccurrence>::new();
    let mut global_labels = BTreeMap::<String, LabelOccurrence>::new();
    collect(
        schematic_path,
        "/",
        None,
        &mut local_labels,
        &mut global_labels,
    )?;

    let mut out = ViolationMap::new();

    for (text, global) in global_labels {
        let Some(local) = local_labels.get(&text) else {
            continue;
        };

        out.entry(global.sheet_path.clone())
            .or_default()
            .push(PendingViolation {
                severity,
                description: "Local and global labels have same name".to_string(),
                violation_type: "same_local_global_label".to_string(),
                items: vec![
                    point_item(
                        format!("Global Label '{}'", global.text),
                        crate::schematic::render::Point {
                            x: (global.x_mm * 10_000.0) as i64,
                            y: (global.y_mm * 10_000.0) as i64,
                        },
                    ),
                    point_item(
                        format!("Label '{}'", local.text),
                        crate::schematic::render::Point {
                            x: (local.x_mm * 10_000.0) as i64,
                            y: (local.y_mm * 10_000.0) as i64,
                        },
                    ),
                ],
            });
    }

    Ok(out)
}

pub(crate) fn merge_pending_maps(out: &mut ViolationMap, incoming: ViolationMap) {
    for (path, violations) in incoming {
        let entry = out.entry(path).or_default();
        entry.extend(violations);
        dedup_pending_violations(entry);
    }
}

fn subtree_global_power_driver_nets(
    schematic_path: &Path,
    current_instance_path: Option<&str>,
) -> Result<BTreeSet<String>, KiError> {
    let schema = parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let nets = resolve_nets(&schema);
    let mut out = nets
        .iter()
        .filter(|net| net.labels.iter().any(|label| label.label_type == "global_label"))
        .filter(|net| {
            net.nodes.iter().any(|node| {
                matches!(
                    node.pin_type.as_deref(),
                    Some("power_out") | Some("output") | Some("bidirectional")
                )
            })
        })
        .map(|net| net.name.clone())
        .collect::<BTreeSet<_>>();

    let Some(root_dir) = schematic_path.parent() else {
        return Ok(out);
    };

    for sheet in sheet_refs(schematic_path, current_instance_path)? {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }

        out.extend(subtree_global_power_driver_nets(
            &child_path,
            Some(&sheet.instance_path),
        )?);
    }

    Ok(out)
}

fn collect_hierarchical_sheet_violations(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    project_rule_severities: &RuleSeverityMap,
    root_screen_violations: &mut ViolationMap,
    out: &mut ViolationMap,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };
    let parent_schema = parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;

    let sheet_refs = sheet_refs(schematic_path, current_instance_path)?;
    let repeated_files = sheet_refs
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut counts, sheet| {
            *counts.entry(sheet.file.clone()).or_default() += 1;
            counts
        });
    let parent_has_multiple_bus_sheets = repeated_files.keys().count() > 1
        && sheet_refs
            .iter()
            .filter(|sheet| sheet.pins.iter().any(|pin| looks_like_bus_name(pin)))
            .count()
            > 1;
    for sheet in sheet_refs {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }
        let child_sheet_path = child_sheet_path(current_sheet_path, &sheet);

        let mut child_schema = parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
            .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
        apply_sheet_text_vars(&mut child_schema, &sheet);
        let child_has_prefixed_descendants =
            child_has_prefixed_bus_alias_descendants(&child_path, &sheet);

        let child_logical_nets = resolve_nets(&child_schema);
        let allow_bus_member_export =
            repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1;
        let label_has_single_pin_like_connection = |label: &crate::schematic::render::LabelInfo| {
            if let Some(net) = child_logical_nets.iter().find(|net| {
                net.labels.iter().any(|other| {
                    other.point == label.point
                        && other.label_type == label.label_type
                        && other.text == label.text
                })
            }) {
                return net.nodes.len() == 1;
            }

            connected_pin_like_count_for_label(label, &child_schema) == 1
        };
        let label_net_exports_to_parent = |label: &crate::schematic::render::LabelInfo| {
            logical_label_net_exports_to_parent(
                label,
                &child_schema,
                &child_logical_nets,
                &sheet,
                allow_bus_member_export,
            )
        };
        let isolated_hier_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "hierarchical_label")
            .filter(|label| !label_has_no_connect(label, &child_schema))
            .filter(|label| label_has_single_pin_like_connection(label))
            .cloned()
            .collect::<Vec<_>>();
        let isolated_local_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter(|label| !label_has_no_connect(label, &child_schema))
            .filter(|label| label_has_single_pin_like_connection(label))
            .cloned()
            .collect::<Vec<_>>();
        let isolated_global_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "global_label")
            .filter(|label| !is_generated_power_label(label, &child_schema))
            .filter(|label| !label_has_no_connect(label, &child_schema))
            .filter(|label| label_has_single_pin_like_connection(label))
            .cloned()
            .collect::<Vec<_>>();
        let isolated_hier_points = isolated_hier_labels
            .iter()
            .map(|label| label.point)
            .collect::<BTreeSet<_>>();
        let isolated_local_points = isolated_local_labels
            .iter()
            .map(|label| label.point)
            .collect::<BTreeSet<_>>();
        let isolated_hier_segments = isolated_hier_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .collect::<BTreeSet<_>>();
        let isolated_local_segments = isolated_local_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .collect::<BTreeSet<_>>();
        let dangling_labels = child_schema
            .labels
            .iter()
            .filter(|label| matches!(label.label_type.as_str(), "hierarchical_label" | "label"))
            .filter(|label| !looks_like_bus_name(&label.text))
            .filter(|label| {
                !isolated_hier_points.contains(&label.point)
                    && !isolated_local_points.contains(&label.point)
            })
            .filter(|label| is_dangling_logical_label(label, &child_schema, &child_logical_nets))
            .cloned()
            .collect::<Vec<_>>();
        let dangling_segments = dangling_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .filter(|_| sheet.uses_prefixed_bus_alias_pins())
            .collect::<BTreeSet<_>>();

        out.entry(child_sheet_path.clone()).or_default().extend(
            child_schema
                .labels
                .iter()
                .filter(|label| label.label_type == "hierarchical_label")
                .filter(|label| !sheet.pins.contains(&label.text))
                .filter_map(|label| {
                    let severity = project_rule_severity(
                        project_rule_severities,
                        "hier_label_mismatch",
                        Severity::Error,
                    )?;
                    Some(PendingViolation::single(
                        severity,
                        "hier_label_mismatch",
                        format!(
                            "Hierarchical label {} has no matching sheet pin in the parent sheet",
                            label.text
                        ),
                        point_item(format!("Hierarchical Label '{}'", label.raw_text), label.point),
                    ))
                }),
        );

        let mut root_violations = Vec::new();

        if !sheet.uses_prefixed_bus_alias_pins() || !parent_has_multiple_bus_sheets {
            root_violations.extend(isolated_hier_labels.iter().filter(|label| {
                !sheet.pins.contains(&label.text)
            }).map(|label| {
                PendingViolation::single(
                    Severity::Warning,
                    "isolated_pin_label",
                    "Label connected to only one pin",
                    label_item(label),
                )
            }));
        }
        if current_sheet_path == "/" && (!sheet.uses_prefixed_bus_alias_pins() || !parent_has_multiple_bus_sheets) {
            root_violations.extend(isolated_local_labels.iter().filter_map(|label| {
                if label_is_hierarchical_bus_member_stub(
                    label,
                    &child_schema,
                    parent_has_multiple_bus_sheets,
                ) || label_net_exports_to_parent(label)
                    || child_has_prefixed_descendants
                    || (child_has_prefixed_descendants
                        && label_is_bus_member_stub(label, &child_schema))
                {
                    return None;
                }

                Some(PendingViolation::single(
                    Severity::Warning,
                    "isolated_pin_label",
                    "Label connected to only one pin",
                    label_item(label),
                ))
            }));
            root_violations.extend(isolated_global_labels.iter().filter(|label| {
                !parent_schema.labels.iter().any(|parent_label| {
                    parent_label.label_type == "global_label"
                        && !is_generated_power_label(parent_label, &parent_schema)
                        && parent_label.text == label.text
                })
            }).map(|label| {
                PendingViolation::single(
                    Severity::Warning,
                    "isolated_pin_label",
                    "Label connected to only one pin",
                    label_item(label),
                )
            }));
        }

        root_violations.extend(dangling_labels.iter().filter(|label| {
            !logical_label_net_exports_to_parent(
                label,
                &child_schema,
                &child_logical_nets,
                &sheet,
                allow_bus_member_export,
            )
                && !child_logical_nets.iter().any(|net| {
                    net.labels.iter().any(|other| {
                        other.point == label.point
                            && other.label_type == label.label_type
                            && other.text == label.text
                    }) && net.nodes.is_empty()
                        && !net.labels.is_empty()
                        && net.labels.len() > 1
                        && net
                            .labels
                            .iter()
                            .all(|other| other.label_type == "hierarchical_label")
                        && net
                            .labels
                            .iter()
                            .all(|other| sheet.pins.contains(&other.text))
                })
        }).map(|label| {
            PendingViolation::single(
                Severity::Error,
                "label_dangling",
                "Label not connected",
                label_item(label),
            )
        }));
        root_violations.extend(sheet.pin_refs.iter().filter_map(|pin| {
            is_dangling_sheet_pin(pin.point, &pin.name, &parent_schema).then(|| {
                PendingViolation::single(
                    Severity::Error,
                    "pin_not_connected",
                    "Pin not connected",
                    point_item(format!("Hierarchical Sheet Pin '{}'", pin.name), pin.point),
                )
            })
        }));

        root_violations.extend(
            child_schema
                .wires
                .iter()
                .filter_map(|segment| {
                    if isolated_hier_segments.contains(&segment_key(segment))
                        || isolated_local_segments.contains(&segment_key(segment))
                        || dangling_segments.contains(&segment_key(segment))
                    {
                        return None;
                    }
                    let dangling_endpoints =
                        dangling_segment_endpoint_count(segment, &child_schema, &[]);
                    if dangling_endpoints == 0 {
                        return None;
                    }
                    let (x_mm, y_mm) = segment_anchor_mm(segment);
                    Some(
                        (0..dangling_endpoints)
                            .map(|_| {
                                PendingViolation::single(
                                    Severity::Warning,
                                    "unconnected_wire_endpoint",
                                    "Unconnected wire endpoint",
                                    segment_item(segment, x_mm, y_mm),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .flatten(),
        );
        if repeated_files.get(&sheet.file).copied().unwrap_or(0) > 1 {
            root_violations.retain(|violation| violation.violation_type != "isolated_pin_label");
        }
        extend_unique_violations(
            root_screen_violations
                .entry(child_path.to_string_lossy().into_owned())
                .or_default(),
            root_violations,
        );

        collect_hierarchical_sheet_violations(
            &child_path,
            &child_sheet_path,
            Some(&sheet.instance_path),
            project_rule_severities,
            root_screen_violations,
            out,
        )?;
    }

    Ok(())
}

fn collect_descendant_sheet_violations(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &RuleSeverityMap,
    global_power_drivers: &BTreeSet<String>,
    root_screen_violations: &mut ViolationMap,
    child_screen_violations: &mut ChildScreenViolationMap,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };
    let parent_schema = parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;

    let sheet_refs = sheet_refs(schematic_path, current_instance_path)?;
    let projected_pin_conflicts = repeated_child_pin_to_pin_violations(
        root_dir,
        current_sheet_path,
        &parent_schema,
        &sheet_refs,
        project_rule_severities,
    )?;
    let repeated_files = sheet_refs
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut counts, sheet| {
            *counts.entry(sheet.file.clone()).or_default() += 1;
            counts
        });
    let parent_has_multiple_bus_sheets = repeated_files.keys().count() > 1
        && sheet_refs
            .iter()
            .filter(|sheet| sheet.pins.iter().any(|pin| looks_like_bus_name(pin)))
            .count()
            > 1;
    let parent_helper_power_labels = parent_schema
        .pin_nodes
        .iter()
        .filter(|pin| pin.pin_type.as_deref() == Some("power_in"))
        .filter(|pin| crate::cmd::schematic::erc::is_helper_power_symbol(pin))
        .filter_map(|pin| pin.pin_function.clone())
        .collect::<BTreeSet<_>>();
    let mut grouped_global_power_violations =
        BTreeMap::<String, (String, PendingViolation)>::new();
    let mut repeated_child_multiple_net_name_violations =
        BTreeMap::<(String, String, Vec<String>), (String, PendingViolation)>::new();

    for sheet in sheet_refs {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }
        let child_sheet_path = if current_sheet_path == "/" {
            sheet.path.clone()
        } else {
            format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
        };

        let mut child_schema = parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
            .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
        apply_sheet_text_vars(&mut child_schema, &sheet);
        let project_prefixed_descendant_net_not_bus_member =
            should_project_prefixed_bus_alias_descendant_conflicts(
                &child_path,
                &parent_schema,
                &sheet,
            );
        let child_symbol_libs = load_project_symbol_libraries(&child_path);
        let child_footprint_libs = load_project_footprint_libraries(&child_path);
        let child_nets = resolve_nets(&child_schema);
        let child_physical_groups = resolve_physical_groups(&child_schema);
        let child_netclass_assignments = load_project_netclass_assignments(&child_path);
        let child_parameterized_netclasses = load_project_parameterized_netclasses(&child_path);
        let child_violations = power_pin_not_driven_violations_with_global_drivers(
            &child_schema,
            &child_nets,
            global_power_drivers,
        )
            .into_iter()
            .chain(pin_not_connected_violations(
                &child_schema,
                &child_physical_groups,
                &child_nets,
                &child_netclass_assignments,
                &child_parameterized_netclasses,
            ))
            .chain(multiple_net_names_violations(
                &child_schema,
                &child_nets,
                project_rule_severities,
            ))
            .chain(child_nets.iter().flat_map(|net| {
                let pins = net.nodes.iter().collect::<Vec<_>>();

                reduced_pin_conflicts(&pins)
                    .into_iter()
                    .filter_map(|(first, second, level)| {
                        let (primary, other) = order_pin_conflict_items(first, second);
                        let (description_first, description_second) =
                            order_pin_conflict_description(first, second);

                        let severity = project_rule_severity(
                            project_rule_severities,
                            "pin_to_pin",
                            match level {
                                PinConflictLevel::Warning => Severity::Warning,
                                PinConflictLevel::Error => Severity::Error,
                            },
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
                                point_item(format_pin_item_description(primary), primary.point),
                                point_item(format_pin_item_description(other), other.point),
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            }))
            .collect::<Vec<_>>();
        let mut direct_child_violations = Vec::new();
        for violation in child_violations {
            if let Some(label) = helper_power_input_label(&violation) {
                if parent_helper_power_labels.contains(&label) {
                    continue;
                }
                grouped_global_power_violations
                    .entry(label)
                    .and_modify(|(_, grouped)| *grouped = violation.clone())
                    .or_insert_with(|| (child_sheet_path.clone(), violation));
            } else if violation.violation_type == "pin_not_connected"
                && helper_power_symbol_label(&violation)
                    .is_some_and(|label| global_power_drivers.contains(&label))
            {
                continue;
            } else if repeated_files.get(&sheet.file).copied().unwrap_or(0) > 1
                && is_repeated_hierarchical_multiple_net_names(&violation)
            {
                let key = (
                    sheet.file.clone(),
                    violation.violation_type.clone(),
                    violation
                        .items
                        .iter()
                        .map(|item| item.description.clone())
                        .collect::<Vec<_>>(),
                );
                repeated_child_multiple_net_name_violations
                    .entry(key)
                    .and_modify(|entry| *entry = (child_sheet_path.clone(), violation.clone()))
                    .or_insert_with(|| (child_sheet_path.clone(), violation));
            } else {
                direct_child_violations.push(violation);
            }
        }
        child_screen_violations.insert(
            child_sheet_path.clone(),
            (child_sheet_path.clone(), direct_child_violations),
        );

        let child_connection_grid_mm = load_project_connection_grid_mm(&child_path);
        let mut root_violations = if repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1 {
            endpoint_off_grid_violations(&child_schema, child_connection_grid_mm)
        } else {
            Vec::new()
        };
        if repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1
            && current_sheet_path == "/"
            && !parent_has_multiple_bus_sheets
            && (!sheet.uses_prefixed_bus_alias_pins()
                || project_prefixed_descendant_net_not_bus_member)
        {
            root_violations.extend(
                net_not_bus_member_violations(&child_schema, project_rule_severities)
                    .into_iter()
                    .map(|violation| {
                        rewrite_descendant_net_not_bus_member_violation(
                            violation,
                            &child_sheet_path,
                            &parent_schema,
                            &sheet,
                        )
                    }),
            );
        }
        if repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1
            && current_sheet_path == "/"
            && !parent_has_multiple_bus_sheets
            && sheet.uses_prefixed_bus_alias_pins()
            && child_schema
                .labels
                .iter()
                .filter(|label| label.label_type == "label")
                .all(|label| !looks_like_bus_name(&label.text))
        {
            let existing_root_net_not_bus_member_descriptions = root_violations
                .iter()
                .filter(|violation| violation.violation_type == "net_not_bus_member")
                .map(|violation| violation.description.clone())
                .collect::<BTreeSet<_>>();
            let synthetic = synthetic_leaf_prefixed_bus_member_conflicts(
                &child_schema,
                &child_sheet_path,
                &parent_schema,
                &sheet,
                project_rule_severities,
            );
            root_violations.extend(synthetic.into_iter().filter(|violation| {
                !existing_root_net_not_bus_member_descriptions.contains(&violation.description)
            }));
        }
        root_violations.extend(child_schema.symbols.iter().filter_map(|symbol| {
            build_lib_symbol_issue(symbol, &child_symbol_libs).map(|description| {
                PendingViolation::single(
                    Severity::Warning,
                    "lib_symbol_issues",
                    description,
                    point_item(format!("Symbol {} [{}]", symbol.reference, symbol.part.as_deref().unwrap_or("?")), symbol.at),
                )
            })
        }));
        root_violations.extend(child_schema.symbols.iter().flat_map(|symbol| {
            build_footprint_link_issues(
                symbol,
                &child_schema,
                &child_symbol_libs,
                &child_footprint_libs,
            )
            .into_iter()
            .map(|description| {
                PendingViolation::single(
                    Severity::Warning,
                    "footprint_link_issues",
                    description,
                    point_item(
                        format!(
                            "Symbol {} [{}]",
                            symbol.reference,
                            symbol.part.as_deref().unwrap_or("?")
                        ),
                        symbol.at,
                    ),
                )
            })
            .collect::<Vec<_>>()
        }));
        if current_sheet_path == "/" && parent_has_multiple_bus_sheets {
            root_violations.extend(
                lib_symbol_mismatch_violations(
                    &child_schema,
                    &child_symbol_libs,
                    project_rule_severities,
                )
                    .into_iter()
                    .filter(|violation| {
                        helper_power_symbol_label(violation).is_some_and(|label| {
                            grouped_global_power_violations.contains_key(&label)
                        })
                    }),
            );
        }
        if !root_violations.is_empty() {
            root_screen_violations.insert(child_path.to_string_lossy().into_owned(), root_violations);
        }

        collect_descendant_sheet_violations(
            &child_path,
            &child_sheet_path,
            Some(&sheet.instance_path),
            symbol_libs,
            project_rule_severities,
            global_power_drivers,
            root_screen_violations,
            child_screen_violations,
        )?;
    }

    for (_label, (sheet_path, violation)) in grouped_global_power_violations {
        child_screen_violations
            .entry(sheet_path.clone())
            .or_insert_with(|| (sheet_path.clone(), Vec::new()))
            .1
            .push(violation);
    }

    for (_key, (sheet_path, violation)) in repeated_child_multiple_net_name_violations {
        child_screen_violations
            .entry(sheet_path.clone())
            .or_insert_with(|| (sheet_path.clone(), Vec::new()))
            .1
            .push(violation);
    }

    for (path, violations) in projected_pin_conflicts {
        child_screen_violations
            .entry(path.clone())
            .or_insert_with(|| (path, Vec::new()))
            .1
            .extend(violations);
    }

    Ok(())
}
