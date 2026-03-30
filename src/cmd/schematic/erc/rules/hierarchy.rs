use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::extract::sym_lib::ProjectSymbolLibraryIndex;
use crate::schematic::render::{parse_schema, resolve_nets};

use super::super::connectivity::{
    connected_pin_like_count_for_label, connected_wire_segments,
    dangling_segment_endpoint_count, is_dangling_label, looks_like_bus_name, segment_key,
};
use super::super::geom::segment_anchor_mm;
use super::super::hierarchy::{apply_sheet_text_vars, sheet_refs};
use super::super::items::{label_item, point_item, segment_item};
use super::super::project::{project_rule_severity, RuleSeverityMap};
use super::super::report::ViolationMap;
use super::connectivity::{
    endpoint_off_grid_violations, multiple_net_names_violations, power_pin_not_driven_violations,
};
use super::symbol::{build_lib_symbol_issue, lib_symbol_mismatch_violations};
use super::super::{PendingViolation, Severity};
use crate::error::KiError;

type ChildScreenViolationMap = BTreeMap<String, (String, Vec<PendingViolation>)>;

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
    collect_descendant_sheet_violations(
        schematic_path,
        "/",
        None,
        symbol_libs,
        project_rule_severities,
        &mut root_screen_violations,
        &mut child_screen_violations,
    )?;
    out.entry("/".to_string())
        .or_default()
        .extend(root_screen_violations.into_values().flatten());
    for (sheet_path, violations) in child_screen_violations.into_values() {
        out.entry(sheet_path).or_default().extend(violations);
    }
    Ok(out)
}

pub(crate) fn merge_pending_maps(out: &mut ViolationMap, incoming: ViolationMap) {
    for (path, violations) in incoming {
        out.entry(path).or_default().extend(violations);
    }
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

    let sheet_refs = sheet_refs(schematic_path, current_instance_path)?;
    let repeated_files = sheet_refs
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut counts, sheet| {
            *counts.entry(sheet.file.clone()).or_default() += 1;
            counts
        });

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

        let isolated_hier_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "hierarchical_label")
            .filter(|label| connected_pin_like_count_for_label(label, &child_schema) == 1)
            .cloned()
            .collect::<Vec<_>>();
        let isolated_local_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter(|label| connected_pin_like_count_for_label(label, &child_schema) == 1)
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
            .filter(|label| {
                !isolated_hier_points.contains(&label.point)
                    && !isolated_local_points.contains(&label.point)
            })
            .filter(|label| is_dangling_label(label, &child_schema))
            .filter(|label| {
                !sheet.uses_prefixed_bus_alias_pins() || !looks_like_bus_name(&label.text)
            })
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

        if !sheet.uses_prefixed_bus_alias_pins() {
            root_violations.extend(isolated_hier_labels.iter().map(|label| {
                PendingViolation::single(
                    Severity::Warning,
                    "isolated_pin_label",
                    "Label connected to only one pin",
                    label_item(label),
                )
            }));
            root_violations.extend(isolated_local_labels.iter().map(|label| {
                PendingViolation::single(
                    Severity::Warning,
                    "isolated_pin_label",
                    "Label connected to only one pin",
                    label_item(label),
                )
            }));
        }

        root_violations.extend(dangling_labels.iter().map(|label| {
            PendingViolation::single(
                Severity::Error,
                "label_dangling",
                "Label not connected",
                label_item(label),
            )
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
        if repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1 {
            root_screen_violations.insert(child_path.to_string_lossy().into_owned(), root_violations);
        }

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
    root_screen_violations: &mut ViolationMap,
    child_screen_violations: &mut ChildScreenViolationMap,
) -> Result<(), KiError> {
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

        let mut child_schema = parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
            .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
        apply_sheet_text_vars(&mut child_schema, &sheet);
        let child_nets = resolve_nets(&child_schema);

        let child_violations = power_pin_not_driven_violations(&child_schema, &child_nets)
            .into_iter()
            .chain(multiple_net_names_violations(
                &child_schema,
                &child_nets,
                project_rule_severities,
            ))
            .collect::<Vec<_>>();
        child_screen_violations.insert(
            child_path.to_string_lossy().into_owned(),
            (child_sheet_path.clone(), child_violations),
        );

        let mut root_violations = endpoint_off_grid_violations(&child_schema);
        root_violations.extend(child_schema.symbols.iter().filter_map(|symbol| {
            build_lib_symbol_issue(symbol, symbol_libs).map(|item| {
                PendingViolation::single(
                    Severity::Warning,
                    "lib_symbol_issues",
                    format!(
                        "The current configuration does not include the symbol library '{}'",
                        symbol.lib.as_deref().unwrap_or_default()
                    ),
                    item,
                )
            })
        }));
        root_violations.extend(
            lib_symbol_mismatch_violations(&child_schema, symbol_libs, project_rule_severities)
                .into_iter()
                .filter(|violation| {
                    violation
                        .items
                        .iter()
                        .any(|item| item.description.starts_with("Symbol #PWR? ["))
                }),
        );
        root_screen_violations.insert(child_path.to_string_lossy().into_owned(), root_violations);

        collect_descendant_sheet_violations(
            &child_path,
            &child_sheet_path,
            Some(&sheet.instance_path),
            symbol_libs,
            project_rule_severities,
            root_screen_violations,
            child_screen_violations,
        )?;
    }

    Ok(())
}
