# BRIEF — the drain reports its own store time

Sample `sum-store-calls` and `sum-store-ns` at the drain's phase boundaries and report the delta
**per queue**. `wat-scripts/fanout/circuit.wat` only.

An **instrument**. It resolves a fork. **Do not act on what it shows.**

## Read in order

1. **`docs/excursus/2026/08/001-sns-sqs/store-ns/SCORE.md`** and its GRADING — why the whole-run counter cannot answer the fork:
   fill is 84–89 % of the working time, so a run-wide figure is a fill measurement.
2. **`sqs.wat:976`** — the `stats` arm. **Every `Queue/stats` costs 2 store calls** and adds
   `depth-ns` to `store-ns`. This is why the numbers need the observation term named.
3. **`circuit.wat:1848`** (`sum-store-calls`) and **`:1862`** (`sum-store-ns`) — the two functions
   this stone calls twice each. **No new function is needed.**
4. **`circuit.wat`, the `t-drain0` / `t-collect0` bindings** — where the samples go, and the
   `phases` format where the results surface.

## The work

**1. Four samples.** `sum-store-calls` and `sum-store-ns` at `t-drain0`, and again at `t-collect0`.

**2. Two reported fields**, both **divided by `m`**:

```
drain-store-ms=    (ns-after − ns-before) / 1000000 / m
drain-store-calls= (calls-after − calls-before) / m
```

⚠ **The `/ m` is load-bearing.** `sum-store-*` sums across the m queues; setting that beside a
single wall-clock `drain` is a category error this arc has already made three times.

**3. Nothing else changes.** No `sqs.wat` edit, no `StatsResponse` field, no new counter — therefore
**no ripple**, and the 15-site list is not in play.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**.

## STOP triggers

- **STOP-1** — if sampling at `t-drain0` requires moving the timestamp, **STOP and say so.** The
  sample and the timestamp must bracket the same span; report which order you used.
- **STOP-2** — if the no-args run differs in any **pre-existing** summary or phases field, **STOP.**
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not touch `sqs.wat` or anything under `wat/`.**
- **STOP-5** — **do not act on the number, and do not net out the poller's share.** Report the raw
  delta; the poller's contribution is derivable from `poll-calls` and belongs in the grading, not in
  the instrument.
- **STOP-6** — **leave the tree parsing.**

## What is deliberately NOT gated

⚠ `store-calls`/pair and `store-ms` vary run to run — measured at n=1000, same code, zero retries:
**4959 / 4982 / 4996 / 5002**. Report them; a band would police noise. Four rows in this arc have
already made that mistake.

⚠ **The two boundary samples cost 8 store calls each** and land inside the delta. That is expected;
report it, do not correct for it.
