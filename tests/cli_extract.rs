mod common;

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

use common::{compare_against_kicad_cli, extract_fixture, extract_test_fixture, fixture};

#[test]
fn extract_help_succeeds() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args(["extract", "--help"]);
    cmd.assert().success();
}

#[test]
fn extract_schematic_outputs_canonical_json() {
    let path = extract_test_fixture("resistor_gnd.kicad_sch");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", path.to_str().unwrap()])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["source"]["project"], "resistor_gnd");
    assert!(json["components"]
        .as_array()
        .is_some_and(|arr| !arr.is_empty()));
    assert!(json["lib_parts"]
        .as_array()
        .is_some_and(|arr| !arr.is_empty()));
    assert!(json.get("nets").is_none());
}

#[test]
fn extract_autoloads_project_sym_lib_table() {
    let path = extract_test_fixture("resistor_gnd.kicad_sch");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", path.to_str().unwrap()])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");

    let resistor = json["components"]
        .as_array()
        .expect("components should be array")
        .iter()
        .find(|component| component["ref"] == "R1")
        .expect("R1 should exist");
    assert_eq!(
        resistor["footprint"].as_str(),
        Some("Resistor_SMD:R_0603_1608Metric")
    );
    assert_eq!(
        resistor["datasheet"].as_str(),
        Some("https://example.com/resistor.pdf")
    );
}

#[test]
fn extract_schematic_outputs_json_with_optional_sections_to_file() {
    let path = extract_test_fixture("resistor_gnd.kicad_sch");

    let temp = TempDir::new().expect("tempdir should exist");
    let out = temp.path().join("extract.json");

    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "extract",
        path.to_str().unwrap(),
        "--include-nets",
        "--include-diagnostics",
        "--output",
        out.to_str().unwrap(),
    ]);
    cmd.assert().success();

    let json: Value =
        serde_json::from_str(&fs::read_to_string(out).expect("output should be readable"))
            .expect("output file should contain json");
    assert_eq!(json["schema_version"], 2);
    assert!(json["lib_parts"]
        .as_array()
        .is_some_and(|arr| !arr.is_empty()));
    assert!(json["components"]
        .as_array()
        .is_some_and(|arr| !arr.is_empty()));
    assert!(json["nets"].as_array().is_some_and(|arr| !arr.is_empty()));
    assert!(json["diagnostics"].as_array().is_some());
}

#[test]
fn extract_rejects_non_schematic_input() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args(["extract", "not-a-schematic.xml"]);
    cmd.assert().code(2);
}

#[test]
fn extract_verbose_logs_autoloaded_libs() {
    let path = extract_test_fixture("resistor_gnd.kicad_sch");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", path.to_str().unwrap(), "--verbose"])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("autoloading sym-lib"));
    assert!(stderr.contains("Device.kicad_sym"));
}

#[test]
fn extract_matches_kicad_cli_for_resistor_gnd_fixture() {
    compare_against_kicad_cli(&extract_test_fixture("resistor_gnd.kicad_sch"));
}

#[test]
fn extract_matches_kicad_cli_for_hierarchical_fixture() {
    compare_against_kicad_cli(&fixture("hierarchical.kicad_sch"));
}

#[test]
fn extract_matches_kicad_cli_for_configured_fixture() {
    let Some(path) = extract_fixture() else {
        return;
    };
    compare_against_kicad_cli(&path);
}
