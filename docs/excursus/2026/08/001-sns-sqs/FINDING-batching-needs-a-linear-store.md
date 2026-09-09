# FINDING — batching needs a linear store, and the reservoir just moves

Measured 2026-09-02, grading `the wire carries a batch`. The strike reported STOP-5 (throughput did
not improve) and was **right to stop**. The conclusion it implies — that batching does not help — is
wrong, and only a complete 2×2 shows why.

## The 2×2, all at `cap 16`, N=2000 M=4 J=3

| | `mem-store` | `sqlite-store` |
|---|---|---|
| **unbatched** | publish 12.0 s, drain 0.1 s → **661/s**, e2e max **200 ms** | publish 10.0 s, drain 0.14 s → **789/s**, e2e max **185 ms** |
| **batched (K=10)** | publish 27–29 s → **282/s**, e2e max **36–42 s** | publish 2.46 s, drain 2.64 s → **1568/s**, e2e max **2.6 s** |

Attribution:

```
batching on mem       661 -> 282   = 0.43x     actively harmful
batching on sqlite    789 -> 1568  = 1.99x     it works
store, unbatched      661 -> 789   = 1.19x     the "~1%" measured this morning
store, batched        282 -> 1568  = 5.6x
```

**The wire batching is correct.** Row 1 is green (`calls=2;msgs=20;shape=batch`), and it pays 2.0× —
just under the decomposed 3–5×, short by exactly the per-message CPU (`uuid::v4`, two `edn::write`s,
one `StoredRow`) that the estimate said would not amortise.

## Why mem collapses: the interaction, not either factor

Batching makes the queues **deep** — the SCORE saw it (`t3→t4 >1 s` on ~7000 of 8000). `mem-store`'s
writes are **O(table size)** (measured this morning: 1000/2000/4000 rows → 6.5/20.8/90.0 s). Deep
queue → big table → slow writes → deeper queue. Positive feedback.

★ **This retires my own ruling, and the lesson is sharper than the usual one.** On 2026-09-02 I
measured the store at **~1% of the circuit** and closed the lane. That measurement was *correct* —
and correct **only for a workload whose queues stayed shallow**. The store's cost is not a property
of the store; it is a property of queue depth, which is a property of the workload shape. **A perf
finding measured under one regime does not transfer when the regime changes**, and I had written
"closed as a perf lane" into the record.

## Why latency regresses even on sqlite: backpressure stops at the topic

Batched + sqlite is 2× throughput but **13× worse latency** (185 ms → 2.6 s). The reason is
structural and it is the same shape this arc already found once:

**The topic's outbox is capped. The queue is not.** `Queue/send` always accepts. So bounding the
topic did not create end-to-end backpressure — it moved the reservoir one stage downstream, where
nothing bounds it. The SCORE names this exactly: *"the reservoir moved into the queues."*

The builder's model — *"one function call, lockstep all the way down, organic backpressure"* —
requires **every** stage bounded, not the first one. We bounded one stage and called it backpressure.

## What follows

1. **Bound the queue.** `Queue/send` refuses (or parks) at a depth cap, so backpressure propagates
   worker → queue → adapter → topic → producer. That is the lockstep model, and it should restore
   flat latency *while keeping* batching's 2×.
2. **The store must be linear** for batching to pay at all. Either adopt `sqlite-store` in the
   circuit, or fix `mem-store`'s quadratic write path — it is the differential **oracle**, so an
   O(n²) oracle also makes every differential slower as the corpus grows.
3. **Re-run the 2×2 after (1).** With the queue bounded the tables stay shallow, and the store's
   share may fall back toward the 1.19× it shows unbatched — in which case (2) is a correctness/
   oracle concern rather than a perf one.

## Method note

Three cells of this table were in hand and the fourth was assumed. The first attempt at the fourth
was **confounded** — HEAD carries `cap 4096`, so it measured *unbatched + sqlite + buffered* (784/s,
e2e max 9.2 s), which is a different experiment. It had to be re-run with `cap 16` stashed in to
match. **A 2×2 with one variable uncontrolled is not a 2×2**, and the numbers it produces are worse
than none.
