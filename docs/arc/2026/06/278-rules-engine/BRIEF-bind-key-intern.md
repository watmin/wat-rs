# BRIEF — bind_pool keys are a fire-scoped index

## The work

`bind_pool` stores `(u32, Value)`. Unique bind-variable keys
live once in `bind_keys`. Populate intern-writes. Token stays
two spans. Do not intern record `names`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2ad is this intern.
2. `DESIGN-STONE-drop-memories-split.md` weigh (B is key Arc).
3. `DESIGN-STONE-match-pool-fact-as-index.md` weigh (isolated
   leftover is B **0.78**).
4. `DESIGN-STONE-bind-key-intern.md`.
5. `kernel.rs` `bind_pool`, `span_from_pairs`, `extend_token`,
   `pool_slice` / `Bindings::get`.
6. `compiled_cond.rs` `materialize_into` `slot_keys[i].clone()`.

## Sketch

```rust
bind_keys: Vec<Value>
bind_pool: Vec<(u32, Value)>

fn intern_key(keys: &mut Vec<Value>, k: &Value) -> u32;
struct BindView<'a> { keys: &'a [Value], pairs: &'a [(u32, Value)] }
impl Bindings for BindView<'_> { … }
```

## STOP

1. **STOP-1** — intern record `names`. Put facts in `bind_pool`.
2. **STOP-2** — process-lifetime intern of `?var` strings.
3. **STOP-3** — fatten Token / retry 2e / 2o / arena-and-forget.
4. **STOP-4** — gate FIRE on a wall. Persist gather. 297.

## Done

- `bind_pool` is `(u32, Value)`. Census `[200 200]` prints drop
  + FIRE. rete lib green. clippy `-D warnings` silent.

Leave dirty.
