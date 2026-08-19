# BRIEF — split the in-fire probe gap

## The work

Apportion probe **12.30 − E 7.08**. Tight loops. If the
largest of wrapper / growth / push is ≥ 1 ms, one intern
(hoist or reserve). Token stays two BindSpans.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2s is this stone.
2. `DESIGN-STONE-probe-extend-split.md` weigh (gap ≈ 5.2).
3. `DESIGN-STONE-probe-gap-split.md`.
4. `kernel.rs` `join_extend` / catch-up probe.

## Sketch

```rust
fn probe_gap_cost_split() {
    // R rematch  S has_seed_cmp  P push
    // E extend reserved  J join_extend reserved  G extend × 40k unreserved
}
```

## STOP

1. **STOP-1** — largest < 1 ms: do not touch the engine.
2. **STOP-2** — `Token.extra` / SmallVec / intern `names`.
3. **STOP-3** — gate FIRE on a wall. Two intern in one stone.

## Done

- Table printed. J > 0. If implemented: Token still Copy.
  Probe printed.

Leave dirty.
