# BRIEF — RHS bind is a slot; Token stays thin

## The work

Print slice-get vs slot on the Pair form. If the 40k-scaled
delta is ≥ 1 ms, production reads slots from the first token.
Token stays two BindSpans.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2q is this stone.
2. `DESIGN-STONE-rhs-construct-split.md` weigh (A0 = 30% of D).
3. `DESIGN-STONE-elem-bindings-inline.md` weigh (fatter Element).
4. `DESIGN-STONE-rhs-bind-slot.md`.
5. `compiled_rhs.rs` `exec_compiled_rhs` Bind arm.
6. `kernel.rs` `operand_slot` / production loop.

## Sketch

```rust
fn rhs_bind_slot_split() { /* A0_pmap / A0_slice / A0_slot / D */ }

// only if (slice − slot) × 40k ≥ 1 ms:
// slots from first token; exec_compiled_rhs_at(pairs, slots)
```

## STOP

1. **STOP-1** — cut < 1 ms: do not touch the engine.
2. **STOP-2** — Token field / slot on `CompiledRhs` / intern `names`.
3. **STOP-3** — gate FIRE on a wall. Retry 2o.

## Done

- Table printed. If implemented: Token still Copy, two spans.
  Census printed. rete + clippy.

Leave dirty.
