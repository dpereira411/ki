# KiCad ERC Parity Inventory

This inventory tracks source-backed ERC parity work for `ki schematic erc`.

Final search notes for the last deep source/CLI sweep are recorded in
`tests/erc_parity/search_audit.md`.

## Implemented Cases

### `missing_input_file`

- KiCad source: `/Users/Daniel/Desktop/kicad/kicad/cli/command_sch_erc.cpp`
- Trigger: input schematic path does not exist
- KiCad message:
  `Schematic file does not exist or is not accessible`
- Severity: error
- Fixture: none
- Status: parity-passing

### `basic_test_errors_only`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:60`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:47`
- Trigger: run ERC on KiCad's `basic_test` sample with `--severity-error`
- Expected KiCad findings:
  - two `power_pin_not_driven` violations for `U1` pins `5` and `2`
- Severity: error only
- Fixture: `tests/fixtures/erc_parity/basic_test_errors_only`
- Oracle: `tests/fixtures/erc_parity/basic_test_errors_only/oracle.json`
- Status: parity-passing

### `missing_symbol_library_in_configuration`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a schematic symbol uses embedded lib id `MissingLib:R` with no project symbol-table entry
- Expected KiCad finding:
  - `lib_symbol_issues` warning for `R1`
- Fixture: `tests/fixtures/erc_parity/missing_symbol_library_in_configuration`
- Oracle: `tests/fixtures/erc_parity/missing_symbol_library_in_configuration/oracle.json`
- Status: parity-passing

### `pin_not_connected`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
- Trigger: a single passive pin is left floating without a no-connect marker
- Expected KiCad findings:
  - `pin_not_connected` error for `TP1` pin `1`
  - `lib_symbol_issues` warning for `OnePin` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/pin_not_connected`
- Oracle: `tests/fixtures/erc_parity/pin_not_connected/oracle.json`
- Status: parity-passing

### `endpoint_off_grid`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1966`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:48`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:43`
- Trigger: a symbol pin endpoint is placed off the 1.27 mm connection grid
- Expected KiCad findings:
  - `endpoint_off_grid` warning for `R1` pin `1`
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/endpoint_off_grid`
- Oracle: `tests/fixtures/erc_parity/endpoint_off_grid/oracle.json`
- Status: parity-passing

### `single_global_label`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3473`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4366`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:124`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:56`
- Trigger: exactly one explicit global label with a given name appears in the schematic
- Expected KiCad findings:
  - `single_global_label` warning for `SIG`
  - two `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/single_global_label`
- Oracle: `tests/fixtures/erc_parity/single_global_label/oracle.json`
- Status: parity-passing

### `four_way_junction`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:872`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:88`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:91`
- Trigger: four distinct connectable items meet at the same connection point without an explicit junction split
- Expected KiCad findings:
  - `four_way_junction` warning at the center crosspoint
  - four `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/four_way_junction`
- Oracle: `tests/fixtures/erc_parity/four_way_junction/oracle.json`
- Status: parity-passing

### `label_dangling`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4198`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:104`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:52`
- Trigger: a local label is placed with no wire, pin, or no-connect touching its anchor point
- Expected KiCad findings:
  - `label_dangling` error for `SIG`
- Fixture: `tests/fixtures/erc_parity/label_dangling`
- Oracle: `tests/fixtures/erc_parity/label_dangling/oracle.json`
- Status: parity-passing

### `isolated_pin_label`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4310`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4066`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:108`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:244`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:76`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:93`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a local label sits on a net with exactly one pin and the same wire also has a dangling endpoint
- Expected KiCad findings:
  - `isolated_pin_label` warning for `SIG`
  - `unconnected_wire_endpoint` warning for the short horizontal wire
  - `lib_symbol_issues` warning for `TP1` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/isolated_pin_label`
- Oracle: `tests/fixtures/erc_parity/isolated_pin_label/oracle.json`
- Status: parity-passing

### `no_connect_dangling`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3877`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:100`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:51`
- Trigger: a no-connect marker is placed without any pin, label, or wire endpoint attached
- Expected KiCad findings:
  - `no_connect_dangling` warning for the lone marker
- Fixture: `tests/fixtures/erc_parity/no_connect_dangling`
- Oracle: `tests/fixtures/erc_parity/no_connect_dangling/oracle.json`
- Status: parity-passing

### `no_connect_connected`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3851`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:96`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:50`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a no-connect marker is attached to a real two-pin net at one pin location
- Expected KiCad findings:
  - `no_connect_connected` warning on `TP1` pin `1` plus the marker
  - two `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/no_connect_connected`
- Oracle: `tests/fixtures/erc_parity/no_connect_connected/oracle.json`
- Status: parity-passing

### `label_multiple_wires`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:795`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:92`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:92`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a local label is placed at a point where more than one wire passes underneath it without using wire endpoints
- Expected KiCad findings:
  - `label_multiple_wires` warning for `SIG`
  - four `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/label_multiple_wires`
- Oracle: `tests/fixtures/erc_parity/label_multiple_wires/oracle.json`
- Status: parity-passing

### `wire_dangling`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4113`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4072`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:188`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:244`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:80`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:93`
- Trigger: a wire-only subgraph exists with no pin, label, or other driver attached
- Expected KiCad findings:
  - `wire_dangling` error for the isolated wire
  - two `unconnected_wire_endpoint` warnings, one for each dangling endpoint
- Fixture: `tests/fixtures/erc_parity/wire_dangling`
- Oracle: `tests/fixtures/erc_parity/wire_dangling/oracle.json`
- Status: parity-passing

### `similar_labels`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1571`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:112`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:53`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4198`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:104`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:52`
- Trigger: two labels on the same sheet differ only by case
- Expected KiCad findings:
  - two `label_dangling` errors because the minimal fixture uses naked labels
  - one `similar_labels` warning comparing `SIG` and `sig`
- Fixture: `tests/fixtures/erc_parity/similar_labels`
- Oracle: `tests/fixtures/erc_parity/similar_labels/oracle.json`
- Status: parity-passing

### `field_name_whitespace`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:467`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:172`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:95`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
- Trigger: a symbol has a custom property whose field name includes leading and trailing spaces
- Expected KiCad findings:
  - `field_name_whitespace` warning for ` Custom `
  - `pin_not_connected` error for the probe pin in this minimal fixture
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/field_name_whitespace`
- Oracle: `tests/fixtures/erc_parity/field_name_whitespace/oracle.json`
- Status: parity-passing

### `same_local_global_label`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1538`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:128`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:57`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4198`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:104`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:52`
- Trigger: a local label and a global label use the exact same name on the same schematic
- Expected KiCad findings:
  - `same_local_global_label` warning comparing the two labels
  - one `label_dangling` error for the naked local label in this minimal fixture
- Fixture: `tests/fixtures/erc_parity/same_local_global_label`
- Oracle: `tests/fixtures/erc_parity/same_local_global_label/oracle.json`
- Status: parity-passing

### `pin_not_driven`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1192`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:56`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:45`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a net connects two input pins and no output-capable or passive driver
- Expected KiCad findings:
  - one `pin_not_driven` error on `U1` pin `1`
  - two `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/pin_not_driven`
- Oracle: `tests/fixtures/erc_parity/pin_not_driven/oracle.json`
- Status: parity-passing

### `stacked_pin_name`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1460`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:168`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:94`
  - `/Users/Daniel/Desktop/kicad/common/string_utils.cpp:1589`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
- Trigger: a symbol pin number uses malformed stacked-pin notation such as `[3-1]`
- Expected KiCad findings:
  - `stacked_pin_name` warning on the malformed pin
  - `pin_not_connected` error for the same floating pin in this minimal fixture
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/stacked_pin_name`
- Oracle: `tests/fixtures/erc_parity/stacked_pin_name/oracle.json`
- Status: parity-passing

### `footprint_link_invalid_identifier`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1818`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:83`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
- Trigger: a symbol footprint field contains a malformed identifier with no valid library nickname
- Expected KiCad findings:
  - `footprint_link_issues` warning reporting missing footprint library `''`
  - `pin_not_connected` error for the same floating pin in this minimal fixture
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/footprint_link_invalid_identifier`
- Oracle: `tests/fixtures/erc_parity/footprint_link_invalid_identifier/oracle.json`
- Status: parity-passing

### `similar_label_and_power`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1581`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:120`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:55`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:60`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:47`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a local label differs only by case from the name of an isolated power symbol
- Expected KiCad findings:
  - `similar_label_and_power` warning comparing `gnd` with the power pin
  - `label_dangling`, `pin_not_connected`, and `power_pin_not_driven` on the isolated items
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/similar_label_and_power`
- Oracle: `tests/fixtures/erc_parity/similar_label_and_power/oracle.json`
- Status: parity-passing

### `similar_power`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1577`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:116`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:54`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:60`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:47`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: two isolated power symbols differ only by case, such as `GND` and `gnd`
- Expected KiCad findings:
  - `similar_power` warning comparing the two power pins
  - `pin_not_connected` and `power_pin_not_driven` for both isolated power symbols
  - `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/similar_power`
- Oracle: `tests/fixtures/erc_parity/similar_power/oracle.json`
- Status: parity-passing

### `missing_input_pin`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:571`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:216`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:220`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:88`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:89`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a 2-unit symbol has only unit `A` placed, while unit `B` contains an input pin
- Expected KiCad findings:
  - `missing_unit` warning reporting unplaced unit `[ B ]`
  - `missing_input_pin` warning reporting input pins in unplaced unit `[ B ]`
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/missing_input_pin`
- Oracle: `tests/fixtures/erc_parity/missing_input_pin/oracle.json`
- Status: parity-passing

### `missing_bidi_pin`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:571`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:216`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:224`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:88`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:90`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a 2-unit symbol has only unit `A` placed, while unit `B` contains a bidirectional pin
- Expected KiCad findings:
  - `missing_unit` warning reporting unplaced unit `[ B ]`
  - `missing_bidi_pin` warning reporting bidirectional pins in unplaced unit `[ B ]`
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/missing_bidi_pin`
- Oracle: `tests/fixtures/erc_parity/missing_bidi_pin/oracle.json`
- Status: parity-passing

### `missing_power_pin`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:571`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:216`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:228`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:88`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:91`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a 2-unit symbol has only unit `A` placed, while unit `B` contains a power input pin
- Expected KiCad findings:
  - `missing_unit` warning reporting unplaced unit `[ B ]`
  - `missing_power_pin` error reporting power-input pins in unplaced unit `[ B ]`
  - `lib_symbol_issues` warning for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/missing_power_pin`
- Oracle: `tests/fixtures/erc_parity/missing_power_pin/oracle.json`
- Status: parity-passing

### `different_unit_footprint`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:510`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:136`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:59`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1818`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:200`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:83`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: two placed units of the same symbol reference assign different footprint fields
- Expected KiCad findings:
  - `different_unit_footprint` error comparing `U1A` and `U1B`
  - two `footprint_link_issues` warnings because the fixture uses nonexistent footprint libraries
  - two `lib_symbol_issues` warnings for `MissingLib` because this fixture intentionally has no active symbol-library entry
- Fixture: `tests/fixtures/erc_parity/different_unit_footprint`
- Oracle: `tests/fixtures/erc_parity/different_unit_footprint/oracle.json`
- Status: parity-passing

### `ground_pin_not_ground`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1376`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:164`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:75`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:60`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:47`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a symbol has one power pin on a ground net and another power pin named `GND` on a non-ground net
- Expected KiCad findings:
  - `ground_pin_not_ground` warning on the mismatched `GND` pin
  - `power_pin_not_driven` errors on both isolated power-input nets
  - three `lib_symbol_issues` warnings for the intentionally missing libraries
- Fixture: `tests/fixtures/erc_parity/ground_pin_not_ground`
- Oracle: `tests/fixtures/erc_parity/ground_pin_not_ground/oracle.json`
- Status: parity-passing

### `undefined_netclass`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:705`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:180`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:77`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a symbol instance has a `Netclass` property that names a class absent from the project net settings
- Expected KiCad findings:
  - `undefined_netclass` error for `BogusClass`
  - `pin_not_connected` error for the floating one-pin probe in this minimal fixture
  - `lib_symbol_issues` warning because this fixture intentionally omits a configured `OnePin` library
- Fixture: `tests/fixtures/erc_parity/undefined_netclass`
- Oracle: `tests/fixtures/erc_parity/undefined_netclass/oracle.json`
- Status: parity-passing

### `unresolved_variable`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:186`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:176`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:96`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a symbol property contains an unresolved text variable such as `${DOES_NOT_EXIST}`
- Expected KiCad findings:
  - `unresolved_variable` error on the symbol
  - `pin_not_connected` error for the floating one-pin probe in this minimal fixture
  - `lib_symbol_issues` warning because this fixture intentionally omits a configured `OnePin` library
- Fixture: `tests/fixtures/erc_parity/unresolved_variable`
- Oracle: `tests/fixtures/erc_parity/unresolved_variable/oracle.json`
- Status: parity-passing

### `generic_warning`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:201`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:76`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a symbol property contains `${ERC_WARNING Custom warning}`
- Expected KiCad findings:
  - `generic-warning` warning with description `Custom warning`
  - `pin_not_connected` error for the floating one-pin probe in this minimal fixture
  - `lib_symbol_issues` warning because this fixture intentionally omits a configured `OnePin` library
- Fixture: `tests/fixtures/erc_parity/generic_warning`
- Oracle: `tests/fixtures/erc_parity/generic_warning/oracle.json`
- Status: parity-passing

### `generic_error`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:225`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:80`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:52`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: a symbol property contains `${ERC_ERROR Custom error}`
- Expected KiCad findings:
  - `generic-error` error with description `Custom error`
  - `pin_not_connected` error for the floating one-pin probe in this minimal fixture
  - `lib_symbol_issues` warning because this fixture intentionally omits a configured `OnePin` library
- Fixture: `tests/fixtures/erc_parity/generic_error`
- Oracle: `tests/fixtures/erc_parity/generic_error/oracle.json`
- Status: parity-passing

## Deferred Findings

The following KiCad ERC finding families are documented for follow-up but not implemented in this
first slice.

### `pin_to_pin`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1094`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1184`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:68`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:72`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:48`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:49`
- Trigger: incompatible pin electrical types are connected on the same net
- Confirmed KiCad CLI fixture:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/erc_multiple_pin_to_pin.kicad_sch`
- Observed KiCad CLI findings:
  - two `pin_to_pin` errors
- Fixture: `tests/fixtures/erc_parity/pin_to_pin`
- Oracle: `tests/fixtures/erc_parity/pin_to_pin/oracle.json`
- Status: parity-passing

### `power_pin_not_driven`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1202`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:60`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:47`
- Trigger: a power input pin is connected, but the net has no power-output driver
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/basic_test_errors_only/basic_test.kicad_sch`
- Observed KiCad CLI findings:
  - two `power_pin_not_driven` errors for `U1` pins `5` and `2`
- Status: parity-passing

### `lib_symbol_issues`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1686`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:192`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:81`
- Trigger: the symbol library nickname in a schematic symbol cannot be resolved, or the symbol cannot be found in that library
- Confirmed KiCad CLI fixtures:
  - `tests/fixtures/erc_parity/missing_symbol_library_in_configuration/missing_symbol_library_in_configuration.kicad_sch`
  - `tests/fixtures/erc_parity/basic_test_errors_only/basic_test.kicad_sch`
- Observed KiCad CLI findings:
  - warning text like `The current configuration does not include the symbol library 'MissingLib'`
  - warning text like `Symbol 'X' not found in symbol library 'Y'`
- Status: parity-passing

### `missing_unit`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:571`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:216`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:65`
- Trigger: a multi-unit symbol has library-defined units that are not placed in the schematic
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/missing_unit/missing_unit.kicad_sch`
- Observed KiCad CLI behavior:
  - `missing_unit` warning: `Symbol U1 has unplaced units [ B ]`
  - accompanying `lib_symbol_issues` warning for `MissingLib` because this minimal fixture intentionally has no active symbol-library entry
- Status: parity-passing

### `unconnected_wire_endpoint`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4066`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:244`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:93`
- Trigger: a wire endpoint does not connect to any other connectable item
- Confirmed KiCad CLI fixtures:
  - `tests/fixtures/erc_parity/wire_dangling/wire_dangling.kicad_sch`
  - `tests/fixtures/erc_parity/isolated_pin_label/isolated_pin_label.kicad_sch`
- Observed KiCad CLI findings:
  - one warning per dangling endpoint
- Status: parity-passing

### `lib_symbol_mismatch`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1759`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:196`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:82`
- Trigger: a placed symbol differs from the current library definition while the library nickname resolves
- Confirmed KiCad CLI fixtures:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/erc_multiple_pin_to_pin.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/unconnected_bus_entry_qa.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/multinetclasses/multinetclasses.kicad_sch`
- Observed KiCad CLI behavior:
  - warning text like `Symbol 'Conn_01x01' doesn't match copy in library 'Connector_Generic'`
  - on KiCad `10.0.0`, `multinetclasses.kicad_sch` no longer emits `lib_symbol_mismatch`; the family remains confirmed by the other QA fixtures above
- Current native `ki` status:
  - project-local `${KIPRJMOD}` symbol library resolution is implemented for ERC
  - `tests/fixtures/erc_parity/lib_symbol_mismatch/lib_symbol_mismatch.kicad_sch` now emits native `lib_symbol_mismatch` instead of falling back to `lib_symbol_issues`
  - official KiCad CLI still does not reproduce that repo-local fixture without promoting the library into global config
  - global KiCad symbol libraries now resolve natively on demand, which makes native ERC match the official CLI on `/Users/Daniel/Desktop/kicad/qa/data/eeschema/erc_multiple_pin_to_pin.kicad_sch`
- Status: parity-passing

### `footprint_filter`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1887`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:204`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:85`
- Trigger: the assigned footprint resolves successfully but does not match the symbol's footprint filters
- Confirmed KiCad CLI fixture:
  - synthetic probe project at `/tmp/erc_probe_footprint_filter_builtin/footprint_filter_builtin.kicad_sch`
- Observed KiCad CLI behavior:
  - emitted as `type: "footprint_link_issues"` with description `Assigned footprint (to-92_inline) doesn't match footprint filters (R_*)`
  - accompanied by `pin_not_connected` and `lib_symbol_mismatch` in the synthetic probe
- Current native `ki` status:
  - project-local footprint libraries from `fp-lib-table` resolve during ERC
  - `tests/fixtures/erc_parity/footprint_filter/footprint_filter.kicad_sch` emits native `footprint_link_issues` with `Assigned footprint (PkgA) doesn't match footprint filters (R_*)`
  - native output does not emit the bogus missing-footprint-library warning that official KiCad CLI reports for the same local fixture
  - built-in KiCad footprint libraries now resolve natively on demand, matching the official CLI on `tests/fixtures/erc_parity/footprint_filter_builtin/footprint_filter_builtin.kicad_sch`
- Fixture: `tests/fixtures/erc_parity/footprint_filter_builtin`
- Oracle: `tests/fixtures/erc_parity/footprint_filter_builtin/oracle.json`
- Status: parity-passing

### `undefined_netclass`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:705`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:182`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:79`
- Trigger: a netclass name is referenced in the schematic but missing from the project netclass configuration
- Confirmed KiCad CLI fixture:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/multinetclasses/multinetclasses.kicad_sch`
- Observed KiCad CLI findings:
  - seven `undefined_netclass` errors alongside connectivity warnings
- Current native `ki` status:
  - the dedicated repo fixture `tests/fixtures/erc_parity/undefined_netclass` is already parity-passing
  - the QA project remains useful as a larger confirmation case for future broad-coverage work
- Status: parity-passing

### `different_unit_net`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1243`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:140`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:66`
- Trigger: shared pins on different units of the same multi-unit symbol resolve to different nets
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/different_unit_net/different_unit_net.kicad_sch`
- Observed KiCad CLI findings:
  - `different_unit_net` error: `Pin 1 is connected to both /NET_A and /NET_B`
  - accompanying `isolated_pin_label`, `unconnected_wire_endpoint`, and `lib_symbol_issues` warnings in the minimal fixture
- Status: parity-passing

### `hier_label_mismatch`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3467`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:4497`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:84`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:49`
- Trigger: a hierarchical sheet pin and a hierarchical label disagree in name or connectivity across sheet boundaries
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/hier_label_mismatch/hier_label_mismatch.kicad_sch`
- Observed KiCad CLI findings:
  - one `hier_label_mismatch` error: `Hierarchical label OUT has no matching sheet pin in the parent sheet`
  - accompanying child-sheet `label_dangling` and `unconnected_wire_endpoint` findings in the root report for the minimal fixture
- Status: parity-passing

### `same_local_global_power`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1538`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:132`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:58`
- Trigger: a local power symbol and a global power symbol share the same name
- Confirmed KiCad internal QA:
  - `/Users/Daniel/Desktop/kicad/qa/tests/eeschema/erc/test_erc_label_names.cpp`
  - expected count: two `ERCE_SAME_LOCAL_GLOBAL_POWER` violations when the rule is enabled
- Current CLI status:
  - the repo fixture `tests/fixtures/erc_parity/same_local_global_power/same_local_global_power.kicad_sch` now loads in the official CLI
  - under KiCad `10.0.0`, `/Users/Daniel/Desktop/kicad/qa/data/eeschema/same_local_global_power.kicad_sch` emits `lib_symbol_mismatch` and `no_connect_connected`
  - no tested official CLI path emits `same_local_global_power`
  - this currently behaves as a source-tested internal ERC path that is not reproducible through `kicad-cli sch erc`
- Status: not-applicable

### `duplicate_sheet_names`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:132`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2091`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:44`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:42`
- Trigger: two hierarchical sheets under the same parent sheet use the same sheet name
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/duplicate_sheet_names/duplicate_sheet_names.kicad_sch`
- Observed KiCad CLI finding:
  - one `duplicate_sheet_names` error with two `Hierarchical Sheet 'dup'` items
- Status: parity-passing

### `duplicate_pins`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1294`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2140`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:64`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:99`
- Trigger: a symbol contains duplicate pin numbers and those duplicate pins resolve to different nets
- Confirmed KiCad CLI fixture:
  - `tests/fixtures/erc_parity/duplicate_pins/duplicate_pins.kicad_sch`
- Observed KiCad CLI finding:
  - one `duplicate_pins` error, e.g. `Pin 1 on symbol 'U1' is connected to different nets: Net-(U1-A) and Net-(U1-B)`
- Status: parity-passing

### `multiple_net_names`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3410`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3505`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:148`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:69`
- Trigger: more than one bus or net name driver is attached to the same connected items
- Confirmed KiCad CLI fixtures:
  - `tests/fixtures/erc_parity/multiple_net_names/multiple_net_names.kicad_sch`
- Observed KiCad CLI findings:
  - label-vs-label form, e.g. `Both A0 and A2 are attached to the same items; A0 will be used in the netlist`
  - this minimal fixture also carries two `lib_symbol_issues` warnings because it intentionally has no active symbol-library entry
- Status: parity-passing

### `bus_to_net_conflict`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3436`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3524`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:160`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:73`
- Trigger: a bus item and a scalar net item are graphically connected in the same subgraph
- Confirmed KiCad CLI fixtures:
  - `tests/fixtures/erc_parity/bus_to_net_conflict/bus_to_net_conflict.kicad_sch`
- Observed KiCad CLI finding:
  - `bus_to_net_conflict` with description `Invalid connection between bus and net items`
- Status: parity-passing

### `net_not_bus_member`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3442`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3665`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:152`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:70`
- Trigger: a bus entry connects a scalar net to a bus, but that net is not a member of the bus definition
- Confirmed KiCad CLI fixtures:
  - `tests/fixtures/erc_parity/net_not_bus_member/net_not_bus_member.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/prefix_bus_alias/subsheet2.kicad_sch`
- Observed KiCad CLI findings:
  - `net_not_bus_member`, e.g. `Net /SIG is graphically connected to bus /A[1..2] but is not a member of that bus`
- Oracle: `tests/fixtures/erc_parity/net_not_bus_member/oracle.json`
- Status: parity-passing

### `bus_definition_conflict`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:144`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:68`
- Trigger: conflicting bus alias definitions across sheets
- Current source/CLI status:
  - `ERCE_BUS_ALIAS_CONFLICT` exists in KiCad’s ERC registry, but no confirmed emitting site was found during source tracing
  - no official CLI JSON fixture has been reproduced yet
  - this currently appears to be a registered-but-unemitted rule key rather than an active CLI ERC family
- Status: not-applicable

### `bus_entry_needed`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:240`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:90`
- Trigger: a bus entry should exist but was not auto-placed by an importer/conversion path
- Confirmed KiCad source status:
  - ERC registry entry exists, but the observed emitting sites are in `eeschema/sch_io/eagle/sch_io_eagle.cpp`
  - no normal `.kicad_sch` CLI ERC producer was found during source tracing or project sweeps
  - this should be treated as an importer-side conversion artifact, not a standalone schematic ERC parity target
- Status: not-applicable

### `bus_to_bus_conflict`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3427`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3636`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:156`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:71`
- Trigger: two bus objects are graphically connected but share no common bus members
- Confirmed KiCad source status:
  - ERC registry entry exists, but no clean standalone CLI fixture has been reproduced yet
  - synthetic probes with vector buses, group buses, sheet pins, and child-sheet hierarchical buses all resolved to other observable findings such as `multiple_net_names` or `pin_not_connected`
  - targeted official CLI probes against likely QA projects and a broader `qa/data/eeschema` sweep did not yield any `bus_to_bus_conflict` JSON marker
  - this is currently source-defined but not observed as a reproducible `kicad-cli sch erc` family
- Status: not-applicable

### `simulation_model_issue`

- KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2061`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:184`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:79`
- Trigger: KiCad detects an error while creating or validating a SPICE simulation model for a symbol
- Current CLI status:
  - the rule is present and emitted by source through `SIM_LIB_MGR::CreateModel`
  - repeated probes across the full `qa/data/eeschema/spice_netlists` corpus still yielded zero `simulation_model_issue` JSON findings from the official CLI
  - this is currently source-defined but not observed as a reproducible `kicad-cli sch erc` family
- Status: not-applicable

Additional ERC families still lacking a confirmed CLI fixture in this repo but present in KiCad sources:

- `hier_label_mismatch`
- `bus_definition_conflict`
- `bus_entry_needed`
- `bus_to_bus_conflict`
- `footprint_filter`
- `simulation_model_issue`
- exclusion reporting
- multi-sheet ERC behavior

KiCad source families currently excluded from CLI ERC parity because `kicad-cli sch erc` does not call `SCH_REFERENCE_LIST::CheckAnnotation`:

- `duplicate_reference`
- `unannotated`
- `extra_units`
- `different_unit_value`

Additional source/CLI notes:

- `bus_definition_conflict` currently has no emitting site in the observed KiCad `eeschema` sources. `ERCE_BUS_ALIAS_CONFLICT` exists in `erc_settings.h` and `erc_item.cpp`, but no producer was found during source tracing.
- `footprint_filter` is CLI-visible, but KiCad serializes it under `type: "footprint_link_issues"` with a footprint-filter-specific message rather than as a dedicated `footprint_filter` type.
- `simulation_model_issue` has a dedicated producer in `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2056`, but repeated probes across KiCad QA SPICE projects that enable the rule did not yield any official CLI JSON findings of that type.
