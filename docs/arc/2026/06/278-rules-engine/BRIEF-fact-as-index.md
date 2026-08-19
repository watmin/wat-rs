# BRIEF — Element does not own a fact clone

## The work

`Element.fact` is a `u32` into the fire-lived worklist.
Populate writes the index. Readers take `fact_at`. Token
stays thin. Element becomes `Copy`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2ab is this intern.
2. `DESIGN-STONE-drop-memories-split.md` weigh (A **1.06**).
3. `DESIGN-STONE-bind-pool.md` (span shape to copy).
4. `DESIGN-STONE-elem-bindings-inline.md` weigh (2e
   reverted — fatter Element ate the drop).
5. `DESIGN-STONE-fact-as-index.md`.
6. `kernel.rs` `Element`, `make_element`, `alpha_activate_fact`,
   `drop-memories` four clears, `element_fact_bindings`
   readers, `join_extend` / leftover rematch / encode.

## Sketch

```rust
#[derive(Clone, Copy)]
struct Element { fact: u32, binds: BindSpan }

fn fact_at<'a>(facts: &'a Value, derived: &'a [Value], n_input: u32, idx: u32) -> &'a Value;

// seed:  make_element(i as u32, off, len)
// later: intern into derived_facts; store n_input + offset
```

## STOP

1. **STOP-1** — inline-enum / SmallVec / arena-and-forget.
2. **STOP-2** — clone facts into a pool that `drop-memories`
   clears (same Arc Drop).
3. **STOP-3** — put facts in `bind_pool`. Intern `match_pool`.
4. **STOP-4** — gate FIRE on a wall. Persist gather. 297.

## Done

- `Element.fact` is `u32`. Census `[200 200]` prints drop +
  FIRE. rete lib green. clippy `-D warnings` silent.

Leave dirty.
