# BRIEF — bindings live in a fire-scoped pool; `Element` is a span

## The work

80k unique `Arc` slices are malloc+free once. Put the pairs in
`WorkingMemory.bind_pool`. `Element` holds `(off, len)`. Indices,
not pointers. Drop the pool once at `drop-memories`.

## Read in order

1. `DESIGN-STONE-elem-bindings-inline.md` weigh — do not retry inline.
2. `DESIGN-STONE-bind-pool.md`.
3. `kernel.rs` `Element` / `make_element` / `element_fact_bindings`.
   `compiled_cond.rs` `exec_compiled` / `materialize`.

## Sketch

```rust
struct BindSpan { off: u32, len: u16 }
struct Element { fact: Value, binds: BindSpan }
// exec_compiled(..., pool: &mut Vec<(Value,Value)>, fact) -> Option<BindSpan>
// drop: alpha.clear(); beta.clear(); bind_pool.clear();
```

## STOP

1. **STOP-1** — `mem::forget` / skip pool `Drop`.
2. **STOP-2** — rete differential red.
3. **STOP-3** — `unsafe`, raw pointers, or inline-enum again.

## Done

- Populate writes the pool. Element is a span. Census fold/snapshot
  green. rete + clippy.

Leave dirty.
