# BRIEF — split leftover probe without a third intern

## The work

Apportion `hj:catchup:probe` **12.30**. Tight loops on the
fanout extend shape. Do not change the engine.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2r is this census.
2. `DESIGN-STONE-extend-no-left-copy.md` weigh (2o reverted).
3. `DESIGN-STONE-prod-leftover-split.md` (do not nest 40k marks).
4. `DESIGN-STONE-probe-extend-split.md`.
5. `kernel.rs` `extend_token` / `key_of` / catch-up probe.

## Sketch

```rust
fn probe_extend_cost_split() {
    // B bind-append  M match+fact  E extend_token
    // K key_of @ 2k  H HashMap::get @ 2k
}
```

## STOP

1. **STOP-1** — nest `phase_start` in the join token loop.
2. **STOP-2** — `Token.extra` / SmallVec / intern `names`.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Table printed. E > 0. Largest drawable lump named.

Leave dirty.
