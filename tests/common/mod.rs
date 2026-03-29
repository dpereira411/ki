#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::TempDir;

pub fn kiutils_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kiutils-rs/crates/kiutils_kicad/tests/fixtures")
}

pub fn fixture(path: &str) -> PathBuf {
    kiutils_fixture_root().join(path)
}

pub fn copy_fixture(temp: &TempDir, name: &str) -> PathBuf {
    let src = fixture(name);
    let dst = temp.path().join(name);
    fs::copy(src, &dst).expect("fixture should copy");
    dst
}

pub fn copy_project_fixture_set(temp: &TempDir) -> PathBuf {
    for name in [
        "sample.kicad_pro",
        "sample.kicad_sch",
        "sample.kicad_pcb",
        "fp-lib-table",
        "sym-lib-table",
    ] {
        copy_fixture(temp, name);
    }
    temp.path().join("sample.kicad_pro")
}

pub fn extract_fixture() -> Option<PathBuf> {
    std::env::var_os("KI_EXTRACT_TEST_SCH")
        .map(PathBuf::from)
        .filter(|candidate| candidate.exists())
}

pub fn extract_test_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("extract")
        .join(path)
}

pub fn extract_parity_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("extract_parity")
        .join(path)
}

pub fn erc_parity_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("erc_parity")
        .join(path)
}

pub fn upstream_erc_fixture(path: &str) -> PathBuf {
    Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema").join(path)
}

pub fn upstream_erc_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("erc_upstream_qa")
        .join("manifest.json")
}

pub fn upstream_erc_project(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("erc_upstream_qa")
        .join("projects")
        .join(path)
}

pub fn upstream_erc_oracle(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("erc_upstream_qa")
        .join("oracles")
        .join(path)
}

fn kicad_cli() -> Option<&'static str> {
    let candidate = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli";
    Path::new(candidate).exists().then_some(candidate)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticOracle {
    pub exit_code: i32,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErcOracle {
    pub exit_code: i32,
    pub report: NormalizedErcReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedErcReport {
    pub coordinate_units: String,
    pub included_severities: Vec<String>,
    pub ignored_checks: Vec<NormalizedIgnoredCheck>,
    pub sheets: Vec<NormalizedErcSheet>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NormalizedIgnoredCheck {
    pub key: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NormalizedErcSheet {
    pub path: String,
    pub violations: Vec<NormalizedErcViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NormalizedErcViolation {
    pub violation_type: String,
    pub severity: String,
    pub description: String,
    pub items: Vec<NormalizedErcItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NormalizedErcItem {
    pub description: String,
    pub x: String,
    pub y: String,
}

pub fn normalize_output_messages(output: &[u8], error: &[u8]) -> Vec<String> {
    let mut messages = String::from_utf8_lossy(output)
        .lines()
        .map(str::trim)
        .map(strip_kicad_log_prefix)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.contains("Failed to remove lock file"))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    messages.extend(
        String::from_utf8_lossy(error)
            .lines()
            .map(str::trim)
            .map(strip_kicad_log_prefix)
            .filter(|line| !line.is_empty())
            .filter(|line| !line.contains("Failed to remove lock file"))
            .map(ToOwned::to_owned),
    );
    messages
}

fn strip_kicad_log_prefix(line: &str) -> &str {
    line.find("Warning:")
        .or_else(|| line.find("Error:"))
        .map(|idx| &line[idx..])
        .unwrap_or(line)
}

pub fn kicad_cli_extract_diagnostics(schematic: &Path) -> Option<DiagnosticOracle> {
    let kicad_cli = kicad_cli()?;
    let temp = TempDir::new().expect("tempdir should exist");
    let xml_path = temp.path().join("oracle.xml");

    let output = StdCommand::new(kicad_cli)
        .args([
            "sch",
            "export",
            "netlist",
            "--format",
            "kicadxml",
            "-o",
            xml_path.to_str().unwrap(),
            schematic.to_str().unwrap(),
        ])
        .output()
        .expect("kicad-cli should run");

    Some(DiagnosticOracle {
        exit_code: output.status.code().unwrap_or(1),
        messages: normalize_output_messages(&output.stdout, &output.stderr),
    })
}

pub fn ki_extract_diagnostics(schematic: &Path) -> DiagnosticOracle {
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args([
            "extract",
            schematic.to_str().unwrap(),
            "--include-diagnostics",
        ])
        .output()
        .expect("native extract should run");

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid json");
    let messages = json["diagnostics"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|diag| diag["message"].as_str().map(ToOwned::to_owned))
        .collect();

    DiagnosticOracle {
        exit_code: output.status.code().unwrap_or(1),
        messages,
    }
}

pub fn kicad_cli_extract_raw(schematic: &Path) -> Option<DiagnosticOracle> {
    let kicad_cli = kicad_cli()?;
    let temp = TempDir::new().expect("tempdir should exist");
    let xml_path = temp.path().join("oracle.xml");

    let output = StdCommand::new(kicad_cli)
        .args([
            "sch",
            "export",
            "netlist",
            "--format",
            "kicadxml",
            "-o",
            xml_path.to_str().unwrap(),
            schematic.to_str().unwrap(),
        ])
        .output()
        .expect("kicad-cli should run");

    Some(DiagnosticOracle {
        exit_code: output.status.code().unwrap_or(1),
        messages: normalize_output_messages(&output.stdout, &output.stderr),
    })
}

pub fn kicad_cli_extract_raw_no_output(schematic: &Path) -> Option<DiagnosticOracle> {
    let kicad_cli = kicad_cli()?;

    let output = StdCommand::new(kicad_cli)
        .args([
            "sch",
            "export",
            "netlist",
            "--format",
            "kicadxml",
            schematic.to_str().unwrap(),
        ])
        .output()
        .expect("kicad-cli should run");

    Some(DiagnosticOracle {
        exit_code: output.status.code().unwrap_or(1),
        messages: normalize_output_messages(&output.stdout, &output.stderr),
    })
}

pub fn ki_extract_raw(schematic: &Path) -> DiagnosticOracle {
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", schematic.to_str().unwrap()])
        .output()
        .expect("native extract should run");

    let mut messages = normalize_output_messages(&output.stdout, &output.stderr);

    if output.status.code().unwrap_or(1) != 0 && messages.is_empty() {
        messages.push("Failed to load schematic".to_string());
    }

    DiagnosticOracle {
        exit_code: output.status.code().unwrap_or(1),
        messages,
    }
}

pub fn kicad_cli_erc_json(
    schematic: &Path,
    extra_args: &[&str],
    output_path: Option<&Path>,
) -> Option<ErcOracle> {
    let kicad_cli = kicad_cli()?;
    let temp = TempDir::new().expect("tempdir should exist");
    let oracle_path = temp.path().join("oracle.json");
    let target_path = output_path.unwrap_or(&oracle_path);

    let mut args = vec!["sch", "erc", "--format=json"];
    args.extend(extra_args.iter().copied());
    args.push("-o");
    args.push(target_path.to_str().unwrap());
    args.push(schematic.to_str().unwrap());

    let output = StdCommand::new(kicad_cli)
        .args(args)
        .output()
        .expect("kicad-cli should run");

    let raw: Value = serde_json::from_slice(&fs::read(target_path).ok()?).ok()?;

    Some(ErcOracle {
        exit_code: output.status.code().unwrap_or(1),
        report: normalize_erc_report(&raw),
    })
}

pub fn ki_erc_json(schematic: &Path, extra_args: &[&str]) -> ErcOracle {
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["schematic", "erc", schematic.to_str().unwrap(), "--json"])
        .args(extra_args)
        .output()
        .expect("native erc should run");

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid json");

    ErcOracle {
        exit_code: output.status.code().unwrap_or(1),
        report: normalize_erc_report(&json),
    }
}

pub fn normalize_erc_report(raw: &Value) -> NormalizedErcReport {
    let mut sheets = raw["sheets"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|sheet| {
            let mut violations = sheet["violations"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|violation| {
                    let mut items = violation["items"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|item| NormalizedErcItem {
                            description: normalize_erc_item_description(
                                item["description"].as_str().unwrap_or_default(),
                            ),
                            x: format_decimal(item["pos"]["x"].as_f64()),
                            y: format_decimal(item["pos"]["y"].as_f64()),
                        })
                        .collect::<Vec<_>>();
                    items.sort_by(|a, b| {
                        a.description
                            .cmp(&b.description)
                            .then_with(|| a.x.cmp(&b.x))
                            .then_with(|| a.y.cmp(&b.y))
                    });
                    NormalizedErcViolation {
                        violation_type: violation["type"].as_str().unwrap_or_default().to_string(),
                        severity: violation["severity"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        description: violation["description"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        items,
                    }
                })
                .collect::<Vec<_>>();
            violations.sort_by(|a, b| {
                a.violation_type
                    .cmp(&b.violation_type)
                    .then_with(|| a.description.cmp(&b.description))
                    .then_with(|| a.items.cmp(&b.items))
            });
            NormalizedErcSheet {
                path: sheet["path"].as_str().unwrap_or_default().to_string(),
                violations,
            }
        })
        .collect::<Vec<_>>();
    sheets.sort_by(|a, b| a.path.cmp(&b.path));

    let mut included_severities = raw["included_severities"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|severity| severity.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    included_severities.sort();

    NormalizedErcReport {
        coordinate_units: raw["coordinate_units"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        included_severities,
        ignored_checks: Vec::new(),
        sheets,
        source: raw["source"].as_str().unwrap_or_default().to_string(),
    }
}

fn normalize_erc_item_description(description: &str) -> String {
    description.replace("[~, ", "[")
}

fn format_decimal(value: Option<f64>) -> String {
    value
        .map(|number| {
            let negative = number.is_sign_negative();
            let rendered = format!("{:.8}", number.abs());
            let (whole, frac) = rendered.split_once('.').unwrap_or((&rendered, ""));
            let mut frac_digits = frac.chars().collect::<Vec<_>>();
            while frac_digits.len() < 5 {
                frac_digits.push('0');
            }

            let mut keep = frac_digits[..4]
                .iter()
                .map(|ch| ch.to_digit(10).unwrap_or(0))
                .collect::<Vec<_>>();
            let next = frac_digits[4].to_digit(10).unwrap_or(0);
            let rest_nonzero = frac_digits[5..]
                .iter()
                .any(|ch| ch.to_digit(10).unwrap_or(0) != 0);

            let round_up = next > 5
                || (next == 5
                    && (rest_nonzero || keep.last().copied().unwrap_or(0) % 2 == 1));

            let mut whole_value = whole.parse::<u64>().unwrap_or(0);

            if round_up {
                let mut carry = 1;
                for digit in keep.iter_mut().rev() {
                    let updated = *digit + carry;
                    *digit = updated % 10;
                    carry = updated / 10;
                    if carry == 0 {
                        break;
                    }
                }
                if carry > 0 {
                    whole_value += carry as u64;
                }
            }

            let mut rendered = format!(
                "{}{}.{}",
                if negative { "-" } else { "" },
                whole_value,
                keep.iter()
                    .map(|digit| char::from_digit(*digit, 10).unwrap())
                    .collect::<String>()
            );
            while rendered.contains('.') && rendered.ends_with('0') {
                rendered.pop();
            }
            if rendered.ends_with('.') {
                rendered.pop();
            }
            rendered
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OracleComponent {
    ref_: String,
    value: String,
    lib: String,
    part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OracleLibPin {
    num: String,
    name: String,
    pin_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OracleLibPart {
    lib: String,
    part: String,
    description: String,
    pins: Vec<OracleLibPin>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OracleNode {
    ref_: String,
    pin: String,
    pin_function: String,
    pin_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OracleNet {
    name: String,
    nodes: Vec<OracleNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OracleDoc {
    components: Vec<OracleComponent>,
    lib_parts: Vec<OracleLibPart>,
    nets: Vec<OracleNet>,
}

fn normalize_pin_function(pin_function: &str, pin: &str) -> String {
    pin_function
        .strip_suffix(&format!("_{pin}"))
        .unwrap_or(pin_function)
        .to_string()
}

pub fn compare_against_kicad_cli(schematic: &Path) {
    let Some(kicad_cli) = kicad_cli() else {
        return;
    };

    let temp = TempDir::new().expect("tempdir should exist");
    let xml_path = temp.path().join("oracle.xml");

    let status = StdCommand::new(kicad_cli)
        .args([
            "sch",
            "export",
            "netlist",
            "--format",
            "kicadxml",
            "-o",
            xml_path.to_str().unwrap(),
            schematic.to_str().unwrap(),
        ])
        .status()
        .expect("kicad-cli should run");
    assert!(
        status.success(),
        "kicad-cli export failed for {:?}",
        schematic
    );

    let native = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", schematic.to_str().unwrap(), "--include-nets"])
        .output()
        .expect("native extract should run");
    assert!(
        native.status.success(),
        "native extract failed for {:?}",
        schematic
    );

    let native_json: Value =
        serde_json::from_slice(&native.stdout).expect("native full output should be valid json");
    let xml_text = fs::read_to_string(xml_path).expect("oracle xml should be readable");
    let oracle_xml = Document::parse(&xml_text).expect("oracle xml should parse");

    assert_eq!(
        normalize_native_doc(&native_json),
        normalize_kicad_cli_doc(&oracle_xml),
        "native extract should match kicad-cli netlist for {:?}",
        schematic
    );
}

fn normalize_native_doc(doc: &Value) -> OracleDoc {
    let mut components = doc["components"]
        .as_array()
        .expect("components should be array")
        .iter()
        .map(|component| OracleComponent {
            ref_: component["ref"].as_str().unwrap_or_default().to_string(),
            value: component["value"].as_str().unwrap_or_default().to_string(),
            lib: component["lib_part_id"]
                .as_str()
                .and_then(|id| id.split_once(':').map(|(lib, _)| lib.to_string()))
                .unwrap_or_default(),
            part: component["lib_part_id"]
                .as_str()
                .and_then(|id| id.split_once(':').map(|(_, part)| part.to_string()))
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    components.sort();

    let mut lib_parts = doc["lib_parts"]
        .as_array()
        .expect("lib_parts should be array")
        .iter()
        .map(|lib_part| {
            let mut pins = lib_part["pins"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|pin| OracleLibPin {
                    num: pin["num"].as_str().unwrap_or_default().to_string(),
                    name: pin["name"].as_str().unwrap_or_default().to_string(),
                    pin_type: pin["electrical_kind"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect::<Vec<_>>();
            pins.sort();
            pins.dedup();

            let (lib, part) = lib_part["id"]
                .as_str()
                .and_then(|id| id.split_once(':'))
                .map(|(lib, part)| (lib.to_string(), part.to_string()))
                .unwrap_or_default();

            OracleLibPart {
                lib,
                part,
                description: lib_part["description"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                pins,
            }
        })
        .collect::<Vec<_>>();
    lib_parts.sort();

    let mut nets = doc["nets"]
        .as_array()
        .expect("nets should be array")
        .iter()
        .map(|net| {
            let mut nodes = net["nodes"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|node| {
                    let pin = node["pin_num"].as_str().unwrap_or_default().to_string();
                    OracleNode {
                        ref_: node["component_ref"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        pin_function: normalize_pin_function(
                            node["pin_name"].as_str().unwrap_or_default(),
                            &pin,
                        ),
                        pin,
                        pin_type: node["pin_electrical_kind"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    }
                })
                .collect::<Vec<_>>();
            nodes.sort();
            OracleNet {
                name: net["name"].as_str().unwrap_or_default().to_string(),
                nodes,
            }
        })
        .collect::<Vec<_>>();
    nets.sort();

    OracleDoc {
        components,
        lib_parts,
        nets,
    }
}

fn normalize_kicad_cli_doc(doc: &Document<'_>) -> OracleDoc {
    let mut components = doc
        .descendants()
        .filter(|node| node.has_tag_name("comp"))
        .map(|component| OracleComponent {
            ref_: component.attribute("ref").unwrap_or_default().to_string(),
            value: component
                .children()
                .find(|child| child.has_tag_name("value"))
                .and_then(|value| value.text())
                .unwrap_or_default()
                .to_string(),
            lib: component
                .children()
                .find(|child| child.has_tag_name("libsource"))
                .and_then(|node| node.attribute("lib"))
                .unwrap_or_default()
                .to_string(),
            part: component
                .children()
                .find(|child| child.has_tag_name("libsource"))
                .and_then(|node| node.attribute("part"))
                .unwrap_or_default()
                .to_string(),
        })
        .collect::<Vec<_>>();
    components.sort();

    let mut lib_parts = doc
        .descendants()
        .filter(|node| node.has_tag_name("libpart"))
        .map(|lib_part| {
            let mut pins = lib_part
                .children()
                .find(|child| child.has_tag_name("pins"))
                .into_iter()
                .flat_map(|pins| pins.children().filter(|child| child.has_tag_name("pin")))
                .map(|pin| OracleLibPin {
                    num: pin.attribute("num").unwrap_or_default().to_string(),
                    name: pin.attribute("name").unwrap_or_default().to_string(),
                    pin_type: pin.attribute("type").unwrap_or_default().to_string(),
                })
                .collect::<Vec<_>>();
            pins.sort();
            pins.dedup();

            OracleLibPart {
                lib: lib_part.attribute("lib").unwrap_or_default().to_string(),
                part: lib_part.attribute("part").unwrap_or_default().to_string(),
                description: lib_part
                    .children()
                    .find(|child| child.has_tag_name("description"))
                    .and_then(|description| description.text())
                    .unwrap_or_default()
                    .to_string(),
                pins,
            }
        })
        .collect::<Vec<_>>();
    lib_parts.sort();

    let mut nets = doc
        .descendants()
        .filter(|node| node.has_tag_name("net"))
        .map(|net| {
            let mut nodes = net
                .children()
                .filter(|child| child.has_tag_name("node"))
                .map(|node| {
                    let pin = node.attribute("pin").unwrap_or_default().to_string();
                    OracleNode {
                        ref_: node.attribute("ref").unwrap_or_default().to_string(),
                        pin_function: normalize_pin_function(
                            node.attribute("pinfunction").unwrap_or_default(),
                            &pin,
                        ),
                        pin,
                        pin_type: node.attribute("pintype").unwrap_or_default().to_string(),
                    }
                })
                .collect::<Vec<_>>();
            nodes.sort();
            OracleNet {
                name: net.attribute("name").unwrap_or_default().to_string(),
                nodes,
            }
        })
        .collect::<Vec<_>>();
    nets.sort();

    OracleDoc {
        components,
        lib_parts,
        nets,
    }
}
