# ki

Rust CLI for KiCad IPC refresh operations and direct KiCad file editing.

## Scope

`ki` now combines:

- IPC refresh support via `kicad-ipc-rs`
- file editing commands backed by the sibling `kiutils-rs` library

Current top-level commands:

- `ki refresh`
- `ki extract ...`
- `ki project ...`
- `ki schematic ...`
- `ki symbol-lib ...`
- `ki pcb ...`
- `ki lib-table ...`

## Development dependency

This repo depends on the local sibling checkout at `../kiutils-rs/crates/kiutils`.
That repo is treated as the library source of truth and is not modified by this CLI.

## Build

```bash
just build
```

## Examples

Refresh the PCB editor through KiCad IPC:

```bash
just run refresh --frame pcb
```

Inspect a schematic as JSON:

```bash
just run schematic inspect path/to/file.kicad_sch --json
```

Extract a schematic netlist/topology as JSON:

```bash
just run extract path/to/file.kicad_sch --pretty
```

Add a PCB trace:

```bash
just run pcb add-trace board.kicad_pcb 10 10 20 10 0.25 F.Cu 1
```

Validate a project:

```bash
just run project validate project.kicad_pro --json
```

## Output and exit codes

File-editing commands support:

- `--json` for machine-readable stdout
- `--diagnostics` for JSON diagnostics on stderr
- `--hierarchical` on schematic net/unconnected queries

File-editing exit codes:

- `0` success
- `1` validation warnings or errors found
- `2` parse or IO error

Project behavior:

- `project open` is informational and still exits `0` when library-table diagnostics are present
- `project validate` is the command that promotes diagnostics/load errors to exit `1`

Refresh exit behavior:

- `0` on success
- `1` on KiCad IPC connection failures or API errors
- `refresh --silent` suppresses refresh output and returns `0` even on refresh errors

## Justfile

```bash
just build
just release
just test
just check
just fmt
just run refresh --frame pcb
just run schematic inspect ../kiutils-rs/crates/kiutils_kicad/tests/fixtures/sample.kicad_sch --json
just install
```
