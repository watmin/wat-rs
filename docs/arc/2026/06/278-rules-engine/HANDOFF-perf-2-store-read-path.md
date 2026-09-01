# HANDOFF — perf 2: the store's read path

`mem-store`'s reads cost O(table), not O(result). Measured: **46 ms per scan returning one row** at
1000 rows, and the fan-out circuit takes **287 seconds** for 8000 outcomes — CPU-bound, correct
throughout.

Start here, in order:

1. `DESIGN-STONE-the-store-read-path.md` — the contract decision and the oracle.
2. `BRIEF-perf-2-store-read-path.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat/query/mem.wat:38-44` — the header that has documented this defect since the file was
   written, and named this stone. Update it when you land.

Three things to hold:

**The durable Record does not change shape.** `:durable` is the soul — EDN, crosses the wire,
survives hibernation — and this service genuinely runs at process locus (`circuit.wat:255`). The
index goes in `:ephemeral`, derived at `:init` and maintained on put/delete. `wat/cache.wat:198`
already holds a data structure there; this is the existing shape, not a new liberty. A rebuild at
resume then comes free, by construction.

**`scan-index` is the hot path, not `scan`.** The queue's `receive` is a `scan-index` on
`by-visible-at`. A fix that only speeds the base table will pass the micro-benchmark and leave the
circuit slow.

**★ You have an oracle, and it was built for something else.** Five mem-vs-sqlite differentials pin
this service's behaviour against an independent implementation of the same surface. They were written
for excursus 001's correctness work; they are now what makes rewriting these internals safe. If one
goes red, behaviour moved — STOP and report which, and never adjust the test. Do not touch
`sqlite-store`: changing both sides of a differential destroys the thing protecting you.

This is a cost change. Ordering, pagination, cursors, `limit`, `delete`, put-is-a-replace and every
response variant stay exactly as they are.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-perf-2-store-read-path.md` when done. It will be graded by re-running, and by re-running
the circuit.
