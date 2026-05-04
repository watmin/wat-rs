# Arc 148 Slice 5 — SCORE

**Sweep:** sonnet, agent `a66fe6214bac5d6dc`
**Wall clock:** ~25 min (493s) — UNDER predicted 25-40 min Mode A
band; used 41% of the 60-min time-box.
**Output verified:** orchestrator independently re-ran FM 9
baselines + spot-checked sweep completeness + confirmed per-Type
leaves absent from `src/`.

**Verdict:** **MODE A CLEAN SHIP.** 10/10 hard rows pass; 4/4
soft rows pass. Net deletion of 18 LOC; no honest deltas
surfaced; rhythm holds.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` + `src/check.rs` + 4 test files. NO new files. NO new tests. NO `eval_eq`/`eval_compare`/`eval_not_eq`/`values_compare`/`values_equal` body changes. |
| 2 | 10 per-Type leaves retired | ✅ All 10 names removed from `src/runtime.rs:2607-2620` (dispatch arms) + freeze pipeline pure-redex list + `src/check.rs:9013-9040` (TypeSchemes). Independent grep returns ZERO occurrences in src/. |
| 3 | `infer_polymorphic_compare` handled | ✅ Path (a) chosen — RENAMED to `infer_comparison`; body unchanged. Sonnet's rationale: substrate's TypeScheme system has `type_params` but no trait-bounded parametric inference; flat `∀T. (T, T) → bool` would lose cross-numeric acceptance. The "polymorphic-handler anti-pattern" framing was the issue, not the function. |
| 4 | Call-site sweep complete | ✅ 9 sites swept across 5 files (3 inline tests + 2 doc comments in `src/runtime.rs`; `wat_arc072_letstar_parametric.rs` (2); `wat_parametric_enum_typecheck.rs` (1); `wat_arc098_form_matches_runtime.rs` (1); `wat_polymorphic_arithmetic.rs` (4)). |
| 5 | Baseline tests still green | ✅ `wat_arc146_dispatch_mechanism` 7/7; `wat_polymorphic_arithmetic` 20/20; `wat_arc148_ord_buildout` 46/46; FM 9 set 45/45. |
| 6 | Polymorphic comparison still works end-to-end | ✅ All comparison-touching tests pass post-sweep; values_compare/values_equal handle the universal cases including mixed-numeric. |
| 7 | Strict type-locking achievable via param types | ✅ The 2 `_rejects_` tests in `wat_polymorphic_arithmetic.rs` demonstrate the pattern — wrapper helpers with `(:i64,:i64)` / `(:f64,:f64)` params replace the per-Type leaf's check error with a binding-site check error. UX preserved. |
| 8 | Full workspace `cargo test` passes | ✅ Single-threaded canonical view: only documented `CacheService.wat` noise (`deftest_wat_lru_test_lru_raw_send_no_recv`). Identical failure profile pre/post slice. |
| 9 | No new clippy warnings | ✅ 33 warnings pre-existing; the `too many arguments (8/7)` warning at `infer_comparison` was already there as `infer_polymorphic_compare` — same line family as 6 other `infer_polymorphic_*` siblings. |
| 10 | Honest report | ✅ ~250-word report covers all required sections; calibration explicit. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (50-200) | ✅ 75 insertions / 93 deletions = NET -18 LOC. Delete-heavy as predicted. |
| 12 | Style consistency | ✅ Sweep is exact char-substitution where possible; the 2 `_rejects_` tests received clean wrapper-helper refactor in convention-consistent style. |
| 13 | clippy clean | ✅ No new warnings. |
| 14 | Audit-first discipline | ✅ Sonnet's rationale for path (a) named explicitly; no improvisation. |

## Honest deltas

**None surfaced** — slice's scope was contained and well-trodden.

Two doc-comment touch-ups beyond strict letter (NOT honest deltas
in the disrupting sense; just incidental cleanup discovered during
the sweep):

1. `src/runtime.rs:1915` — lexer doc cited `:wat::core::f64::<` as the
   "ends with `<` no closing `>`" example. Updated to bare
   `:wat::core::<` so the structural example uses a name that still
   exists post-retirement.
2. `src/runtime.rs:4499` — `values_equal` doc cited the retired
   per-Type names; updated to point at the param-types-at-call-site
   pattern.

Neither touches behavior.

## Calibration record

- **Predicted Mode A (~80%)**: ACTUAL Mode A. Calibration matched.
- **Predicted runtime (25-40 min)**: ACTUAL ~25 min. AT the lower
  bound; cleanup slice as designed. Same shape as arc 146 slice 4
  (similarly small, similarly mechanical).
- **Time-box (60 min)**: NOT triggered. Used 41%.
- **Predicted LOC (50-200)**: ACTUAL net -18 LOC. UNDER positive band
  (delete-heavy). Honest scope.
- **Honest deltas (predicted 0-1; actual 0)**: matched. Cleanup
  slices have small surface area for surprises.

## Workspace failure profile (pre/post slice)

- **Pre-slice baseline** (post-slice-3): single-threaded clean
  except `deftest_wat_lru_test_lru_raw_send_no_recv` (CacheService.wat
  noise — pre-existing arc 130 issue).
- **Post-slice (single-threaded):** SAME — only the CacheService.wat
  noise. Identical failure profile.
- **Post-slice (multi-threaded):** pre-existing concurrency flakes
  per slice 2 SCORE Delta — NOT introduced by this slice.
  Single-threaded is the deterministic canonical view.

## What this slice closes

- 10 per-Type comparison leaves RETIRED (`:wat::core::{i64,f64}::{=,<,>,<=,>=}`)
- The "polymorphic-handler anti-pattern" framing in
  `infer_polymorphic_compare` REPLACED with the honest name
  `infer_comparison`
- Comparison surface reaches its FINAL shape: 6 polymorphic bare-name
  entities; zero per-Type leaves; zero comma-typed comparison leaves;
  one cleaned check-side inference function

## What this slice unlocks

- **Slice 4** — numeric arithmetic migration (32 names; the largest
  remaining slice in arc 148)
- **Slice 6** — closure paperwork (after slice 4)
- **Arc 146 slice 5** — closure (BLOCKED on arc 148 completion;
  awaits slice 6)

The substrate's comparison surface is now LLM-natural by
construction: no comma-typed crutches; bare polymorphic names
handle every legitimate case; type system enforces correctness
at call-site param types.

## Pivot signal analysis

NO PIVOT. Clean ship; no surprises.

The methodology IS the proof. The rhythm holds.
