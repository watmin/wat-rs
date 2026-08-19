# BRIEF — match_pool does not own a fact clone

## The work

`match_pool` stores `(u32, i64)` — the same fact index
Element already holds. Root-join / `extend_token` write the
index. Token stays two `BindSpan`s.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2ac is this intern.
2. `DESIGN-STONE-fact-as-index.md` weigh (M **1.08** leftover).
3. `DESIGN-STONE-match-pool.md` (span shape).
4. `DESIGN-STONE-match-pool-fact-as-index.md`.
5. `kernel.rs` `push_match`, `extend_token`, `match_slice`,
   `native_token_to_value` / `value_token_to_native`, root-join
   `fact_at(…).clone()`, drop census M.

## Sketch

```rust
match_pool: Vec<(u32, i64)>

fn push_match(pool: &mut Vec<(u32, i64)>, fact: u32, alpha_id: i64) -> BindSpan;

// root-join:  push_match(&mut wm.match_pool, el.fact, *node_id)
// extend:     copy left (Copy) + (el.fact, alpha_id)
```

## STOP

1. **STOP-1** — fatten Token / inline `[T; 2]` / skip matches.
2. **STOP-2** — clone facts into a pool that `drop-memories`
   clears.
3. **STOP-3** — put facts in `bind_pool`. Retry 2e / 2o.
4. **STOP-4** — gate FIRE on a wall. Persist gather. 297.

## Done

- `match_pool` is `(u32, i64)`. Census `[200 200]` prints drop
  + FIRE. rete lib green. clippy `-D warnings` silent.

Leave dirty.
