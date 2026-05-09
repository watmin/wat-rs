# Arc 168 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-90 minutes (opus agent).**

Reasoning: this slice is approximately arc 167's slice 2 in
shape — substrate consumer + walker + tests — but with two
extensions:
- `eval_fn` / `infer_fn` body multi-form (purely additive)
- Function::body representation choice (orchestrator-discretion
  decision; opus picks at substrate-judgment level)

Arc 167 slice 2 ran ~75 min opus (Mode A clean, 13 rows). Arc
168 slice 1 should land in similar band. Implicit-do additions
are small per-function changes; the walker is mechanically
mirrored from `BareLegacyLetStar` (arc 154); the Function::body
choice is the only real judgment call.

**Time-box (2× upper-bound): 180 minutes.** If opus is still
iterating at 90 min, in-flight check; hard cap at 180.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `parse_let_binding` consumes flat (binder, expr) chunks | `eval_let` accepts new shape; tests 1-6 pass | ✓ |
| B — `eval_let` accepts Vector outer + multi-form body | tests 1-5, 10, 14 pass | ✓ |
| C — `infer_let` parallel update | type-check passes for new-shape; test 11 surfaces type mismatch correctly | ✓ |
| D — `eval_fn` accepts multi-form body | tests 12, 15 pass | ✓ |
| E — `infer_fn` parallel update | fn body type-check works for multi-form | ✓ |
| F — `try_parse_fn_shape_def` arity expansion | defn pre-registration still works (arc 166 tests still pass) | ✓ |
| G — `BareLegacyLetBindings` variant + Display + Diagnostic | git diff confirms variant + Display impl with verbatim migration text from BRIEF | ✓ |
| H — Walker fires on legacy outer-List shape | test 7 passes | ✓ |
| I — Walker wired into pipeline | `freeze.rs` user-source pre-pass region | ✓ accepted |
| J — Walker recurses into Vector children | mirror arc 167 slice 3 substrate gap fix; if missing, surface as honest delta | ✓ accepted |
| K — Defn macro forwards multi-form body | test 13 passes; macro shape unchanged in `wat/core.wat` (verify, don't edit) | ✓ |
| L — Migration message text load-bearing | test 8 passes (verbatim text assertion) | ✓ |
| M — Empty cases handled | empty bindings (test 4), empty body (test 5), odd-count error (test 9) all pass | ✓ |
| N — `cargo build --release --workspace` green | substrate compiles cleanly | ✓ |
| O — `cargo test --release --test wat_arc168_let_flat_shape` 15/15 | full new-test-file pass | ✓ |
| P — Arc 167 tests still pass (regression) | `cargo test --release --test wat_arc167_fn_flat_signature` 9/9 | ✓ |
| Q — Lib unit tests stay 793/0 | substrate-internal `mod tests` use single-body; not affected | ✓ |
| R — Workspace failure count reported | `./scripts/cargo-test-summary.sh` shows the let-callsite failure stream (~563 sites; this is slice 2's input) | ✓ |
| S — Slice branch on remote | `arc-168-let-flat-shape` carries opus's commits; main untouched | ✓ |
| T — Function::body representation choice documented | report names choice (a/b/c/own) + rationale | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Function::body shape conflict.** If choice (b) (synthesize
  `do` AST for multi-body) hits the existing `do`'s empty-arity
  check, that's a real substrate conflict. STOP and report;
  orchestrator decides between extending `do` vs picking a
  different body representation.
- **Walker recursion gap.** If `walk_for_bare_primitives` Vector
  arm needs extension to cover nested Vector destructure binders
  (Vector-inside-Vector), surface as a separate substrate gap
  the same way arc 167 slice 3 surfaced its Vector-recursion
  fix. The fix is structurally identical; surface as honest delta
  before applying.
- **Defn macro forwarding fails.** If splicing `,@rest` doesn't
  forward N body forms cleanly through fn expansion, that's a
  macro-engine substrate gap. STOP and report.
- **Type-check on intermediate body forms.** If `infer_let` /
  `infer_fn` need a new "infer-and-discard" pass that doesn't
  exist yet, surface the gap. The existing `do` form's
  type-checking is the precedent — if `do` already does this
  correctly, mirror it; if `do` does it differently, surface the
  divergence.
- **Function::body call-site iteration.** Wherever Function::body
  is consumed (call_function, etc.), the multi-form representation
  needs to thread through. If a hidden caller depends on body
  being a single AST, surface that.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 60-90 min band.

Function::body choice: ___ (a / b / c / own).

Honest deltas surfaced: ___ (count + brief).

## What's next (orchestrator-side, post-slice-1)

When slice 1 ships green:

- Slice 2 BRIEF + EXPECTATIONS for the sweep (sonnet — first
  real sonnet sweep on the new `.claude/settings.json`
  permission discipline; this is calibration data the discovery
  in arc 167 slice 4b paid for)
- ~563 sites across `wat/`, `wat-tests/`, `tests/`
- The recipe: outer `((n e) (n e))` → outer `[n e n e]`; legacy
  typed-single `((name :T) rhs)` → `[name rhs]` (arc 159
  retired user-side typed-single but the parser still accepts;
  arc 168 slice 1 walker fires on the typed-single via the
  outer-List pattern); destructure `((a b c) rhs)` → `[[a b c]
  rhs]`

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1.md
to slice branch after scoring all rows + reviewing the diff.
