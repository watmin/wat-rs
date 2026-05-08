# Arc 167 slice 2 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-90 minutes (opus agent).**

Reasoning: substrate-judgment work — fn-sig parser rewrite (both
runtime + check), `BareLegacyFnSignature` walker (mirrors arc 154's
walker shape), defn macro shape change with rest-binder, 9 tests.
Smaller surface than slice 1 (no new AST variant; just consuming
the variant slice 1 minted). Comparable to arc 154's substrate
shipment (~70 min for the let* retirement substrate side).

**Time-box (2× upper-bound): 180 minutes.** If opus is still
iterating at 90 min, an in-flight check confirms progress; hard
cap at 180 via TaskStop + Mode B-time-violation.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | `parse_fn_signature` consumes `WatAST::Vector` body | `eval_fn` accepts the new 5-element fn-form shape; tests 1, 4, 7 pass |
| B   | `parse_fn_signature_for_check` parallel update | type-check infers correctly for flat shape; tests 1, 7 pass |
| C   | `BareLegacyFnSignature` variant + Display added | git diff confirms variant + Display impl with the EXACT migration text from BRIEF (paste comparison) |
| D   | Walker `walk_for_legacy_fn_signature` fires on legacy | tests 5, 6 pass; legacy-shape fn anywhere in user source fires |
| E   | Walker wired into `check_program` | runs as part of the standard check pipeline; ordered before main inference |
| F   | Defn macro shape updated with rest-binder | git diff confirms the new variadic shape; macro expansion produces the new fn-form |
| G   | Tests 1-4, 7-9 pass | 7/9 success cases pass |
| H   | Tests 5, 6 pass | walker-firing assertions pass |
| I   | `cargo build --release --workspace` green | substrate compiles cleanly |
| J   | `cargo test --release --test wat_arc167_fn_flat_signature` | 9/9 pass |
| K   | Full workspace test count + failure count reported | numerical accuracy of "X passed / Y failed" — Y is slice 3's input; expect tens-to-hundreds of legacy-site failures |
| L   | Slice branch up-to-date on remote | `arc-167-slice-2-fn-sig-consumer` exists at origin with all WIP commits |
| M   | Main untouched | `git log origin/main` hasn't moved during the work |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Rest-binder type annotation shape.** The BRIEF guesses
  `:AST<wat::core::Vector<wat::core::nil>>` for the variadic
  defmacro rest-binder. If defmacro accepts a different shape per
  the existing variadic-define / quasiquote-template precedent in
  `wat/runtime.wat` or `wat/test.wat`, report the actual shape and
  why.
- **`eval_fn` arity reshape.** The BRIEF says fn now takes 4 args
  (sig-vec, arrow, ret, body) after the head. If the eval_fn
  layout differs (e.g., needs to keep accepting an old shape during
  defmacro expansion intermediate states), report.
- **Walker false-positives.** If the walker fires on cases that
  AREN'T legacy fn-sigs (e.g., a list at args[0] of fn that's
  actually a quasi-quoted template or some other valid shape that
  happens to be a List), report the false-positive pattern and how
  you scoped the walker to avoid it.
- **Legacy lib unit tests in `src/runtime.rs`.** The substrate's
  own unit tests (in `src/runtime.rs::tests`) likely use the legacy
  fn-sig shape. They will fire the walker post-substrate-rewrite.
  Slice 2 should NOT fix these — they're slice 3 territory along
  with everything else. Report the count of failing lib tests.
- **Macro expansion span quality.** If the legacy defn shape fires
  the walker via macro expansion, the error span should point at
  the user's source (the legacy defn call), not the macro
  expansion's synthetic fn-form. If span quality is poor, report.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode C
failed). Compare to predicted 60-90 min band.

## Slice-3 setup expectations

This slice INTENTIONALLY leaves the workspace broken at the end.
Slice 3's BRIEF (drafted by orchestrator after this slice ships)
will use the failure count + diagnostic stream to drive the sweep.
The cleaner the migration message in the walker, the easier slice
3's mechanical translation will be.

The number of failing tests at slice 2 ship is the calibration
input for slice 3 prediction:
- < 50 failures: slice 3 is ~30 min sonnet
- 50-200 failures: slice 3 is ~60-120 min sonnet
- 200+ failures: slice 3 is ~120-180+ min sonnet (or split into
  sub-slices)

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-2.md
after scoring all rows + reviewing the branch diff.
