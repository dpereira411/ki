This folder catalogs the upstream KiCad QA schematics we use for maintained ERC regressions.

The actual schematic files remain in the local KiCad checkout under:

- `kicad/qa/data/eeschema`

`manifest.json` is the sync source of truth for:

- which upstream ERC files are tracked here
- whether each one is expected to match exactly or only via targeted assertions
- why a file is in the broad-regression set

When updating KiCad versions, refresh this catalog first, then rerun the broad ERC regression tests.
