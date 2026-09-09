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

## Blast radius — EIGHT files (amended after STOP-1)

⚠ The first version of this brief said "two files, no `sns-fanout.wat`." **It was wrong**, and a
`grep` on the callers of the type being changed would have caught it. `Queue::StatsResponse::Ok` is
matched at **14 sites**; five of the extra files are `every_wat_scripts_file_loads` members, so a
green floor and a two-file blast could not both be true.

**Where the instrument lives (6 sites):**
`wat-scripts/queue/sqs.wat` `:856` `:1293` `:1303` · `wat-scripts/fanout/circuit.wat` `:1103`
`:1820` `:1834`.

**Where a `_` is appended and NOTHING else (8 sites, 6 files):**

| file | lines |
|---|---|
| `wat-scripts/topic/sns-fanout.wat` | `:219` (topic `stats` arm), `:801` (`:demo::q-depth`) |
| `wat-scripts/scratch-pad/probe-three-waiters-wake.wat` | `:136`, `:145` |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | `:31` |
| `wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat` | `:50` |
| `wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` | `:70` |
| `wat-scripts/scratch-pad/probe-does-anyone-hold-a-service-name.wat` | `:62` |

⚠ **`Topic::StatsResponse` stays `[n ticks]`.** `sns-fanout.wat` does not grow a `store-calls` of its
own; it only absorbs the extra binding.

⚠ **The probes are not optional.** `every_wat_scripts_file_loads` type-checks every `.wat` under
`wat-scripts/`.

No `wat/`. **No timing, no clock reads, no policy** — a counter and a field.

## STOP triggers

- **STOP-1** — if adding a field to `StatsResponse::Ok` breaks a caller outside **the eight files
  named above**, **STOP and name it.** The list was produced by
  `grep -rn 'queue::Queue::StatsResponse::Ok' --include=*.wat .` and verified independently; a
  fifteenth site means the grep is wrong, not that the list should be quietly extended.
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

`DESIGN/BRIEF/EXPECTATIONS/docs/excursus/2026/08/001-sns-sqs/the-poller-sweeps-once/SCORE.md` — the same kind of stone: a counter that
turns an estimate into a measurement, gated on the counter and explicitly **not** on the wall clock.
