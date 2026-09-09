# DESIGN — a give-back is counted

**Make the give-back observable.** `wat-scripts/fanout/circuit.wat` only. Correctness of the
*instrument*, not of the consumer. No perf work.

## WHY — the fix is currently as invisible as the bug was

The previous stone made retry-exhaustion release the envelope instead of killing the worker.
It works. **Nothing counts it:**

```
grep -c "gave-back\|give-back\|exhausted"  →  0
```

A run that gives back five envelopes and a run that gives back zero **print identical output**.
So that stone's acceptance rows — *6/6 terminate, `total=100`* — cannot distinguish

- *the give-back path ran and behaved correctly*, from
- *exhaustion never happened in these six runs*

and exhaustion was measured at only **~1 run in 6**.

★★ **This is the same shape as the defect that stone repaired.** The crash read `0/6` because
the injector was aimed elsewhere; the fix now reads green because nothing counts it. Twice in
two stones, the thing that would tell us it works did not exist.

## ⛔ THE ONE CONTRACT DECISION

**A give-back is fault telemetry and rides the channel that already exists.**

`:fanout::Worker` already has a `disrupts` feature whose response carries the worker's chaos
counters, and the whole path is live in this file:

| stage | site |
|---|---|
| durable counter on the worker | `circuit.wat:327-329` — `disrupt-hits` / `disrupt-draws` / `disrupt-points` |
| aggregation across workers | `circuit.wat:1010` — `:fanout::sum-disrupts` |
| printed in the summary | `circuit.wat:1381` — `disrupts={dh}` |

`gave-back` becomes a fourth field on `DisruptsResponse::Ok`, incremented in the give-back arm
at `circuit.wat:474-477`, and printed beside `disrupts=`.

★ **No new mechanism, and therefore no new probe.** `disrupt-hits` is not a described shape —
it is a counter doing exactly this, in this file, today. The exemplar is cited, not paraphrased.

## ⛔ WHAT THIS UNLOCKS — the row we could not state before

With a counter, the previous stone's claim becomes conditional and checkable:

> **When `gave-back > 0`, `total` must still be 100 and `dup` must still be 0.**

That is the row that proves a give-back loses nothing. Today it cannot be written, because we
cannot tell which runs took the path.

## FILES

`wat-scripts/fanout/circuit.wat` only. `held-worker`'s `disrupts` stub (`:607`) needs the
fourth field to stay type-correct; it stays a stub.

## OUT OF SCOPE = REJECTED

- **Counting anything else** — dropped checks, dropped marks, redials. Each is a real gap and
  each would confound this stone's one number.
- **Queue-side drop knobs**, the redelivery fixture, the rung-3 census, all perf work.
- **Changing the retry budget or the drop rate to make `gave-back` fire more often.** The rate
  is the configuration under test; tuning it to produce a nicer number is the opposite of the
  point. If the signal is too rare at 6 runs, **run more runs** — that is EXPECTATIONS row 1.
