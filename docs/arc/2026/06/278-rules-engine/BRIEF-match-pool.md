# BRIEF — `Token.matches` is a span

## The work

Stop minting a `matches` Vec on every join. Edges live in
`wm.match_pool`. Root-join pushes one. Extend appends.
Token becomes two spans. `Copy`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2n is this stone.
2. `DESIGN-STONE-token-bind-pool.md` weigh.
3. `DESIGN-STONE-match-pool.md`.
4. `kernel.rs` `Token` / `extend_token`.

## Sketch

```rust
struct Token { matches: BindSpan, binds: BindSpan } // Copy
// seed: match_pool.push((fact, alpha)); span of 1
// extend: concat left edges + (fact, alpha_id)
```

## STOP

1. **STOP-1** — SmallVec / `[T; 2]` (2e).
2. **STOP-2** — skip matches on `fire-rules`.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Token is two spans. Census printed. rete + clippy.

Leave dirty.
