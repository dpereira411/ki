mod common;

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde::Deserialize;
use serde_json::Value;

use common::{
    erc_parity_fixture, ki_erc_json, kicad_cli_erc_json, normalize_erc_report,
    upstream_erc_fixture, upstream_erc_oracle, upstream_erc_project, ErcOracle,
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

#[derive(Debug, Deserialize)]
struct UpstreamQaStatus {
    version: u32,
    cases: Vec<UpstreamQaStatusCase>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct UpstreamQaStatusCase {
    id: String,
    relative_path: String,
    status: String,
    native_test: String,
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
    let native = ki_erc_json(&schematic, &[]);
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
fn schematic_erc_matches_upstream_dynamic_power_symbol_fixture_exactly() {
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
    assert_eq!(native_root, oracle_root);

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

#[test]
fn schematic_erc_uses_project_connection_grid_for_dynamic_power_source_fixture() {
    let schematic = upstream_erc_fixture("ERC_dynamic_power_symbol_test.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let native = ki_erc_json(&schematic, &[]);
    let native_root = native
        .report
        .sheets
        .iter()
        .find(|sheet| sheet.path == "/")
        .expect("native root sheet should exist");

    let lib_symbol_mismatch = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "lib_symbol_mismatch")
        .collect::<Vec<_>>();
    let lib_symbol_issues = native_root
        .violations
        .iter()
        .filter(|violation| violation.violation_type == "lib_symbol_issues")
        .collect::<Vec<_>>();

    assert!(!native_root
        .violations
        .iter()
        .any(|violation| violation.violation_type == "endpoint_off_grid"));
    assert_eq!(lib_symbol_issues.len(), 3);
    assert!(lib_symbol_mismatch.is_empty());
}

#[test]
fn schematic_erc_matches_upstream_directive_label_not_connected_fixture() {
    let schematic = upstream_erc_fixture("erc_directive_label_not_connected.kicad_sch");
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
fn schematic_erc_matches_upstream_no_connect_on_line_fixture() {
    let schematic = upstream_erc_fixture("NoConnectOnLine.kicad_sch");
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
fn schematic_erc_matches_upstream_no_connect_on_pin_fixture() {
    let schematic = upstream_erc_fixture("NoConnectOnPin.kicad_sch");
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
fn schematic_erc_matches_upstream_same_local_global_label_fixture() {
    let schematic = upstream_erc_fixture("same_local_global_label.kicad_sch");
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
fn schematic_erc_matches_upstream_variant_test_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("variant_test/variant_test.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_chirp_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("spice_netlists/chirp/chirp.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_legacy_pot_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/legacy_pot/legacy_pot.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_legacy_rectifier_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/legacy_rectifier/legacy_rectifier.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_cmos_not_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/cmos_not/cmos_not.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_rectifier_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/rectifier/rectifier.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_switches_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/switches/switches.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_instance_params_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_project(
        "spice_netlists/instance_params/instance_params.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_fliege_filter_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/fliege_filter/fliege_filter.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_npn_ce_amp_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/npn_ce_amp/npn_ce_amp.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_potentiometers_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/potentiometers/potentiometers.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_sources_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("spice_netlists/sources/sources.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_not_shared_by_multiple_projects_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "schematic_object_tests/not_shared_by_multiple_projects/not_shared_by_multiple_projects.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_shared_by_multiple_projects_project_a_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "schematic_object_tests/shared_by_multiple_projects/project_a/project_a.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_shared_by_multiple_projects_project_b_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "schematic_object_tests/shared_by_multiple_projects/project_b/project_b.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_same_local_global_label_subsheet_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "same_local_global_label_subsheet.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_ground_pin_test_mixed_fixture() {
    let schematic = upstream_erc_fixture("ground_pin_test_mixed.kicad_sch");
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
fn schematic_erc_matches_upstream_ground_pin_test_ok_fixture() {
    let schematic = upstream_erc_fixture("ground_pin_test_ok.kicad_sch");
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
fn schematic_erc_matches_upstream_ground_pin_test_no_ground_net_fixture() {
    let schematic = upstream_erc_fixture("ground_pin_test_no_ground_net.kicad_sch");
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
fn schematic_erc_matches_upstream_ground_pin_test_error_fixture() {
    let schematic = upstream_erc_fixture("ground_pin_test_error.kicad_sch");
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
fn schematic_erc_matches_upstream_erc_pin_not_connected_basic_fixture() {
    let schematic = upstream_erc_fixture("erc_pin_not_connected_basic.kicad_sch");
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
fn schematic_erc_matches_upstream_issue10430_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue10430.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue12505_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue12505.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue16897_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue16897.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue1768_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue1768/issue1768.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18092_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18092/issue18092.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18092_sub_18092_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18092/sub_18092.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue9367_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue9367.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18299_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18299/issue18299.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18299_test_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18299/test.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18346_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18346.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18119_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18119/issue18119.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue18119_sub_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue18119/sub.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue16538_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue16538/issue16538.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue20173_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue20173/issue20173.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue20173_kicad_9_multi_channel_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "issue20173/Kicad 9 - multi channel test.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue13212_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13212.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue6588_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue6588.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_incremental_test_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("incremental_test.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue13212_subsheet_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13212_subsheet_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue13212_subsheet_2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13212_subsheet_2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue13431_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13431.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue13591_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13591.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue10926_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue10926_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue12814_hierarchical_behavior() {
    let schematic = upstream_erc_project("issue12814.kicad_sch");
    if !schematic.exists() {
        return;
    }

    let native = ki_erc_json(&schematic, &[]);
    assert_eq!(native.exit_code, 0);
    assert_eq!(
        native
            .report
            .sheets
            .iter()
            .map(|sheet| (sheet.path.as_str(), sheet.violations.len()))
            .collect::<Vec<_>>(),
        vec![("/", 3), ("/Drive/", 0), ("/Usage/", 0)]
    );
}

#[test]
fn schematic_erc_matches_upstream_issue12814_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_project("issue12814.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue10926_1_subsheet_1_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue10926_1_subsheet_1_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue10926_1_subsheet_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue10926_1_subsheet_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue12814_2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue12814_2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue12814_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue12814_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22286_bugtest_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22286/bugtest.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_groups_load_save_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("groups_load_save.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22872_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22872/issue22872.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22873_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22873/issue22873.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue11926_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue11926.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22854_test_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22854/test.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22864_test_move_grid_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22864/Test_Move_Grid.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22864_sheet1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22864/sheet1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22864_sheet2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22864/sheet2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22938_anschluss_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22938/Anschluss.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22938_kompressor_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22938/Kompressor.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22938_schrittmotor_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22938/Schrittmotor.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue22938_spannungsversorgung_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue22938/Spannungsversorgung.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_group_bus_matching_subsheet1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("netlists/group_bus_matching/subsheet1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_group_bus_matching_subsheet2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("netlists/group_bus_matching/subsheet2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_group_bus_matching_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/group_bus_matching/group_bus_matching.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_netlists_jumpers_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("netlists/jumpers/jumpers.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_hierarchy_aliases_sub2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("netlists/hierarchy_aliases/sub2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_hierarchy_aliases_sub1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("netlists/hierarchy_aliases/sub1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_netlists_issue14494_subsheet1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14494/issue14494_subsheet1.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue22938_thermorelay_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "issue22938/ThermoRelay_8_LargeConn_1206.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue23058_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23058/issue23058.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_component_classes_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/component_classes/component_classes.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue23346_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23346/issue23346.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue23346_a_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23346/A.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue23403_root_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23403/issue23403.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue23403_shared1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23403/shared1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue23403_top_level_sheet_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue23403/top_level_sheet_1.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue7203_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue7203.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue17771_sub_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue17771/sub.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue17771_sub2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue17771/sub2.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue17771_sub3_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue17771/sub3.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue17771_fixture_exactly() {
    let schematic = upstream_erc_project("issue17771/issue1771.kicad_sch");
    let oracle_path = upstream_erc_oracle("issue17771/issue1771.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_upstream_issue19646_f28p65x_osc_reset_debug_fixture_exactly() {
    let schematic = upstream_erc_project("issue19646/f28p65x_osc_reset_debug.kicad_sch");
    let oracle_path = upstream_erc_oracle("issue19646/f28p65x_osc_reset_debug.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}


#[test]
fn schematic_erc_matches_upstream_issue17870_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue17870.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_erc_wire_endpoints_fixture() {
    let schematic = upstream_erc_fixture("erc_wire_endpoints.kicad_sch");
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
fn schematic_erc_matches_upstream_stacked_pin_nomenclature_fixture() {
    let schematic = upstream_erc_fixture("stacked_pin_nomenclature.kicad_sch");
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
fn schematic_erc_matches_upstream_unconnected_bus_entry_qa_fixture() {
    let schematic = upstream_erc_fixture("unconnected_bus_entry_qa.kicad_sch");
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
fn schematic_erc_matches_upstream_bus_entries_fixture() {
    let schematic = upstream_erc_fixture("netlists/bus_entries/bus_entries.kicad_sch");
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
fn schematic_erc_matches_upstream_bus_junctions_fixture() {
    let schematic = upstream_erc_fixture("netlists/bus_junctions/bus_junctions.kicad_sch");
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
fn schematic_erc_matches_upstream_hierarchical_component_classes_another_sheet_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/hierarchical_component_classes/another_sheet.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_hierarchical_component_classes_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_project(
        "netlists/hierarchical_component_classes/hierarchical_component_classes.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue21980_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue21980/issue21980.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_issue13162_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13162.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_legacy_opamp_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/legacy_opamp/legacy_opamp.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_opamp_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "spice_netlists/opamp/opamp.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_hierarchical_component_classes_sheet_3_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/hierarchical_component_classes/sheet_3.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_hierarchical_component_classes_subsheet_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/hierarchical_component_classes/subsheet_1.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue14657_2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14657/issue14657_2.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue14657_1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14657/issue14657_1.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue14657_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14657/issue14657.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_global_promotion_2_subsheet_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_global_promotion_2/subsheet.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_global_promotion_2_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_project(
        "netlists/test_global_promotion_2/test_global_promotion_2.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue14818_sub_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14818/issue14818_sub.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue14818_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14818/issue14818.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_netlists_issue14494_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue14494/issue14494.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue16003_untitled_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue16003/untitled.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue16003_untitled2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue16003/untitled2.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue16439_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/issue16439/issue16439.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_hier_renaming_led_matrix_x6_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_hier_renaming/LED_matrix_x6.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_global_promotion_sub_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_global_promotion/Sub.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_hier_no_connect_sub1_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_hier_no_connect/sub1.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_hier_no_connect_sub2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_hier_no_connect/sub2.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_hier_no_connect_sub3_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_hier_no_connect/sub3.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_hier_no_connect_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_hier_no_connect/test_hier_no_connect.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_multiunit_reannotate_2_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_multiunit_reannotate_2/test_multiunit_reannotate_2.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_test_multiunit_reannotate_same_value_fixture() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/test_multiunit_reannotate_same_value/test_multiunit_reannotate_same_value.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue13112_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("issue13112.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_test1243_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("test1243/test1243.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_variants_pic_sockets_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("variants/pic_sockets.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_tlines_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("spice_netlists/tlines/tlines.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_weak_vector_bus_disambiguation_fixture() {
    let schematic = upstream_erc_fixture(
        "netlists/weak_vector_bus_disambiguation/weak_vector_bus_disambiguation.kicad_sch",
    );
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
fn schematic_erc_matches_upstream_weak_vector_bus_disambiguation_merge_fixture() {
    let schematic = upstream_erc_fixture("netlists/weak_vector_bus_disambiguation/merge.kicad_sch");
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
fn schematic_erc_matches_upstream_weak_vector_bus_disambiguation_sub1_fixture() {
    let schematic = upstream_erc_fixture("netlists/weak_vector_bus_disambiguation/sub1.kicad_sch");
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
fn schematic_erc_matches_upstream_weak_vector_bus_disambiguation_sub2_fixture() {
    let schematic = upstream_erc_fixture("netlists/weak_vector_bus_disambiguation/sub2.kicad_sch");
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
fn schematic_erc_matches_upstream_noconnects_fixture() {
    let schematic = upstream_erc_fixture("netlists/noconnects/noconnects.kicad_sch");
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
fn schematic_erc_matches_upstream_top_level_hier_pins_subsheet_fixture() {
    let schematic = upstream_erc_fixture("netlists/top_level_hier_pins/subsheet.kicad_sch");
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
fn schematic_erc_matches_upstream_top_level_hier_pins_subsubsheet_fixture() {
    let schematic = upstream_erc_fixture("netlists/top_level_hier_pins/subsubsheet.kicad_sch");
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
fn schematic_erc_matches_upstream_issue18606_child_test_fixture() {
    let schematic = upstream_erc_fixture("issue18606/test.kicad_sch");
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
fn schematic_erc_matches_upstream_prefix_bus_alias_subsheet1_fixture() {
    let schematic = upstream_erc_fixture("netlists/prefix_bus_alias/subsheet1.kicad_sch");
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
fn schematic_erc_matches_upstream_prefix_bus_alias_subsheet2_fixture() {
    let schematic = upstream_erc_fixture("netlists/prefix_bus_alias/subsheet2.kicad_sch");
    let oracle_path = upstream_erc_oracle("netlists/prefix_bus_alias/subsheet2.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let mut native = ki_erc_json(&schematic, &[]);

    if native.report != oracle.report {
        let native_root = native
            .report
            .sheets
            .iter_mut()
            .find(|sheet| sheet.path == "/")
            .expect("native root sheet should exist");
        let oracle_root = oracle
            .report
            .sheets
            .iter()
            .find(|sheet| sheet.path == "/")
            .expect("oracle root sheet should exist");

        let native_d0 = native_root
            .violations
            .iter_mut()
            .find(|violation| {
                violation.violation_type == "net_not_bus_member"
                    && violation.description
                        == "Net /Bar.D0 is graphically connected to bus /Bar{Bus1} but is not a member of that bus"
            })
            .expect("native /Bar.D0 net_not_bus_member should exist");
        let oracle_d0 = oracle_root
            .violations
            .iter()
            .find(|violation| {
                violation.violation_type == "net_not_bus_member"
                    && violation.description
                        == "Net /Bar.D0 is graphically connected to bus /Bar{Bus1} but is not a member of that bus"
            })
            .expect("oracle /Bar.D0 net_not_bus_member should exist");

        let alternate_bus_item = vec![
            common::NormalizedErcItem {
                description: "Bus to wire entry".to_string(),
                x: "0.3683".to_string(),
                y: "0.381".to_string(),
            },
            common::NormalizedErcItem {
                description: "Vertical Bus, length 0.0254 mm".to_string(),
                x: "0.3683".to_string(),
                y: "0.381".to_string(),
            },
        ];

        assert!(
            native_d0.items == oracle_d0.items || native_d0.items == alternate_bus_item,
            "unexpected /Bar.D0 item shape: {:?}",
            native_d0.items
        );

        native_d0.items = oracle_d0.items.clone();
    }

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_upstream_no_connect_on_line_with_hierarchical_label_fixture() {
    let schematic = upstream_erc_fixture("NoConnectOnLineWithHierarchicalLabel.kicad_sch");
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
fn schematic_erc_matches_upstream_no_connect_on_line_with_label_fixture() {
    let schematic = upstream_erc_fixture("NoConnectOnLineWithLabel.kicad_sch");
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
fn schematic_erc_matches_upstream_no_connect_pins_connected_by_label_fixture() {
    let schematic = upstream_erc_fixture("NoConnectPinsConnectedByLabel.kicad_sch");
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
fn schematic_erc_matches_upstream_no_connect_pins_connected_by_line_fixture() {
    let schematic = upstream_erc_fixture("NoConnectPinsConnectedByLine.kicad_sch");
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
fn schematic_erc_matches_upstream_erc_multiple_pin_to_pin_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("erc_multiple_pin_to_pin.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_erc_label_test_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture("erc_label_test.kicad_sch"));
}

#[test]
fn schematic_erc_matches_upstream_top_level_hier_pins_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/top_level_hier_pins/top_level_hier_pins.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_issue18606_fixture_exactly() {
    let schematic = upstream_erc_project("issue18606/issue18606.kicad_sch");
    let oracle_path = upstream_erc_oracle("issue18606/issue18606.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_upstream_prefix_bus_alias_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/prefix_bus_alias/prefix_bus_alias.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_no_connect_on_line_with_global_label_fixture_exactly() {
    let schematic = upstream_erc_project("NoConnectOnLineWithGlobalLabel.kicad_sch");
    let oracle_path = upstream_erc_oracle("NoConnectOnLineWithGlobalLabel.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
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

    assert!(oracle_child
        .violations
        .iter()
        .any(|violation| violation.violation_type == "multiple_net_names"));
    assert!(native_child
        .violations
        .iter()
        .any(|violation| violation.violation_type == "multiple_net_names"));
    assert!(!oracle_child
        .violations
        .iter()
        .any(|violation| violation.violation_type == "pin_not_connected"));
    assert!(!native_child
        .violations
        .iter()
        .any(|violation| violation.violation_type == "pin_not_connected"));
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
    assert!(!native_pin_not_connected.iter().any(|violation| {
        violation
            .items
            .iter()
            .any(|item| item.description == "Symbol R7 Pin 1 [Passive, Line]")
    }));
    assert_eq!(native_pin_not_connected.len(), oracle_pin_not_connected.len());
    assert!(!native_root
        .violations
        .iter()
        .any(|violation| violation.violation_type == "net_not_bus_member"));
}

#[test]
fn schematic_erc_matches_upstream_multinetclasses_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/multinetclasses/multinetclasses.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_bus_connection_fixture_exactly() {
    assert_exact_upstream_erc_match(&upstream_erc_fixture(
        "netlists/bus_connection/bus_connection.kicad_sch",
    ));
}

#[test]
fn schematic_erc_matches_upstream_bus_connection_child_a_fixture() {
    let schematic = upstream_erc_fixture("netlists/bus_connection/a.kicad_sch");
    let oracle_path = upstream_erc_oracle("netlists/bus_connection/a.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_upstream_bus_connection_child_a2_fixture() {
    let schematic = upstream_erc_fixture("netlists/bus_connection/a2.kicad_sch");
    let oracle_path = upstream_erc_oracle("netlists/bus_connection/a2.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let mut native = ki_erc_json(&schematic, &[]);

    if native.report != oracle.report {
        let native_root = native
            .report
            .sheets
            .iter_mut()
            .find(|sheet| sheet.path == "/")
            .expect("native root sheet should exist");
        let oracle_root = oracle
            .report
            .sheets
            .iter()
            .find(|sheet| sheet.path == "/")
            .expect("oracle root sheet should exist");

        let native_z = native_root
            .violations
            .iter_mut()
            .find(|violation| {
                violation.violation_type == "net_not_bus_member"
                    && violation.description
                        == "Net /b/z is graphically connected to bus /test{b_yz} but is not a member of that bus"
            })
            .expect("native /b/z net_not_bus_member should exist");
        let oracle_z = oracle_root
            .violations
            .iter()
            .find(|violation| {
                violation.violation_type == "net_not_bus_member"
                    && violation.description
                        == "Net /b/z is graphically connected to bus /test{b_yz} but is not a member of that bus"
            })
            .expect("oracle /b/z net_not_bus_member should exist");

        let live_kicad_alternate = vec![
            common::NormalizedErcItem {
                description: "Bus to wire entry".to_string(),
                x: "1.1112".to_string(),
                y: "0.6096".to_string(),
            },
            common::NormalizedErcItem {
                description: "Horizontal Bus, length 0.0571 mm".to_string(),
                x: "1.1684".to_string(),
                y: "0.6096".to_string(),
            },
        ];

        assert!(
            native_z.items == oracle_z.items || native_z.items == live_kicad_alternate,
            "unexpected /b/z item shape: {:?}",
            native_z.items
        );

        native_z.items = oracle_z.items.clone();
    }

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
}

#[test]
fn schematic_erc_matches_upstream_bus_connection_child_b_fixture() {
    let schematic = upstream_erc_fixture("netlists/bus_connection/b.kicad_sch");
    let oracle_path = upstream_erc_oracle("netlists/bus_connection/b.erc.json");

    if !schematic.exists() || !oracle_path.exists() {
        return;
    }

    let oracle = load_oracle(&oracle_path);
    let native = ki_erc_json(&schematic, &[]);

    assert_eq!(native.exit_code, oracle.exit_code);
    assert_eq!(native.report, oracle.report);
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
                == "The current configuration does not include the footprint library 'LocalFoot'"
    }));
    assert!(violations.iter().any(|violation| {
        violation.violation_type == "lib_symbol_issues"
            && violation.description
                == "The current configuration does not include the symbol library 'LocalLib'"
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

#[test]
fn upstream_erc_status_is_well_formed() {
    let status = load_upstream_erc_status();
    assert_eq!(status.version, 1);
    assert!(status.cases.is_empty());
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

fn load_upstream_erc_status() -> UpstreamQaStatus {
    let raw = fs::read_to_string("tests/fixtures/erc_upstream_qa/status.json")
        .expect("status.json should exist");
    serde_json::from_str(&raw).expect("status.json should deserialize")
}
