# Upstream ERC Corpus Comparison Summary

This summary records a full side-by-side ERC comparison over every schematic listed in:

- `tests/fixtures/erc_upstream_qa/all_files.txt`

Comparison command shape:

- official: `/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli sch erc --format json`
- native: `ki schematic erc --json`

Current baseline:

- upstream corpus size: `189` `.kicad_sch` files
- exact matches: `21`
- diffs: `168`

Notes:

- the maintained broad regression catalog is still the curated subset in:
  - `tests/fixtures/erc_upstream_qa/manifest.json`
- this full-corpus sweep is intended as a sync/reporting pass, not yet as a checked-in exact-equality test for all `189` upstream files
- the main value of this file is that future KiCad-version syncs can rerun the same corpus against the same file list and compare the totals directly

Known high-signal exact-match upstream cases already maintained in tests:

- `erc_multiple_pin_to_pin.kicad_sch`
- `netlists/top_level_hier_pins/top_level_hier_pins.kicad_sch`
- `issue18606/issue18606.kicad_sch`
- `netlists/prefix_bus_alias/prefix_bus_alias.kicad_sch`

Known targeted broad-regression cases already maintained in tests:

- `netlists/bus_connection/bus_connection.kicad_sch`
- `netlists/multinetclasses/multinetclasses.kicad_sch`

Current top-level conclusion:

- `ki` is now strong on the curated ERC parity set and selected upstream broad regressions
- full upstream-corpus parity is not complete yet; the corpus sweep still exposes `168` differing files
