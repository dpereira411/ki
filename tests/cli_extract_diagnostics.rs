mod common;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::Value;
use tempfile::TempDir;

use common::{
    extract_parity_fixture, ki_extract_diagnostics, ki_extract_raw, kicad_cli_extract_diagnostics,
    kicad_cli_extract_raw,
};

fn assert_no_visible_diagnostics(schematic: &Path) {
    let Some(kicad) = kicad_cli_extract_diagnostics(schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

fn assert_matching_diagnostics(schematic: &Path) {
    let Some(kicad) = kicad_cli_extract_diagnostics(schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert_eq!(ki.messages, kicad.messages);
}

fn assert_failed_load_message(schematic: &Path) {
    let Some(kicad) = kicad_cli_extract_raw(schematic) else {
        return;
    };
    let ki = ki_extract_raw(schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_annotation_error_diagnostics() {
    let schematic = extract_parity_fixture("annotation_errors/annotation_errors.kicad_sch");
    assert_matching_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_duplicate_sheet_name_diagnostics() {
    let schematic = extract_parity_fixture("duplicate_sheet_names/duplicate_sheet_names.kicad_sch");
    assert_matching_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_recursive_sheet_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("recursive_sheet/recursive_sheet.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_missing_library_symbol_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("missing_library_symbol/missing_library_symbol.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_missing_child_sheet_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("missing_child_sheet/missing_child_sheet.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_lib_id_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("invalid_lib_id/invalid_lib_id.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_name_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("invalid_symbol_name/invalid_symbol_name.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_library_id_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("invalid_symbol_library_id/invalid_symbol_library_id.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_missing_sheet_name_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("missing_sheet_name/missing_sheet_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.exit_code, ki.exit_code);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_missing_sheet_file_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("missing_sheet_file/missing_sheet_file.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_instances_page_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_instances_page/invalid_sheet_instances_page.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_instances_path_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_instances_path/invalid_sheet_instances_path.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_instances_numeric_path_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_instances_numeric_path/invalid_sheet_instances_numeric_path.kicad_sch",
    );
    assert_failed_load_message(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_instances_numeric_page_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_instances_numeric_page/invalid_sheet_instances_numeric_page.kicad_sch",
    );
    assert_failed_load_message(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_reference_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_reference/invalid_symbol_instances_reference.kicad_sch",
    );
    assert_failed_load_message(&schematic);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_numeric_path_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_numeric_path/invalid_symbol_instances_numeric_path.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_numeric_reference_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_numeric_reference/invalid_symbol_instances_numeric_reference.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_unit_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_unit/invalid_symbol_instances_unit.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_numeric_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_numeric_value/invalid_symbol_instances_numeric_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_numeric_footprint_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_numeric_footprint/invalid_symbol_instances_numeric_footprint.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_instances_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_instances_value/invalid_symbol_instances_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_invalid_reference_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_invalid_reference/default_instance_invalid_reference.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_numeric_reference_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_numeric_reference/default_instance_numeric_reference.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_invalid_unit_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_invalid_unit/default_instance_invalid_unit.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_invalid_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_invalid_value/default_instance_invalid_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_numeric_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_numeric_value/default_instance_numeric_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_numeric_footprint_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_numeric_footprint/default_instance_numeric_footprint.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_instances_missing_project_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_instances_missing_project_name/symbol_instances_missing_project_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_instances_numeric_project_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_instances_numeric_project/symbol_instances_numeric_project.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_instances_missing_path_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_instances_missing_path_value/symbol_instances_missing_path_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_instances_numeric_path_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_instances_numeric_path/symbol_instances_numeric_path.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_failed_load_message() {
    let Some(kicad) = ({
        let temp = TempDir::new().expect("tempdir should exist");
        let bad = temp.path().join("bad.kicad_sch");
        fs::write(&bad, "not a schematic\n").expect("fixture should write");
        kicad_cli_extract_raw(&bad)
    }) else {
        return;
    };

    let temp = TempDir::new().expect("tempdir should exist");
    let bad = temp.path().join("bad.kicad_sch");
    fs::write(&bad, "not a schematic\n").expect("fixture should write");
    let ki = ki_extract_raw(&bad);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_duplicate_reference_failed_load_message() {
    let schematic = extract_parity_fixture("duplicate_references/duplicate_references.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_property_name_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_property_name/invalid_property_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_property_value_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_property_value/invalid_property_value.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_string_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_text_string/invalid_text_string.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_global_label_shape_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_global_label_shape/invalid_global_label_shape.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_global_label_at_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_global_label_at_arity/invalid_global_label_at_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_hierarchical_label_shape_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_hierarchical_label_shape/invalid_hierarchical_label_shape.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_directive_label_shape_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_directive_label_shape/invalid_directive_label_shape.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_label_at_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_label_at_arity/invalid_label_at_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_hyperlink_url_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_hyperlink_url/invalid_hyperlink_url.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_cannot_parse_header_failed_load_message() {
    let schematic = extract_parity_fixture("cannot_parse_header/cannot_parse_header.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_name_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_pin_name/invalid_pin_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_number_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_pin_number/invalid_pin_number.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_alternate_pin_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_alternate_pin_name/invalid_alternate_pin_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_type_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_pin_type/invalid_pin_type.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_shape_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_pin_shape/invalid_pin_shape.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_alternate_pin_type_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_alternate_pin_type/invalid_alternate_pin_type.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_alternate_pin_shape_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_alternate_pin_shape/invalid_alternate_pin_shape.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_box_string_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_text_box_string/invalid_text_box_string.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_bus_alias_members_failed_load_message() {
    let schematic =
        extract_parity_fixture("bus_alias_weird_members/bus_alias_weird_members.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_at_arity_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_text_at_arity/invalid_text_at_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_box_size_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_text_box_size_arity/invalid_text_box_size_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_title_block_comment_number_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_title_block_comment_number/invalid_title_block_comment_number.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_effects_font_size_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_effects_font_size_arity/invalid_text_effects_font_size_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_effects_font_size_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_effects_font_size_token/invalid_text_effects_font_size_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_effects_color_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_effects_color_arity/invalid_text_effects_color_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_effects_color_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_effects_color_token/invalid_text_effects_color_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_effects_justify_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_effects_justify_token/invalid_effects_justify_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_effects_bold_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_effects_bold_token/invalid_effects_bold_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_effects_italic_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_effects_italic_token/invalid_effects_italic_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_junction_at_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_junction_at_arity/invalid_junction_at_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_stroke_width_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_stroke_width_token/invalid_stroke_width_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_stroke_type_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_stroke_type_token/invalid_stroke_type_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_stroke_color_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_stroke_color_arity/invalid_stroke_color_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_fill_type_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_fill_type_token/invalid_fill_type_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_no_connect_at_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_no_connect_at_arity/invalid_no_connect_at_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_bus_entry_at_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_bus_entry_at_arity/invalid_bus_entry_at_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_bus_entry_size_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_bus_entry_size_arity/invalid_bus_entry_size_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_rectangle_start_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_rectangle_start_arity/invalid_rectangle_start_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_circle_center_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_circle_center_arity/invalid_circle_center_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_arc_start_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_arc_start_arity/invalid_arc_start_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_circle_radius_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_circle_radius_token/invalid_circle_radius_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_rectangle_fill_color_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_rectangle_fill_color_arity/invalid_rectangle_fill_color_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_fill_color_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_fill_color_token/invalid_fill_color_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_bezier_point_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_bezier_point_arity/invalid_bezier_point_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_wire_pts_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_wire_pts_arity/invalid_wire_pts_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_polyline_pts_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_polyline_pts_arity/invalid_polyline_pts_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_bus_pts_arity_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_bus_pts_arity/invalid_bus_pts_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_rule_area_pt_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_rule_area_pt_arity/invalid_rule_area_pt_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_parent_symbol_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_parent_symbol_name/invalid_parent_symbol_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_unit_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_unit_name/invalid_symbol_unit_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_unit_number_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_unit_number/invalid_symbol_unit_number.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_body_style_number_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_body_style_number/invalid_symbol_body_style_number.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_unit_suffix_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_unit_suffix/invalid_symbol_unit_suffix.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_pin_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_pin_name/invalid_sheet_pin_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_pin_type_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_pin_type/invalid_sheet_pin_type.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_pin_position_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_pin_position_arity/invalid_sheet_pin_position_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_pin_orientation_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_pin_orientation/invalid_sheet_pin_orientation.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_at_angle_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_at_angle/invalid_sheet_at_angle.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_title_block_property_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_title_block_property_value/invalid_title_block_property_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_title_block_property_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_title_block_property_name/invalid_title_block_property_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_group_name_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("invalid_group_name/invalid_group_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_fonts_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "no_schematic_object_embedded_fonts/no_schematic_object_embedded_fonts.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_library_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_library_name/invalid_symbol_library_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_uuid_missing_value_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_uuid_missing_value/symbol_uuid_missing_value.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_unit_missing_value_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_unit_missing_value/symbol_unit_missing_value.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_body_style_missing_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_body_style_missing_value/symbol_body_style_missing_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_at_orientation_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_at_orientation/invalid_symbol_at_orientation.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_mirror_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_mirror_token/invalid_symbol_mirror_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_missing_number_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_pin_missing_number/symbol_pin_missing_number.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_uuid_missing_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_pin_uuid_missing_value/symbol_pin_uuid_missing_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_unknown_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_pin_unknown_child/symbol_pin_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_uuid_numeric_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_pin_uuid_numeric/symbol_pin_uuid_numeric.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_variant_field_numeric_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("variant_field_numeric_name/variant_field_numeric_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_variant_field_numeric_value_failed_load_message() {
    let schematic =
        extract_parity_fixture("variant_field_numeric_value/variant_field_numeric_value.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_variant_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture("variant_unknown_child/variant_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_numeric_reference_failed_load_message() {
    let schematic = extract_parity_fixture(
        "nested_instance_numeric_reference/nested_instance_numeric_reference.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "nested_instance_unknown_child/nested_instance_unknown_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_value_missing_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "nested_instance_value_missing_value/nested_instance_value_missing_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_footprint_missing_value_failed_load_message() {
    let schematic = extract_parity_fixture(
        "nested_instance_footprint_missing_value/nested_instance_footprint_missing_value.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_unit_numeric_float_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "nested_instance_unit_numeric_float/nested_instance_unit_numeric_float.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_value_tilde_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("nested_instance_value_tilde/nested_instance_value_tilde.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_footprint_tilde_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "nested_instance_footprint_tilde/nested_instance_footprint_tilde.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_value_tilde_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "default_instance_value_tilde/default_instance_value_tilde.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_footprint_tilde_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "default_instance_footprint_tilde/default_instance_footprint_tilde.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_reference_tilde_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "default_instance_reference_tilde/default_instance_reference_tilde.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_unknown_child/default_instance_unknown_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_duplicate_uuid_child_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("symbol_duplicate_uuid_child/symbol_duplicate_uuid_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_unknown_top_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_unknown_top_child/symbol_unknown_top_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_no_cells_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_table_no_cells/invalid_table_no_cells.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_group_library_id_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("invalid_group_library_id/invalid_group_library_id.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "no_schematic_object_embedded_files/no_schematic_object_embedded_files.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_missing_reference_failed_load_message() {
    let schematic = extract_parity_fixture("missing_reference/missing_reference.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_missing_uuid_failed_load_message() {
    let schematic = extract_parity_fixture("missing_uuid/missing_uuid.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_page_type_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_page_type/invalid_page_type.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_empty_sheet_pin_name_failed_load_message() {
    let schematic = extract_parity_fixture("empty_sheet_pin_name/empty_sheet_pin_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_polyline_too_few_points_no_visible_diagnostics() {
    let schematic =
        extract_parity_fixture("polyline_too_few_points/polyline_too_few_points.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.exit_code, ki.exit_code);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_font_color_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_font_color_token/invalid_font_color_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_font_color_arity_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_font_color_arity/invalid_font_color_arity.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_font_thickness_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_font_thickness_token/invalid_font_thickness_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_font_line_spacing_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_font_line_spacing_token/invalid_font_line_spacing_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_font_face_payload_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_font_face_payload/invalid_font_face_payload.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_hide_token_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_hide_token/invalid_hide_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_junction_diameter_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_junction_diameter_token/invalid_junction_diameter_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_junction_color_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_junction_color_token/invalid_junction_color_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_junction_color_arity_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_junction_color_arity/invalid_junction_color_arity.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_directive_length_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_directive_length_token/invalid_directive_length_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_box_margins_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_box_margins_token/invalid_text_box_margins_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_box_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_box_exclude_from_sim_token/invalid_text_box_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_exclude_from_sim_token/invalid_text_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_text_exclude_from_sim_failed_load_message() {
    let schematic =
        extract_parity_fixture("bare_text_exclude_from_sim/bare_text_exclude_from_sim.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_bare_exclude_from_sim_failed_load_message() {
    let schematic =
        extract_parity_fixture("label_bare_exclude_from_sim/label_bare_exclude_from_sim.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_global_label_bare_exclude_from_sim_failed_load_message() {
    let schematic = extract_parity_fixture(
        "global_label_bare_exclude_from_sim/global_label_bare_exclude_from_sim.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_bare_exclude_from_sim_failed_load_message() {
    let schematic = extract_parity_fixture(
        "hier_label_bare_exclude_from_sim/hier_label_bare_exclude_from_sim.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_directive_label_bare_exclude_from_sim_failed_load_message() {
    let schematic = extract_parity_fixture(
        "directive_label_bare_exclude_from_sim/directive_label_bare_exclude_from_sim.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_text_fields_autoplaced_token/invalid_text_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_invalid_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "label_invalid_fields_autoplaced_token/label_invalid_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_global_label_invalid_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "global_label_invalid_fields_autoplaced_token/global_label_invalid_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_invalid_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "hier_label_invalid_fields_autoplaced_token/hier_label_invalid_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_show_name_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_show_name_token/invalid_show_name_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_do_not_autoplace_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_do_not_autoplace_token/invalid_do_not_autoplace_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_top_embedded_fonts_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_top_embedded_fonts_token/invalid_top_embedded_fonts_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_top_embedded_fonts_failed_load_message() {
    let schematic =
        extract_parity_fixture("bare_top_embedded_fonts/bare_top_embedded_fonts.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_directive_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_directive_fields_autoplaced_token/invalid_directive_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_directive_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_directive_exclude_from_sim_token/invalid_directive_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_invalid_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "label_invalid_exclude_from_sim_token/label_invalid_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_global_label_invalid_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "global_label_invalid_exclude_from_sim_token/global_label_invalid_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_invalid_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "hier_label_invalid_exclude_from_sim_token/hier_label_invalid_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_directive_fields_autoplaced_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "bare_directive_fields_autoplaced/bare_directive_fields_autoplaced.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_text_box_span_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_text_box_span_token/invalid_text_box_span_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_column_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_table_column_token/invalid_table_column_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_header_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_table_header_token/invalid_table_header_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_border_external_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_table_border_external_token/invalid_table_border_external_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_border_header_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_table_border_header_token/invalid_table_border_header_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_col_widths_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_table_col_widths_token/invalid_table_col_widths_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_row_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_table_row_token/invalid_table_row_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_row_heights_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_table_row_heights_token/invalid_table_row_heights_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_table_separators_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_table_separators_token/invalid_table_separators_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_size_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_size_token/invalid_sheet_size_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_dnp_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_dnp_token/invalid_sheet_dnp_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_fields_autoplaced_token/invalid_sheet_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_exclude_from_sim_token/invalid_sheet_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_variant_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_variant_exclude_from_sim_token/invalid_sheet_variant_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_sheet_variant_exclude_from_sim_failed_load_message() {
    let schematic = extract_parity_fixture(
        "bare_sheet_variant_exclude_from_sim/bare_sheet_variant_exclude_from_sim.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_in_bom_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_sheet_in_bom_token/invalid_sheet_in_bom_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_sheet_on_board_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_sheet_on_board_token/invalid_sheet_on_board_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_dnp_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_dnp_token/invalid_symbol_dnp_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_exclude_from_sim_token/invalid_symbol_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_in_bom_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_symbol_in_bom_token/invalid_symbol_in_bom_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_on_board_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_on_board_token/invalid_symbol_on_board_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_in_pos_files_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_in_pos_files_token/invalid_symbol_in_pos_files_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_symbol_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_symbol_fields_autoplaced_token/invalid_symbol_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_names_offset_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_pin_names_offset_token/invalid_pin_names_offset_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_names_hide_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_pin_names_hide_token/invalid_pin_names_hide_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_numbers_hide_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_pin_numbers_hide_token/invalid_pin_numbers_hide_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_pin_hide_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_pin_hide_token/invalid_pin_hide_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_pin_names_hide_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("bare_pin_names_hide/bare_pin_names_hide.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_pin_hide_failed_load_message() {
    let schematic = extract_parity_fixture("bare_pin_hide/bare_pin_hide.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_pin_numbers_hide_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("bare_pin_numbers_hide/bare_pin_numbers_hide.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_rule_area_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_rule_area_exclude_from_sim_token/invalid_rule_area_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_rule_area_dnp_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_rule_area_dnp_token/invalid_rule_area_dnp_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_lib_symbol_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_lib_symbol_exclude_from_sim_token/invalid_lib_symbol_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_lib_symbol_fonts_embedded_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_lib_symbol_fonts_embedded_token/invalid_lib_symbol_fonts_embedded_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_lib_symbol_jumpers_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_lib_symbol_jumpers_token/invalid_lib_symbol_jumpers_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_variant_dnp_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_variant_dnp_token/invalid_variant_dnp_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_variant_in_bom_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "invalid_variant_in_bom_token/invalid_variant_in_bom_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_variant_name_failed_load_message() {
    let schematic = extract_parity_fixture("invalid_variant_name/invalid_variant_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_invalid_variant_field_name_failed_load_message() {
    let schematic =
        extract_parity_fixture("invalid_variant_field_name/invalid_variant_field_name.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_symbol_fields_autoplaced_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "bare_symbol_fields_autoplaced/bare_symbol_fields_autoplaced.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_sheet_fields_autoplaced_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "bare_sheet_fields_autoplaced/bare_sheet_fields_autoplaced.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_bold_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("bare_bold/bare_bold.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_italic_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("bare_italic/bare_italic.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bare_hide_no_visible_diagnostics() {
    let schematic = extract_parity_fixture("bare_hide/bare_hide.kicad_sch");
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "netclass_flag_no_visible_diagnostics/netclass_flag_no_visible_diagnostics.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert_eq!(ki.messages, kicad.messages);
    assert_eq!(ki.exit_code, kicad.exit_code);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_extra_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_extra_child/netclass_flag_extra_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_paper_landscape_token_failed_load_message() {
    let schematic = extract_parity_fixture("paper_landscape_token/paper_landscape_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_with_property_failed_load_message() {
    let schematic = extract_parity_fixture("text_with_property/text_with_property.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_with_property_failed_load_message() {
    let schematic =
        extract_parity_fixture("text_box_with_property/text_box_with_property.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_with_iref_failed_load_message() {
    let schematic = extract_parity_fixture("label_with_iref/label_with_iref.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_with_property_failed_load_message() {
    let schematic = extract_parity_fixture("label_with_property/label_with_property.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_global_label_with_property_failed_load_message() {
    let schematic =
        extract_parity_fixture("global_label_with_property/global_label_with_property.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_hier_label_with_property_failed_load_message() {
    let schematic =
        extract_parity_fixture("hier_label_with_property/hier_label_with_property.kicad_sch");
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_directive_label_with_property_failed_load_message() {
    let schematic = extract_parity_fixture(
        "directive_label_with_property/directive_label_with_property.kicad_sch",
    );
    assert_no_visible_diagnostics(&schematic);
}

#[test]
fn extract_matches_kicad_cli_label_property_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "label_property_name_list_child/label_property_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_property_value_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "label_property_value_list_child/label_property_value_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_global_label_property_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "global_label_property_name_list_child/global_label_property_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_global_label_property_value_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "global_label_property_value_list_child/global_label_property_value_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_property_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "hier_label_property_name_list_child/hier_label_property_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_property_value_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "hier_label_property_value_list_child/hier_label_property_value_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_directive_label_property_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "directive_label_property_name_list_child/directive_label_property_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_directive_label_property_value_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "directive_label_property_value_list_child/directive_label_property_value_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_label_with_length_failed_load_message() {
    let schematic = extract_parity_fixture("label_with_length/label_with_length.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_directive_label_with_iref_failed_load_message() {
    let schematic =
        extract_parity_fixture("directive_label_with_iref/directive_label_with_iref.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_hier_label_with_iref_failed_load_message() {
    let schematic = extract_parity_fixture("hier_label_with_iref/hier_label_with_iref.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_with_iref_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_with_iref/netclass_flag_with_iref.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_with_href_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_with_href/netclass_flag_with_href.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_effects_href_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_effects_href/netclass_flag_effects_href.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_uuid_numeric_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_uuid_numeric/netclass_flag_uuid_numeric.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_uuid_list_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_uuid_list/netclass_flag_uuid_list.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_invalid_shape_failed_load_message() {
    let schematic =
        extract_parity_fixture("netclass_flag_invalid_shape/netclass_flag_invalid_shape.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_invalid_length_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "netclass_flag_invalid_length_token/netclass_flag_invalid_length_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_fields_autoplaced_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "netclass_flag_fields_autoplaced_token/netclass_flag_fields_autoplaced_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_netclass_flag_exclude_from_sim_token_failed_load_message() {
    let schematic = extract_parity_fixture(
        "netclass_flag_exclude_from_sim_token/netclass_flag_exclude_from_sim_token.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bezier_too_many_points_failed_load_message() {
    let schematic =
        extract_parity_fixture("bezier_too_many_points/bezier_too_many_points.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_bezier_too_many_points_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_bezier_too_many_points/symbol_bezier_too_many_points.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_wire_too_many_points_failed_load_message() {
    let schematic = extract_parity_fixture("wire_too_many_points/wire_too_many_points.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bus_too_many_points_failed_load_message() {
    let schematic = extract_parity_fixture("bus_too_many_points/bus_too_many_points.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_polyline_empty_pts_failed_load_message() {
    let schematic = extract_parity_fixture("polyline_empty_pts/polyline_empty_pts.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_paper_user_missing_height_failed_load_message() {
    let schematic =
        extract_parity_fixture("paper_user_missing_height/paper_user_missing_height.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_page_user_missing_height_no_visible_diagnostics() {
    let schematic = extract_parity_fixture(
        "page_user_missing_height_no_visible_diagnostics/page_user_missing_height_no_visible_diagnostics.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_diagnostics(&schematic) else {
        return;
    };
    let ki = ki_extract_diagnostics(&schematic);

    assert_eq!(kicad.exit_code, 0);
    assert!(kicad.messages.is_empty());
    assert_eq!(ki.messages, kicad.messages);
    assert_eq!(ki.exit_code, kicad.exit_code);
}

#[test]
fn extract_matches_kicad_cli_page_user_portrait_token_failed_load_message() {
    let schematic =
        extract_parity_fixture("page_user_portrait_token/page_user_portrait_token.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_circle_extra_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_circle_extra_child/symbol_circle_extra_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_rectangle_extra_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_rectangle_extra_child/symbol_rectangle_extra_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_polyline_extra_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_polyline_extra_child/symbol_polyline_extra_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_text_extra_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_text_extra_child/symbol_text_extra_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_text_box_extra_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("symbol_text_box_extra_child/symbol_text_box_extra_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_missing_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_missing_name/embedded_files_file_missing_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_checksum_before_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_checksum_before_name/embedded_files_file_checksum_before_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_type_before_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_type_before_name/embedded_files_file_type_before_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_data_before_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_data_before_name/embedded_files_file_data_before_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_unknown_child_payload_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_unknown_child_payload/embedded_files_unknown_child_payload.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_unknown_after_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_unknown_after_name/embedded_files_file_unknown_after_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_unknown_before_name_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_unknown_before_name/embedded_files_file_unknown_before_name.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_future_sch_bare_failed_load_message() {
    let schematic = extract_parity_fixture("future_sch_bare/future_sch_bare.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_background_failed_load_message() {
    let schematic = extract_parity_fixture("text_box_background/text_box_background.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_length_failed_load_message() {
    let schematic = extract_parity_fixture("text_box_length/text_box_length.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_fields_autoplaced_failed_load_message() {
    let schematic =
        extract_parity_fixture("text_box_fields_autoplaced/text_box_fields_autoplaced.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_iref_failed_load_message() {
    let schematic = extract_parity_fixture("text_box_iref/text_box_iref.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_effects_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture("effects_unknown_child/effects_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_fill_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture("fill_unknown_child/fill_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_stroke_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture("stroke_unknown_child/stroke_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_fill_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("fill_bare_atom/fill_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_stroke_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("stroke_bare_atom/stroke_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_effects_font_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("effects_font_bare_atom/effects_font_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_effects_font_unknown_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("effects_font_unknown_child/effects_font_unknown_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_property_unknown_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_property_unknown_child/symbol_property_unknown_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_fill_color_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("fill_color_bare_atom/fill_color_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_stroke_color_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("stroke_color_bare_atom/stroke_color_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_property_bare_atom_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("property_bare_atom_child/property_bare_atom_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_text_box_margins_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("text_box_margins_bare_atom/text_box_margins_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_border_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_border_bare_atom/table_border_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_separators_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_separators_bare_atom/table_separators_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_cells_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("table_cells_bare_atom/table_cells_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_row_heights_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_row_heights_bare_atom/table_row_heights_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_col_widths_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_col_widths_bare_atom/table_col_widths_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_columns_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_columns_bare_atom/table_columns_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_rows_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("table_rows_bare_atom/table_rows_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_header_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("table_header_bare_atom/table_header_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_border_external_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_border_external_bare_atom/table_border_external_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_separators_rows_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_separators_rows_bare_atom/table_separators_rows_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_separators_cols_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_separators_cols_bare_atom/table_separators_cols_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_border_header_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_border_header_bare_atom/table_border_header_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_border_stroke_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_border_stroke_bare_atom/table_border_stroke_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_table_separators_stroke_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "table_separators_stroke_bare_atom/table_separators_stroke_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bus_alias_members_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "bus_alias_members_list_child/bus_alias_members_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_sheet_instances_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "sheet_instances_extra_bare_atom/sheet_instances_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_instances_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_instances_extra_bare_atom/symbol_instances_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_default_instance_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "default_instance_extra_bare_atom/default_instance_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_variant_extra_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("variant_extra_bare_atom/variant_extra_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_nested_instance_project_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "nested_instance_project_extra_bare_atom/nested_instance_project_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_group_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("group_extra_bare_atom/group_extra_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_sheet_property_bare_atom_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "sheet_property_bare_atom_child/sheet_property_bare_atom_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_image_data_list_child_failed_load_message() {
    let schematic = extract_parity_fixture("image_data_list_child/image_data_list_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_group_locked_bare_atom_failed_load_message() {
    let schematic =
        extract_parity_fixture("group_locked_bare_atom/group_locked_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_group_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture("group_name_list_child/group_name_list_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bus_alias_name_list_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("bus_alias_name_list_child/bus_alias_name_list_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_embedded_files_file_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "embedded_files_file_name_list_child/embedded_files_file_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_class_label_bare_failed_load_message() {
    let schematic = extract_parity_fixture("class_label_bare/class_label_bare.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_bitmap_data_list_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("bitmap_data_list_child/bitmap_data_list_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_alternate_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_pin_alternate_bare_atom/symbol_pin_alternate_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_name_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_pin_name_extra_bare_atom/symbol_pin_name_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_symbol_pin_number_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "symbol_pin_number_extra_bare_atom/symbol_pin_number_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_schematic_symbol_pin_extra_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture(
        "schematic_symbol_pin_extra_bare_atom/schematic_symbol_pin_extra_bare_atom.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_effects_justify_list_child_failed_load_message() {
    let schematic =
        extract_parity_fixture("effects_justify_list_child/effects_justify_list_child.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_pin_angles_bare_atom_failed_load_message() {
    let schematic = extract_parity_fixture("pin_angles_bare_atom/pin_angles_bare_atom.kicad_sch");
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_matches_kicad_cli_variant_field_name_list_child_failed_load_message() {
    let schematic = extract_parity_fixture(
        "variant_field_name_list_child/variant_field_name_list_child.kicad_sch",
    );
    let Some(kicad) = kicad_cli_extract_raw(&schematic) else {
        return;
    };
    let ki = ki_extract_raw(&schematic);

    assert_eq!(kicad.messages, vec!["Failed to load schematic".to_string()]);
    assert_eq!(ki.messages, kicad.messages);
}

#[test]
fn extract_parity_manifest_is_well_formed() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/extract_parity/cases.json"
    );
    let cases: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("manifest should read"))
            .expect("manifest should be valid json");
    let cases = cases.as_array().expect("manifest should be an array");
    let mut ids = HashSet::new();

    assert!(!cases.is_empty(), "parity manifest should not be empty");

    for case in cases {
        let id = case["id"].as_str().expect("case id should be string");
        assert!(ids.insert(id.to_string()), "duplicate case id: {id}");
        assert!(
            case["kicad_source"]
                .as_array()
                .is_some_and(|entries| !entries.is_empty()),
            "case should list at least one KiCad source reference"
        );
        assert!(
            case["expected_messages"].is_array(),
            "case should define expected_messages as an array"
        );
        if let Some(dir) = case["fixture_dir"].as_str() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
            assert!(
                path.exists(),
                "fixture dir should exist: {}",
                path.display()
            );
        }
    }
}
