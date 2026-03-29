mod common;

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use common::{
    extract_parity_fixture, ki_extract_diagnostics, ki_extract_raw, kicad_cli_extract_diagnostics,
    kicad_cli_extract_raw, kicad_cli_extract_raw_no_output,
};

#[derive(Clone, Copy)]
enum OracleKind {
    Diagnostics,
    Raw,
}

struct MutationCase {
    id: &'static str,
    family: &'static str,
    seed_fixture: &'static str,
    oracle: OracleKind,
    find: &'static str,
    replace: &'static str,
}

impl MutationCase {
    fn materialize(&self, temp: &TempDir) -> PathBuf {
        let seed = extract_parity_fixture(self.seed_fixture);
        let original = fs::read_to_string(&seed).expect("seed fixture should read");
        let mutated = original.replacen(self.find, self.replace, 1);
        assert_ne!(
            original, mutated,
            "mutation {} did not modify the seed fixture",
            self.id
        );

        let target = temp.path().join(format!("{}.kicad_sch", self.id));
        fs::write(&target, mutated).expect("mutated fixture should write");
        target
    }
}

fn mutation_cases() -> Vec<MutationCase> {
    vec![
        MutationCase {
            id: "mutation_invalid_power_scope",
            family: "optional-enum-token",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(symbol \"Device:R\"\n      (pin_numbers (hide yes))",
            replace: "(symbol \"Device:R\"\n      (power maybe)\n      (pin_numbers (hide yes))",
        },
        MutationCase {
            id: "mutation_invalid_pin_names_child",
            family: "optional-child-kind",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(pin_names (offset 0))",
            replace: "(pin_names (bogus))",
        },
        MutationCase {
            id: "mutation_invalid_jumper_group_member",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(duplicate_pin_numbers_are_jumpers no)",
            replace: "(duplicate_pin_numbers_are_jumpers no)\n      (jumper_pin_groups (1))",
        },
        MutationCase {
            id: "mutation_missing_bus_alias_members_keyword",
            family: "missing-structural-keyword",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "  (sheet_instances\n",
            replace: "  (bus_alias DATA\n    (bogus A0))\n  (sheet_instances\n",
        },
        MutationCase {
            id: "mutation_numeric_bus_alias_member",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "  (sheet_instances\n",
            replace: "  (bus_alias DATA\n    (members 123))\n  (sheet_instances\n",
        },
        MutationCase {
            id: "mutation_invalid_group_header",
            family: "header-token-kind",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "  (sheet_instances\n",
            replace: "  (group 123\n    (uuid \"group-1\")\n    (members))\n  (sheet_instances\n",
        },
        MutationCase {
            id: "mutation_legacy_bare_pin_names_hide",
            family: "legacy-form-acceptance",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(pin_names (offset 0))",
            replace: "(pin_names hide)",
        },
        MutationCase {
            id: "mutation_image_payload_warning",
            family: "payload-decoding",
            seed_fixture: "valid_image_fixture/valid_image_fixture.kicad_sch",
            oracle: OracleKind::Raw,
            find: "iVBORw0KGgo",
            replace: "not-base64-image",
        },
        MutationCase {
            id: "mutation_tilde_default_value",
            family: "sentinel-token",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
            replace: "(default_instance\n      (reference \"R1\")\n      (unit 1)\n      (value ~)\n      (footprint \"\"))\n    (instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
        },
        MutationCase {
            id: "mutation_numeric_symbol_lib_name",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(symbol (lib_id \"Device:R\")",
            replace: "(symbol (lib_name 123) (lib_id \"Device:R\")",
        },
        MutationCase {
            id: "mutation_numeric_symbol_uuid",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(uuid \"sym-rq\")",
            replace: "(uuid 123)",
        },
        MutationCase {
            id: "mutation_quoted_default_instance_unit",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
            replace: "(default_instance\n      (reference \"R1\")\n      (unit \"1\")\n      (value \"10k\")\n      (footprint \"\"))\n    (instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
        },
        MutationCase {
            id: "mutation_quoted_nested_instance_unit",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
            replace: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit \"1\")))))",
        },
        MutationCase {
            id: "mutation_quoted_symbol_unit",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(unit 1)",
            replace: "(unit \"1\")",
        },
        MutationCase {
            id: "mutation_float_symbol_unit",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(unit 1)",
            replace: "(unit 1.5)",
        },
        MutationCase {
            id: "mutation_quoted_symbol_body_style",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(body_style 1)",
            replace: "(body_style \"1\")",
        },
        MutationCase {
            id: "mutation_float_symbol_body_style",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(body_style 1)",
            replace: "(body_style 1.5)",
        },
        MutationCase {
            id: "mutation_float_symbol_instances_unit",
            family: "token-class-boundary",
            seed_fixture: "invalid_symbol_instances_unit/invalid_symbol_instances_unit.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(unit maybe)",
            replace: "(unit 1.5)",
        },
        MutationCase {
            id: "mutation_float_title_block_comment",
            family: "token-class-boundary",
            seed_fixture: "invalid_title_block_comment_number/invalid_title_block_comment_number.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(comment 10 \"oops\")",
            replace: "(comment 1.5 \"oops\")",
        },
        MutationCase {
            id: "mutation_quoted_title_block_comment",
            family: "quoted-vs-unquoted",
            seed_fixture: "invalid_title_block_comment_number/invalid_title_block_comment_number.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(comment 10 \"oops\")",
            replace: "(comment \"1\" \"oops\")",
        },
        MutationCase {
            id: "mutation_quoted_symbol_at_angle",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(at 50 50 0)",
            replace: "(at 50 50 \"0\")",
        },
        MutationCase {
            id: "mutation_quoted_pin_length",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(length 1.27)",
            replace: "(length \"1.27\")",
        },
        MutationCase {
            id: "mutation_quoted_pin_names_offset",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(offset 0)",
            replace: "(offset \"0\")",
        },
        MutationCase {
            id: "mutation_quoted_stroke_width",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(width 0.254)",
            replace: "(width \"0.254\")",
        },
        MutationCase {
            id: "mutation_quoted_text_font_size",
            family: "quoted-vs-unquoted",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(size 1.27 1.27)",
            replace: "(size \"1.27\" 1.27)",
        },
        MutationCase {
            id: "mutation_quoted_junction_diameter",
            family: "quoted-vs-unquoted",
            seed_fixture: "invalid_junction_diameter_token/invalid_junction_diameter_token.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(diameter nope)",
            replace: "(diameter \"1.27\")",
        },
        MutationCase {
            id: "mutation_duplicate_effects_color",
            family: "duplicate-optional-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(effects (font (size 1.27 1.27))))",
            replace: "(effects (color 0 0 0 1) (color 255 0 0 1) (font (size 1.27 1.27))))",
        },
        MutationCase {
            id: "mutation_root_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(uuid \"ann-err-root\")",
            replace: "(uuid (bad))",
        },
        MutationCase {
            id: "mutation_root_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(uuid \"ann-err-root\")",
            replace: "(uuid 123)",
        },
        MutationCase {
            id: "mutation_duplicate_table_header",
            family: "duplicate-optional-child",
            seed_fixture: "invalid_table_header_token/invalid_table_header_token.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(header maybe)",
            replace: "(headers yes) (headers no)",
        },
        MutationCase {
            id: "mutation_duplicate_table_stroke",
            family: "duplicate-optional-child",
            seed_fixture: "invalid_table_border_header_token/invalid_table_border_header_token.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(border (header maybe))",
            replace: "(stroke (width 0.254) (type default)) (stroke (width 0.3) (type default))",
        },
        MutationCase {
            id: "mutation_empty_pts_then_valid",
            family: "repeated-structural-block",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(wire (pts) (pts (xy 0 0) (xy 10 0)))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_duplicate_nested_variant",
            family: "duplicate-optional-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Diagnostics,
            find: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1)))))",
            replace: "(instances (project \"annotation_errors\" (path \"/ann-err-root\" (reference \"R?\") (unit 1) (variant (name \"A\")) (variant (name \"B\"))))))",
        },
        MutationCase {
            id: "mutation_global_label_with_directive_length",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(global_label \"NET\" (at 10 10 0) (shape input) (length 1.27) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_with_label_shape",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text \"X\" (at 10 10 0) (shape input) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_duplicate_text_href",
            family: "duplicate-optional-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text \"hello\" (at 10 10 0) (href \"a\") (href \"b\") (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_box_with_shape",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text_box \"hello\" (at 10 10 0) (size 5 5) (shape input) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_box_with_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text_box \"hello\" (at 10 10 0) (size 5 5) (href \"x\") (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_image_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "valid_image_fixture/valid_image_fixture.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(uuid bfa2aa0c-8730-4906-9ef4-dba33beae055)",
            replace: "(uuid 42)",
        },
        MutationCase {
            id: "mutation_text_box_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text_box \"hello\" (at 10 10 0) (size 5 5) (effects (href \"x\") (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_box_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text_box \"hello\" (at 10 10 0) (size 5 5) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_box_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text_box \"hello\" (at 10 10 0) (size 5 5) (uuid (bad)) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_cell_with_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (cells (table_cell \"x\" (at 10 10) (size 5 5) (href \"x\") (effects (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_cell_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (cells (table_cell \"x\" (at 10 10) (size 5 5) (effects (href \"x\") (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_cell_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (cells (table_cell \"x\" (at 10 10) (size 5 5) (uuid 42) (effects (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_cell_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (cells (table_cell \"x\" (at 10 10) (size 5 5) (uuid (bad)) (effects (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_junction_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(junction (at 10 10) (diameter 1.27) (color 0 0 0 0) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_no_connect_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(no_connect (at 10 10) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_bus_entry_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(bus_entry (at 10 10) (size 1.27 1.27) (stroke (width 0) (type default)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_wire_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(wire (pts (xy 0 0) (xy 10 0)) (stroke (width 0) (type default)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_bus_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(bus (pts (xy 0 0) (xy 10 0)) (stroke (width 0) (type default)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_polyline_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(polyline (pts (xy 0 0) (xy 10 0) (xy 10 10)) (stroke (width 0) (type default)) (fill (type none)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_arc_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(arc (start 0 0) (mid 5 5) (end 10 0) (stroke (width 0) (type default)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_circle_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(circle (center 0 0) (radius 5) (stroke (width 0) (type default)) (fill (type none)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_rectangle_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(rectangle (start 0 0) (end 10 10) (stroke (width 0) (type default)) (fill (type none)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_bezier_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(bezier (pts (xy 0 0) (xy 3 3) (xy 7 3) (xy 10 0)) (stroke (width 0) (type default)) (fill (type none)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_text_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text \"hi\" (at 10 10 0) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_group_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(group \"g\" (uuid (bad)) (members))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_rule_area_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(rule_area (pts (xy 0 0) (xy 10 0) (xy 10 10) (xy 0 10)) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_rule_area_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(rule_area (pts (xy 0 0) (xy 10 0) (xy 10 10) (xy 0 10)) (uuid (bad)))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (uuid 42) (cells (table_cell \"x\" (at 10 10) (size 5 5) (effects (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_table_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(table (column 1) (row 1) (uuid (bad)) (cells (table_cell \"x\" (at 10 10) (size 5 5) (effects (font (size 1.27 1.27))))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid 42))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid (bad)))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_pin_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid \"s1\") (pin \"SIG\" input (at 10 20 0) (uuid 42)))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_pin_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid \"s1\") (pin \"SIG\" input (at 10 20 0) (uuid (bad))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_pin_with_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid \"s1\") (pin \"SIG\" input (at 10 20 0) (href \"x\")))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_sheet_pin_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid \"s1\") (pin \"SIG\" input (at 10 20 0) (effects (href \"x\") (font (size 1.27 1.27)))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_label_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(label \"NET\" (at 10 10 0) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_label_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(label \"NET\" (at 10 10 0) (uuid (bad)) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_global_label_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(global_label \"NET\" (shape input) (at 10 10 0) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_global_label_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(global_label \"NET\" (shape input) (at 10 10 0) (uuid (bad)) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_hier_label_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(hierarchical_label \"NET\" (shape input) (at 10 10 0) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_hier_label_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(hierarchical_label \"NET\" (shape input) (at 10 10 0) (uuid (bad)) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_directive_label_uuid_numeric",
            family: "token-class-boundary",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(directive_label \"NET\" (shape input) (at 10 10 0) (uuid 42) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_directive_label_uuid_list",
            family: "list-vs-atom",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(directive_label \"NET\" (shape input) (at 10 10 0) (uuid (bad)) (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_label_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(label \"NET\" (at 10 10 0) (effects (href \"x\") (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_global_label_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(global_label \"NET\" (shape input) (at 10 10 0) (effects (href \"x\") (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_hier_label_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(hierarchical_label \"NET\" (shape input) (at 10 10 0) (effects (href \"x\") (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "mutation_directive_label_effects_href",
            family: "cross-context-child",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(directive_label \"NET\" (shape input) (at 10 10 0) (effects (href \"x\") (font (size 1.27 1.27))))\n  (sheet_instances",
        },
    ]
}

fn compare_mutation(case: &MutationCase, schematic: &Path) -> Option<String> {
    match case.oracle {
        OracleKind::Diagnostics => {
            let Some(kicad) = kicad_cli_extract_diagnostics(schematic) else {
                return None;
            };
            let ki = ki_extract_diagnostics(schematic);

            if (kicad.exit_code == 0) != (ki.exit_code == 0) || kicad.messages != ki.messages {
                return Some(format!(
                    "{} [{}]\n  seed: {}\n  kicad: exit={} messages={:?}\n  ki: exit={} messages={:?}",
                    case.id,
                    case.family,
                    case.seed_fixture,
                    kicad.exit_code,
                    kicad.messages,
                    ki.exit_code,
                    ki.messages
                ));
            }
        }
        OracleKind::Raw => {
            let Some(kicad) = kicad_cli_extract_raw(schematic) else {
                return None;
            };
            let ki = ki_extract_raw(schematic);

            if (kicad.exit_code == 0) != (ki.exit_code == 0) || kicad.messages != ki.messages {
                return Some(format!(
                    "{} [{}]\n  seed: {}\n  kicad: exit={} messages={:?}\n  ki: exit={} messages={:?}",
                    case.id,
                    case.family,
                    case.seed_fixture,
                    kicad.exit_code,
                    kicad.messages,
                    ki.exit_code,
                    ki.messages
                ));
            }
        }
    }

    None
}

fn dual_oracle_cases() -> Vec<MutationCase> {
    vec![
        MutationCase {
            id: "dual_text_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(text \"NET\" (at 10 10 0) (href \"x\") (effects (font (size 1.27 1.27))))\n  (sheet_instances",
        },
        MutationCase {
            id: "dual_symbol_property_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(property \"Reference\" \"R?\" (at 47.968 50 90) (show_name no) (do_not_autoplace no) (effects (font (size 1.27 1.27))))",
            replace: "(property \"Reference\" \"R?\" (at 47.968 50 90) (show_name no) (do_not_autoplace no) (href \"x\") (effects (font (size 1.27 1.27))))",
        },
        MutationCase {
            id: "dual_sheet_property_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(sheet_instances",
            replace: "(sheet (at 10 10) (size 20 20) (uuid \"s\") (property \"Sheetname\" \"S\" (at 10 10 0) (href \"x\") (effects (font (size 1.27 1.27)))))\n  (sheet_instances",
        },
        MutationCase {
            id: "dual_lib_symbol_property_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(property \"Reference\" \"R\" (at 2.032 0 90) (show_name no) (do_not_autoplace no) (effects (font (size 1.27 1.27))))",
            replace: "(property \"Reference\" \"R\" (at 2.032 0 90) (show_name no) (do_not_autoplace no) (href \"x\") (effects (font (size 1.27 1.27))))",
        },
        MutationCase {
            id: "dual_lib_pin_name_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(name \"\" (effects (font (size 1.27 1.27))))",
            replace: "(name \"\" (href \"x\") (effects (font (size 1.27 1.27))))",
        },
        MutationCase {
            id: "dual_lib_pin_number_with_href",
            family: "cli-shape",
            seed_fixture: "annotation_errors/annotation_errors.kicad_sch",
            oracle: OracleKind::Raw,
            find: "(number \"1\" (effects (font (size 1.27 1.27))))",
            replace: "(number \"1\" (href \"x\") (effects (font (size 1.27 1.27))))",
        },
    ]
}

#[test]
fn extract_mutation_matrix_matches_kicad_cli() {
    let mut mismatches = Vec::new();

    for case in mutation_cases() {
        let temp = TempDir::new().expect("tempdir should exist");
        let schematic = case.materialize(&temp);

        if let Some(mismatch) = compare_mutation(&case, &schematic) {
            mismatches.push(mismatch);
        }
    }

    assert!(
        mismatches.is_empty(),
        "mutation mismatches detected:\n{}",
        mismatches.join("\n\n")
    );
}

#[test]
fn extract_dual_cli_shape_matrix_is_stable() {
    let mut mismatches = Vec::new();

    for case in dual_oracle_cases() {
        let temp = TempDir::new().expect("tempdir should exist");
        let schematic = case.materialize(&temp);

        let Some(with_output) = kicad_cli_extract_raw(&schematic) else {
            return;
        };
        let Some(without_output) = kicad_cli_extract_raw_no_output(&schematic) else {
            return;
        };

        if with_output.exit_code != without_output.exit_code
            || with_output.messages != without_output.messages
        {
            mismatches.push(format!(
                "{} [{}]\n  seed: {}\n  with -o: exit={} messages={:?}\n  without -o: exit={} messages={:?}",
                case.id,
                case.family,
                case.seed_fixture,
                with_output.exit_code,
                with_output.messages,
                without_output.exit_code,
                without_output.messages
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "dual cli-shape mismatches detected:\n{}",
        mismatches.join("\n\n")
    );
}
