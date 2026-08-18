# BRIEF — `Aggregate` carries its structural fingerprint

## The work

`setup:seen` still walks 40k Aggregates. Stamp `identity: u64` at
construction. Hash writes the u64. Eq unchanged.

## Read in order

1. `DESIGN-STONE-setup-fxhash.md` — leftover is the walk.
2. `DESIGN-STONE-aggregate-identity.md`.
3. `value.rs` `AggregateValue` / `impl Hash for Value` Aggregate arm.
   `runtime.rs` `Record/assoc` rebuild (~18855).

## Sketch

```rust
fn from_parts(...) -> Self {
    let mut h = FxHasher::default();
    nature.hash(&mut h); class.hash(&mut h); fields.hash(&mut h);
    Self { ..., identity: h.finish() }
}
// Hash: a.identity.hash(state)
```

## STOP

1. **STOP-1** — pointer identity, or inputs omitted from `seen`.
2. **STOP-2** — rete differential red.
3. **STOP-3** — Debug golden includes `identity` (it is a cache).

## Done

- Hash of Aggregate is one u64. `setup:seen` printed. rete + clippy.

Leave dirty.
