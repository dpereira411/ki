mod connectivity;
mod format;
mod geom;
mod hierarchy;
mod items;
mod pin_conflict;
mod project;
mod report;
mod rules;
mod sexpr;
mod text;

use std::path::Path;

use self::hierarchy::{
    child_sheet_paths, descendant_global_label_texts, load_project_footprint_libraries,
    load_project_symbol_libraries, sheet_refs,
    sheet_pin_points,
};
use self::project::{
    load_project_connection_grid_mm, load_project_netclass_assignments, load_project_netclasses,
    load_project_parameterized_netclasses, load_project_rule_severities,
};
use self::report::{
    build_report, filter_and_sort_violation_map, filter_and_sort_violations, sort_violations,
    write_report,
};
use self::rules::hierarchy::{
    descendant_sheet_violations, hierarchical_sheet_violations, merge_pending_maps,
    same_local_global_label_hierarchy_violations,
};
use self::rules::root::collect_root_violations;
use crate::error::KiError;
use crate::output::Flags;
use crate::schematic::render::{
    parse_schema, resolve_nets, resolve_physical_groups, LabelInfo, ParsedSchema, PinNode, Point,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Severity {
    Error,
    Warning,
    Exclusion,
}

impl Severity {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Exclusion => "exclusion",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Units {
    Mm,
    In,
    Mils,
}

impl Units {
    fn parse(value: &str) -> Result<Self, KiError> {
        match value {
            "mm" => Ok(Self::Mm),
            "in" => Ok(Self::In),
            "mils" => Ok(Self::Mils),
            _ => Err(KiError::Message("Invalid units specified".to_string())),
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Mm => "mm",
            Self::In => "in",
            Self::Mils => "mils",
        }
    }

    fn from_mm(self, value: f64) -> f64 {
        match self {
            Self::Mm => value,
            Self::In => value / 25.4,
            Self::Mils => value / 0.0254,
        }
    }

    pub(crate) fn json_pos(self, value_mm: f64) -> f64 {
        let value = self.from_mm(value_mm) / 100.0;
        round_half_even(value, 5)
    }
}

fn round_half_even(value: f64, decimals: usize) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    let scaled = value * factor;
    let lower = scaled.floor();
    let upper = scaled.ceil();
    let diff = scaled - lower;
    let halfway = (diff - 0.5).abs() < 1e-9;

    let rounded = if diff < 0.5 && !halfway {
        lower
    } else if diff > 0.5 && !halfway {
        upper
    } else if (lower as i64) % 2 == 0 {
        lower
    } else {
        upper
    };

    rounded / factor
}

fn bus_group_suffix(name: &str) -> Option<&str> {
    let trimmed = name.trim_start_matches('/');
    let start = trimmed.find('{')?;
    trimmed.get(start..)
}

fn has_prefixed_bus_group(name: &str) -> bool {
    name.split_once('{')
        .is_some_and(|(prefix, rest)| !prefix.is_empty() && rest.contains('}'))
}

fn net_not_bus_member_violation_bus_name(description: &str) -> Option<&str> {
    let prefix = "Net ";
    let infix = " is graphically connected to bus ";
    let suffix = " but is not a member of that bus";
    let rest = description.strip_prefix(prefix)?;
    let (_, rest) = rest.split_once(infix)?;
    rest.strip_suffix(suffix)
}

fn net_not_bus_member_violation_net_name(description: &str) -> Option<&str> {
    let prefix = "Net ";
    let infix = " is graphically connected to bus ";
    let rest = description.strip_prefix(prefix)?;
    let (net_name, _) = rest.split_once(infix)?;
    Some(net_name)
}

fn suppress_intermediate_prefixed_bus_alias_root_violations(
    input: &Path,
    pending: &mut Vec<PendingViolation>,
) {
    let direct_child_bus_suffixes = match sheet_refs(input, None) {
        Ok(refs) => refs
            .iter()
            .filter(|sheet| sheet.uses_prefixed_bus_alias_pins())
            .flat_map(|sheet| sheet.pins.iter())
            .filter(|pin| has_prefixed_bus_group(pin))
            .filter_map(|pin| bus_group_suffix(pin).map(str::to_string))
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    if direct_child_bus_suffixes.is_empty() {
        return;
    }

    let suppressed_bus_items = pending
        .iter()
        .filter(|violation| violation.violation_type == "net_not_bus_member")
        .filter(|violation| {
            let is_direct_child_surface = net_not_bus_member_violation_net_name(&violation.description)
                .is_some_and(|net_name| net_name.trim_start_matches('/').split('/').count() == 1);
            is_direct_child_surface
                && net_not_bus_member_violation_bus_name(&violation.description)
                    .and_then(bus_group_suffix)
                    .is_some_and(|suffix| direct_child_bus_suffixes.iter().any(|entry| entry == suffix))
        })
        .filter_map(|violation| violation.items.get(1))
        .map(|item| (item.description.clone(), item.x_mm.to_bits(), item.y_mm.to_bits()))
        .collect::<Vec<_>>();

    pending.retain(|violation| {
        if violation.violation_type == "net_not_bus_member" {
            let is_direct_child_surface = net_not_bus_member_violation_net_name(&violation.description)
                .is_some_and(|net_name| net_name.trim_start_matches('/').split('/').count() == 1);
            return !is_direct_child_surface
                || net_not_bus_member_violation_bus_name(&violation.description)
                    .and_then(bus_group_suffix)
                    .is_none_or(|suffix| !direct_child_bus_suffixes.iter().any(|entry| entry == suffix));
        }

        if violation.violation_type == "bus_to_net_conflict" {
            return !violation.items.iter().any(|item| {
                suppressed_bus_items.iter().any(|suppressed| {
                    suppressed.0 == item.description
                        && suppressed.1 == item.x_mm.to_bits()
                        && suppressed.2 == item.y_mm.to_bits()
                })
            });
        }

        true
    });
}

fn collapse_duplicate_root_net_not_bus_member_violations(
    pending: &mut Vec<PendingViolation>,
) {
    let mut chosen = std::collections::BTreeMap::<
        (String, String, String),
        PendingViolation,
    >::new();

    for violation in pending
        .iter()
        .filter(|violation| violation.violation_type == "net_not_bus_member")
    {
        let key = (
            violation.severity.as_str().to_string(),
            violation.violation_type.clone(),
            violation.description.clone(),
        );

        chosen
            .entry(key)
            .and_modify(|existing| {
                let candidate_items = violation
                    .items
                    .iter()
                    .map(|item| {
                        (
                            item.description.clone(),
                            item.x_mm.to_bits(),
                            item.y_mm.to_bits(),
                        )
                    })
                    .collect::<Vec<_>>();
                let existing_items = existing
                    .items
                    .iter()
                    .map(|item| {
                        (
                            item.description.clone(),
                            item.x_mm.to_bits(),
                            item.y_mm.to_bits(),
                        )
                    })
                    .collect::<Vec<_>>();

                if candidate_items < existing_items {
                    *existing = violation.clone();
                }
            })
            .or_insert_with(|| violation.clone());
    }

    pending.retain(|violation| {
        if violation.violation_type != "net_not_bus_member" {
            return true;
        }

        let key = (
            violation.severity.as_str().to_string(),
            violation.violation_type.clone(),
            violation.description.clone(),
        );

        chosen.get(&key).is_some_and(|selected| {
            selected.items.len() == violation.items.len()
                && selected
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct SeverityFilter {
    error: bool,
    warning: bool,
    exclusion: bool,
}

impl SeverityFilter {
    fn from_flags(
        severity_all: bool,
        severity_error: bool,
        severity_warning: bool,
        severity_exclusions: bool,
    ) -> Self {
        if severity_all {
            return Self {
                error: true,
                warning: true,
                exclusion: true,
            };
        }

        if severity_error || severity_warning || severity_exclusions {
            return Self {
                error: severity_error,
                warning: severity_warning,
                exclusion: severity_exclusions,
            };
        }

        Self {
            error: true,
            warning: true,
            exclusion: false,
        }
    }

    fn includes(self, severity: Severity) -> bool {
        match severity {
            Severity::Error => self.error,
            Severity::Warning => self.warning,
            Severity::Exclusion => self.exclusion,
        }
    }

    pub(crate) fn included_severities(self) -> Vec<&'static str> {
        let mut severities = Vec::new();
        if self.error {
            severities.push(Severity::Error.as_str());
        }
        if self.warning {
            severities.push(Severity::Warning.as_str());
        }
        if self.exclusion {
            severities.push(Severity::Exclusion.as_str());
        }
        severities
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PendingViolation {
    pub(crate) severity: Severity,
    pub(crate) description: String,
    pub(crate) violation_type: String,
    pub(crate) items: Vec<PendingItem>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingItem {
    pub(crate) description: String,
    pub(crate) x_mm: f64,
    pub(crate) y_mm: f64,
}

impl PendingViolation {
    fn new(
        severity: Severity,
        violation_type: impl Into<String>,
        description: impl Into<String>,
        items: Vec<PendingItem>,
    ) -> Self {
        Self {
            severity,
            description: description.into(),
            violation_type: violation_type.into(),
            items,
        }
    }

    fn single(
        severity: Severity,
        violation_type: impl Into<String>,
        description: impl Into<String>,
        item: PendingItem,
    ) -> Self {
        Self::new(severity, violation_type, description, vec![item])
    }
}

impl PendingItem {
    fn new(description: impl Into<String>, x_mm: f64, y_mm: f64) -> Self {
        Self {
            description: description.into(),
            x_mm,
            y_mm,
        }
    }

    fn from_point(description: impl Into<String>, point: Point) -> Self {
        Self::new(
            description,
            point.x as f64 / 10_000.0,
            point.y as f64 / 10_000.0,
        )
    }
}

pub fn run_erc(
    path: &str,
    output_path: Option<&str>,
    units: &str,
    severity_all: bool,
    severity_error: bool,
    severity_warning: bool,
    severity_exclusions: bool,
    exit_code_violations: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let input = Path::new(path);
    if !input.exists() {
        return Err(KiError::Message(
            "Schematic file does not exist or is not accessible".to_string(),
        ));
    }

    let units = Units::parse(units)?;
    let severity_filter = SeverityFilter::from_flags(
        severity_all,
        severity_error,
        severity_warning,
        severity_exclusions,
    );

    let schema = parse_schema(path, None)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let nets = resolve_nets(&schema);
    let physical_groups = resolve_physical_groups(&schema);
    let resolved_symbol_libs = load_project_symbol_libraries(input);
    let available_footprint_libs = load_project_footprint_libraries(input);
    let defined_netclasses = load_project_netclasses(input);
    let project_netclass_assignments = load_project_netclass_assignments(input);
    let parameterized_netclasses = load_project_parameterized_netclasses(input);
    let project_rule_severities = load_project_rule_severities(input);
    let connection_grid_mm = load_project_connection_grid_mm(input);
    let descendant_global_labels = descendant_global_label_texts(input)?;
    let mut root_attached_points = sheet_pin_points(input)?;
    root_attached_points.extend(schema.bus_entries.iter().map(|entry| entry.wire_point));
    let mut child_sheet_violations =
        hierarchical_sheet_violations(input, &project_rule_severities)?;
    merge_pending_maps(
        &mut child_sheet_violations,
        descendant_sheet_violations(input, &resolved_symbol_libs, &project_rule_severities)?,
    );
    merge_pending_maps(
        &mut child_sheet_violations,
        same_local_global_label_hierarchy_violations(input, &project_rule_severities)?,
    );

    let mut pending = collect_root_violations(
        input,
        &schema,
        &nets,
        &physical_groups,
        &resolved_symbol_libs,
        &available_footprint_libs,
        &defined_netclasses,
        &project_netclass_assignments,
        &parameterized_netclasses,
        &project_rule_severities,
        connection_grid_mm,
        &root_attached_points,
        &descendant_global_labels,
    )?;
    suppress_intermediate_prefixed_bus_alias_root_violations(input, &mut pending);

    filter_and_sort_violations(&mut pending, severity_filter);
    filter_and_sort_violation_map(&mut child_sheet_violations, severity_filter);

    if let Some(root_child_violations) = child_sheet_violations.remove("/") {
        pending.extend(root_child_violations);
        suppress_intermediate_prefixed_bus_alias_root_violations(input, &mut pending);
        collapse_duplicate_root_net_not_bus_member_violations(&mut pending);
        sort_violations(&mut pending);
    }

    let report = build_report(
        input,
        path,
        units,
        severity_filter,
        &pending,
        &child_sheet_violations,
        child_sheet_paths(input)?,
    );

    write_report(&report, output_path, flags)?;

    if exit_code_violations && !pending.is_empty() {
        return Err(KiError::Validation);
    }

    Ok(())
}
pub(super) fn is_helper_power_symbol(node: &PinNode) -> bool {
    node.reference.starts_with("#PWR")
}

pub(super) fn is_generated_power_label(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    schema
        .pin_nodes
        .iter()
        .any(|pin| {
            let pin_name = pin.pin_function.as_deref().unwrap_or("");
            is_helper_power_symbol(pin)
                && pin.point == label.point
                && (pin_name.is_empty()
                    || pin_name == "~"
                    || pin_name == label.text
                    || pin_name == label.raw_text)
        })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_defaults_match_kicad_cli_shape() {
        let filter = SeverityFilter::from_flags(false, false, false, false);
        assert_eq!(filter.included_severities(), vec!["error", "warning"]);
    }

    #[test]
    fn parses_supported_units() {
        assert!(matches!(Units::parse("mm"), Ok(Units::Mm)));
        assert!(matches!(Units::parse("in"), Ok(Units::In)));
        assert!(matches!(Units::parse("mils"), Ok(Units::Mils)));
    }
}
