mod common;

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde::Deserialize;
use serde_json::Value;

use common::{
    erc_parity_fixture, ki_erc_json, kicad_cli_erc_json, normalize_erc_report,
    upstream_erc_fixture, upstream_erc_manifest, upstream_erc_oracle, upstream_erc_project,
    ErcOracle, NormalizedErcSheet,
};

#[derive(Debug, Deserialize)]
struct ErcParityCase {
    id: String,
    fixture_dir: Option<String>,
    oracle_path: Option<String>,
    ki_cli_args: Vec<String>,
    expected_exit_code: i32,
    #[serde(rename = "expected_messages")]
    _expected_messages: Vec<String>,
}

#[test]
fn schematic_erc_reports_missing_input_file_like_kicad_cli() {
    let missing = Path::new("tests/fixtures/erc_parity/does_not_exist.kicad_sch");
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args(["schematic", "erc", missing.to_str().unwrap(), "--json"])
        .output()
        .expect("native erc should run");

    assert_eq!(output.status.code().unwrap_or(1), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Schematic file does not exist or is not accessible"));
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_basic_test_errors_only() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "basic_test_errors_only")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("basic_test.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_missing_symbol_library_warning() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "missing_symbol_library_in_configuration")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("missing_symbol_library_in_configuration.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_resolves_local_symbol_library_and_reports_mismatch() {
    let schematic =
        Path::new("tests/fixtures/erc_parity/lib_symbol_mismatch/lib_symbol_mismatch.kicad_sch");
    let native = ki_erc_json(schematic, &[]);
    let violations = &native.report.sheets[0].violations;

    assert!(violations.iter().any(|violation| {
        violation.violation_type == "lib_symbol_mismatch"
            && violation.description == "Symbol 'Probe' doesn't match copy in library 'LocalLib'"
    }));
    assert!(!violations
        .iter()
        .any(|violation| violation.violation_type == "lib_symbol_issues"));
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_lib_symbol_mismatch_on_kicad_qa_fixture() {
    let schematic = upstream_erc_fixture("erc_multiple_pin_to_pin.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_documents_upstream_dynamic_power_symbol_line_marker_bug() {
    let schematic = upstream_erc_project("ERC_dynamic_power_symbol_test.kicad_sch");
    let oracle_path = upstream_erc_oracle("ERC_dynamic_power_symbol_test.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    let oracle_endpoint_off_grid = oracle_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "endpoint_off_grid")
        .count();
    let native_endpoint_off_grid = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "endpoint_off_grid")
        .count();

    assert_eq!(native_endpoint_off_grid, oracle_endpoint_off_grid + 1);
    assert!(native_root.violations.iter().any(|violation| {
        violation.violation_type == "endpoint_off_grid"
            && violation.items.iter().any(|item| {
                item.description == "Horizontal Wire, length 0.1143 mm"
                    && item.x == "0.6477"
                    && item.y == "0.5652"
            })
    }));

    let filtered_native_root = NormalizedErcSheet {
        path: native_root.path.clone(),
        violations: native_root
            .violations
            .iter()
            .filter(|violation| {
                !(violation.violation_type == "endpoint_off_grid"
                    && violation.items.iter().any(|item| {
                        item.description == "Horizontal Wire, length 0.1143 mm"
                            && item.x == "0.6477"
                            && item.y == "0.5652"
                    }))
            })
            .cloned()
            .collect(),
    };

    assert_eq!(&filtered_native_root, oracle_root);

    let other_native_sheets = native
        .report
        .sheets
        .iter()
        .filter(|sheet| sheet.path != "/")
        .cloned()
        .collect::<Vec<_>>();
    let other_oracle_sheets = oracle
        .report
        .sheets
        .iter()
        .filter(|sheet| sheet.path != "/")
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(other_native_sheets, other_oracle_sheets);
}

fn assert_exact_upstream_erc_match(schematic: &Path) {
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_selected_upstream_kicad_qa_projects_exactly() {
    assert!(upstream_erc_manifest().exists());

    for schematic in [
        upstream_erc_fixture("erc_multiple_pin_to_pin.kicad_sch"),
        upstream_erc_fixture("netlists/top_level_hier_pins/top_level_hier_pins.kicad_sch"),
        upstream_erc_fixture("issue18606/issue18606.kicad_sch"),
        upstream_erc_fixture("netlists/prefix_bus_alias/prefix_bus_alias.kicad_sch"),
    ] {
        assert_exact_upstream_erc_match(&schematic);
    }
}

#[test]
fn schematic_erc_tracks_broad_hierarchical_kicad_qa_regression() {
    let schematic =
        upstream_erc_fixture("netlists/top_level_hier_pins/top_level_hier_pins.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    assert!(native_root.violations.iter().any(|violation| {
        (violation.violation_type == "pin_not_connected")
            && (violation.description
                == "Hierarchical label 'INPUT' in root sheet cannot be connected to non-existent parent sheet")
    }));

    for label in [
        "INPUT",
        "SUB_INPUT",
        "SUB_OUTPUT",
        "SUBSUB_INPUT",
        "SUBSUB_OUTPUT",
    ] {
        assert!(oracle_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Hierarchical Label '{label}'"))
        }));
        assert!(native_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Hierarchical Label '{label}'"))
        }));
    }

    let oracle_power_mismatches = oracle_root
        .violations
        .iter()
        .filter(|violation| {
            (violation.violation_type == "lib_symbol_mismatch")
                && (violation.description == "Symbol 'GND' doesn't match copy in library 'power'")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == "Symbol #PWR? [GND]")
        })
        .count();
    let native_power_mismatches = native_root
        .violations
        .iter()
        .filter(|violation| {
            (violation.violation_type == "lib_symbol_mismatch")
                && (violation.description == "Symbol 'GND' doesn't match copy in library 'power'")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == "Symbol #PWR? [GND]")
        })
        .count();
    assert_eq!(oracle_power_mismatches, 2);
    assert_eq!(native_power_mismatches, 2);

    let oracle_subsub = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/Subsheet1/SubSubSheet/")
        .expect("oracle grandchild sheet should exist");
    let native_subsub = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/Subsheet1/SubSubSheet/")
        .expect("native grandchild sheet should exist");

    assert!(oracle_subsub.violations.iter().any(|violation| {
        (violation.violation_type == "power_pin_not_driven")
            && (violation.description == "Input Power pin not driven by any Output Power pins")
    }));
    assert!(native_subsub.violations.iter().any(|violation| {
        (violation.violation_type == "power_pin_not_driven")
            && (violation.description == "Input Power pin not driven by any Output Power pins")
            && violation.items.iter().any(|item| {
                item.description == "Symbol #PWR? Hidden pin 1 [GND, Power input, Line]"
            })
    }));
}

#[test]
fn schematic_erc_tracks_issue18606_root_regression() {
    let schematic = upstream_erc_fixture("issue18606/issue18606.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    for label in ["A0", "A1", "A2", "A3"] {
        assert!(oracle_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Label '{label}'"))
        }));
        assert!(native_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Label '{label}'"))
        }));
    }

    assert!(oracle_root.violations.iter().any(|violation| {
        (violation.violation_type == "lib_symbol_issues")
            && (violation.description
                == "The current configuration does not include the symbol library 'Resistors'")
    }));
    assert!(native_root.violations.iter().any(|violation| {
        (violation.violation_type == "lib_symbol_issues")
            && (violation.description
                == "The current configuration does not include the symbol library 'Resistors'")
    }));

    let native_child = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/test/")
        .expect("native child sheet should exist");
    let oracle_child = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/test/")
        .expect("oracle child sheet should exist");

    assert!(oracle_child.violations.iter().any(|violation| {
        (violation.violation_type == "multiple_net_names")
            && (violation.description
                == "Both A0 and A2 are attached to the same items; A0 will be used in the netlist")
    }));
    assert!(native_child.violations.iter().any(|violation| {
        (violation.violation_type == "multiple_net_names")
            && (violation.description
                == "Both A0 and A2 are attached to the same items; A0 will be used in the netlist")
    }));
}

#[test]
fn schematic_erc_tracks_multinetclasses_bus_group_regression() {
    let schematic = upstream_erc_fixture("netlists/multinetclasses/multinetclasses.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    for label in ["BUS.SIGNAL", "BUS.A0", "BUS.A1", "BUS.A2"] {
        assert!(oracle_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Label '{label}'"))
        }));
        assert!(native_root.violations.iter().any(|violation| {
            (violation.violation_type == "isolated_pin_label")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == format!("Label '{label}'"))
        }));
    }

    let oracle_undefined_count = oracle_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "undefined_netclass")
        .count();
    let native_undefined_count = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "undefined_netclass")
        .count();
    assert_eq!(oracle_undefined_count, 7);
    assert_eq!(native_undefined_count, 7);

    let oracle_pin_not_connected = oracle_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "pin_not_connected")
        .collect::<Vec<_>>();
    let native_pin_not_connected = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "pin_not_connected")
        .collect::<Vec<_>>();
    assert!(!oracle_pin_not_connected.iter().any(|violation| {
        violation
            .items
            .iter()
            .any(|item| item.description == "Symbol R7 Pin 1 [Passive, Line]")
    }));
    assert!(native_pin_not_connected.iter().any(|violation| {
        violation
            .items
            .iter()
            .any(|item| item.description == "Symbol R7 Pin 1 [Passive, Line]")
    }));
    assert_eq!(
        native_pin_not_connected.len(),
        oracle_pin_not_connected.len() + 1
    );
    assert!(!native_root
        .violations
        .iter()
        .any(|violation| violation.violation_type == "net_not_bus_member"));
}

#[test]
fn schematic_erc_documents_upstream_multinetclasses_r7_marker_bug() {
    let schematic = upstream_erc_fixture("netlists/multinetclasses/multinetclasses.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    assert!(!oracle_root.violations.iter().any(|violation| {
        violation
            .items
            .iter()
            .any(|item| item.description == "Symbol R7 Pin 1 [Passive, Line]")
    }));
    assert!(native_root.violations.iter().any(|violation| {
        violation
            .items
            .iter()
            .any(|item| item.description == "Symbol R7 Pin 1 [Passive, Line]")
    }));
}

#[test]
fn schematic_erc_tracks_bus_connection_regression() {
    let schematic = upstream_erc_fixture("netlists/bus_connection/bus_connection.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    let oracle_endpoint_count = oracle_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "endpoint_off_grid")
        .count();
    let native_endpoint_count = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "endpoint_off_grid")
        .count();
    assert_eq!(oracle_endpoint_count, 35);
    assert_eq!(native_endpoint_count, 35);

    let oracle_lib_issue_count = oracle_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "lib_symbol_issues")
        .count();
    let native_lib_issue_count = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "lib_symbol_issues")
        .count();
    assert_eq!(oracle_lib_issue_count, 2);
    assert_eq!(native_lib_issue_count, 2);

    assert!(oracle_root.violations.iter().any(|violation| {
        (violation.violation_type == "label_dangling")
            && violation
                .items
                .iter()
                .any(|item| item.description == "Label 'test.y'")
    }));
    assert!(native_root.violations.iter().any(|violation| {
        (violation.violation_type == "label_dangling")
            && violation
                .items
                .iter()
                .any(|item| item.description == "Label 'test.y'")
    }));

    for pin_item in [
        "Symbol J1 Pin 2 [Pin_2, Passive, Line]",
        "Symbol J1 Pin 3 [Pin_3, Passive, Line]",
        "Symbol J2 Pin 2 [Pin_2, Passive, Line]",
        "Symbol J2 Pin 3 [Pin_3, Passive, Line]",
    ] {
        assert!(!oracle_root.violations.iter().any(|violation| {
            (violation.violation_type == "endpoint_off_grid")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == pin_item)
        }));
        assert!(!native_root.violations.iter().any(|violation| {
            (violation.violation_type == "endpoint_off_grid")
                && violation
                    .items
                    .iter()
                    .any(|item| item.description == pin_item)
        }));
    }
}

#[test]
fn schematic_erc_tracks_prefix_bus_alias_regression() {
    let schematic = upstream_erc_fixture("netlists/prefix_bus_alias/prefix_bus_alias.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let Some(oracle) = kicad_cli_erc_json(&schematic, &[], None) else {
        return;
    };
    let native = ki_erc_json(&schematic, &[]);

    let oracle_root = oracle
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("oracle root sheet should exist");
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    let oracle_root_types = oracle_root
        .violations
        .iter()
        .map(|violation| violation.violation_type.as_str())
        .collect::<Vec<_>>();
    let native_root_types = native_root
        .violations
        .iter()
        .map(|violation| violation.violation_type.as_str())
        .collect::<Vec<_>>();

    assert!(oracle_root_types.is_empty());
    assert!(native_root_types.is_empty());

    for sheet_path in ["/Subsheet 1/", "/Subsheet2/"] {
        let oracle_sheet = oracle
            .report
            .sheets
            .iter()
            .find(|sheet| sheet.path == sheet_path)
            .expect("oracle child sheet should exist");
        let native_sheet = native
            .report
            .sheets
            .iter()
            .find(|sheet| sheet.path == sheet_path)
            .expect("native child sheet should exist");

        assert!(oracle_sheet.violations.is_empty());
        assert!(native_sheet.violations.is_empty());
    }
}

#[test]
fn schematic_erc_resolves_local_footprint_library_and_reports_filter_mismatch() {
    let schematic =
        Path::new("tests/fixtures/erc_parity/footprint_filter/footprint_filter.kicad_sch");
    let native = ki_erc_json(schematic, &[]);
    let violations = &native.report.sheets[0].violations;

    assert!(violations.iter().any(|violation| {
        violation.violation_type == "footprint_link_issues"
            && violation.description
                == "Assigned footprint (pkga) doesn't match footprint filters (R_*)"
    }));
    assert!(!violations.iter().any(|violation| {
        violation.violation_type == "footprint_link_issues"
            && violation.description
                == "The current configuration does not include the footprint library 'LocalFoot'"
    }));
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_footprint_filter_builtin() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "footprint_filter_builtin")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("footprint_filter_builtin.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_pin_not_connected() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "pin_not_connected")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("pin_not_connected.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_duplicate_sheet_names() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "duplicate_sheet_names")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("duplicate_sheet_names.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_duplicate_pins() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "duplicate_pins")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("duplicate_pins.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_multiple_net_names() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "multiple_net_names")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("multiple_net_names.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_hier_label_mismatch() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "hier_label_mismatch")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("hier_label_mismatch.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_bus_to_net_conflict() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "bus_to_net_conflict")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("bus_to_net_conflict.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_net_not_bus_member() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "net_not_bus_member")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("net_not_bus_member.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_endpoint_off_grid() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "endpoint_off_grid")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("endpoint_off_grid.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_single_global_label() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "single_global_label")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("single_global_label.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_four_way_junction() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "four_way_junction")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("four_way_junction.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_label_dangling() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "label_dangling")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("label_dangling.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_isolated_pin_label() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "isolated_pin_label")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("isolated_pin_label.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_no_connect_dangling() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "no_connect_dangling")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("no_connect_dangling.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_no_connect_connected() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "no_connect_connected")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("no_connect_connected.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_label_multiple_wires() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "label_multiple_wires")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("label_multiple_wires.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_wire_dangling() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "wire_dangling")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("wire_dangling.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_similar_labels() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "similar_labels")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("similar_labels.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_field_name_whitespace() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "field_name_whitespace")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("field_name_whitespace.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_same_local_global_label() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "same_local_global_label")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("same_local_global_label.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_pin_not_driven() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "pin_not_driven")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("pin_not_driven.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_stacked_pin_name() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "stacked_pin_name")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("stacked_pin_name.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_footprint_link_invalid_identifier() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "footprint_link_invalid_identifier")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("footprint_link_invalid_identifier.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_similar_label_and_power() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "similar_label_and_power")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("similar_label_and_power.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_similar_power() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "similar_power")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("similar_power.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_missing_input_pin() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "missing_input_pin")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("missing_input_pin.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_missing_bidi_pin() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "missing_bidi_pin")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("missing_bidi_pin.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_missing_power_pin() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "missing_power_pin")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("missing_power_pin.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_missing_unit() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "missing_unit")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("missing_unit.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_different_unit_net() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "different_unit_net")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("different_unit_net.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_different_unit_footprint() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "different_unit_footprint")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("different_unit_footprint.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_ground_pin_not_ground() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "ground_pin_not_ground")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("ground_pin_not_ground.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_undefined_netclass() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "undefined_netclass")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("undefined_netclass.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_unresolved_variable() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "unresolved_variable")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("unresolved_variable.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_generic_warning() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "generic_warning")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("generic_warning.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_generic_error() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "generic_error")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("generic_error.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_exit_code_violations_returns_validation_status() {
    let schematic = erc_parity_fixture("basic_test_errors_only/basic_test.kicad_sch");
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args([
            "schematic",
            "erc",
            schematic.to_str().unwrap(),
            "--json",
            "--severity-error",
            "--exit-code-violations",
        ])
        .output()
        .expect("native erc should run");

    assert_eq!(output.status.code().unwrap_or(0), 1);

    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    let normalized = normalize_erc_report(&report);
    assert_eq!(normalized.sheets[0].violations.len(), 2);
}

#[test]
fn schematic_erc_units_in_matches_expected_scaled_json_positions() {
    let schematic = erc_parity_fixture("basic_test_errors_only/basic_test.kicad_sch");
    let output = Command::cargo_bin("ki")
        .expect("binary should build")
        .args([
            "schematic",
            "erc",
            schematic.to_str().unwrap(),
            "--json",
            "--severity-error",
            "--units",
            "in",
        ])
        .output()
        .expect("native erc should run");

    assert_eq!(output.status.code().unwrap_or(1), 0);
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    let normalized = normalize_erc_report(&report);
    assert_eq!(normalized.coordinate_units, "in");
    let mut points = normalized.sheets[0]
        .violations
        .iter()
        .flat_map(|violation| {
            violation
                .items
                .iter()
                .map(|item| (item.x.clone(), item.y.clone()))
        })
        .collect::<Vec<_>>();
    points.sort();
    assert_eq!(
        points,
        vec![
            ("0.0555".to_string(), "0.028".to_string()),
            ("0.0555".to_string(), "0.034".to_string())
        ]
    );
}

#[test]
fn schematic_erc_matches_kicad_oracle_for_pin_to_pin() {
    let case = load_cases()
        .into_iter()
        .find(|case| case.id == "pin_to_pin")
        .expect("case should exist");
    let fixture_dir = Path::new(
        case.fixture_dir
            .as_deref()
            .expect("fixture dir should exist"),
    );
    let schematic = fixture_dir.join("pin_to_pin.kicad_sch");
    let oracle_path = Path::new(
        case.oracle_path
            .as_deref()
            .expect("oracle path should exist"),
    );
    let oracle = load_oracle(oracle_path);

    let args = case
        .ki_cli_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let native = ki_erc_json(&schematic, &args);

    assert_eq!(native.exit_code, case.expected_exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn erc_parity_cases_manifest_is_well_formed() {
    let cases = load_cases();
    assert!(cases.iter().any(|case| case.id == "missing_input_file"));
    assert!(cases.iter().any(|case| case.id == "basic_test_errors_only"));
    assert!(cases
        .iter()
        .any(|case| case.id == "missing_symbol_library_in_configuration"));
    assert!(cases.iter().any(|case| case.id == "pin_not_connected"));
    assert!(cases.iter().any(|case| case.id == "endpoint_off_grid"));
    assert!(cases.iter().any(|case| case.id == "single_global_label"));
    assert!(cases.iter().any(|case| case.id == "four_way_junction"));
    assert!(cases.iter().any(|case| case.id == "multiple_net_names"));
    assert!(cases.iter().any(|case| case.id == "hier_label_mismatch"));
    assert!(cases.iter().any(|case| case.id == "bus_to_net_conflict"));
    assert!(cases.iter().any(|case| case.id == "net_not_bus_member"));
    assert!(cases.iter().any(|case| case.id == "similar_power"));
    assert!(cases.iter().any(|case| case.id == "missing_input_pin"));
    assert!(cases.iter().any(|case| case.id == "missing_bidi_pin"));
    assert!(cases.iter().any(|case| case.id == "missing_power_pin"));
    assert!(cases.iter().any(|case| case.id == "missing_unit"));
    assert!(cases.iter().any(|case| case.id == "different_unit_net"));
    assert!(cases
        .iter()
        .any(|case| case.id == "different_unit_footprint"));
    assert!(cases.iter().any(|case| case.id == "ground_pin_not_ground"));
    assert!(cases.iter().any(|case| case.id == "undefined_netclass"));
    assert!(cases.iter().any(|case| case.id == "unresolved_variable"));
    assert!(cases.iter().any(|case| case.id == "generic_warning"));
    assert!(cases.iter().any(|case| case.id == "generic_error"));
    assert!(cases.iter().any(|case| case.id == "pin_to_pin"));
}

fn load_cases() -> Vec<ErcParityCase> {
    let raw = fs::read_to_string("tests/erc_parity/cases.json").expect("cases.json should exist");
    serde_json::from_str(&raw).expect("cases.json should deserialize")
}

fn load_oracle(path: &Path) -> ErcOracle {
    let raw = fs::read_to_string(path).expect("oracle should exist");
    let json: Value = serde_json::from_str(&raw).expect("oracle should be json");
    ErcOracle {
        exit_code: 0,
        report: normalize_erc_report(&json),
    }
}
