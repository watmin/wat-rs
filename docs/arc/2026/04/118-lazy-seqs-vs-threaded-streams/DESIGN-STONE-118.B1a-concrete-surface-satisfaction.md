# DESIGN STONE — 118.B1a · a CONCRETE instantiation of a parametric surface

**Builder, 2026-08-18: *"strike B1a yourself, then hand B2 to the shadowdancer."***

Unblocks B2. Measured cause: `MEASURED-118.B2-blocked-the-var-gate.md`.

## The defect, in one line

`(:wat::core::Vector :i64 …)` does not satisfy a parameter typed `Seqable<wat::core::i64>`, though
it satisfies `Seqable<?N>` — because 118.3-B's bind-and-unify path is gated on the expected args
still containing an unbound `Var`.

```
:d::a<T> [s <- Seqable<T>]                + Vector<i64>   →  GREEN   (T free → Var → gate fires)
:d::b<T> [probe <- :T  s <- Seqable<T>]   + Vector<i64>   →  RED     (T pinned → concrete → gate misses)
```

**Every verb B2 collapses takes `f`/`sep`/`idx` before the collection**, so every one pins `T` first.

## The site, and why the gate is removable

`src/check.rs`, the `(Parametric actual, Parametric expected)` block:

```
arm 1  :14865   if ah != eh && <exact-string full-args edge>     → return nature_floor_ok(…)
arm 2  :14894   else if eargs.iter().any(|t| matches!(t, Var))   → bind-and-unify (118.3-B)
```

★ **Arm 2 is an `else if` on arm 1.** The gate's own comment justifies itself by saying the arm's
tenants — `Dialable` / `TypedCapability` / `Handle` — are baked with concrete args and *"the
exact-string arm above already decides them, byte-identical, and this branch is unreached for their
calls."*

**That is exactly why the gate can go.** If arm 1 decides a tenant, arm 2 never runs for it —
`else if`. So the gate is not what protects them; **arm 1 is.** The gate's only effect is to exclude
cases arm 1 did *not* decide, and the `Seqable`-over-a-builtin case is precisely such a case: a
builtin's name can never string-match a surface's, so arm 1 must fail for it, always.

## What actually provides soundness — arm 2's inner guards, unchanged

```rust
types.get(&bare) is a Surface                                  // the expected head IS a surface
surf.type_params.len() == eargs.len()                          // arity matches the declaration
aargs.len() == eargs.len()                                     // arity matches the actual
transport_edge_keys(&a).any(satisfies_bare_surface(k, &bare))  // the actual's family really extends it
aargs.zip(eargs).all(unify(...).is_ok())                       // INVARIANT args — this is the swap-gate
```

The swap-gate the arm worries about (`echo'::Handle` must match `Dialable<Echo::Op,Echo::Reply>`,
never `Dialable<Kv::Op,Kv::Reply>`) is enforced by **`unify` on the args**, not by the Var gate:
two different concrete instantiations do not unify. Removing the gate does not weaken it.

## The change

Replace `else if eargs.iter().any(|t| matches!(t, TypeExpr::Var(_)))` with a plain `else`, and
rewrite the justifying comment so the next reader learns the guards are the argument, not the gate.

## The four questions

- **Obvious? YES.** "A concrete type satisfies a parametric surface when its family extend-types
  that surface and the args unify" is one sentence, and it is what the arm already does for the
  polymorphic case.
- **Simple? YES.** One condition deleted. No new helper, no new concept, no new registry.
- **Honest? YES.** It removes an accidental restriction rather than adding a special case, and the
  stone's load-bearing row is the *regression* proof, not the feature.
- **Good UX? YES.** Today `[s <- Seqable<T>]` works or fails depending on the position of an
  unrelated parameter. That is not a rule anyone can hold in their head.

## ⚠ The gate — and row 2 is the stone, NOT row 1

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY** — restore the B2 probe; it must be RED with the **same 5 errors**, captured |
| 1 | after the change, the B2 probe RUNS and prints `2,4 \| 2,4 \| 2,4 \| 2,4` / `0,1,2,3,4` / `0,2,4` |
| 2 | ★★ **THE STONE — `Dialable` / `TypedCapability` / `Handle` observationally UNCHANGED.** These are live service/capability tenants. Widening reaches their arm; proving they did not move is the whole risk |
| 3 | the discriminator `:d::b` goes GREEN |
| 4 | `:d::a` still GREEN — the Var path is not disturbed |
| 5 | ★ **NEGATIVE CONTROL — a type that does NOT extend-type the surface is still REJECTED.** A widening proved only by what it now accepts is a widening nobody has tested. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]` |
| 6 | floor GREEN via `scripts/floor.sh` — the Summary line |
| 7 | `cargo clippy --release --all-targets` → 0 |
| 8 | `#[ignore]` count **13** |

**Row 5 is not optional.** This stone makes a satisfaction check accept more. The only way that is
honest is to demonstrate it still refuses what it must refuse.

## Out of scope

- **Collapsing any verb** — B2, and B2 is the rider's.
- **Deleting `extract_lazyable_elem`** — B2.
- **Surfaces with >1 type param over builtins** — untested before and after; this stone does not
  claim it. Named, not deferred: it has no consumer, and B2's verbs are all single-param.
