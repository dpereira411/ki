use kiutils_rs::SchematicFile;
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(serde::Deserialize)]
struct IntentFile {
    #[allow(dead_code)]
    version: Option<u32>,
    #[serde(default)]
    nets: Vec<NetIntent>,
    #[serde(default)]
    values: Vec<ValueIntent>,
    #[serde(default)]
    footprints: Vec<FootprintIntent>,
    #[serde(default)]
    properties: Vec<PropertyIntent>,
}

#[derive(serde::Deserialize)]
struct NetIntent {
    name: String,
}

#[derive(serde::Deserialize)]
struct ValueIntent {
    reference: String,
    expected: String,
}

#[derive(serde::Deserialize)]
struct FootprintIntent {
    reference: String,
    expected: String,
}

#[derive(serde::Deserialize)]
struct PropertyIntent {
    reference: String,
    key: String,
    expected: String,
}

#[derive(Serialize)]
struct Violation {
    rule: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    net: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual: Option<String>,
    message: String,
}

#[derive(Serialize)]
struct IntentResponse {
    schema_version: u32,
    ok: bool,
    violation_count: usize,
    violations: Vec<Violation>,
}

pub fn check_intent(path: &str, intent_path: &str, flags: &Flags) -> Result<(), KiError> {
    let raw = std::fs::read_to_string(intent_path)
        .map_err(|e| KiError::Message(format!("cannot read intent file {intent_path:?}: {e}")))?;
    let intent: IntentFile = serde_json::from_str(&raw)
        .map_err(|e| KiError::Message(format!("invalid intent JSON in {intent_path:?}: {e}")))?;

    let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let netlist = doc.netlist();
    let instances = doc.symbol_instances();

    let mut violations: Vec<Violation> = Vec::new();

    for rule in &intent.nets {
        let found = netlist
            .nets
            .iter()
            .any(|n| n.name.as_deref() == Some(&rule.name));
        if !found {
            violations.push(Violation {
                rule: "net",
                net: Some(rule.name.clone()),
                reference: None,
                key: None,
                expected: None,
                actual: None,
                message: format!("net {:?} not found in schematic", rule.name),
            });
        }
    }

    for rule in &intent.values {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(Violation {
                rule: "value",
                reference: Some(rule.reference.clone()),
                net: None,
                key: None,
                expected: Some(rule.expected.clone()),
                actual: None,
                message: format!("component {:?} not found", rule.reference),
            }),
            Some(sym) => {
                let actual = sym.value.as_deref().unwrap_or("");
                if actual != rule.expected {
                    violations.push(Violation {
                        rule: "value",
                        reference: Some(rule.reference.clone()),
                        net: None,
                        key: None,
                        expected: Some(rule.expected.clone()),
                        actual: Some(actual.to_string()),
                        message: format!(
                            "{} Value is {:?}, expected {:?}",
                            rule.reference, actual, rule.expected
                        ),
                    });
                }
            }
        }
    }

    for rule in &intent.footprints {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(Violation {
                rule: "footprint",
                reference: Some(rule.reference.clone()),
                net: None,
                key: None,
                expected: Some(rule.expected.clone()),
                actual: None,
                message: format!("component {:?} not found", rule.reference),
            }),
            Some(sym) => {
                let actual = sym.footprint.as_deref().unwrap_or("");
                if actual != rule.expected {
                    violations.push(Violation {
                        rule: "footprint",
                        reference: Some(rule.reference.clone()),
                        net: None,
                        key: None,
                        expected: Some(rule.expected.clone()),
                        actual: Some(actual.to_string()),
                        message: format!(
                            "{} Footprint is {:?}, expected {:?}",
                            rule.reference, actual, rule.expected
                        ),
                    });
                }
            }
        }
    }

    for rule in &intent.properties {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(Violation {
                rule: "property",
                reference: Some(rule.reference.clone()),
                net: None,
                key: Some(rule.key.clone()),
                expected: Some(rule.expected.clone()),
                actual: None,
                message: format!("component {:?} not found", rule.reference),
            }),
            Some(sym) => {
                let actual = sym
                    .properties
                    .iter()
                    .find(|(k, _)| k == &rule.key)
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                if actual != rule.expected {
                    violations.push(Violation {
                        rule: "property",
                        reference: Some(rule.reference.clone()),
                        net: None,
                        key: Some(rule.key.clone()),
                        expected: Some(rule.expected.clone()),
                        actual: Some(actual.to_string()),
                        message: format!(
                            "{}.{} is {:?}, expected {:?}",
                            rule.reference, rule.key, actual, rule.expected
                        ),
                    });
                }
            }
        }
    }

    let ok = violations.is_empty();

    if flags.format == OutputFormat::Json {
        output::print_json(&IntentResponse {
            schema_version: SCHEMA_VERSION,
            ok,
            violation_count: violations.len(),
            violations,
        })?;
    } else if ok {
        println!("ok: all intent rules pass");
    } else {
        println!("FAIL: {} violation(s)", violations.len());
        for v in &violations {
            println!("  [{}] {}", v.rule, v.message);
        }
    }

    if !ok {
        return Err(KiError::Validation);
    }
    Ok(())
}
