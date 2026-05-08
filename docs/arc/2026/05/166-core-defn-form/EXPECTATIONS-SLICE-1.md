# Arc 166 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-60 minutes.**

Reasoning: macro is ~6 lines. Tests are 10 cases, each requiring a
minimal `:user::main` program. Lower-bound (30 min) assumes all
tests work first cargo-test pass; upper-bound (60 min) assumes 1-2
iteration cycles for the no-arg-sig syntax detail (case 7) and
position-rule edge cases (case 6) plus any surprises in macro
expansion.

**Time-box (2× upper-bound): 120 minutes.** Orchestrator schedules
the 90-min wakeup as the soft check; if sonnet is still iterating at
that point, it gets a check-in; hard cap at 120 min via TaskStop +
Mode B-time-violation scoring.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | `wat/core.wat` has the new defmacro section | section comment + defmacro form match BRIEF spec |
| B   | Macro expansion shape | post-expansion AST matches `(:wat::core::def name (:wat::core::fn sig body))` (verifiable via reflection in case 10) |
| C   | Test 1 — simple defn add(2,3)=5 | PASS |
| D   | Test 2 — recursive defn fact(5)=120 | PASS |
| E   | Test 3 — defn at file-root | PASS |
| F   | Test 4 — defn inside `(:wat::core::do ...)` | PASS |
| G   | Test 5 — defn inside top-level `let` body | PASS |
| H   | Test 6 — defn inside `if` branch is REJECTED | startup_err matches def's position-rule diagnostic |
| I   | Test 7 — zero-arg defn | PASS (or honest delta if no-arg sig shape differs from `(-> :T)`) |
| J   | Test 8 — body type-mismatch surfaces | startup_err matches `ReturnTypeMismatch` or equivalent |
| K   | Test 9 — redef same name forbidden | startup_err matches `DefRedefForbidden` |
| L   | Test 10 — reflection lookup-form resolves post-defn | PASS |
| M   | `cargo test --release --workspace --no-fail-fast` clean | 0 failed; total ≥ baseline + 10 new |
| N   | Pre-existing tests unchanged | no regressions; the 10 new tests are additive |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Zero-arg fn sig shape (case 7).** Arc 155's fn-form syntax may
  use `(-> :T)` for no-args OR may require a unit param — the BRIEF
  guesses `(-> :T)`. If this doesn't compile, report the actual
  syntax + adjust the test fixture. Don't bridge a substrate change.
- **Macro arg AST type annotation.** The BRIEF uses
  `:AST<wat::core::nil>` matching the deftest precedent. If macro
  type-checking rejects this for some position (e.g., name needs to
  be `:AST<wat::core::keyword>`), report the diagnostic and adjust.
- **Position rule propagation through macro expansion.** Case 6
  expects the post-expansion def's position-rule walker to fire on
  the synthetic def form inside the if-branch. If the walker walks
  pre-expansion AST and doesn't see the synthetic def, the position
  rule WOULDN'T propagate — that's a substrate gap to surface, not
  bridge. Report the diagnostic; orchestrator decides whether
  arc 166 expands scope to fix the propagation OR opens a follow-up
  arc.
- **Recursive name binding through macro expansion.** Case 2 expects
  the fact body inside `(:wat::core::fn ...)` (post-expansion) to
  see `:user::fact` as bound. If def's name-registration timing
  doesn't reach the fn body inside the post-expansion form, that's
  a surprise to report.
- **Reflection across the macro boundary (case 10).** Reflection on
  `:user::add` should work because the macro expansion lands in the
  SymbolTable through def's standard register path. If reflection
  returns None, the macro-expansion-to-symbol-table path has a gap.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode C
failed). Compare to predicted 30-60 min band; flag if outside.

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-1.md
after scoring all rows.
