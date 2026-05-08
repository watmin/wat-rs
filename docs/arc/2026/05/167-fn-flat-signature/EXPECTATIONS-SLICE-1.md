# Arc 167 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-120 minutes (opus agent).**

Reasoning: substrate-judgment-heavy work — new AST variant + parser
path + cross-cutting match-arm propagation across ~5-15 sites + new
error arms in eval/check + 5 tests. The match-arm propagation count
is the variable. Comparable in shape to arc 157 slice 1a-i (def
substrate mint, ~390 LOC) which ran ~90 min for opus-tier work.

**Time-box (2× upper-bound): 240 minutes.** Orchestrator schedules a
60-min wakeup as the soft check; if opus is still iterating at the
upper bound, an in-flight check confirms it's making progress; hard
cap at 240 min via TaskStop + Mode B-time-violation scoring.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | `WatAST::Vector(Vec<WatAST>, Span)` variant added to `src/ast.rs` | git diff confirms variant + span() arm + any other trait impls updated |
| B   | Parser produces `WatAST::Vector` from `[...]` brackets | tests 1-3 pass; manual debug-print confirms shape |
| C   | `eval` arm errors on `WatAST::Vector` at value position | test 4 passes; error message contains the literal "vector literals at value position are not supported" string |
| D   | `check`/`infer` arm errors on `WatAST::Vector` at value position | test 5 passes |
| E   | Match-arm propagation: cargo build green | `cargo build --release --workspace` 0 errors |
| F   | Existing tests unaffected by the new variant | pre-arc-167 baseline (`117 OK / 0 FAILED` from arc 166's closure) preserved |
| G   | New tests in `tests/wat_arc167_vector_ast.rs` | 5 cases all pass |
| H   | No clippy regressions | `cargo clippy --release --workspace`: no new warnings (existing warnings unchanged) |
| I   | Slice branch on remote | `arc-167-slice-1-vector-foundation` exists on origin and contains all WIP commits |
| J   | Main untouched | `git log origin/main..origin/arc-167-slice-1-vector-foundation` shows new commits; `git log origin/main` hasn't gained new commits during the work |
| K   | Test names follow convention | per arc 153/154/155 precedent: snake_case, descriptive, single-purpose |
| L   | Error message text matches BRIEF | the exact "vector literals at value position are not supported in arc 167" prose appears in the eval AND check arms |

## Honest-delta categories (if surfaced, report; don't bridge)

- **EDN round-trip behavior.** If `Value::Vector` ↔ `WatAST::Vector`
  conversion in `src/edn_shim.rs` requires a substrate-architectural
  decision (preserve vector semantics across serialization, vs
  collapse to list, vs error), report and let orchestrator decide.
  Default safe answer: preserve vector semantics (Value::Vector
  round-trips to WatAST::Vector and back).
- **Parser ambiguity at empty form.** `()` is the unit value; `[]`
  is empty Vector. The parser must distinguish them cleanly. If
  the existing tokenizer doesn't distinguish, that's a substrate
  gap — report.
- **Display impl quirks.** If `WatAST::Display` exists and is used
  in error messages, the Vector arm should render as `[a b c]`
  (matching the source syntax). If it renders as `(a b c)` to
  reuse the List path, that's confusing — fix or report.
- **Walker recursion behavior.** Walkers in `src/check.rs` that
  recurse through children should treat Vector children identically
  to List children for the recursion (children are walked; the
  Vector itself doesn't carry semantic weight in arc 167). If a
  walker has position-aware logic that needs to distinguish Vector
  from List (e.g., "this can only be a List in arg position"),
  report — that's slice 2's territory.
- **Comment/docstring/data-position uses.** If wat-edn parses a
  vector inside a docstring or some other "data" position that
  pre-existed but was never exercised, the new Vector arm might
  fire a spurious error. Audit and report.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode C
failed). Compare to predicted 60-120 min band; flag if outside.

Number of sites needing explicit `WatAST::Vector` arms: ___ (predict
5-15; report actual for future arc-prediction calibration).

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1.md
after scoring all rows + reviewing the branch diff.
