# Arc 138 Slice 5 — Pre-handoff expectations

**Brief:** `BRIEF-SLICE-5.md`
**Target:** src/config.rs only.

## Setup — workspace state pre-spawn

- Baseline: F4c commit `55c21f6`. All 4 cracks closed.
- 8 ConfigError variants; 2 use `form_index: usize`, 6 use struct fields without span.
- ~40 emission sites in src/config.rs (collect_entry_file + helpers).
- Cross-file: src/freeze.rs wraps ConfigError, no destructure (invisible to restructure).
- 6/6 arc138 canaries pass.

## Hard scorecard (7 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | src/config.rs only. |
| 2 | All 8 variants gain span | 6 plain additions + 2 form_index→span migrations. |
| 3 | All 8 Display arms prefix coords via span_prefix | + 2 with intentional content changes (drop form_index suffix). |
| 4 | Emission sites use real spans | ≥ 90% real-spanned. Remaining Span::unknown() carries rationale. |
| 5 | Canary added + passes | New `config::tests::arc138_config_error_message_carries_span` triggers a variant + asserts `<test>:` substring. |
| 6 | Workspace tests pass | All 7 arc138 canaries (6 existing + 1 new). |
| 7 | No commits | working tree only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 8 | form_index migration documented | Both intentional Display content changes named in report. |
| 9 | Calibration | ≤ 15 min sonnet (matches F4a/4b shape). |
| 10 | Honest report | per-variant + per-arm confirmation. |

## Independent prediction

- **Most likely (~75%):** 7/7 + 3/3, sonnet 8-15 min. Same shape as slice 4a but smaller + no cross-file.
- **Test pattern updates (~15%):** existing tests in src/config.rs may have form_index in destructure patterns; sonnet handles mechanically.
- **freeze.rs build break (~5%):** the From<ConfigError> impl in freeze.rs may need adjustment if the variant signatures changed (it shouldn't — it just re-wraps). cargo build catches.
- **Cross-file regression (~5%):** rare.

## Methodology

Standard verify (diff stat, grep counts, canaries, workspace) → score → commit + push → queue F-NAMES-1 (wat::test! macro emit per NAMES-AUDIT).

## What this slice tells us

- All clean → ConfigError closes the original arc 138 variant-restructure work. Last variant-restructure slice; remaining work is F-NAMES-1 + slice 6 closure.
