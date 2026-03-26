# ki

An agentic-first, zero-dependency Rust CLI for direct KiCad file editing, designed to bring most KiCad functionality to the command line.

## Current scope

The first command is `refresh`, which asks KiCad to refresh a specific editor frame.

Default target:
- `schematic`

Supported frames:
- `schematic`
- `pcb`
- `project-manager`
- `spice`
- `symbol`
- `footprint`
- `drawing-sheet`

## Build

```bash
just build
```

This tool uses the remote `kicad-ipc-rs` library pinned to a specific Git revision.

## Justfile

Common development commands:

```bash
just build
just release
just test
just check
just fmt
just run refresh --frame pcb
just install
```

`just install` builds the release binary and installs it to `/usr/local/bin/ki` by default.

You can override the install location with:

```bash
INSTALL_DIR=/custom/bin just install
INSTALL_PATH=/custom/bin/ki just install
```

## Usage

Refresh the open schematic editor:

```bash
just run refresh
```

Refresh the PCB editor:

```bash
just run refresh --frame pcb
```

Suppress all output and always return success:

```bash
just run refresh --frame pcb --silent
```

Override KiCad IPC connection settings:

```bash
just run \
  --socket <uri> \
  --token <token> \
  --client-name ki \
  --timeout-ms 5000 \
  refresh --silent
```

## Exit behavior

- Exit `0` on success
- Exit `1` on KiCad IPC connection failures or API errors
- `refresh --silent` suppresses all refresh output and always exits `0`

## Compatibility note

`ki refresh` first tries `RefreshEditor` through `kicad-ipc-rs`.

If KiCad returns `AS_UNHANDLED` for `RefreshEditor`, `ki` falls back to `RevertDocument` for document-backed frames.

In practice:

- `pcb` can fall back successfully on KiCad builds where board reload is exposed through IPC
- non-PCB frames may still fail because KiCad IPC refresh/reload support is effectively PCB-focused as of KiCad 10
