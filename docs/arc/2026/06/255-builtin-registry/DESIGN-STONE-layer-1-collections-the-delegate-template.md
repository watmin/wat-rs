# DESIGN — STONE layer-1: collections return to the impl layer, and the delegate gets a TEMPLATE

> **Builder, 2026-08-30:** *"`src/intrinsic/<ns>.rs` … these are for registration and delegation.
> `src/<some-mod>/<some-files>.rs` … these are for actual impls. In the medium term we will move
> the vast majority of wat's code into `crates/wat-*/`. Getting the registry and re-homing done is
> the first step towards breaking wat up into many crates."* Then: *"collections go first."*

## Why this stone, and why it is ours to fix

**Stone P6-c-W6 (`5725ab10d`, three hours ago) moved four implementations OUT of the impl layer.**
`eval_rest` left `src/collection/eval.rs`; `eval_vec_last`/`eval_vec_reverse`/`eval_vec_range` left
`src/collection/transform.rs`. They went into `src/intrinsic/collection.rs` — the registration
layer — because the brief said "home the verb" and never named the layer discipline, which did not
exist in writing at the time.

That makes this the ideal first case: the destination is exactly where the code lived this morning,
`git show 5725ab10d` holds the known-good before-state, and reversing our own move is the cheapest
way to derive the template rather than invent it.

## ⛔ The measurement that decides the SHAPE — and the one that failed

**Measured, 404 attributed fns under `src/intrinsic/`: the median body is EIGHT LINES.** The
delegate pattern is already the majority — `i64.rs`'s `:wat::i64::+` is a four-line body calling
`crate::runtime::eval_i64_arith`. Roughly 90 functions carry real implementation, ~3,700 body
lines, and `holon/atom.rs` is 40% of that alone.

⚠ **There is no convention being restored here.** The homes run a continuum from 19.8 to 121.5
lines per verb with no bimodality; both patterns shipped and neither was ever ruled. This is a
FORWARD discipline, not a cleanup of drift.

⛔ **AND A GATE IS DELIBERATELY NOT IN THIS STONE, because two candidate predicates both FAILED:**

- `RuntimeErrorKind::` / `match`-on-value → flags `eval_program_env_intrinsic`, which is a CORRECT
  delegate: it calls `crate::services::current_program_env()` and adapts `None` to a wat error.
  **Error adaptation is the door's job**, so the predicate condemns the thing it should bless.
- "calls out to a non-`intrinsic` module" → flags 41 of `time.rs`'s fns at ~12 lines each, which
  are almost certainly fine; a delegate can call out through an import with no `crate::` path.

The reason is structural, not a bad regex: `eval_holon_from_holon` mixes arity checking and
`-> :T` annotation parsing (**the door's job**) with the conversion itself (**the impl's job**).
The boundary is a judgement per verb. **So this stone's real product is the worked template the
gate's predicate will later be DERIVED from.** Writing the gate first would be a design shipped
before anything consumed it. `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

## The seven, and where each impl belongs

```
  56  eval_length        → src/collection/eval.rs       (came from runtime.rs)
  73  eval_empty         → src/collection/eval.rs       (came from runtime.rs)
 148  eval_nth           → src/collection/eval.rs       (came from runtime.rs)
 120  eval_rest          → src/collection/eval.rs       ← REVERT: W6 took it from there
   8  eval_vec_last      → src/collection/transform.rs  ← REVERT
  56  eval_vec_reverse   → src/collection/transform.rs  ← REVERT
  15  eval_vec_range     → src/collection/transform.rs  ← REVERT
 ---
 476  body lines returning to the impl layer
```

★ `length`/`empty?`/`nth` never lived in `src/collection/` — they came from the megafile. They go
there anyway, because **`src/collection/eval.rs` already holds the 50 `*_inner` helpers their
bodies call** (`vector_length_inner`, `hashmap_length_inner`, `record_length_inner`, …). Their
dispatch belongs beside the helpers it dispatches to.

★ `eval_vec_last` (8 lines) and `eval_vec_range` (15) are already delegate-sized. They still move —
the axis is WHERE THE IMPLEMENTATION LIVES, not how long it is. A short impl in the registration
layer is still an impl in the registration layer, and a size-based rule is exactly the predicate
that just failed.

## The one contract decision, pinned

**The `#[wat_intrinsic]` attribute and its doc block STAY in `src/intrinsic/collection.rs`.** Only
the body moves. The attributed fn keeps its name, its signature, its declared arity and its whole
doc contract, and its body becomes a single call.

This is forced, not chosen: `#[wat_intrinsic]` is an attribute ON a function, and
`rete::purity::completeness_gate::dispatch_verbs` scans `src/intrinsic/` for it. Move the attributed
fn out and the verb vanishes from the completeness population — the blind spot documented in
`NOTE-the-completeness-gate-cannot-see-a-home-outside-one-directory.md`. `i64.rs` proves the
attribute sits happily on a four-line delegate.

## Out of scope = REJECTED

- **No gate, no lint, no ledger.** Derived from this stone's result, in a later one.
- **No other home is touched.** `holon/atom.rs` is 40% of the remaining mass and is not this stone.
- **No behaviour change.** Every body moves VERBATIM; only its file changes.
- **No `#[wat_intrinsic]` leaves `src/intrinsic/`.**

## Calibration

Predicted 30–50 min. Comparable: W6 itself, which moved the same seven the other way.
