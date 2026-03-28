mod common;

use assert_cmd::Command;
use tempfile::TempDir;

use common::copy_project_fixture_set;

#[test]
fn project_open_stays_successful_when_tables_have_diagnostics() {
    let temp = TempDir::new().expect("tempdir should exist");
    let project = copy_project_fixture_set(&temp);
    let sym_table = temp.path().join("sym-lib-table");

    let mut mutate = Command::cargo_bin("ki").expect("binary should build");
    mutate.args([
        "lib-table",
        "add",
        sym_table.to_str().unwrap(),
        "A",
        "${KIPRJMOD}/duplicate.kicad_sym",
    ]);
    mutate.assert().success();

    let mut open = Command::cargo_bin("ki").expect("binary should build");
    open.args(["project", "open", project.to_str().unwrap()]);
    open.assert().success();
}
