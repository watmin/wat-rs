# ⛔ NOTE (arc 109) — every `conj` onto a HashSet FULL-CLONES the set. Accumulation is O(n²), MEASURED.

**Parked here 2026-08-18 by the builder's ruling — *"we chase it later"*.** Written with a number so
the next self does not have to re-derive one, and so nobody re-argues it on taste.

## The measurement

```wat
(foldl (fn [acc x] (conj acc x)) (HashSet :i64) (range 0 n))
```

| n | wall | ratio vs previous |
|---|---|---|
| 2,000 | 12 ms | — |
| 4,000 | 53 ms | **×4.4** |
| 8,000 | 223 ms | **×4.2** |
| 16,000 | 875 ms | **×3.9** |

Each doubling of `n` roughly **quadruples** the time. That is O(n²), not a constant factor.
Non-vacuity: the returned counts were exactly 2,000 / 4,000 / 8,000 / 16,000, so every set really was
built. Capped run, `./target/release/wat` at `29dc5862`.

Extrapolated from the 16,000 point: **100,000 elements ≈ 34 s; 1,000,000 ≈ an hour.** Any wat program
that accumulates a set of non-trivial size is already paying this.

## The mechanism — one line

`src/collection/eval.rs:613`:

```rust
let mut out: HashSet<Value> = (**s).clone();
out.insert(item.clone());
Ok(Value::wat__std__HashSet(Arc::new(out)))
```

`conj` is persistent-by-value, so it must not mutate the caller's set — and the implementation buys
that with a **full clone of every element on every insert**. Building a set of n elements clones
0 + 1 + 2 + … + (n−1) = n(n−1)/2 values.

## What this is NOT

- **Not a Stream problem.** Arc 118 route B is complete and this is untouched by it. It bites every
  HashSet accumulation in the language, lazy or eager.
- **Not `distinct`'s OOM.** That was the *interaction* of this clone with the retained memo — stone
  118.B3 removed the memory half (`b1d876f6`), which is why `distinct` at n=16,000 completes today.
  **This half was never fixed**, and its own note said so at the time: *"B3 removes the half that
  turns transient allocation into permanent retention."* The O(n²) **time** is still here.
- **Not ruled.** The obvious fix is a persistent set with structural sharing (the `rpds` family
  already in the tree for `PersistentVector`/`PersistentMap`), but that is a representation change
  to a core container and it is the builder's call, not a rider's.

## The shape of the fix, when it is chased

`Value::wat__std__HashSet` is a `std::collections::HashSet` behind an `Arc`. Every other persistent
container in this substrate already uses `rpds` — `PersistentVector`, `PersistentMap`. A `HashTrieSet`
would make `conj` O(log n) with sharing instead of O(n) with copying, and the narrow waist
(`collection/seq_container.rs`) means the capability table would carry the change to both classifiers
at once.

⚠ **Before that lands, check what else reads the concrete type.** `Value::wat__std__HashSet(Arc<HashSet<Value>>)`
is a public shape; a representation swap ripples wherever the inner type is named. That census is the
first step of the stone, not an afterthought.

## Kin

- `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/MEASURED-distinct-ooms-at-8000.md` — where the
  clone was first caught, as one half of a two-part interaction.
- `DESIGN-STONE-118.B3-delete-the-memos.md` — the stone that fixed the other half and named this one
  as out of scope.
- The 294 seam's still-open list.
