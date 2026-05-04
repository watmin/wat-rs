# Arc 148 Slice 4 — Pre-handoff expectations

**Drafted 2026-05-03.** The big slice. Substrate work spans
`src/runtime.rs` + `src/check.rs` + `wat/core.wat` + possibly
`tests/wat_polymorphic_arithmetic.rs`. Predicted MEDIUM-LARGE slice
(Mode A ~50%; Mode B-architecture-pivot ~25% — Q1 path call may
flip mid-slice; Mode B-test-shape-update ~15%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-4.md`
**Output:** EDITS to `src/runtime.rs` (retire eval_poly_arith + add
8 mixed-type Rust primitives + dispatch arms + freeze entries) +
`src/check.rs` (retire/rename infer_polymorphic_arith + add 8 mixed
TypeScheme registrations) + `wat/core.wat` (4 Dispatch entities + 8
same-type variadic wat fns + possibly 4 polymorphic variadic wat fns
if Path A/B). Possibly EDITS to `tests/wat_polymorphic_arithmetic.rs`
if test shapes change.

## Setup — workspace state pre-spawn

- Arc 148 slices 1, 2, 3, 5 shipped (audit; rename per-Type leaves to
  `,2`; values_compare buildout; comparison cleanup).
- Arc 150 (variadic define) shipped including the TypeScheme inline-
  field cleanup. `:wat::core::define` accepts `& (rest :Vector<T>)`.
- Workspace baseline (per FM 9, post-arc-150): 9 substrate-foundation
  test files green = 133/133. Workspace failure profile is the
  documented arc 130 HologramCacheService noise + pre-existing
  `call_stack_populates_on_assertion` panicking-test.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (eval_poly_arith retire + 8 mixed primitives + dispatch arms + freeze entries) + `src/check.rs` (infer_polymorphic_arith retire/rename + 8 TypeScheme registrations) + `wat/core.wat` (4 Dispatch entities + 8+ wat-defined variadic fns). NO new files in `src/`. NO new test files (existing tests may be updated for new shape). |
| 2 | 8 mixed-type Rust primitives shipped | All 8 names registered: `:wat::core::+,i64-f64`, `:+,f64-i64`, `:-,i64-f64`, `:-,f64-i64`, `:*,i64-f64`, `:*,f64-i64`, `:/,i64-f64`, `:/,f64-i64`. Each has dispatch arm + TypeScheme + freeze-pipeline entry. |
| 3 | 4 binary Dispatch entities shipped | All 4 named `:wat::core::<v>,2` declared in `wat/core.wat` mirroring arc 146 pattern. Each has 4 arms covering (i64,i64), (f64,f64), (i64,f64), (f64,i64) routing to per-Type `,2` leaves and mixed leaves. |
| 4 | 8 same-type variadic wat fns shipped | All 8 named `:wat::core::<Type>::<v>` in `wat/core.wat`. Use arc 150's variadic-define `& (xs :Vector<T>)` syntax. Body handles arity branching (0/1/2+) per Lisp/Clojure rules: `+`/`*` 0-ary returns identity; `-`/`/` 0-ary errors; 1-ary returns arg (for `+`/`*`) or inserts identity-on-left (for `-`/`/`); 2+-ary folds over `,2` leaf. |
| 5 | Polymorphic variadic surface shipped (Path A/B/C decision) | EITHER 4 polymorphic variadic wat fns (Path A/B) OR 4 polymorphic substrate primitives via custom inference (Path C). Path chosen + rationale named in report. The user-facing call shape `(:wat::core::+ x y z ...)` works including mixed-numeric per the DESIGN's worked example `(:wat::core::+ 0 40.0 2) => :f64 42.0`. |
| 6 | RETIRED: eval_poly_arith + dispatch arms + freeze entries | `eval_poly_arith` function gone. 4 polymorphic dispatch arms at runtime.rs:2744-2747 gone. 4 freeze-pipeline pure-redex polymorphic entries at runtime.rs:15889-15892 gone. PolyOp enum gone (if no other consumer). |
| 7 | RETIRED: infer_polymorphic_arith handling | If Path A/B: function + dispatch site retired. If Path C: function renamed (e.g., to `infer_arithmetic`) + body simplified per slice 5 precedent; dispatch site updated to new name. |
| 8 | All baseline tests still green | All 9 baselines: `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_polymorphic_arithmetic` (number TBD — may change shape); `wat_arc148_ord_buildout` 46/46; `wat_arc150_variadic_define` 16/16; `wat_variadic_defmacro` 6/6. |
| 9 | Worked example from DESIGN passes | `(:wat::core::+ 0 40.0 2)` evaluates to `:f64 42.0` (mixed-numeric variadic via dispatch + per-pair routing). Add or verify a test that exercises this. |
| 10 | Honest report | ~300-word report covers all required sections from BRIEF. Path chosen + rationale; counts of new/retired entities; test results; workspace state; honest deltas. |

**Hard verdict:** all 10 must pass. Rows 4 + 5 + 6 are the load-
bearing rows (variadic surface working + polymorphic surface working
+ legacy retired).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 600-1500 LOC (substrate edits ~300-600 + new wat fns ~200-400 + retirement deletions ~100-300 + possible test updates ~100-200). >2000 LOC = re-evaluate scope. |
| 12 | Style consistency | Mixed-type Rust primitives mirror eval_poly_arith's per-pair impl style. Dispatch entity shape mirrors arc 146 length/get/etc. Variadic wat fns mirror the foldl-over-rest pattern from `tests/wat_arc150_variadic_define.rs`. If Path C: renamed function mirrors slice 5's `infer_comparison` shape. |
| 13 | clippy clean | No new warnings. |
| 14 | Audit-first discipline | If Path A/B blocked, surface clearly + pivot to Path C with rationale. Don't silently bridge over a substrate gap. If retire-eval_poly_arith breaks something, surface honestly + don't paper over. |

## Independent prediction

- **Most likely (~50%) — Mode A clean ship via Path C.** Slice 5's
  precedent + arc 146 Dispatch pattern + arc 150 variadic define all
  provide templates. Path C avoids the polymorphic-variadic typing
  gap. ~60-90 min wall-clock.
- **Mode B-architecture-pivot (~25%):** sonnet attempts Path A/B,
  hits the substrate gap (no clean way to type Vector<numeric> for
  the polymorphic surface), pivots to Path C mid-slice. Adds 15-30
  min vs predicted; surfaces in report.
- **Mode B-test-shape-update (~15%):** the existing
  `wat_polymorphic_arithmetic.rs` tests have assertions tied to the
  OLD eval_poly_arith error messages or shapes; sonnet needs to
  update test assertions. Honest delta.
- **Mode C (~10%):** unforeseen substrate edge — e.g., wat-defined
  variadic fn calling Dispatch entity has a check/runtime mismatch;
  freeze pipeline pure-redex entry needs different handling for the
  new Dispatch+wat-fn shape; etc.

## Time-box

120 min wall-clock (≈1.3× the predicted upper-bound of 90 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

**Arc 148 slice 6** — closure paperwork (small).
**Arc 146 slice 5** — closure (was blocked on arc 148; unblocks at
slice 6).
**Arc 144 closure** — verification + paperwork queue (becomes
tractable post-arc-148).
**Arc 109 v1 closure trajectory** — one major chain link closes.

The polymorphic-handler anti-pattern for arithmetic is retired.
Every arithmetic op is first-class. The substrate's "extend the
carrier" + "Dispatch + per-Type leaves" + "variadic wat fn over
binary dispatch" patterns are all exercised end-to-end.

## After sonnet completes

- Re-read the audit's arithmetic section against the SCORE
- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows (4, 5, 6) by running tests + spot-checking
  the new entity registrations
- Write `SCORE-SLICE-4.md`
- Commit the SCORE before drafting slice 6's BRIEF (calibration
  preserved)
