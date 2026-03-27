use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

fn fixture(path: &str) -> PathBuf {
    Path::new("/Users/Daniel/Desktop/modular/tools/kiutils-rs")
        .join("crates/kiutils_kicad/tests/fixtures")
        .join(path)
}

fn copy_fixture(temp: &TempDir, name: &str) -> PathBuf {
    let src = fixture(name);
    let dst = temp.path().join(name);
    fs::copy(src, &dst).expect("fixture should copy");
    dst
}

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
