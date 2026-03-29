mod common;

use std::fs;

use assert_cmd::Command;
use kiutils_rs::SchematicFile;
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

#[test]
fn schematic_query_net_resolves_anonymous_driver_name_and_geometry() {
    let temp = TempDir::new().expect("tempdir should exist");
    let schematic = temp.path().join("anonymous_net.kicad_sch");
    fs::write(
        &schematic,
        concat!(
            "(kicad_sch (version 20260101) (generator \"test\") (uuid \"u1\") (paper \"A4\")\n",
            "  (lib_symbols\n",
            "    (symbol \"Conn:Test\"\n",
            "      (property \"Reference\" \"J\" (id 0))\n",
            "      (property \"Value\" \"Test\" (id 1))\n",
            "      (symbol \"Test_1_1\"\n",
            "        (pin input line (at 0 0 0) (length 2.54) (name \"SIG\") (number \"1\"))\n",
            "      )\n",
            "    )\n",
            "  )\n",
            "  (symbol (lib_id \"Conn:Test\") (at 10 10 0) (unit 1)\n",
            "    (property \"Reference\" \"J1\" (id 0))\n",
            "    (property \"Value\" \"Test\" (id 1))\n",
            "  )\n",
            "  (wire (pts (xy 10 10) (xy 20 10)) (uuid \"w1\"))\n",
            ")\n",
        ),
    )
    .expect("fixture should write");

    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args([
            "schematic",
            "query",
            "net",
            schematic.to_str().unwrap(),
            "Net-(J1-SIG)",
            "--json",
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(json["net"], "Net-(J1-SIG)");
    assert_eq!(json["pin_count"], 1);
    assert_eq!(json["pins"][0]["reference"], "J1");
    assert_eq!(json["pins"][0]["pin_number"], "1");
    assert_eq!(json["wires"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["wires"][0]["x1"], 10.0);
    assert_eq!(json["wires"][0]["y1"], 10.0);
    assert_eq!(json["wires"][0]["x2"], 20.0);
    assert_eq!(json["wires"][0]["y2"], 10.0);
    assert_eq!(json["placement"]["x"], 10.0);
    assert_eq!(json["placement"]["y"], 10.0);
    assert_eq!(json["placement"]["angle"], 0.0);
}

#[test]
fn update_from_lib_preserves_requested_instance_property_in_ki() {
    let temp = TempDir::new().expect("tempdir should exist");
    let schematic = temp.path().join("demo.kicad_sch");
    let sym_table = temp.path().join("sym-lib-table");
    let symbol_lib = temp.path().join("MyLib.kicad_sym");

    fs::write(
        &schematic,
        concat!(
            "(kicad_sch (version 20260101) (generator \"eeschema\") (uuid \"u1\")\n",
            "  (lib_symbols\n",
            "    (symbol \"MyLib:R\"\n",
            "      (property \"Reference\" \"R\" (at 0 0 0))\n",
            "      (property \"Value\" \"R_old\" (at 0 0 0))\n",
            "      (property \"MPN\" \"GENERIC\" (at 0 0 0))))\n",
            "  (symbol (lib_id \"MyLib:R\") (at 100 50 0) (unit 1) (uuid \"s1\")\n",
            "    (property \"Reference\" \"R1\" (at 100 55 0))\n",
            "    (property \"Value\" \"R_old\" (at 100 45 0))\n",
            "    (property \"MPN\" \"SPECIFIC-123\" (at 100 40 0))))\n",
        ),
    )
    .expect("fixture should write");

    fs::write(
        &sym_table,
        concat!(
            "(sym_lib_table (version 7)\n",
            "  (lib (name \"MyLib\") (type \"KiCad\") (uri \"${KIPRJMOD}/MyLib.kicad_sym\") (options \"\") (descr \"\")))\n",
        ),
    )
    .expect("sym-lib-table should write");

    fs::write(
        &symbol_lib,
        concat!(
            "(kicad_symbol_lib (version 20260101) (generator kicad_symbol_editor)\n",
            "  (symbol \"MyLib:R\"\n",
            "    (property \"Reference\" \"R\" (at 0 0 0))\n",
            "    (property \"Value\" \"R_new\" (at 0 0 0))\n",
            "    (property \"Datasheet\" \"new-datasheet\" (at 0 0 0))\n",
            "    (property \"MPN\" \"GENERIC\" (at 0 0 0))))\n",
        ),
    )
    .expect("symbol lib should write");

    let mut cmd = Command::cargo_bin("ki").expect("binary should build");
    cmd.args([
        "schematic",
        "update-from-lib",
        schematic.to_str().unwrap(),
        "MyLib",
        "R1",
        "--preserve-property",
        "MPN",
    ]);
    cmd.assert().success();

    let reread = SchematicFile::read(&schematic).expect("updated schematic should be readable");
    let instance = reread
        .symbol_instances()
        .into_iter()
        .find(|symbol| symbol.reference.as_deref() == Some("R1"))
        .expect("R1 should exist");

    assert!(instance
        .properties
        .iter()
        .any(|(key, value)| key == "MPN" && value == "SPECIFIC-123"));
    assert!(instance
        .properties
        .iter()
        .any(|(key, value)| key == "Datasheet" && value == "new-datasheet"));
}
