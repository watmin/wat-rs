# BRIEF — split insert − conj (defclause vs Session rebuild)

## The work

Add an `insert'` arm to `probe-insert-cost-split.wat`. Print
`P − C` (rebuild) and `I − P` (defclause). Intern the one
delta ≥ 0.5 µs. 2-ary `insert` stays `insert'`, never
`insert-all`.

## Read in order

1. `DESIGN-STONE-insert-prime-split.md`
2. `wat-scripts/scratch-pad/probe-insert-cost-split.wat`
3. `kernel.rs` `eval_insert_native`
4. `wat/rete.wat` `defclause :wat::rete::insert`

## Sketch

```
C  PersistentVector/conj
P  insert'
I  insert
```

If `I − P` ≥ 0.5 µs: native 2-ary `insert` → `eval_insert_native`.
If `P − C` ≥ 0.5 µs and `Arc::get_mut` is live: unique-owner
facts conj. If unique-owner is dead: STOP rebuild intern.

## STOP

1. **STOP-1** — Session-`Vec` / hardcoded facts index.
2. **STOP-2** — 2-ary through insert-all.
3. **STOP-3** — intern two lumps, or intern a dead `make_mut`.
