# BRIEF — the queue reports time inside its store

Add a second counter to `Queue::StatsResponse`: nanoseconds spent inside `Store/*` calls. Sum it
across the subscriber queues and report `store-ms=` on the phases line.

An **instrument**. It splits the drain into *time in the store* and *everything else*, and resolves
a two-branch fork. **Do not act on what it shows.**

## Read in order

1. **`docs/excursus/2026/08/001-sns-sqs/the-queue-counts-its-store-round-trips/SCORE.md`** — the previous stone and its GRADING. It
   established flat `store-calls` per pair (1.24 → 1.27) against +59 % wall time per pair, and it
   records what over-reading a cumulative counter costs.
2. **`wat-scripts/scratch-pad/probe-what-a-clock-read-costs.wat`** — **the overhead is priced:
   ~322 ns per read, 6.4 ms for 20 000 reads, 0.20 % of a 3 200 ms drain.** This is why STOP-5 from
   the previous brief is lifted.
3. **`sqs.wat:911`** — the `stats` arm constructor. `store-ns` joins `store-calls` as the **last**
   field.
4. **The eight `Store/*` call sites** — the same ones `store-calls` already increments. Each gains a
   clock pair around the call.
5. **`circuit.wat:1848`** — `sum-store-calls`, added by the previous stone. **Copy its shape** for
   `sum-store-ns`.

## Blast radius — EIGHT files, FIFTEEN sites

`grep -rn 'queue::Queue::StatsResponse::Ok' --include=*.wat .` — **run before this brief was
written**, not after.

**Where the instrument lives (7 sites):**
`sqs.wat` `:911` `:1351` `:1361` · `circuit.wat` `:1103` `:1820` `:1834` `:1848`

**Where a `_` is appended and NOTHING else (8 sites, 6 files):**

| file | lines |
|---|---|
| `wat-scripts/topic/sns-fanout.wat` | `:219`, `:801` |
| `wat-scripts/scratch-pad/probe-three-waiters-wake.wat` | `:136`, `:145` |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | `:31` |
| `wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat` | `:50` |
| `wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` | `:70` |
| `wat-scripts/scratch-pad/probe-does-anyone-hold-a-service-name.wat` | `:62` |

⚠ **`Topic::StatsResponse` stays `[n ticks]`.** ⚠ **The probes are floor members** —
`every_wat_scripts_file_loads` type-checks every `.wat` under `wat-scripts/`. No `wat/`.

## The work

**1. `store-ns <- :wat::core::i64`** on the queue's `:ephemeral` state beside `store-calls`,
initialised `0`, carried through every `State` reconstruction that already carries `store-calls`.

**2. A clock pair at each of the eight sites.** `(epoch-nanos (now))` before and after the
`Store/*` call; add the difference. **Every site that increments `store-calls` also accumulates
`store-ns`** — a site that counts but does not time is the asymmetry to avoid.

**3. `store-ns` last on `StatsResponse::Ok`**, after `store-calls`. Update `:911`.

**4. `sum-store-ns` in `circuit.wat`**, shaped like `sum-store-calls` at `:1848`, reported as
`store-ms=` (divide by 1 000 000 at the format site, keep nanos on the wire).

## STOP triggers

- **STOP-1** — if a **sixteenth** `Queue::StatsResponse::Ok` site exists, **STOP and name it.** The
  list above came from the grep; a site outside it means the grep is wrong, not that the list should
  be quietly extended.
- **STOP-2** — if the no-args run differs in any **pre-existing** summary or phases field, **STOP.**
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not touch `wat/query/sqlite-store.wat` or anything under `wat/`.** Timing inside
  the store is the *next* question and only if the fork points there.
- **STOP-5** — **do not act on the number.** Report it. The fork decides the next stone; pre-empting
  it wastes the measurement.
- **STOP-6** — if `store-ms` exceeds `drain` at any depth, **STOP and report both.** That would mean
  the counter is accumulating outside the phase (fill, or the send path) and the number is not what
  its name says.

## Shape to copy

The previous stone in this directory — same instrument shape, same eight files, same discipline of
gating on the counter and never on what it reveals.
