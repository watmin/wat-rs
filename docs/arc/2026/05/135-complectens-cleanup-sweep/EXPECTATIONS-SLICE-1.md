# Arc 135 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-03, AFTER spawning sonnet, BEFORE deliverable.

**Brief:** `BRIEF-SLICE-1.md`
**Targets:** `wat-tests/service-template.wat` + `wat-tests/console.wat`

## Setup — workspace state pre-spawn

- Baseline: `cargo test --release --workspace` exit=0; ~1773 individual tests passing across 100 result blocks.
- Both target files have monolithic deftest bodies (106 + 101 lines worst).
- *complectēns* SKILL recently extended with three edge-case sections (two-prelude, cross-function tracing, pop-before-finish) from the HologramCacheService calibration.
- HologramCacheService is the calibrated demonstration of the discipline.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Two-file diff | Only `wat-tests/service-template.wat` and `wat-tests/console.wat` modified. No substrate. No other tests. No docs. No Rust files. |
| 2 | Helpers added | Each file gains layered helpers in its `make-deftest` prelude(s). Estimated 6-15 helpers per file. |
| 3 | Each existing deftest body shrinks | `service-template.wat :svc::test-template-end-to-end` 106→3-7 lines. `console.wat :test-multi-writer` 101→3-7 lines. `console.wat :test-hello-world` 44→3-7 lines. |
| 4 | Per-helper deftests added | Each new helper has its own `(:deftest ...)` proving it. |
| 5 | No forward references | Helpers reference only earlier helpers in their prelude. Top-down. |
| 6 | **Outcomes preserved** | `cargo test --release --workspace` exit=0; existing deftests' outcomes unchanged; new helper deftests' outcomes consistent with their helper's pattern. |
| 7 | No commits | Working tree shows uncommitted modifications. |
| 8 | Honest report | ~300 words; per-file BEFORE→AFTER body line counts; helpers listed; per-helper deftests listed; outcomes verified; honest deltas; four questions applied. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Helper count | 6-20 helpers TOTAL across both files. |
| 10 | Average body shrink | ≥60% across the three flagged deftests. |
| 11 | Workspace runtime | Total `cargo test --release --workspace` runtime ≤ baseline + 20%. |
| 12 | Edge-case usage | Sonnet correctly applies the SKILL's edge-case guidance — uses two preludes if outcomes mix; doesn't factor `make-bounded-channel` into helpers; pops before finish on lifecycle. |

## Independent prediction

The HologramCacheService calibration shipped 8/8+4/4 with 3 honest deltas. With those deltas now baked into the SKILL's edge-case section, this slice should ship cleaner.

- **Most likely (~65%):** all 8 hard + 4 soft pass cleanly. The new SKILL edge-case sections plus the two worked demos (single-prelude + two-prelude) cover the cases. Sonnet ships in 15-25 min.
- **Two-prelude split needed (~20%):** at least one of the two files has mixed-outcome deftests (clean + should-panic in same file). Sonnet uses two preludes per the SKILL guidance. Ships clean.
- **A new edge case surfaces (~10%):** a Console-specific quirk (multi-writer / cross-thread stdio) the documents didn't anticipate. Sonnet names it in honest deltas; we refine the SKILL.
- **Per-helper deftest gap (~3%):** sonnet adds helpers but skips per-helper deftests on some. Hard row 4 fails partially. Re-spawn or finish manually.
- **Outcome regression (~2%):** test counts diverge from baseline. Likely cause: a `:should-panic` test stops panicking because a helper rewrite changed the deadlock pattern.

## Methodology

After agent reports back:

1. Read this file FIRST.
2. Score each row.
3. `git diff --stat` → 2 files modified.
4. `cargo test --release --workspace` for outcomes.
5. Read each rewritten file top-down; verify dependency direction; verify per-helper deftests.
6. Read honest deltas; assess whether SKILL needs further refinement.
7. Score; commit `SCORE-SLICE-1.md`.

## What this slice tells us

- All clean → the SKILL with its edge-case section teaches reproducibly. Future slices can dispatch with confidence.
- Edge case surfaced → the SKILL still has gaps. Refine before slice 2.
- Hard fail → the discipline has a deeper problem we missed. Diagnose and reframe.

The clean-delegation hypothesis continues: every slice is a measurement.

## What follows

- Score → commit slice 1 → start slice 2 (telemetry/Console.wat + telemetry/Service.wat) with whatever document refinements this slice surfaced.
