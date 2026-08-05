# DESIGN STONE — per-type equality RESTORED; 237.8d part B reversed

**Ruled 2026-08-05 by the builder.** *"i think both `:wat::core::=` and `:wat::core::i64::=` should be
a thing."*

⛔ **THIS REVERSES A RECORDED HARD CUT. Read this before you "restore consistency" by re-cutting.**
Stone 237.8d (`docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.8d.md`) deleted
`:wat::core::i64::=`, `:i64::not=`, `:f64::=`, `:f64::not=` and installed a guard set asserting they
stay dead. That cut is being reversed **in part**, on the ground below. The guard set inverts; it is
not deleted.

## What 237.8d got RIGHT, and is NOT touched

Its load-bearing claim stands and this stone does not disturb it:

> *"The clause matcher checks each arg against a fixed named type independently (`assignable`
> per-position) and **never unifies arg0's type with arg1's**; equality *is* that cross-argument
> unification (`infer_equality` does `unify(a,b)`, ∀T). A monomorphic clause cannot express it."*

**Equality is an intrinsic, not a defclause.** `eval_eq` / `eval_not_eq` / `values_equal` /
`infer_equality` are untouched. `docs/DISPATCH.md`'s projective/relational partition is untouched.
Part A of 237.8d (the source-marking) is untouched. **Nothing here proposes equality-as-defclause.**

## What it got WRONG — the test does not distinguish, proven on the disk

237.8d justified cutting the *aliases* on this test:

> *"`:i64::=` … are **fake per-Type leaves** for a uniform op: each dispatches *directly* to
> `eval_eq`/`eval_not_eq`, parallel to `:wat::core::=` — **not as a leaf of any defclause** (unlike
> `:i64::+`, which the `+` defclause genuinely calls)."*

Applied consistently, that test cuts the **ordering** aliases too. It did not. Grounded
2026-08-05, `src/runtime.rs`:

| head | dispatches to | a defclause leaf? | 237.8d verdict |
|---|---|---|---|
| `:wat::core::=` (`:5189`) | `eval_eq` | no | kept (canonical) |
| `:wat::core::i64::=` | `eval_eq` | no | **CUT** |
| `:wat::core::>` (`:5192`) | `eval_compare` | no | kept (canonical) |
| `:wat::core::i64::>` (`:5208`) | `eval_compare` | no | **KEPT** |
| `:wat::core::i64::+` | the `+` defclause genuinely calls it (`wat/core.wat:58`) | **yes** | kept |

`:wat::core::>` is a **native polymorphic comparator inside `eval_compare`**, not a defclause dispatch
— corrected on the record in `DESIGN-STONE-where-admits-only-rete-ops.md`. So `i64::>` is a "fake
per-Type leaf" by 237.8d's own definition and survived. The rule was applied to one family and not to
its twin. That asymmetry, not the relational-intrinsic argument, is what this stone corrects.

## What the alias BUYS — measured, not asserted

Generic `=` requires the two operands be the **same type**; it does not lock them to a *specific*
type. Proven by run (`target/release/wat --check`, 2026-08-05):

```
(:wat::core::=       "a" 1)  →  TypeMismatch: parameter #2 expects :wat::core::String; got :wat::core::i64   exit 1
(:wat::rete::String::= "a" 1) →  TypeMismatch: parameter #2 expects :wat::core::String; got :wat::core::i64   exit 1
```

Both reject the *mismatch*. Neither is the point. The point is that generic `=` **accepts**
`(= "a" "b")`, and `:wat::core::i64::=` would refuse it — the operands are locked to i64. That is
precisely the value `:wat::core::i64::>` already carries over generic `>`, and it is what FQDN-always
buys everywhere else: the form states the type it is about.

**Honest bound: this is SURFACE consistency, not behaviour.** No program that works today changes.
The rete equality rows minted in round 1c (`6d5af2c8`) already work routed through the generic; they
re-point to the per-type door for consistency, and behave identically either way.

## The strike

**Reverse Part B of 237.8d only. Part A and the equality impl stay.**

1. **`src/check.rs:3738-3746`** — delete the `UnknownCallee` rejection arm for the four heads.
2. **`src/check.rs:15818`** (i64 ordering array) — add `":wat::core::i64::="`, `":wat::core::i64::not="`.
   Same `(T,T) -> bool` scheme as its siblings; update the 237.8d comment above it to cite THIS stone.
3. **`src/check.rs:15878`** (f64 ordering array) — add `":wat::core::f64::="`, `":wat::core::f64::not="`.
4. **`src/runtime.rs:~5208`** — four dispatch arms beside the ordering ones:
   - i64 via `eval_compare` (`|o| o == Ordering::Equal` / `!=`)
   - f64 via `eval_f64_compare` (`|a,b| a == b` / `a != b`) — already NaN-correct, so IEEE
     `NaN ≠ NaN` falls out; do NOT special-case it.
5. **`src/rete/purity.rs`** — classify all four as pure ∧ det ∧ total, beside their ordering twins
   (`:399`/`:519` for i64, `:400`/`:524` for f64). **This is the act that was impossible before** —
   the builder's standing rule: *when a per-type op is minted in core it is classified in the same act.*
6. **The guard set INVERTS, it is not deleted** (the arc-153 precedent, R59: a gate whose subject
   reversed gets inverted so the record keeps the coverage):
   - `tests/types/probe_arc237_8d_equality_intrinsic_cut_{i64,f64}_{eq,not_eq}.wat.bad` ×4 — these
     assert the heads FAIL TO PARSE/CHECK. They must become **positive** fixtures asserting they
     work, or move to wherever the ordering family's positive fixtures live. Do not silently delete
     them; the coverage is real.
   - `tests/types/probe_arc237_8d_equality_intrinsic.rs:97/105/113/121` — four
     `assert!(r.is_err(), ":i64::= must be cut")` invert to assert success. **Rename the test fn** —
     `[[feedback_a_gates_name_is_where_the_lie_lives]]`; a test named `..._cut_...` that asserts the
     opposite is a lie in its own name.
   - Its **UNIFORM-EQUALITY REGRESSION** block stays exactly as-is — generic `=` over i64/f64/records
     must remain green. That is the part of 237.8d's probe that is still true.

## ⛔ STOPs

- **⛔ STOP-1 — do NOT touch `eval_eq` / `eval_not_eq` / `values_equal` / `infer_equality`.** The
  equality implementation is correct and 237.8d's relational-intrinsic argument for it stands.
- **⛔ STOP-2 — do NOT make equality a defclause.** That is the thing 237.8d proved impossible and it
  is still impossible. If any step seems to require it, STOP and report.
- **⛔ STOP-3 — do NOT delete the four `.wat.bad` fixtures or the four asserts.** Invert them. A
  deleted guard is coverage lost; an inverted guard is coverage kept.
- **⛔ STOP-4 — do NOT re-point the round-1c rete rows in this strike.** `src/rete/vocabulary.rs` is
  a separate act (it pairs with minting the missing f64 comparator rows). Keep the blast radius to
  `check.rs` / `runtime.rs` / `purity.rs` / `tests/types/`.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.**
- **⛔ Do not commit, stash, push, or touch git.**

## The floor

`cargo build --release` · `cargo nextest run --release` (SOLO) · `cargo clippy --release --all-targets`.
Floor at strike time: **`4348 / 4348 / 0 / 262`**, clippy clean, `check-where-shapes.sh` →
`9 pair(s), 98 rows`.

Note: 237.8d's own gate was `grep -rn "i64::=\|f64::=" src/ wat/` → **zero**. That gate is now
obsolete BY RULING; if any doc or CI step still runs it, it must be retired in this strike or it will
fail on purpose-built work.

## Related, NOT in this stone

**The rete surface has no f64 comparators at all** — `:wat::rete::i64::{> < >= <=}` exist,
`:wat::rete::f64::*` has none, so a rule cannot compare two floats. Flagged in
`DESIGN-STONE-where-admits-only-rete-ops.md` (*"f64::> — not in this document's f64 row at all"*) and
never picked up. That is four missing rows in `vocabulary.rs`, and it pairs with re-pointing the
round-1c equality rows. **Tracked as the immediate next stone; not this one.**
