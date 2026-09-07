# BRIEF — the queue counts its store round trips

Add one counter to `Queue::StatsResponse`: how many round trips this queue has made to its store.
Sum it across the subscriber queues in the circuit and report it on the phases line.
`wat-scripts/queue/sqs.wat` + `wat-scripts/fanout/circuit.wat`.

An **instrument**. It answers a fork, not a question.

## Read in order

1. **`sqs.wat:73-75`** — `StatsResponse::Ok [receive-calls ticks visible unacked]`. **The precedent:**
   two pure counters already live here. The new field joins them.
2. **`sqs.wat:120-130`** — the service's `:ephemeral` block where `receive-calls` and `ticks` are
   held. The new counter lives beside them.
3. **The eight `Store/*` call sites** — `:168` (scan-index), `:204` (put), `:256` (count-index),
   `:290` (count-index), `:384` (put), `:729` (delete), `:990` (put), `:1039` (delete). **Every one
   increments.**
4. **`sqs.wat:856-860`** — the `stats` arm that builds the response. The new field is read out here.
5. **`circuit.wat` `sum-calls`** — folds `Queue/stats` over `qclients` and sums `receive-calls`.
   **Copy this shape** for the new sum.
6. **`circuit.wat:2300-2320`** — the `phases` format, where `store-calls=` surfaces.

## The work

**1. The counter.** `store-calls <- :wat::core::i64` on the queue's `:ephemeral` state, initialised
`0`, incremented **once per `Store/*` round trip** at all eight sites.

⚠ Every arm that rebuilds `State` must carry it forward. `receive-calls` is threaded through roughly
a dozen `State` reconstructions; the new field goes everywhere that one does. Missing a site is a
compile error, not a silent zero.

**2. The response field.** `StatsResponse::Ok` gains `store-calls` as its **last** field, so existing
positional destructures shift only by appending. Update the arm at `:856`.

**3. The callers.** Every `StatsResponse::Ok` match in `sqs.wat` and `circuit.wat` gains the extra
binding. `circuit.wat` has several (`depth-of`, `sum-calls`, `sum-ticks`, the drain sweep) — most
will bind it `_`.

**4. The report.** A `sum-store-calls` in `circuit.wat` shaped like `sum-calls`, surfaced as
`store-calls=` on the phases line.

## Blast radius

`wat-scripts/queue/sqs.wat` and `wat-scripts/fanout/circuit.wat`. No `wat/`, no `sns-fanout.wat`.
**No timing, no clock reads, no policy** — a counter and a field.

## STOP triggers

- **STOP-1** — if adding a field to `StatsResponse::Ok` breaks a caller outside these two files,
  **STOP and name it.** The blast radius is stated as two files; if it is wider, the design is wrong.
- **STOP-2** — if the counter cannot be threaded without touching an arm the brief did not name,
  **STOP and list the arms.** Do not restructure `State` to make it fit.
- **STOP-3** — if the no-args run differs in any **pre-existing** summary or phases field, **STOP.**
  The new field is additive only.
- **STOP-4** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-5** — **do not add timing.** No `time::now` around store calls. That is the next stone and
  its overhead would land inside the phase being measured.
- **STOP-6** — **do not "fix" anything the number reveals.** Report it. The fork this instrument
  resolves decides what the next stone is; pre-empting it wastes the measurement.

## Shape to copy

`DESIGN/BRIEF/EXPECTATIONS/SCORE-the-poller-sweeps-once.md` — the same kind of stone: a counter that
turns an estimate into a measurement, gated on the counter and explicitly **not** on the wall clock.
