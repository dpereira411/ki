use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::cmd::schematic::erc::Severity;

pub(crate) type RuleSeverityMap = HashMap<String, String>;

pub(crate) fn load_project_netclasses(schematic_path: &Path) -> HashSet<String> {
    let Some(dir) = schematic_path.parent() else {
        return HashSet::new();
    };
    let Some(stem) = schematic_path.file_stem().and_then(|stem| stem.to_str()) else {
        return HashSet::new();
    };
    let project_path = dir.join(format!("{stem}.kicad_pro"));
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

pub(crate) fn load_project_rule_severities(schematic_path: &Path) -> RuleSeverityMap {
    let Some(dir) = schematic_path.parent() else {
        return HashMap::new();
    };
    let Some(stem) = schematic_path.file_stem().and_then(|stem| stem.to_str()) else {
        return HashMap::new();
    };
    let project_path = dir.join(format!("{stem}.kicad_pro"));
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
