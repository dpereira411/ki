#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use roxmltree::Document;
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

fn kicad_cli() -> Option<&'static str> {
    let candidate = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli";
    Path::new(candidate).exists().then_some(candidate)
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

            OracleLibPart {
                lib: lib_part["lib"].as_str().unwrap_or_default().to_string(),
                part: lib_part["part"].as_str().unwrap_or_default().to_string(),
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
                        ref_: node["component_ref"].as_str().unwrap_or_default().to_string(),
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
