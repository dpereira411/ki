mod common;

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

use common::{copy_fixture, fixture};

#[test]
fn schematic_inspect_json() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "schematic",
        "inspect",
        fixture("sample.kicad_sch").to_str().unwrap(),
        "--json",
    ]);
    cmd.assert().success();
}

#[test]
fn pcb_inspect_json() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "pcb",
        "inspect",
        fixture("sample.kicad_pcb").to_str().unwrap(),
        "--json",
    ]);
    cmd.assert().success();
}

#[test]
fn lib_table_inspect_json() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "lib-table",
        "inspect",
        fixture("sym-lib-table").to_str().unwrap(),
        "--json",
    ]);
    cmd.assert().success();
}

#[test]
fn schematic_set_property_mutates_file() {
    let temp = TempDir::new().expect("tempdir should exist");
    let schematic = copy_fixture(&temp, "sample.kicad_sch");

    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "schematic",
        "set-property",
        schematic.to_str().unwrap(),
        "R1",
        "Value",
        "22k",
        "--json",
    ]);
    cmd.assert().success();

    let updated = fs::read_to_string(&schematic).expect("updated schematic should be readable");
    assert!(updated.contains("22k"));
}

#[test]
fn missing_file_exits_two() {
    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args(["schematic", "inspect", "does-not-exist.kicad_sch"]);
    cmd.assert().code(2);
}

#[test]
fn lib_table_inspect_detects_symbol_table_by_contents() {
    let temp = TempDir::new().expect("tempdir should exist");
    let renamed = temp.path().join("renamed.table");
    fs::copy(fixture("sym-lib-table"), &renamed).expect("fixture should copy");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["lib-table", "inspect", renamed.to_str().unwrap(), "--json"])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(json["table_type"], "symbol");
}

#[test]
fn hierarchical_query_fails_when_subsheet_is_missing() {
    let temp = TempDir::new().expect("tempdir should exist");
    let root = copy_fixture(&temp, "hierarchical.kicad_sch");

    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "schematic",
        "query",
        "net",
        root.to_str().unwrap(),
        "VCC",
        "--hierarchical",
    ]);
    cmd.assert().code(2);
}
