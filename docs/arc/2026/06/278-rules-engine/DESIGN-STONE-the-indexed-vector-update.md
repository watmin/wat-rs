# DESIGN STONE — perf 3: the indexed vector update

**Commissioned 2026-09-01.** perf-2 made the store's reads flat; the circuit moved only 287s → 257s
because the bottleneck moved to writes. This is the write path — and it turns out not to be a
store defect at all.

## Measured

`wat-scripts/scratch-pad/probe-store-write-cost.wat`, into a fresh store each time:

```
puts    250 / 500 / 1000  ->   400 / 1333 / 4887 ms   (3.33x then 3.67x per doubling)
deletes 250 / 500         ->   751 / 2801 ms          (3.73x)
```

Quadratic predicts 4× per doubling. **4.9 ms per put** at 1000 rows and climbing — against
**0.6 ms per scan, flat**, after perf-2. Writes are ~8× dearer per operation *and* they grow with the
table.

The mechanism, read not inferred (`mem.wat:516-531`): `put` is a **nested foldl** — for each incoming
row it folds the entire table to rebuild `kept`, then conj's. `delete` (`:563`) walks it the same way.

## ★ THE CONTRACT DECISION: the defect is in CORE, not in the store

`Record/rows` is a `(PersistentVector :- [StoredRow])`, and `PersistentVector` is backed by `rpds`
(`Cargo.toml:123`) — a bit-partitioned trie whose indexed `set` is **O(log n)**, not a copy.

But **wat exposes no indexed update for it.** The entire surface reachable from wat is
`stream->pvec` / `stream->pvec-spec`. There is no `set`, no `assoc-n`, no `drop-last`.

So `mem-store`'s nested foldl is not a careless implementation — **it is the only shape available.**
A keyed write has no choice but to degrade to a fold when the language cannot address a slot.

> The stem is `mem.wat`'s foldl. The root is a missing core primitive. Fix the root.

Expose indexed update on `PersistentVector`, then let the store use it. That helps every future
consumer, not just this one — and it is an *exposure* of something `rpds` already provides, not an
implementation.

**Two routes were considered and rejected:**

- **Lazily materialize `:durable` and keep the truth in `:ephemeral`.** The projection hooks
  (`stop-project`, `hibernate-project`) are macro-**generated** (`service.wat:682-728`), not
  user-declarable, so this needs a `service.wat` change — a much wider blast radius than a core
  primitive, for a narrower benefit.
- **Change the durable shape** to something keyed. That reverses perf-2's contract decision and
  changes the wire and hibernation formats. Rejected there, rejected here.

## What makes the store side cheap

★ **After perf-2, the durable table's ORDER is semantically irrelevant.** Verified: `Record/rows` is
touched at exactly three sites — `:499` (rebuild the index at `:init`), `:532` (put), `:563`
(delete). **No read path touches it**; reads go through the `:ephemeral` index, which carries
ordering.

So the table is an unordered bag keyed by `(pk, sk)` — put-is-a-replace guarantees no duplicates —
and a `delete` may **swap-remove**: move the last row into the hole and drop the last. Both halves
are O(log n) given the primitives.

## The shape

- **core**: indexed `set` and `drop-last` on `PersistentVector`, with the bounds behaviour of the
  house (an out-of-range index is a located error, never a silent no-op).
- **store**: the `:ephemeral` index also carries key → position; `put`-replace is a `set`,
  `put`-insert a `conj`, `delete` a swap-remove with the moved row's position fixed up.

## The oracle is the same one

The five mem-vs-sqlite differentials pin behaviour against an independent implementation. They caught
nothing in perf-2 because perf-2 changed nothing observable; the same is required here.

★ **And swap-remove is exactly the change most likely to be caught by them** — if any consumer
depended on the durable table's order, a differential goes red. That is the check, not a hope.

## Out of scope = REJECTED

- `sqlite-store` (the oracle, and it has a real database doing indexed writes — 43s on the same
  circuit).
- The durable Record's shape.
- A general `rpds` surface for wat (`take`, `drop`, `split`, …). Expose what this needs, with its
  own evidence; a wide surface with one consumer is an abstraction before its second user.
