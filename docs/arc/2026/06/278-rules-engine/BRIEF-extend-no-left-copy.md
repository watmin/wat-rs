# BRIEF — extend does not copy left binds

## The work

Stop cloning the left binding pairs on every join. Keep
`tok.binds`. Write right-only keys into `extra`. Readers
search both. Token stays `Copy`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2o is this stone.
2. `DESIGN-STONE-match-pool.md` weigh (probe wash).
3. `DESIGN-STONE-extend-no-left-copy.md`.
4. `kernel.rs` `extend_token`.

## Sketch

```rust
struct Token { matches, binds, extra: BindSpan } // Copy
// extend: binds = tok.binds; extra = tok.extra + right-only
// token_binds(tok, pool) implements Bindings
```

## STOP

1. **STOP-1** — SmallVec / skip matches / `key_of` rewrite.
2. **STOP-2** — copy left pairs "to keep one span".
3. **STOP-3** — gate FIRE on a wall.

## Done

- Left pairs uncopied. Census printed. rete + clippy.

Leave dirty.
