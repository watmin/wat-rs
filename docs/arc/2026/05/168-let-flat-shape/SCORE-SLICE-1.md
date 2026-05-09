# Arc 168 slice 1 — SCORE

Substrate consumer + walker + body implicit-do for the let
flat-Vector binding shape and the multi-form body extension to
both let and fn. Mode A clean (with deltas surfaced after
slice 2 closure). Three commits on the slice branch:

- `fcf45fe` — slice 1 WIP (eval_let Vector outer + multi-form body)
- `8b24398` — eval_fn / try_parse_fn_shape_def / step_let multi-form body + walker
- `83000b7` — tests + odd-count check.rs error

Predicted ~120-180 min opus. Actual: not separately measured
(work shipped across two opus session windows pre-compaction).

## Scope as shipped

### Substrate consumer

`src/runtime.rs`:
- `eval_let` / `eval_let_tail` accept `WatAST::Vector` outer; even-
  length required; slice 1 also kept legacy `WatAST::List` fall-
  through (slice 3 retired)
- `parse_let_binding` rewritten to take `(binder, rhs)` where binder
  is `Symbol` (single) or `Vector` of Symbols (destructure)
- `parse_legacy_let_binding` scaffolded as a holding-pattern
  function for the legacy shape (slice 3 retired)
- `bind_let_binding` helper centralized scope-extension
- Body becomes implicit-do over `args[1..]`; empty body returns
  `:wat::core::nil` (per DESIGN's settled answer)
- `eval_fn` accepts `args.len() >= 3`; `synthesize_fn_body` peels
  body forms (empty → nil keyword; single → pass-through; multi →
  `(:wat::core::do f1 f2 ... fN)`). Function::body stays
  `Arc<WatAST>` — representation choice (b) per BRIEF.
- `try_parse_fn_shape_def` accepts 4+ fn-form elements
- `step_let` rewritten for both outer-Vector + outer-List bindings
  AND multi-form body
- `synthesize_let_body` collapses body forms into nil/single/do
- `rebuild_let_with_first_rhs` preserves outer shape

### Check side (`src/check.rs`)

- `infer_let` accepts Vector outer; both shapes desugared to a
  uniform `Vec<2-list pair>` for `process_let_binding`
- Multi-form body synthesized to single body AST mirroring runtime
- `process_let_binding` gains Vector binder arm (destructure)
- `infer_fn` / `parse_fn_signature_for_check_diag` accept 3+ args;
  body synthesized identically to runtime path
- `BareLegacyLetBindings` variant + Display + Diagnostic with
  verbatim migration text
- `validate_legacy_let_bindings` + `walk_for_legacy_let_bindings`
  fire on `(:wat::core::let LIST ...)` shape; recurse through
  Vector + List children

### Freeze (`src/freeze.rs`)

- Walker registered in user-source pre-pass alongside
  `validate_bare_legacy_primitives` (mirrors arc 167 slice 2
  delta A)

### Tests (`tests/wat_arc168_let_flat_shape.rs`)

15 cases per BRIEF: single binding, multiple, sequential refs,
empty bindings, empty body (Clojure-faithful nil), destructure,
walker firing (test 7 — retired in slice 3), verbatim migration
text (test 8 — retired in slice 3), odd-count vector error,
multi-form let body + type-check, multi-form fn body, multi-form
defn body, single-body regressions for let + fn.

All 15 tests passed at slice 1 ship. Arc 167's 7 tests passed
(regression). Lib unit tests stayed 793/0 — note this baseline
held because the lib unit-test fixtures using legacy shape were
walker-skipped (per arc 167 slice 2 delta A) and the legacy
parser fall-through still accepted them. Slice 3 retirement
made these fixtures fail as expected (slice 4 sweep fixed them).

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — Vector outer accepted | `eval_let` + `infer_let` consume `args[0] = Vector` | ✓ |
| B — Symbol binder accepted | `parse_let_binding` Symbol arm; bare-name binding works | ✓ |
| C — Vector destructure accepted | `parse_let_binding` Vector arm; `[a b c]` destructure works | ✓ |
| D — Empty bindings → :nil | `[]` body returns `:wat::core::nil` | ✓ |
| E — Empty body → :nil | `(let [x 1])` returns `:wat::core::nil` | ✓ |
| F — Multi-form body (let) | `(let [x 1] f1 f2 f3)` evaluates f1, f2 for side effect; returns f3 | ✓ |
| G — Multi-form body (fn) | `(fn [x <- :T] -> :R f1 f2 f3)` same shape | ✓ |
| H — Multi-form body (defn) | macro expansion forwards N body forms cleanly | ✓ |
| I — Walker fires on legacy outer-List | `BareLegacyLetBindings` variant fires; verbatim message text | ✓ (retired in slice 3) |
| J — Migration text verbatim | walker diagnostic carries the BRIEF's migration recipe | ✓ (retired in slice 3) |
| K — Odd-count vector error | `[x]` and `[x 1 y]` produce clean `MalformedForm` | ✓ |
| L — Single-body regression (let) | old single-form body still works unchanged | ✓ |
| M — Single-body regression (fn) | old single-form body still works unchanged | ✓ |
| N — Slice branch on remote | `arc-168-let-flat-shape` carries `fcf45fe` + `8b24398` + `83000b7`; main untouched | ✓ |

## Honest deltas (surfaced post-slice-1, fixed in slice 2 follow-up `b220846`)

### Delta A — `register_runtime_defs_form` companion path missed

Slice 1 updated `eval_let` (call-time path) to consume Vector
outer + multi-form body but missed the freeze-time companion
`register_runtime_defs_form` in `src/runtime.rs` which:
- Only handled legacy List outer bindings (the form arc 168 retires)
- Only looked at `items[2]` for body (single-form assumption)

Result: `def` inside arc-168-shape let bodies never registered
into `runtime_def_values`. Surfaced via `def_runtime_let_splice_closure_capture`
(arc 157 test) + `defn_inside_top_level_let_body_works` (arc 166
test) failing post-slice-1.

Fix: rewrote let arm Vector-only (Symbol binder), iterates
`items[2..]` for multi-form body. Mirrors `eval_let`'s discipline.

### Delta B — `collect_splice_defs_ctx` (check.rs) had same multi-body assumption

Parallel to Delta A on the check side. Slice 2 follow-up applied
the same multi-body iteration fix.

### Delta C — Empty-bindings test fixture missed in slice 2 sweep

`empty_bindings_evaluates_body_directly` (arc 154 test) used
legacy outer-list `()` empty bindings. Slice 1 didn't migrate
the degenerate `()` empty form because grep targeted `((`
patterns. Slice 2 sweep didn't catch it for the same reason.
Slice 2 closure surfaced + fixed.

## FM 5 incident — caught + reverted within slice 2 follow-up

First draft of `register_runtime_defs_form` fix (slice 2
follow-up) kept legacy List outer + legacy typed-single binder
support "to make the fix work." User caught the FM 5 pattern —
accepting forms the substrate is retiring creates walker-vs-
registrar disagreement. Reverted to Vector-only.

Recorded as Discipline lesson in slice 2 SCORE delta A. Pattern
reinforced in slice 3 + slice 4 follow-ups (FM 5 held throughout
substrate retirement + parallel walker fixes).

## Calibration row

Slice 1 didn't have a separate runtime measurement — work
shipped across two opus sessions pre-compaction. Closure
paperwork (this SCORE) captures the historical record after
slice 4 closure brings the arc 168 territory to clean state.

## Cross-references

- `BRIEF-SLICE-1.md` — original scope statement
- `EXPECTATIONS-SLICE-1.md` — original prediction + scorecard
- `SCORE-SLICE-2.md` delta A — surfaces slice 1's missed companion paths
- `SCORE-SLICE-3.md` — substrate retirement made walker tests vacuous
- `SCORE-SLICE-4.md` — lib unit-test fixture sweep + parallel walker fixes
