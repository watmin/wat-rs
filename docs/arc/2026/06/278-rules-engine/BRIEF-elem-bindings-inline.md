# BRIEF — `Element.bindings` is inline at width 0–2

## The work

`round:drop-memories` is 10.49 ms of unique `Arc` slices this
fire never shares. Put width 0–2 in an enum. Spill 3+. Fire
materializes `ElemBindings`, not `Arc`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2d landed; this is the drop.
2. `DESIGN-STONE-elem-bindings-inline.md`.
3. `matcher.rs` `Bindings`. `compiled_cond.rs` `materialize`.
   `kernel.rs` `Element` / `make_element` / `slot_i64`.

## Sketch

```rust
enum ElemBindings {
    N0,
    N1((Value, Value)),
    N2([(Value, Value); 2]),
    Many(Vec<(Value, Value)>),
}
// as_slice / Bindings / FromIterator
// exec_compiled → Option<ElemBindings>
```

## STOP

1. **STOP-1** — skip `Drop` / leak Arcs.
2. **STOP-2** — rete differential red.
3. **STOP-3** — a crate, or `Token.bindings` rewritten.

## Done

- Fire populate is `ElemBindings`. Census still green on
  fold/snapshot. rete + clippy.

Leave dirty.
