# DESIGN STONE — perf 2: the store's read path

**Commissioned 2026-09-01.** The `mem-store`'s reads cost O(table) rather than O(result). Its own
header has said so since it was written — *"the whole table, unindexed; `scan`/`scan-index` filter +
`sort-by` + `take` a plain materialized copy on every read (no separate sorted structure — correct,
not fast; a later stone may add per-index structures)."* This is that stone.

## Measured, twice

`wat-scripts/scratch-pad/probe-store-scan-cost.wat` — 200 scans, `limit 1`, so both the scan count
and the **result** size are constant. Only the table grows:

```
200 scans over  250 rows -> 1691ms
                500 rows -> 3489ms   (2.06x)
               1000 rows -> 9204ms   (2.64x)
```

**46 ms per scan returning one row**, at 1000 rows. The climb past 2× is the `sort-by`'s
`n log n` on top of the linear filter.

And in situ: `wat-scripts/fanout/circuit.wat`, 8000 outcomes, **287 seconds** wall — ~36ms per
outcome, CPU-bound, correct throughout (`distinct=8000; dup=0`).

The queue's hot path is the index one: `receive` is a `scan-index` on `by-visible-at`, so
**`scan-index` matters more than `scan`**, and both walk the whole table today (`mem.wat:190, 217`).

## ★ THE CONTRACT DECISION: the durable Record does NOT change shape

`:durable` is *"the soul: EDN, crosses the wire, survives hibernation"*. The table is
`rows <- (PersistentVector :- [StoredRow])` and that is the honest wire form — flat, ordered, no
structure to reconstruct on the far side.

**And it genuinely crosses a wire**: `circuit.wat:255` starts `mem-store` at
`:locus (:wat::spawn::process)`, despite the file header's claim of thread-only. Changing the durable
shape is a wire-format change and a hibernation-format change, for a performance fix. That trade is
not worth making.

> **The index lives in `:ephemeral`, derived at `:init` and maintained on `put`/`delete`.**

`:ephemeral` is *"what I carry"* — and `wat/cache.wat:198` already holds a data structure there
(`cache <- (Lru :- [K V])`), so this is the existing shape, not a new liberty. Durable stays the
soul; the index is body.

This also means a hibernate/resume rebuilds the index from the table, which is correct by
construction rather than by remembering to.

## The shape

- base reads: partition by `pk`, ordered by `sk` → lookup + range + take.
- index reads: per index name, partition by `ipk`, ordered by `isk` → the same.

Both become O(matching + limit) instead of O(table · log table).

## ★ The oracle already exists, and it was built for something else

Five differentials pin `mem` against `sqlite` — an independent implementation of the same surface:

```
tests/rete/probe_ex001_reput_differential.rs
tests/rete/probe_ex001_delete_differential.rs
tests/services/probe_ex001_queue.rs
tests/services/probe_ex001_journal_same_ns.rs
tests/services/probe_arc278_journal_backend_differential.rs
```

They were written for excursus 001's **correctness** work. They are now the safety net for a
performance rewrite: if `mem`'s internals change behaviour at all, `sqlite` disagrees and they go red.

**This is why the stone is safe to take.** A rewrite of a stdlib service's internals with no oracle
would be reckless; with one, the risk is bounded and named.

## What must not change

Ordering, pagination, cursor semantics, the `limit` contract, `delete`, the `put`-is-a-replace rule
(excursus 001 stone 2c), and every response variant. **Behaviour identical; only cost moves.**

## Out of scope = REJECTED

- **`sqlite-store`.** It has a real database underneath; if it is slow that is a different
  investigation with its own measurement.
- **The durable wire format** — see the contract decision.
- **A partial-selection tweak instead of a structure** (take the smallest `limit` in O(n) rather than
  sorting). It removes the `log` factor and leaves the whole-table walk, which is the actual cost.
  Half a fix that would make the real one harder to justify later.
