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

Active narrowed backlog notes:

- `issue19646/F28P65x-power.kicad_sch` is no longer a broad hierarchy mismatch.
  It is narrowed to a symbol-specific upstream ERC difference for `C24` only:
  live `kicad-cli` and the stored oracle both omit `C24` from `lib_symbol_issues` and
  `footprint_link_issues`, while native `ki` still emits those 2 items.
- `issue19646/issue19646.kicad_sch` therefore remains blocked on the same child-sheet `C24`
  omission surface rather than on root-sheet aggregation.
- `issue12814_2.kicad_sch` is now behaviorally aligned on legacy power-symbol mismatch selection:
  only `#PWR05 [VCC]` remains, matching upstream. It is still not exact because the stored oracle
  position normalizes to `1.46` while native `ki` keeps the marker at `1.4605`.
- `issue12814_1.kicad_sch` also regained the expected legacy `VCC` mismatches (`#PWR01`, `#PWR04`),
  but it still differs on the unresolved power-driver classification surface:
  upstream emits one `power_pin_not_driven` item on `U1 Pin 1`, while native `ki` still splits
  that into `pin_not_driven` on `U1 Pin 1` plus `power_pin_not_driven` on `U1 Pin 7 [GND]`.
- `netlists/issue14494/issue14494_subsheet1.kicad_sch` remains the next low-diff hierarchy target.
  The live diff is still one extra local root `pin_not_connected` on hierarchical label `C`;
  upstream omits only that one while keeping the `A`, `B`, and `D` root label errors.
