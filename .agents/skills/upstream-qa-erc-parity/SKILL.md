---
name: upstream-qa-erc-parity
description: Use when adding, promoting, or fixing KiCad upstream QA ERC fixtures in this repository. Covers harvesting exact upstream oracle matches, narrowing remaining diffs, grounding behavior on upstream KiCad source and QA tests under /Users/Daniel/Desktop/kicad, updating the maintained manifest and exact tests, and validating parity with narrow single-threaded ERC checks.
---

# Upstream QA ERC Parity

Use this skill for repo-local ERC parity work against upstream KiCad QA data.

## Goal

Keep the local ERC CLI behavior compatible with KiCad by:

1. Harvesting newly exact upstream oracle-backed fixtures.
2. Promoting verified exact matches into the maintained manifest and explicit tests.
3. Diagnosing non-exact cases against upstream KiCad source and QA tests.
4. Fixing the owning abstraction locally instead of patching only one surface symptom.

Upstream authority lives at:

- `/Users/Daniel/Desktop/kicad/qa/data/eeschema`
- `/Users/Daniel/Desktop/kicad/qa/tests/eeschema`
- `/Users/Daniel/Desktop/kicad/eeschema`

Primary local files are:

- `tests/cli_schematic_erc.rs`
- `tests/fixtures/erc_upstream_qa/manifest.json`
- `tests/fixtures/erc_upstream_qa/oracles/`
- `tests/fixtures/erc_upstream_qa/latest_comparison_results.json`
- `src/cmd/schematic/erc/`
- `src/schematic/render.rs`

## Rules

- Do not infer KiCad ERC behavior when upstream source or QA covers it.
- Prefer the owning upstream abstraction over copying an isolated condition.
- Verify exact-match promotions narrowly before adding them to the maintained set.
- Prefer single-test or small-batch verification with `--test-threads=1`.
- Be cautious with broad `cargo test --test cli_schematic_erc`; an early failure can poison the shared test lock and make the run noisy.
- Do not promote fixtures that only looked exact in a stale sweep; rerun them on the current branch first.
- Keep user-facing counts explicit:
  - maintained upstream-QA exact regressions = curated manifest-backed set
  - full upstream oracle-backed corpus = all oracle-backed cases currently comparable

## Standard Loop

### 1. Refresh the frontier

Use a live sweep to find:

- current exact total
- current diff total
- non-manifest exact matches

If a helper script does not exist, a small one-off `python3` comparison is acceptable. Prefer comparing against:

- `tests/fixtures/erc_upstream_qa/oracles/*.erc.json`
- `tests/fixtures/erc_upstream_qa/latest_comparison_results.json`

### 2. Promote verified exact matches first

For each non-manifest exact candidate:

1. Add an explicit exact test in `tests/cli_schematic_erc.rs`.
2. Add the manifest entry in `tests/fixtures/erc_upstream_qa/manifest.json`.
3. Verify the new exact case narrowly:

```bash
cargo test --test cli_schematic_erc schematic_erc_matches_upstream_<case>_fixture -- --nocapture --test-threads=1
```

4. Re-run the manifest gate:

```bash
cargo test --test cli_schematic_erc erc_parity_cases_manifest_is_well_formed -- --nocapture --test-threads=1
```

If a supposedly exact case still differs, remove it from the promotion batch and treat it as real backlog.

### 3. Pick the next real diff

Prefer:

- the smallest diff count
- a family with an upstream QA test
- a family where the failing abstraction is likely shared and reusable

Typical good next targets:

- hierarchy aggregation
- label/logical-net handling
- bus-member expansion
- sheet pin connectivity
- multi-unit/shared-pin behavior

### 4. Ground the diff upstream

Before editing local code:

1. Open the upstream QA test covering the issue.
2. Open the owning upstream implementation in `eeschema/`.
3. Identify where KiCad resolves the behavior:
   - parsing
   - connection graph construction
   - normalization
   - ERC rule evaluation
   - report formatting

Do not stop at the final error message text if the real decision happens earlier in the graph/model.

### 5. Diff actual vs oracle concretely

Generate the local ERC JSON for the fixture and compare it to the stored oracle.

Focus on:

- missing violations
- extra violations
- wrong item descriptions
- wrong sheet paths
- wrong ordering only if it affects normalized equality

Treat float-only noise separately from behavioral diffs.

### 6. Patch the owning abstraction

Favor fixes in shared layers such as:

- `src/schematic/render.rs`
- `src/cmd/schematic/erc/connectivity.rs`
- `src/cmd/schematic/erc/rules/connectivity.rs`
- `src/cmd/schematic/erc/rules/root.rs`
- `src/cmd/schematic/erc/rules/hierarchy.rs`

Do not refactor for its own sake. Only change structure when it helps match the upstream model.

### 7. Verify narrowly, then guard adjacent regressions

After a fix:

1. Run the exact target case.
2. Run the nearest related maintained regressions.
3. Run the manifest gate.

Example:

```bash
cargo test --test cli_schematic_erc schematic_erc_matches_upstream_issue18299_fixture -- --nocapture --test-threads=1
cargo test --test cli_schematic_erc schematic_erc_matches_upstream_issue18299_test_fixture -- --nocapture --test-threads=1
cargo test --test cli_schematic_erc schematic_erc_matches_upstream_issue1768_fixture -- --nocapture --test-threads=1
cargo test --test cli_schematic_erc schematic_erc_matches_upstream_issue9367_fixture -- --nocapture --test-threads=1
cargo test --test cli_schematic_erc erc_parity_cases_manifest_is_well_formed -- --nocapture --test-threads=1
```

Only use the full test target when it is worth the cost and noise.

## Common Patterns

### Exact but not promoted

This is low-risk backlog. Promote it after narrow verification.

### Sweep said exact, narrow check failed

This usually means one of:

- stale comparison data
- fixture-specific library/project resolution differences
- a comparison script bug

Do not promote until the narrow exact test passes.

### Root vs descendant sheet duplication

Inspect:

- hierarchical merge paths
- repeated sheet instance handling
- dedup behavior when merging pending violations

### Bus/member issues

Inspect:

- bus alias expansion
- label naming normalization
- sheet pin remapping
- bus-entry net naming
- hierarchy propagation in upstream `connection_graph.cpp`

### Label-only net issues

Inspect:

- logical net creation
- whether label-only groups are preserved
- dangling-label logic
- multiple-net-name driver selection

## Reporting Back

When closing a chunk of work, report:

1. what upstream fixture or family was closed
2. which upstream source/test grounded the fix
3. which local abstractions changed
4. exact maintained count if it changed
5. the narrow verification commands that passed

If something remains open, name the next real backlog family instead of stopping at a generic summary.
