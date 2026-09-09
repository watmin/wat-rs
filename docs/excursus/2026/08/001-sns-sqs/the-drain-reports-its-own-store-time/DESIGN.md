# DESIGN — the drain reports its own store time

**`store-calls` and `store-ns` are sampled at the phase boundaries; the drain's delta is reported
per queue.** `wat-scripts/fanout/circuit.wat` only.

The fix for the previous stone, which measured the right thing over the wrong span.

## WHY — `store-ns` is whole-run, and the run is mostly fill

`store-ns` landed and works, but it is cumulative:

```
n=500    fill  4107 ms   drain  531 ms   →  fill is 89 % of the working time
n=2000   fill 17162 ms   drain 3302 ms   →  fill is 84 %
```

So its +16.2 % per-round-trip growth is dominated by **fill** store calls and says almost nothing
about the drain's **+61 %**. Two calls to the existing `sum-store-ns` — one at `t-drain0`, one at
`t-collect0` — turn it into the drain measurement the fork actually needs.

## ⛔ AND A FINDING THAT CHANGES HOW EVERY NUMBER IN THIS ARC READS

`sqs.wat:976` — the `stats` arm:

```wat
:store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 2)
:store-ns    (:wat::i64::+ (:queue::queue::State/store-ns s) depth-ns)
```

★★★ **Every `Queue/stats` costs 2 store round trips**, because `depth` derives visible/unacked from
the index. And `poll-until-drained` calls `Queue/stats` on every queue, every iteration.

From numbers already measured (`poll-calls = iterations × (m+1)`; m of those are `Queue/stats`):

| n | poll-calls | iterations | poller's store calls | of total | **share** |
|---|---|---|---|---|---|
| 500 | 95 | 19 | 152 | 2472 | **6.1 %** |
| 1000 | 245 | 49 | 392 | 4967 | **7.9 %** |
| 2000 | 640 | 128 | 1024 | 10197 | **10.0 %** |

★★★★ **The observation cost grows with depth, in the same direction as the slope.** It is not large
enough to explain 61 % on its own, but two things follow immediately:

- **`store-calls`/pair "flat" contains a growing observation term.** The *message* store work per
  pair is therefore drifting slightly **down**, not flat.
- **`store-ns` includes the poller's store time**, so the +16.2 % is partly self-inflicted.

⚠ This is the failure mode this arc has already recorded twice — *the measurement contains the
measurer*. It is now quantified rather than suspected.

## ⛔ THE ONE CONTRACT DECISION — a delta, per queue, with the observation term named

```
drain-store-ms=   (sum-store-ns at t-collect0  −  sum-store-ns at t-drain0) / 1e6 / m
drain-store-calls= (sum-store-calls at t-collect0 − at t-drain0) / m
```

**Divided by `m`.** `sum-store-*` sums across the m queues; setting that beside a single wall-clock
`drain` is the category error this arc has paid for three times. Per-queue is commensurate.

⚠ **The two boundary samples cost 8 store calls each** (m stats × 2) and land inside the delta. A
known, constant offset — reported, not silently subtracted.

★ And the poller's contribution to the delta is **derivable from `poll-calls` already on the line**:
`(poll-calls / (m+1)) × m × 2` store calls. Nothing new is needed to net it out.

## THE FORK IT RESOLVES

- **`drain-store-ms`/pair grows in step with `drain`/pair (+61 %)** → the drain's time is inside the
  store; the next question is the SQL, in the **stdlib**.
- **`drain-store-ms`/pair flat or far below** → **the store is exonerated for the drain**; the time
  is in the queue's own processing or in scheduling, and the search moves somewhere nobody has
  looked.

★ **No prediction.** The previous two stones each named a winner inside their own fork; this one
states both and picks neither. And unlike the last one, the instrument **can** discriminate — it
measures the phase the fork is about.

## WHAT THIS IS AND IS NOT

⚠ **It does not explain the slope.** It measures the drain instead of the run.

⚠ **It does not separate the store's own work from the IPC.** `store-ns` is the round trip as the
queue sees it. That separation lives in the stdlib and is only worth doing if this points there.

⚠ **`circuit.wat` only.** `sum-store-calls` and `sum-store-ns` already exist; this calls them twice
and does arithmetic. **No `sqs.wat` change, no `StatsResponse` change, so no 15-site ripple.**

## OUT OF SCOPE — REJECTED

- **Making `stats` cheaper.** Its 2 store calls are how `visible`/`unacked` are derived, and the
  arc already ruled against caching depth in service state
  (`docs/excursus/2026/08/001-sns-sqs/stop-fetching-rows-to-get-a-number/SCORE.md`). Naming the cost is this stone; changing it is not.
- **Timing inside the store service** (stdlib). Next, and only if the fork points there.
- **`circuit.wat`'s 36 nested-Tuple chains**, the `Lost`/`Closed` → `Exhausted 0` conflation,
  `collect`, and the `seen-skipped` drift. All open, none on the slope's path.
