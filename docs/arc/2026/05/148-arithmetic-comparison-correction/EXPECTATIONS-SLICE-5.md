# Arc 148 Slice 5 — Pre-handoff expectations

**Drafted 2026-05-03.** Cleanup slice — retire 10 per-Type
comparison leaves + sweep call sites + rename/simplify
`infer_polymorphic_compare`. NO new substrate primitives.
Predicted SMALL slice (Mode A ~80%; Mode B-call-site-resistance
~10%; Mode B-check-side-coupling ~5%; Mode C ~5%).

**Brief:** `BRIEF-SLICE-5.md`
**Output:** EDITS to `src/runtime.rs` (remove dispatch arms +
freeze-pipeline entries) + `src/check.rs` (remove TypeScheme
registrations + rename/simplify `infer_polymorphic_compare`) +
test/wat files containing call sites for retired names. NO new
files. NO new tests.

## Setup — workspace state pre-spawn

- Arc 148 slice 3 shipped (`SCORE-SLICE-3.md`); `values_compare`
  extended for universal ord delegation.
- Workspace baseline (per FM 9, post-slice-3): reflection-layer
  baselines all green (45/45 across 5 test files); `wat_arc148_ord_buildout`
  46/46; `wat_polymorphic_arithmetic` 20/20; workspace failure
  profile is the documented `CacheService.wat` noise.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (dispatch arms + freeze list) + `src/check.rs` (TypeSchemes + handler rename/simplify) + test/wat files containing call sites. NO new Rust files. NO new wat files. NO new tests. NO body changes to `eval_eq`/`eval_compare`/`eval_not_eq`/`values_compare`/`values_equal`. |
| 2 | 10 per-Type leaves retired | All 10 names removed from `src/runtime.rs:2607-2620` (dispatch arms) + `src/runtime.rs:15716-15731` (freeze pipeline) + `src/check.rs:9013-9040` (TypeSchemes). Independent grep confirms zero remaining occurrences in `src/`. |
| 3 | `infer_polymorphic_compare` handled | Either (a) renamed + simplified OR (b) promoted to TypeScheme. Choice + rationale named in report. The 6 polymorphic comparison ops (`:wat::core::{=,not=,<,>,<=,>=}`) still type-check correctly post-handoff. |
| 4 | Call-site sweep complete | `grep -rn ':wat::core::i64::[=<>]'` and same for f64 returns ZERO matches outside of comments / migration-marking text. (Or matches explicitly accounted for in the report.) |
| 5 | Baseline tests still green | `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_polymorphic_arithmetic` 20/20; `wat_arc148_ord_buildout` 46/46. |
| 6 | Polymorphic comparison still works end-to-end | `(:wat::core::< 1 2)` → `:bool true`; `(:wat::core::< 1 2.5)` → `:bool true` (mixed-numeric); `(:wat::core::= "a" "a")` → `:bool true` (universal); `(:wat::core::not= 1 2)` → `:bool true`. Spot-check via existing tests + sonnet's verification. |
| 7 | Strict type-locking achievable via param types | Test or example shows that `(:wat::core::< (a :i64) (b :i64))` enforces same-type at the binding site (i.e., users haven't lost the strict-typing UX entirely; it just moved from per-primitive name to call-site param types). Spot-check via existing test that uses i64 params + polymorphic comparison. |
| 8 | Full workspace `cargo test` passes | Single-threaded; only documented `CacheService.wat` noise. |
| 9 | No new clippy warnings | `cargo clippy` count unchanged from pre-slice baseline. |
| 10 | Honest report | ~250-word report covers all required sections from BRIEF. |

**Hard verdict:** all 10 must pass. Rows 2 + 4 + 6 are the
load-bearing rows (retirements complete + sweep complete +
polymorphic ops still work).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 50-200 LOC (substrate retirements ~20-50 LOC NET DELETED + handler rename ~10 LOC + sweep call sites ~30-150 LOC). Net likely NEGATIVE (more deletions than insertions). |
| 12 | Style consistency | Sweep is exact char-substitution (`:i64::<` → `:<` etc.). Handler rename uses convention-consistent name. |
| 13 | clippy clean | No new warnings. |
| 14 | Audit-first discipline | If sonnet finds a call site that can't trivially convert (per-Type leaf provided value beyond cosmetic type-locking), surface as honest delta with file:line + investigation. Don't improvise. |

## Independent prediction

- **Most likely (~80%) — Mode A clean ship.** Smaller scope than
  slice 2 (10 names + retirement vs slice 2's 8 names + crypto regen
  + complete rename). Brief is detailed; the substrate context is
  well-understood post-slices-1-3. ~25-40 min wall-clock.
- **Mode B-call-site-resistance (~10%):** sonnet finds a call site
  that genuinely relies on per-Type strict type-locking in a way
  the polymorphic form can't replicate. Surfaces as honest delta;
  orchestrator decides whether to refactor caller or keep one leaf.
- **Mode B-check-side-coupling (~5%):** removing/renaming
  `infer_polymorphic_compare` exposes a coupling at check.rs:3292
  that the audit missed; sonnet adapts within scope.
- **Mode C (~5%):** unforeseen substrate edge.

## Time-box

60 min wall-clock (≈1.5× the predicted upper-bound). If the wakeup
fires and sonnet hasn't completed: TaskStop + Mode B score with
the overrun as data.

## What sonnet's success unlocks

Slice 4 (numeric arithmetic migration) is the only remaining
implementation slice. Slice 6 (closure) follows.

The substrate's comparison surface reaches its final shape:
- 6 polymorphic bare-name entities (LLM-natural)
- 0 per-Type comparison leaves
- 0 comma-typed comparison leaves
- 1 cleaned check-side inference function

Maximum LLM affordance for comparison achieved.

## After sonnet completes

- Re-read the audit's OQ2 against the SCORE
- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows (2, 4, 6) by spot-checking sweep + running
  comparison tests
- Write `SCORE-SLICE-5.md`
- Commit the SCORE before drafting slice 4's BRIEF (calibration
  preserved)
