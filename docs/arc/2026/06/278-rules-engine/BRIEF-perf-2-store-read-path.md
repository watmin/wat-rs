# BRIEF — perf 2: the store's read path

Make `mem-store`'s reads cost O(result), not O(table). **Measured**: 46 ms per scan returning one row
at 1000 rows; the fan-out circuit takes 287 s for 8000 outcomes.

Read `DESIGN-STONE-the-store-read-path.md` beside this first — the contract decision (the durable
Record does **not** change shape) and the oracle that makes this safe.

## Read in order, and why you are being sent there

1. **`wat/query/mem.wat:38-44`** — the header that has documented this since it was written:
   *"the whole table, unindexed … correct, not fast; a later stone may add per-index structures."*
   You are that stone. **Update the header when you land it** — leaving it stale is the drift this
   repo has FM-catalogued.
2. **`wat/query/mem.wat:190`** (`scan`) and **`:217`** (`scan-index`) — the two `sort-by`-over-the-
   whole-table sites. **`scan-index` is the hotter one**: the queue's `receive` is a `scan-index` on
   `by-visible-at`.
3. **`wat/cache.wat:198`** — `:ephemeral [cache <- (Lru :- [K V])]`. The precedent for holding a
   derived data structure in `:ephemeral`. This is the shape you are copying.
4. **`wat-scripts/scratch-pad/probe-store-scan-cost.wat`** — the measurement. **Re-run before and
   after**; its numbers are the stone's evidence and belong in the SCORE.
5. **`wat/query/sqlite-store.wat`** — the independent implementation of the same surface. It is the
   oracle, not a thing to change.

## The work

**1. A derived index in `:ephemeral`** — base rows partitioned by `pk` and ordered by `sk`; index
rows partitioned per index-name by `ipk` and ordered by `isk`.

**2. Built at `:init`** from the durable table, so hibernate/resume rebuilds it by construction.

**3. Maintained on `put` and `delete`** — including `put`-is-a-replace (excursus 001 stone 2c).

**4. Reads become lookup + range + take**, with no whole-table filter and no `sort-by`.

**5. Update the file header** to describe what it now does.

## Blast radius

`wat/query/mem.wat` only. **The durable Record keeps its shape** — no wire change, no hibernation
change. No `sqlite-store` change. No surface change. No runtime change.

## STOP triggers

**STOP-1 — the durable Record does not change shape.** It is the soul and it crosses a wire
(`circuit.wat:255` starts this service at process locus). If the index seems to want to live in
`:durable`, STOP: it belongs in `:ephemeral`, derived.

**STOP-2 — the differentials are the gate, not a formality.** If any of the five mem-vs-sqlite tests
goes red, behaviour moved. STOP and report which and how — do not adjust the test.

**STOP-3 — no behaviour change at all.** Ordering, pagination, cursor semantics, `limit`, `delete`,
put-is-a-replace, every response variant. This is a cost change. If you find a case where the old
code was *wrong*, STOP and report it separately rather than fixing it inside a perf stone.

**STOP-4 — do not touch `sqlite-store`.** It is the oracle. Changing both sides of a differential
destroys the thing that makes this safe.

## The gates to write

- **★ the cost:** re-run `probe-store-scan-cost.wat`. Cost per scan must become roughly **flat**
  across 250 / 500 / 1000 rows — the result size is constant, so the time should be too. Baseline
  1691 / 3489 / 9204 ms.
- **the circuit:** re-run `wat-scripts/fanout/circuit.wat`. Same output string
  (`total=8000; distinct=8000; dup=0`), materially less wall time than 287 s. **Report the number.**
- **the differentials:** all five pass, unedited.
- **hibernate/resume rebuilds the index** — a resumed store answers reads identically.

## Prior comparable result

`SCORE-item-b-batched-writer.md` — a no-delta stone whose decisions were made against measurements
taken first. Same discipline here: the numbers above are real and re-runnable, not estimates.
