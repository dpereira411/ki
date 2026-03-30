use std::fs;
use std::path::Path;
use std::collections::BTreeMap;

use serde::Serialize;

use crate::cmd::schematic::erc::{PendingItem, PendingViolation, SeverityFilter, Units};
use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

const COMMON_IGNORED_CHECKS: [(&str, &str); 4] = [
    (
        "single_global_label",
        "Global label only appears once in the schematic",
    ),
    (
        "four_way_junction",
        "Four connection points are joined together",
    ),
    ("footprint_link_issues", "Footprint link issue"),
    (
        "footprint_filter",
        "Assigned footprint doesn't match footprint filters",
    ),
];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcCheck {
    pub(crate) description: &'static str,
    pub(crate) key: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcPos {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcItem {
    pub(crate) description: String,
    pub(crate) pos: ErcPos,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcViolation {
    pub(crate) description: String,
    pub(crate) items: Vec<ErcItem>,
    pub(crate) severity: &'static str,
    #[serde(rename = "type")]
    pub(crate) violation_type: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcSheet {
    pub(crate) path: String,
    pub(crate) violations: Vec<ErcViolation>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ErcReport {
    pub(crate) schema_version: u32,
    pub(crate) coordinate_units: String,
    pub(crate) ignored_checks: Vec<ErcCheck>,
    pub(crate) included_severities: Vec<&'static str>,
    pub(crate) sheets: Vec<ErcSheet>,
    pub(crate) source: String,
}

pub(crate) type ViolationMap = BTreeMap<String, Vec<PendingViolation>>;

pub(crate) fn filter_and_sort_violations(
    violations: &mut Vec<PendingViolation>,
    severity_filter: SeverityFilter,
) {
    violations.retain(|violation| severity_filter.includes(violation.severity));
    sort_violations(violations);
}

pub(crate) fn filter_and_sort_violation_map(
    violations_by_sheet: &mut ViolationMap,
    severity_filter: SeverityFilter,
) {
    for violations in violations_by_sheet.values_mut() {
        filter_and_sort_violations(violations, severity_filter);
    }
}

pub(crate) fn sort_violations(violations: &mut [PendingViolation]) {
    violations.sort_by(|a, b| {
        a.violation_type
            .cmp(&b.violation_type)
            .then_with(|| a.description.cmp(&b.description))
            .then_with(|| a.items[0].description.cmp(&b.items[0].description))
    });
}

pub(crate) fn build_report(
    input: &Path,
    source_path: &str,
    units: Units,
    severity_filter: SeverityFilter,
    pending: &[PendingViolation],
    child_sheet_violations: &std::collections::BTreeMap<String, Vec<PendingViolation>>,
    child_sheet_paths: Vec<String>,
) -> ErcReport {
    let mut sheets = vec![ErcSheet {
        path: "/".to_string(),
        violations: pending.iter().map(|violation| report_violation(violation, units)).collect(),
    }];

    for path in child_sheet_paths {
        let violations = child_sheet_violations.get(&path).cloned().unwrap_or_default();
        sheets.push(ErcSheet {
            path,
            violations: violations
                .iter()
                .map(|violation| report_violation(violation, units))
                .collect(),
        });
    }

    ErcReport {
        schema_version: SCHEMA_VERSION,
        coordinate_units: units.label().to_string(),
        ignored_checks: COMMON_IGNORED_CHECKS
            .iter()
            .map(|(key, description)| ErcCheck { key, description })
            .collect(),
        included_severities: severity_filter.included_severities(),
        sheets,
        source: input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(source_path)
            .to_string(),
    }
}

pub(crate) fn write_report(
    report: &ErcReport,
    output_path: Option<&str>,
    flags: &Flags,
) -> Result<(), KiError> {
    match flags.format {
        OutputFormat::Json => {
            let rendered = serde_json::to_string_pretty(report)?;
            if let Some(path) = output_path {
                fs::write(path, rendered)?;
            } else {
                output::print_json(report)?;
            }
        }
        OutputFormat::Text => {
            let rendered = render_text_report(report);
            if let Some(path) = output_path {
                fs::write(path, rendered)?;
            } else {
                print!("{rendered}");
            }
        }
    }

    Ok(())
}

fn report_violation(violation: &PendingViolation, units: Units) -> ErcViolation {
    ErcViolation {
        description: violation.description.clone(),
        items: violation
            .items
            .iter()
            .map(|item| report_item(item, units))
            .collect(),
        severity: violation.severity.as_str(),
        violation_type: violation.violation_type.clone(),
    }
}

fn report_item(item: &PendingItem, units: Units) -> ErcItem {
    ErcItem {
        description: item.description.clone(),
        pos: ErcPos {
            x: units.json_pos(item.x_mm),
            y: units.json_pos(item.y_mm),
        },
    }
}

fn render_text_report(report: &ErcReport) -> String {
    let mut out = String::new();
    out.push_str("ERC report\n");
    out.push_str(&format!(
        "Report includes: {}\n\n",
        report
            .included_severities
            .iter()
            .map(|severity| match *severity {
                "error" => "Errors",
                "warning" => "Warnings",
                "exclusion" => "Exclusions",
                other => other,
            })
            .collect::<Vec<_>>()
            .join(", ")
    ));

    for sheet in &report.sheets {
        out.push_str(&format!("***** Sheet {}\n", sheet.path));
        for violation in &sheet.violations {
            out.push_str(&format!(
                "[{}]: {}\n",
                violation.violation_type, violation.description
            ));
            out.push_str(&format!("    ; {}\n", violation.severity));
            for item in &violation.items {
                out.push_str(&format!(
                    "    @({:.2} {}, {:.2} {}): {}\n",
                    item.pos.x * 100.0,
                    report.coordinate_units,
                    item.pos.y * 100.0,
                    report.coordinate_units,
                    item.description
                ));
            }
        }
    }

    out
}
