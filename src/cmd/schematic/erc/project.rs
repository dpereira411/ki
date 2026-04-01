use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::cmd::schematic::erc::Severity;

pub(crate) type RuleSeverityMap = HashMap<String, String>;
pub(crate) type NetclassAssignmentMap = HashMap<String, Vec<String>>;

const DEFAULT_CONNECTION_GRID_MM: f64 = 1.27;

fn direct_project_path(schematic_path: &Path) -> Option<PathBuf> {
    let dir = schematic_path.parent()?;
    let stem = schematic_path.file_stem().and_then(|stem| stem.to_str())?;
    let direct = dir.join(format!("{stem}.kicad_pro"));
    direct.exists().then_some(direct)
}

fn resolve_assignment_project_path(schematic_path: &Path) -> Option<PathBuf> {
    if let Some(project_path) = direct_project_path(schematic_path) {
        return Some(project_path);
    }

    let dir = schematic_path.parent()?;
    let mut candidates = fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("kicad_pro"))
        .collect::<Vec<_>>();
    candidates.sort();

    if candidates.len() == 1 {
        return Some(candidates.remove(0));
    }

    let child_name = schematic_path.file_name()?.to_str()?;
    let mut referencing_projects = candidates
        .into_iter()
        .filter(|project_path| {
            let Some(project_stem) = project_path.file_stem().and_then(|stem| stem.to_str()) else {
                return false;
            };
            let schematic_candidate = dir.join(format!("{project_stem}.kicad_sch"));
            let Ok(raw) = fs::read_to_string(schematic_candidate) else {
                return false;
            };

            raw.contains(&format!("(file \"{child_name}\")"))
                || raw.contains(&format!("(property \"Sheetfile\" \"{child_name}\""))
        })
        .collect::<Vec<_>>();
    referencing_projects.sort();
    (referencing_projects.len() == 1).then(|| referencing_projects.remove(0))
}

pub(crate) fn load_project_netclasses(schematic_path: &Path) -> HashSet<String> {
    let Some(project_path) = direct_project_path(schematic_path) else {
        return HashSet::new();
    };
    let Ok(raw) = fs::read_to_string(project_path) else {
        return HashSet::from([String::from("Default")]);
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return HashSet::from([String::from("Default")]);
    };

    let mut classes = json["net_settings"]["classes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["name"].as_str().map(ToOwned::to_owned))
        .collect::<HashSet<_>>();

    if classes.is_empty() {
        classes.insert("Default".to_string());
    }

    classes
}

pub(crate) fn load_project_netclass_assignments(
    schematic_path: &Path,
) -> NetclassAssignmentMap {
    let Some(project_path) = resolve_assignment_project_path(schematic_path) else {
        return HashMap::new();
    };
    let Ok(raw) = fs::read_to_string(&project_path) else {
        return HashMap::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return HashMap::new();
    };

    let mut assignments = json["net_settings"]["netclass_assignments"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| {
            let assignments = if let Some(value) = value.as_str() {
                vec![value.to_string()]
            } else {
                value
                    .as_array()?
                    .iter()
                    .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
                    .collect::<Vec<_>>()
            };
            Some((key.clone(), assignments))
        })
        .collect::<NetclassAssignmentMap>();

    let schematic_stem = schematic_path.file_stem().and_then(|stem| stem.to_str());
    let project_stem = project_path.file_stem().and_then(|stem| stem.to_str());

    if schematic_stem != project_stem {
        let alias_entries = assignments
            .iter()
            .filter_map(|(key, value)| key.rsplit('/').next().map(|leaf| (leaf.to_string(), value.clone())))
            .collect::<Vec<_>>();
        for (alias, value) in alias_entries {
            assignments.entry(alias).or_insert(value);
        }
    }

    assignments
}

pub(crate) fn load_project_parameterized_netclasses(
    schematic_path: &Path,
) -> HashSet<String> {
    let Some(project_path) = direct_project_path(schematic_path) else {
        return HashSet::new();
    };
    let Ok(raw) = fs::read_to_string(project_path) else {
        return HashSet::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return HashSet::new();
    };

    json["net_settings"]["classes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry["name"].as_str()?.to_string();
            let has_explicit_params = [
                "bus_width",
                "clearance",
                "diff_pair_gap",
                "diff_pair_via_gap",
                "diff_pair_width",
                "line_style",
                "microvia_diameter",
                "microvia_drill",
                "track_width",
                "via_diameter",
                "via_drill",
                "wire_width",
            ]
            .iter()
            .any(|key| entry.get(*key).is_some());
            has_explicit_params.then_some(name)
        })
        .collect()
}

pub(crate) fn load_project_rule_severities(schematic_path: &Path) -> RuleSeverityMap {
    let Some(project_path) = direct_project_path(schematic_path) else {
        return HashMap::new();
    };
    let Ok(raw) = fs::read_to_string(project_path) else {
        return HashMap::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return HashMap::new();
    };

    json["erc"]["rule_severities"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
        .collect()
}

pub(crate) fn load_project_connection_grid_mm(schematic_path: &Path) -> f64 {
    let Some(project_path) = direct_project_path(schematic_path) else {
        return DEFAULT_CONNECTION_GRID_MM;
    };
    let Ok(raw) = fs::read_to_string(project_path) else {
        return DEFAULT_CONNECTION_GRID_MM;
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return DEFAULT_CONNECTION_GRID_MM;
    };

    json["schematic"]["connection_grid_size"]
        .as_f64()
        .map(|mils| mils * 0.0254)
        .unwrap_or(DEFAULT_CONNECTION_GRID_MM)
}

pub(crate) fn project_rule_severity(
    severities: &RuleSeverityMap,
    rule: &str,
    default: Severity,
) -> Option<Severity> {
    match severities.get(rule).map(String::as_str) {
        Some("error") => Some(Severity::Error),
        Some("warning") => Some(Severity::Warning),
        Some("exclude") | Some("exclusion") => Some(Severity::Exclusion),
        Some("ignore") => None,
        None => Some(default),
        Some(_) => Some(default),
    }
}
