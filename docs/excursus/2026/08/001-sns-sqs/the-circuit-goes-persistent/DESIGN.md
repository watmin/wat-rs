# DESIGN — the circuit goes persistent

Drawn 2026-09-02. **Not struck.** Small, bounded, and its whole job is to convert a measured
isolated gain into a measured circuit gain.

## Why

`:wat::core::Vector`'s `conj` copies the whole accumulator (`src/collection/eval.rs:280`), so it is
**O(n) per call**. `PersistentVector`'s is rpds-backed with structural sharing, amortised O(1):

```
build n by conj    Vector           PersistentVector
1000               6790 us          1267 us
2000              25322 us (x3.7)   2612 us (x2.1)
4000              91297 us (x3.6)   5245 us (x2.0)
```

The topic's `-deliver` rebuilds its entire outbox with `conj`, once per delivery, to drop the head —
so it is O(n) rebuilds of an O(n²) build. Timed alone, no I/O (`probe-outbox-strategies.wat`):

```
A  Vector + rebuild    500=570   1000=4042   2000=30685 ms   x7.6   CUBIC
B  PVec   + rebuild    500=223   1000= 910   2000= 3742 ms   x4.1   quadratic
C  PVec   + cursor     500=  0   1000=   1   2000=    3 ms   linear
```

**30.7 s of a 73.4 s drain is one `foldl` doing no I/O.**

## What it delivers

A→B: the container swap alone, at both accumulator sites. **8.2× on the isolated loop**, ~27 s of
the 30.7 s.

## Scope: the container only, not the cursor

C is 1000× better than B and it is deliberately **not** this stone. The cursor changes what the
outbox *is* — it never shrinks until compacted, so `cap` must become `count - head` and a compaction
point has to be chosen. That is a semantic change with its own edge cases, and it buys the remaining
**3.7 s** against a drain that will still be ~43 s. **Take the 27 s first, measure, then decide
whether the 3.7 s is the next thing worth doing** — it almost certainly is not.

## The two sites

- **`wat-scripts/topic/sns-fanout.wat`** — `outbox <- (:wat::core::Vector :- [String])` becomes a
  `PersistentVector`. `-deliver`'s rebuild uses `:wat::vector::conj` / `:wat::vector::get`, and
  `get` returns an `Option<T>` that must be faced.
- **`wat-scripts/fanout/circuit.wat`** — `:fanout::worker`'s `outcomes` is the same mistake, smaller:
  ~667 appends per worker, ~222,000 element copies each, ~2.7 M across twelve. A pure container
  swap; nothing removes from it, so there is no rebuild to fix.

## The one contract decision

**`:wat::vector::get` returns `(Option :- [T])` and every call site must face it.** The rebuild's
index is always in range by construction, so `Option/expect` with a located message is correct —
but it is a real outcome and it is not to be `_`-swallowed.

## Why this is worth doing even though a corpus migration is coming

The builder has large migrations landing in `main` and this class will ride one of them. This stone
is not that migration — it is the **measurement** that tells the migration what it is worth, on the
one fixture that can show it. A corpus-wide change justified by an isolated microbenchmark is the
error this campaign has made four times today.

## Out of scope = REJECTED

- **The cursor** (strategy C). Above, with its number.
- **The other 21 `Vector` accumulators.** Most are bounded by structure, not by workload; the axis
  that matters is *unbounded* growth. That census belongs to the corpus migration.
- **A COW `conj`.** Measured and dead: `Arc::make_mut` in `vector_conj_inner` changes nothing,
  because the environment binding holds a second reference and the refcount is never 1.
  (`src/` reverted; diff is empty.)
- **`wat/` and `src/`.** Nothing here needs them.
