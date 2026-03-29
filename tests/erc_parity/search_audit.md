# ERC Search Audit

This note records the final deep-source and CLI sweep performed after the ERC parity inventory reached zero `documented` backlog items.

## Scope

- KiCad source tree searched:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h`
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/eagle/sch_io_eagle.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp`
  - `/Users/Daniel/Desktop/kicad/qa/tests/eeschema/erc/test_erc_label_names.cpp`
- KiCad QA/project corpus searched:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/spice_netlists`
- Repo files checked during this pass:
  - [kicad_inventory.md](/Users/Daniel/Desktop/modular/tools/ki/tests/erc_parity/kicad_inventory.md)
  - [same_local_global_power.kicad_sch](/Users/Daniel/Desktop/modular/tools/ki/tests/fixtures/erc_parity/same_local_global_power/same_local_global_power.kicad_sch)
  - [same_local_global_power_builtin.kicad_sch](/Users/Daniel/Desktop/modular/tools/ki/tests/fixtures/erc_parity/same_local_global_power_builtin/same_local_global_power_builtin.kicad_sch)

## Cases Reviewed

### `same_local_global_power`

- Source files checked:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1477`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:1538`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2169`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:132`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:58`
  - `/Users/Daniel/Desktop/kicad/qa/tests/eeschema/erc/test_erc_label_names.cpp:139`
- Fixtures/probes checked:
  - [same_local_global_power.kicad_sch](/Users/Daniel/Desktop/modular/tools/ki/tests/fixtures/erc_parity/same_local_global_power/same_local_global_power.kicad_sch)
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/same_local_global_power.kicad_sch`
  - [same_local_global_power_builtin.kicad_sch](/Users/Daniel/Desktop/modular/tools/ki/tests/fixtures/erc_parity/same_local_global_power_builtin/same_local_global_power_builtin.kicad_sch)
- Official CLI result:
  - the repo fixture emitted `pin_not_connected`, `power_pin_not_driven`, and `lib_symbol_issues`
  - the downgraded QA-derived built-in fixture emitted `pin_not_connected`, `power_pin_not_driven`, and `lib_symbol_mismatch`
  - no official `same_local_global_power` JSON marker was reproduced
- Conclusion:
  - source rule exists and is covered by KiCad internal tests
  - current `kicad-cli sch erc` does not reproduce it on the available fixtures
  - status moved to `not-applicable`

### `bus_to_bus_conflict`

- Source files checked:
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3427`
  - `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp:3636`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:156`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:71`
- Fixtures/probes checked:
  - synthetic vector bus probes
  - synthetic group bus probes
  - synthetic parent/child sheet-pin bus probes
  - broader `qa/data/eeschema` CLI sweep
- Official CLI result:
  - probes consistently resolved to other findings such as `multiple_net_names`, `pin_not_connected`, or `hier_label_mismatch`
  - no `bus_to_bus_conflict` JSON marker was reproduced
- Conclusion:
  - source rule exists
  - current official CLI behavior remains non-reproducible for this family
  - status moved to `not-applicable`

### `simulation_model_issue`

- Source files checked:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2030`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp:2194`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:184`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:79`
- QA corpus checked:
  - full `/Users/Daniel/Desktop/kicad/qa/data/eeschema/spice_netlists` tree
- Official CLI result:
  - zero `simulation_model_issue` JSON findings across the SPICE QA corpus
- Conclusion:
  - source producer exists via `SIM_LIB_MGR::CreateModel`
  - current official CLI did not surface a reproducible JSON case
  - status moved to `not-applicable`

### `bus_entry_needed`

- Source files checked:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:240`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:90`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/eagle/sch_io_eagle.cpp:2936`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/eagle/sch_io_eagle.cpp:3261`
- Official CLI result:
  - no normal `.kicad_sch` ERC producer was found
- Conclusion:
  - importer/conversion artifact only
  - status moved to `not-applicable`

### `bus_definition_conflict`

- Source files checked:
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_item.cpp:144`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.h:68`
- Official CLI result:
  - ERC registry entry exists, but no producer was found in current KiCad sources
- Conclusion:
  - registered-but-unemitted key
  - status moved to `not-applicable`

## Remaining CLI-Parity Backlog After Final Search

- None

The ERC inventory now contains no remaining `documented` families.

## Broad QA Follow-Up After Family Discovery

The next pass moved from family discovery to broad KiCad QA regression sweeps. These did not expose new ERC families; they only produced wider instances of already-documented ones.

The maintained upstream QA ERC set is cataloged in:

- `tests/fixtures/erc_upstream_qa/manifest.json`

Current tracked broad cases:

- exact equality:
  - `erc_multiple_pin_to_pin`
  - `top_level_hier_pins`
  - `issue18606`
  - `prefix_bus_alias`
- targeted:
  - `bus_connection`
  - `multinetclasses`

### `prefix_bus_alias`

- Files checked:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/prefix_bus_alias/prefix_bus_alias.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/prefix_bus_alias/subsheet1.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/prefix_bus_alias/subsheet2.kicad_sch`
  - official CLI outputs from:
    - `prefix_bus_alias.kicad_sch`
    - `subsheet1.kicad_sch`
    - `subsheet2.kicad_sch`
- Official CLI result:
  - parent project emits only two root `lib_symbol_mismatch` warnings for `J1` and `J2`
  - each child sheet emits standalone `isolated_pin_label` warnings when checked alone
- Conclusion:
  - no new ERC family
  - the parity gap was in root-sheet hoisting for prefixed bus-alias sheet pins like `Foo{Bus1}` and `Bar{Bus1}`
  - native behavior was tightened so these child-label diagnostics are not hoisted to `/` when the parent sheet pins use prefixed bus-alias syntax
  - KiCad `10.0.0` changed the observable root behavior again:
    - `prefix_bus_alias.kicad_sch` now emits no violations at all in the official CLI
    - the earlier two root `lib_symbol_mismatch` warnings for `J1` and `J2` are gone on this build
    - native `ki` still emits those two root `lib_symbol_mismatch` warnings, so this file has become an active 10.0.0 broad-parity backlog item

### `multinetclasses`

- Files checked:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/multinetclasses/multinetclasses.kicad_sch`
  - official CLI output from `multinetclasses.kicad_sch`
- Official CLI result:
  - known families emitted:
    - `isolated_pin_label`
    - `pin_not_connected`
    - `undefined_netclass`
  - counts from the official CLI run:
    - `isolated_pin_label`: 4
    - `pin_not_connected`: 11
    - `undefined_netclass`: 7
- Conclusion:
  - no new ERC family
  - this broad case remains a regression target for already-known families
  - the broad sweep directly helped close missing `undefined_netclass` support for `netclass_flag`
  - after the KiCad `10.0.0` rebaseline and signature-normalization fixes, the remaining native delta is narrow:
    - extra `pin_not_connected` on `R7`
  - follow-up result:
    - a temporary `90`/`270` rotation change over-corrected and flipped the disconnected pin numbering for the entire resistor set
    - that change was reverted
    - the current geometry model already matches the schematic endpoint locations for the resistor family, so the residual `R7` delta looks more like a KiCad-side suppression quirk than a simple transform bug
    - a KiCad-shaped rewrite of native `pin_not_connected` to mirror subgraph-style `has_other_connections`, stacked-pin handling, and named-label suppression did not change the `R7` outcome
    - that keeps the remaining gap on the KiCad side of the rule boundary rather than the old native shortcut logic
  - reduction probe result:
    - a minimal two-resistor probe built from the same `R7`/`R8` geometry does emit official `pin_not_connected` for `R7 Pin 1`
    - therefore the remaining omission in broad `multinetclasses` is not basic resistor-geometry behavior; it only appears in the larger project context
  - additional source-guided probes:
    - KiCad netlist export for the same schematic still contains `unconnected-(R7-Pad1)`, so connectivity generation itself is not dropping the dangling pin
    - KiCad netlist export also confirms the connected side of `/NET_2` is the same as native resolution:
      `R7.2` connected to `R8.1`
    - stripping top-level `netclass_flag` and `rule_area` blocks does not change the remaining omission set (`R7` pin-not-connected still omitted)
    - reordering all top-level symbol blocks does not change the omission set
    - renaming `R7` keeps the omission tied to the same symbol instance, not to the reference text
  - deeper source conclusion:
    - the remaining delta is not in symbol parsing or net extraction
    - KiCad builds ERC on `CONNECTION_SUBGRAPH`s in `eeschema/connection_graph.cpp`, not on the final netlist view
    - `ercCheckNoConnects()` decides `pin_not_connected` from subgraph runtime state (`m_items`, `m_drivers`, `m_driver_connection`, `m_local_label_cache`, `m_global_label_cache`)
    - the official omission therefore happens at ERC marker generation time, after connectivity exists, not because KiCad fails to see the pin or the net
    - native `ki` ERC is still net-centric over `render.rs` `ResolvedNet`s, so this last mismatch is the one place where the simpler model still diverges from KiCad’s subgraph engine
  - final structural check:
    - `R7` is not special in the raw schematic instance data
    - its placed-symbol block matches the same `Device:R` shape and instance-property pattern used by the rest of the resistor set
    - no project ERC exclusions or item-specific suppression metadata were found
    - current remaining delta should therefore be treated as an unexplained official-CLI quirk unless a deeper KiCad runtime path is identified later
  - geometry clue recorded during follow-up:
    - the remaining `R7` mismatch traces to symbol rotation semantics, not a new ERC family
    - `src/schematic/render.rs` was rotating `90` and `270` degree symbols with the opposite handedness for KiCad-style screen coordinates
    - that same clue also explains the remaining extra rotated connector pin items seen in broader `bus_connection` comparisons
  - KiCad `10.0.0` re-baseline:
    - official CLI now emits only:
      - `isolated_pin_label`: 4
      - `pin_not_connected`: 11
      - `undefined_netclass`: 7
    - root cause found:
      - the missing official `pin_not_connected` for `R7 Pin 1` is not a connection-graph parity gap
      - it is an upstream marker-provider/reporting bug
      - a reduced KiCad `10.0.0` probe with only `R7`, `R8`, the joining wire, and a dangling label at the transposed point `(121.92, 77.47)` reproduces the omission:
        - official ERC reports `label_dangling` for the label
        - official ERC omits `R7 Pin 1`
        - official netlist export still contains `unconnected-(R7-Pad1)`
      - source explanation:
        - `SHEETLIST_ERC_ITEMS_PROVIDER::visitMarkers()` in `/Users/Daniel/Desktop/kicad/eeschema/erc/erc_settings.cpp:340` inserts markers into `std::set<SCH_MARKER*, CompareMarkers>`
        - `CompareMarkers` falls back to `item->GetPosition() < item2->GetPosition()` when positions differ
        - `VECTOR2I::operator<` in `/Users/Daniel/Desktop/kicad/libs/kimath/include/math/vector2d.h:578` compares by squared radius from the origin, not lexicographic `(x, y)`
        - transposed points such as `(77.47, 121.92)` and `(121.92, 77.47)` therefore compare neither less-than nor greater-than each other, so the `std::set` treats them as equivalent and drops one marker
      - native `ki` policy:
        - do not mirror this upstream marker-provider collapse
        - keep the physically correct `pin_not_connected` for `R7 Pin 1`
        - track the upstream omission as a documented KiCad bug instead of parity target behavior

### `bus_connection`

- Files checked:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/bus_connection/bus_connection.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/bus_connection/a.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/bus_connection/a2.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/bus_connection/b.kicad_sch`
  - official CLI output from `bus_connection.kicad_sch`
- Official CLI result:
  - only known families were emitted:
    - `endpoint_off_grid`
    - `label_dangling`
    - `lib_symbol_issues`
  - counts from the official CLI run:
    - `endpoint_off_grid`: 35
    - `label_dangling`: 1
    - `lib_symbol_issues`: 2
- Native delta observed during the broad sweep:
  - initial native root `label_dangling` pointed at `top{a_xyz}`, while the official CLI pointed at `test.y`
  - initial native run emitted a reduced `endpoint_off_grid` surface because descendant off-grid checks were not hoisted
- Conclusion:
  - no new ERC family
  - this is a broad bus/hierarchy/off-grid regression target, not a discovery lead
  - the broad sweep led to native fixes for:
    - descendant `endpoint_off_grid` hoisting
    - bus and bus-entry off-grid reporting
    - brace-style bus alias recognition in dangling-label checks
  - after those fixes, native matches the official root family counts for this project:
    - `endpoint_off_grid`: 35
    - `label_dangling`: 1
    - `lib_symbol_issues`: 2

## QA Corpus Emitted-Type Sweep

- Scope:
  - official CLI run over `190` `.kicad_sch` files under `/Users/Daniel/Desktop/kicad/qa/data/eeschema`
- Unique official CLI ERC `type` keys observed:
  - `bus_to_net_conflict`
  - `different_unit_net`
  - `endpoint_off_grid`
  - `footprint_link_issues`
  - `ground_pin_not_ground`
  - `hier_label_mismatch`
  - `isolated_pin_label`
  - `label_dangling`
  - `label_multiple_wires`
  - `lib_symbol_issues`
  - `lib_symbol_mismatch`
  - `missing_bidi_pin`
  - `missing_input_pin`
  - `missing_power_pin`
  - `missing_unit`
  - `multiple_net_names`
  - `net_not_bus_member`
  - `no_connect_connected`
  - `pin_not_connected`
  - `pin_not_driven`
  - `pin_to_pin`
  - `power_pin_not_driven`
  - `same_local_global_label`
  - `similar_label_and_power`
  - `similar_labels`
  - `similar_power`
  - `unconnected_wire_endpoint`
  - `undefined_netclass`
  - `wire_dangling`
- Conclusion:
  - no new official CLI-visible ERC family was discovered by the full QA sweep
  - this emitted-type set is consistent with the current inventory boundary
  - the remaining work is broad parity burn-down on already-known families, not discovery
- Notable CLI load failures during the sweep:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23058/issue23058.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/issue23403.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/shared1.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/top_level_sheet_1.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/same_local_global_power.kicad_sch`

### Load-Boundary Follow-Up

- Files checked:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23058/issue23058.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/issue23403.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/shared1.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue23403/top_level_sheet_1.kicad_sch`
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/same_local_global_power.kicad_sch`
- Observed status:
  - `issue23403` and `same_local_global_power` fail with `Failed to load schematic`
  - `issue23058` does not produce an ERC report under the installed CLI build
- Source-level clue from file headers:
  - `issue23058.kicad_sch` uses schematic version `20260101`
  - `issue23403.kicad_sch` uses schematic version `20260306`
  - `same_local_global_power.kicad_sch` uses schematic version `20260306`
- Conclusion:
  - these are load/version-boundary cases against the installed official CLI, not newly discovered ERC finding families
  - they should be treated as CLI coverage limits for this discovery phase, not as missing ERC inventory entries

## KiCad 10.0.0 Re-Baseline

- Updated CLI binary:
  - `/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli`
- Reported version:
  - `10.0.0`
- High-signal changes relative to the previous installed build:
  - `multinetclasses.kicad_sch`
    - dropped all `lib_symbol_mismatch` findings
    - now emits only `isolated_pin_label=4`, `pin_not_connected=11`, `undefined_netclass=7`
  - `prefix_bus_alias.kicad_sch`
    - now emits no violations at all
  - `same_local_global_power.kicad_sch`
    - no longer fails to load
    - now emits `lib_symbol_mismatch=4` and `no_connect_connected=2`
    - still does not emit `same_local_global_power`
  - `top_level_hier_pins.kicad_sch`
    - still emits known families only:
      `hier_label_mismatch=1`, `isolated_pin_label=5`, `lib_symbol_mismatch=2`, `pin_not_connected=1`, `power_pin_not_driven=1`
  - `issue18606.kicad_sch`
    - still emits known families only:
      `isolated_pin_label=4`, `lib_symbol_issues=1`, `multiple_net_names=1`
  - `bus_connection.kicad_sch`
    - unchanged family counts:
      `endpoint_off_grid=35`, `label_dangling=1`, `lib_symbol_issues=2`

## R7 Connectivity Follow-Up

- Target file:
  - `/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/multinetclasses/multinetclasses.kicad_sch`
- Focus:
  - remaining official/native delta on `pin_not_connected`
  - official KiCad `10.0.0` omits `Symbol R7 Pin 1 [Passive, Line]`
  - native `ki` still emits it
- Source review:
  - KiCad evaluates this in `CONNECTION_GRAPH::ercCheckNoConnects()` in
    `/Users/Daniel/Desktop/kicad/eeschema/connection_graph.cpp`
  - KiCad runs that check on physical `CONNECTION_SUBGRAPH`s, not on final label-merged nets
- Native model update:
  - `src/schematic/render.rs` now exposes physical connection groups separately from label-merged nets
  - `src/cmd/schematic/erc.rs` now runs `pin_not_connected` over those physical groups and only
    uses final net names for the same cache-based suppression step KiCad uses
- Result:
  - no broad ERC regressions were introduced
  - `cargo test --test cli_schematic_erc -- --quiet` still passes
  - the `R7 Pin 1` delta remains
- Current conclusion:
  - the remaining omission is not explained by project ERC exclusions, final net extraction, or
    our previous label-merged net model
  - it appears to depend on deeper KiCad runtime subgraph state than is visible from static
    schematic/project data alone
