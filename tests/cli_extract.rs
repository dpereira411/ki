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
    assert!(resistor.get("footprint").is_none());
    assert!(resistor.get("datasheet").is_none());

    let lib_part = json["lib_parts"]
        .as_array()
        .expect("lib_parts should be array")
        .iter()
        .find(|part| part["id"] == "Device:R")
        .expect("Device:R lib part should exist");
    assert_eq!(
        lib_part["footprint_filters"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item.as_str()),
        Some("Resistor_SMD:R_0603_1608Metric")
    );
    assert_eq!(
        lib_part["datasheet"].as_str(),
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
fn extract_missing_file_matches_kicad_cli_message() {
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", "does-not-exist.kicad_sch"])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Schematic file does not exist or is not accessible"));
}

#[test]
fn extract_malformed_schematic_matches_kicad_cli_message() {
    let temp = TempDir::new().expect("tempdir should exist");
    let path = temp.path().join("bad.kicad_sch");
    fs::write(&path, "not a schematic\n").expect("fixture should write");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", path.to_str().unwrap()])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Failed to load schematic"));
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

#[test]
fn extract_separates_library_fields_from_instance_overrides() {
    let temp = TempDir::new().expect("tempdir should exist");
    let path = temp.path().join("overrides.kicad_sch");

    fs::write(
        &path,
        r#"(kicad_sch
  (version 20260101)
  (generator "eeschema")
  (uuid "override-root")
  (paper "A4")
  (lib_symbols
    (symbol "Device:R"
      (property "Reference" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (property "Value" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (property "Footprint" "" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
      (property "Datasheet" "https://lib.example/datasheet.pdf" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
      (property "Description" "Resistor" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
      (property "MPN" "LIB-MPN" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
      (symbol "R_1_1"
        (pin passive line (at 0 3.81 270) (length 1.27)
          (name "" (effects (font (size 1.27 1.27))))
          (number "1" (effects (font (size 1.27 1.27)))))
        (pin passive line (at 0 -3.81 90) (length 1.27)
          (name "" (effects (font (size 1.27 1.27))))
          (number "2" (effects (font (size 1.27 1.27))))))))
  (symbol (lib_id "Device:R") (at 50 50 0) (unit 1) (body_style 1) (uuid "sym-r1")
    (property "Reference" "R1" (at 0 0 0) (effects (font (size 1.27 1.27))))
    (property "Value" "10k" (at 0 0 0) (effects (font (size 1.27 1.27))))
    (property "Footprint" "" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "https://lib.example/datasheet.pdf" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Description" "Resistor" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "MPN" "OVERRIDE-MPN" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "LCSC" "C12345" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (pin "1" (uuid "r1-pin1"))
    (pin "2" (uuid "r1-pin2"))
    (instances (project "overrides" (path "/override-root" (reference "R1") (unit 1)))))
  (sheet_instances
    (path "/" (page "1"))))"#,
    )
    .expect("fixture should write");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["extract", path.to_str().unwrap()])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");

    let lib_part = json["lib_parts"]
        .as_array()
        .expect("lib_parts should be array")
        .iter()
        .find(|part| part["id"] == "Device:R")
        .expect("Device:R lib part should exist");
    let lib_fields = lib_part["fields"]
        .as_array()
        .expect("lib fields should be array");
    assert!(!lib_fields
        .iter()
        .any(|field| field["name"] == "Description"));
    assert!(!lib_fields.iter().any(|field| field["name"] == "Datasheet"));
    assert!(!lib_fields.iter().any(|field| field["name"] == "Value"));
    assert!(lib_fields
        .iter()
        .any(|field| field["name"] == "MPN" && field["value"] == "LIB-MPN"));

    let component = json["components"]
        .as_array()
        .expect("components should be array")
        .iter()
        .find(|component| component["ref"] == "R1")
        .expect("R1 should exist");
    let properties = component["properties"]
        .as_array()
        .expect("component properties should be array");

    assert!(!properties
        .iter()
        .any(|property| property["name"] == "Reference"));
    assert!(!properties
        .iter()
        .any(|property| property["name"] == "Description"));
    assert!(!properties
        .iter()
        .any(|property| property["name"] == "Datasheet"));
    assert!(properties
        .iter()
        .any(|property| property["name"] == "MPN" && property["value"] == "OVERRIDE-MPN"));
    assert!(properties
        .iter()
        .any(|property| property["name"] == "LCSC" && property["value"] == "C12345"));
    assert!(component.get("footprint").is_none());
    assert!(component.get("datasheet").is_none());
}
