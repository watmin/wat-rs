# BRIEF — `Token.bindings` is a `BindSpan`

## The work

Stop minting a PMap on every join. Token bindings live in
`wm.bind_pool`. Root-join shares the Element span. extend
appends. PMap stays the freeze / query / accum door.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2m is this stone.
2. `DESIGN-STONE-join-extend-no-leftover.md` weigh (probe leftover).
3. `DESIGN-STONE-bind-pool.md` (Element, the same law).
4. `DESIGN-STONE-token-bind-pool.md`.
5. `kernel.rs` `Token` / `extend_token` / `join_extend`.

## Sketch

```rust
struct Token { matches: Vec<(Value, i64)>, binds: BindSpan }
// seed: binds = el.binds
// extend: concat left + right-only into pool
// exec_compiled_rhs<B: Bindings>(...)
```

## STOP

1. **STOP-1** — inline / SmallVec `matches` in this diff.
2. **STOP-2** — bind-by-slot / fatter Token (2e).
3. **STOP-3** — gate FIRE on a wall.

## Done

- Token is a span. Census printed. rete + clippy.

Leave dirty.
