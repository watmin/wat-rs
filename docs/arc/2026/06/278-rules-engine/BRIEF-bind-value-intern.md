# BRIEF — bind_pool fillers are a fire-scoped index

## The work

`bind_pool` stores `(u32, u32)`. Unique fillers live once in
`bind_vals`, interned by `FxHashMap`. Populate intern-writes.
Token stays two spans. Do not intern record `names`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2ae is this intern.
2. `DESIGN-STONE-bind-key-intern.md` weigh (B **0.32** is
   value Drop).
3. `DESIGN-STONE-bind-value-intern.md`.
4. `kernel.rs` `intern_key`, `span_from_pairs`, `extend_token`,
   `BindView`.
5. `compiled_cond.rs` `materialize_into`.

## Sketch

```rust
bind_vals: Vec<Value>
bind_val_ids: FxHashMap<Value, u32>
bind_pool: Vec<(u32, u32)>

fn intern_val(vals, ids, v: Value) -> u32;
struct BindView<'a> { keys, vals, pairs: &'a [(u32, u32)] }
```

## STOP

1. **STOP-1** — intern record `names`. Put facts in `bind_pool`.
2. **STOP-2** — linear-scan intern of values. Process-lifetime.
3. **STOP-3** — fatten Token / retry 2e / 2o / arena-and-forget.
4. **STOP-4** — gate FIRE on a wall. Persist gather. 297.
   Chase fact insertion.

## Done

- `bind_pool` is `(u32, u32)`. Census `[200 200]` prints drop
  + FIRE. rete lib green. clippy `-D warnings` silent.

Leave dirty.
