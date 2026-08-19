# BRIEF — `AggregateValue.class` is `Arc<str>`

## The work

40k derived Pairs each `String::clone` the class. Hold `Arc<str>`
on the value and on `CompiledRhs`. Clone is a refcount.

## Read in order

1. `DESIGN-STONE-prod-no-token-clone.md` weigh.
2. `DESIGN-STONE-class-arc.md`.
3. `value.rs` `AggregateValue`. `compiled_rhs.rs` `Record { class }`.

## Sketch

```rust
pub class: Arc<str>
pub fn record(class: impl Into<Arc<str>>, names, fields) -> Self
CompiledRhs::Record { class: Arc<str>, ... }
exec: AggregateValue::record(class.clone(), names.clone(), Arc::new(fields))
```

## STOP

1. **STOP-1** — rete differential red / Debug golden red.
2. **STOP-2** — a global intern `HashMap`.
3. **STOP-3** — `names` rewritten here.

## Done

- Class clone is Arc. Census printed. rete + clippy.

Leave dirty.
