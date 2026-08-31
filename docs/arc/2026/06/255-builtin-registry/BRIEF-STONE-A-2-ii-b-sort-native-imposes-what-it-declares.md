# BRIEF — STONE A-2-ii-b: `sort$native` imposes what it declares

Make `:wat::core::sort$native` refuse an impure or nondeterministic comparator at its door, and home
it into the registry declaring `Pure`/`Deterministic` — the gate and the declaration in one stone,
because a declaration the door does not enforce is a lie. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-b-sort-native-imposes-what-it-declares.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned coupling, the rulings, and the falsifiable prediction.
2. **`src/freeze.rs`, the sigma-fn purity gate** (search `sigma fns must be pure`) — **the working
   precedent for exactly this imposition**: it loops `[Axis::Pure, Axis::Deterministic, Axis::Total]`,
   calls `find_axis_violation` per axis, handles `FunctionBody::Native` via `classify_native_fn`, and
   raises naming the offending head. Copy its shape; you need only the first two axes.
3. `src/collection/transform.rs`, `eval_vec_sort_by` — where the gate goes. Note the existing
   `Value::wat__core__fn(func)` destructure; your check goes immediately after it and **before**
   `sorted.sort_by(...)`.
4. `src/rete/purity.rs` — `ClassifyCtx`, `find_axis_violation_ctx`, and `classify_closure`, all
   shipped by A-2-i/A-2-ii-a. The comparator is a `Function` carrying `closed_env`; that is the
   environment the classifier needs.
5. `src/intrinsic/collection.rs` — the delegate template (and read its header note on the 1-arity
   `std::slice::from_ref` idiom; `sort$native` is 2-arity, so `&[a.clone(), b.clone()]` is correct
   here).
6. `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`
   — why `Total` is declared but **not imposed**.

## The work

### 1 — the gate

In `eval_vec_sort_by`, after the comparator is destructured to a `Function` and **before any
comparison runs**, refuse a comparator that is not Pure ∧ Deterministic. Classify against the
function's own `closed_env` — `ClassifyCtx::Runtime(..)` when it has one, `Static` when it does not —
mirroring how `classify_closure` already carries a closure's own environment rather than the
caller's.

⛔ **Refuse before sorting, never during.** A refusal raised mid-sort would already have run the
caller's comparator on some pairs, emitting exactly the effects this gate exists to prevent. The
acceptance row is that **zero** effects are observable before the error.

The error should name the offending head, as `freeze.rs`'s does, and carry the call span.

### 2 — the homing

`#[wat_intrinsic(":wat::core::sort$native")]` in `src/intrinsic/collection.rs`, a thin 2-arity
delegate over the existing `eval_vec_sort_by`. Declare:

```
@Purity      Pure            — true because the door above imposes it
@Determinism Deterministic   — same
@Total       Total           — on its own merits; a pathological comparator returns a scrambled
                               well-formed vector, exit 0, no panic (measured)
@ExpandTime  Legal
```

Write the grounding prose per axis as the template does. **`@Purity Pure` must cite the gate** — the
declaration is true *because* of it, and a reader should not have to guess that.

Then remove what now derives: the literal dispatch arm in `src/runtime.rs`, the `KNOWN_UNREVIEWED`
row in `src/rete/purity.rs`, and the `macros/eval.rs` expand-time list entry. Its `check.rs`
TypeScheme **stays** (homing does not retire a scheme).

### 3 — the probe

Write `wat-scripts/scratch-pad/255-probe-sort-imposes-purity.wat`, following the shape of the
existing probes in that directory:

- an **effectful** comparator → the program fails, and **no comparator output appears first**;
- `sort/1`, `sort/2` and `sort-by` with a pure key fn → unchanged results.

## Blast radius

`src/collection/transform.rs` · `src/intrinsic/collection.rs` · `src/runtime.rs` (one arm out) ·
`src/rete/purity.rs` (one row out) · `src/macros/eval.rs` (one entry out) · the new probe. No changes
to `src/freeze.rs`, to `sort`/`sort-by` in `wat/core.wat`, or to any other verb's registration.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — no effects before the refusal.** If you cannot place the check so that a refused
comparator runs zero times, STOP and report what forced it. A gate that fires after two comparisons
has already lost the thing it was protecting.

**STOP-2 — do not impose `Total`.** Declared yes, imposed no. If imposing it seems necessary, re-read
the RULING: every record accessor is `Partial` through `Option/expect`, so a `Total` demand refuses
`wat/query/mem.wat`'s live call sites for no defect.

**STOP-3 — the debt-ledger prediction is falsifiable.** The design predicts **no**
`FROZEN_CHECKER_DEBT_LEDGER` row is needed, because `sort$native` has an `env.register()` TypeScheme
at `src/check.rs:20322`. If you find a row IS required, STOP and report — the measurement is wrong,
and that is a finding, not a row to add quietly.

**STOP-4 — a real caller refused.** The three live shapes were measured passing (a record accessor
keyfn, an inline-`fn` keyfn, the default `<`). If your gate refuses any of them, STOP and report
which and why — that is a defect in the gate or a gap in the classifier, never a corpus to fix.

## Report

Per-file diff summary; where exactly you placed the check and how you guaranteed zero comparisons
run before a refusal; the probe's output from the pre-existing binary; and whether the debt-ledger
prediction held. Then the part the orchestrator cannot reconstruct: what surprised you — a
comparator shape the design did not predict, a place where `closed_env` was not what you expected, or
a caller that came closer to refusal than the measurements implied.
