# ki

`ki` is a Rust CLI for KiCad project, schematic, PCB, symbol-library, and library-table
operations.

KiCad is still primarily built around the interactive editor, not the CLI, so
`kicad-cli` is useful but not the center of the project. `ki` treats CLI compatibility as a
baseline and then fills in the gaps that matter for scripted and agent-driven workflows.


## Scope

Intent:

- match `kicad-cli` funccionlaity for the implemented command surfaces and data paths
- provide additional automation and agent workflow commands on top of that compatibility baseline

Current Features:

- inspect, validate, and modify KiCad files directly
- drive KiCad editor refresh through IPC (for machine-human feedback loops)

Parsing, validation, and command behavior are derived from upstream KiCad where this tool aims to
be compatible.


## Command Surface

Current top-level commands:

- `ki refresh`
- `ki extract`
- `ki project`
- `ki schematic`
- `ki symbol-lib`
- `ki pcb`
- `ki lib-table`

## Repository Layout

This CLI depends on the local sibling checkout:

```text
../kiutils-rs/crates/kiutils
```

`kiutils-rs` provides the file-model and editing primitives used by this CLI. This repository
contains the CLI layer, validation logic, parity tests, and command behavior.

## Build

```bash
just build
```

## Common Commands

Refresh the PCB editor through KiCad IPC:

```bash
just run refresh --frame pcb
```

Inspect a schematic as JSON:

```bash
just run schematic inspect path/to/file.kicad_sch --json
```

Extract schematic netlist/topology data as JSON:

```bash
just run extract path/to/file.kicad_sch --pretty
```

Run ERC with JSON output:

```bash
just run schematic erc path/to/file.kicad_sch --json
```

Validate a project:

```bash
just run project validate path/to/project.kicad_pro --json
```

Add a PCB trace:

```bash
just run pcb add-trace board.kicad_pcb 10 10 20 10 0.25 F.Cu 1
```

## Extract Output Semantics

`ki extract` separates library-level symbol data from instance-level overrides.

- `lib_parts[*].fields` contains library or class symbol properties
- `components[*].properties` contains instance-only overrides and instance-only custom properties
- `components[*].footprint` and `components[*].datasheet` are instance overrides only
- inherited library defaults remain on the matching `lib_parts[*]`

To reconstruct the effective property set for a component, overlay
`components[*].properties` onto the corresponding `lib_parts[*].fields`.

## Schema Validation

`ki extract` validates `.kicad_sch` input before extraction. The validator is intended to follow
the accept/reject behavior of KiCad's schematic parser on the implemented file-format surface.
Invalid schematics fail with the same top-level `Failed to load schematic` message shape used by
`kicad-cli`.

Validation currently covers the main schematic object families and shared parser branches used by
extract parity, including:

- top-level schematic structure
- properties and mandatory-field aliases
- symbols, sheets, instances, variants, and default instances
- pins, labels, text, text boxes, tables, rule areas, and images
- embedded files, groups, bus aliases, UUID/token-class constraints, and legacy compatibility forms

The authoritative parity record for extract validation lives in:

- [tests/extract_parity/cases.json](/Users/Daniel/Desktop/modular/tools/ki-validation-wt/tests/extract_parity/cases.json)
- [tests/extract_parity/kicad_inventory.md](/Users/Daniel/Desktop/modular/tools/ki-validation-wt/tests/extract_parity/kicad_inventory.md)

## Output Modes

File-oriented commands support some combination of:

- `--json` for machine-readable stdout
- `--pretty` for formatted JSON
- `--diagnostics` for machine-readable diagnostics
- `--include-diagnostics` to embed diagnostics in extract output
- `--hierarchical` for schematic net and unconnected queries

Command-specific availability depends on the subcommand.

## Exit Codes

File-editing and validation-oriented commands:

- `0` success
- `1` validation findings
- `2` parse or I/O failure

Extract command:

- `3` failed schematic load

Project commands:

- `project open` is informational and exits `0` even when library-table diagnostics are present
- `project validate` promotes load and validation diagnostics to exit `1`

Refresh command:

- `0` success
- `1` IPC connection or API failure
- `refresh --silent` suppresses refresh output and still exits `0` on refresh failure

## Development Commands

```bash
just build
just release
just test
just check
just fmt
just install
```

Example inspection command against the sibling fixture set:

```bash
just run schematic inspect ../kiutils-rs/crates/kiutils_kicad/tests/fixtures/sample.kicad_sch --json
```
