# FINDING — the buffer was the bottleneck

Measured 2026-09-02, immediately after `the message carries its trace` made it visible.
**One variable: the topic's outbox `cap`, 4096 → 16.** Nothing else changed.

## The result

| | buffered (`cap 4096`) | backpressured (`cap 16`) |
|---|---|---|
| wall | 26.8 – 31.9 s | **26.8 / 26.9 / 26.7 s** (±0.4%) |
| publish | 0.8 s | 11.7 – 12.1 s |
| drain | 12.8 – 20.5 s | **92 / 156 / 179 ms** |
| **e2e max** | **12.8 – 20.5 s** | **184 – 202 ms** |
| outbox residency max | 11.9 – 13.6 s | **169 – 181 ms** |
| `t3→t4` `>1 s` count | 161 – 652 | **0** |
| `qticks` | 872 – 4227 | **99 – 185** |
| invariant | `distinct=8000; dup=0` | `distinct=8000; dup=0` |

**End-to-end latency fell ~100×. The variance vanished. Wall time did not get worse.**

Throughput, end to end (publish + drain):

```
buffered, best run    8000 / 13.6 s  =  588 deliveries/s
buffered, worst run   8000 / 21.3 s  =  376 /s
backpressured         8000 / 12.1 s  =  661 /s      -- every run
```

## What it means

`publish` moving 0.8 s → 12 s is not a cost imposed; it is **the honest rate becoming visible**. It
was never 0.8 s of work — it was 0.8 s of dumping into a 2000-deep reservoir and 12–20 s of paying
for it afterwards. With a shallow cap the producer is paced by the consumers, which is what
backpressure *is*.

The one genuine cost: the producer's thread is blocked for 12 s instead of 0.8 s. Free in this
fixture; an argument for a *deeper* cap, never an unbounded one, in a producer with other work.

## What it retires

The entire variance investigation was chasing a symptom of buffer depth:

- the park-duration hypothesis (killed by measurement)
- the lost-wakeup-window hypothesis (killed by reading)
- the three queue disposition counters (**now unnecessary** — with no reservoir there is no `>1 s`
  tail to explain, no `qticks` swing, no bimodality)

`FINDING-the-drain-variance.md` is answered: the mechanism was queueing delay in a deep FIFO. The
counters stay undrawn.

## What it does NOT retire

Every stone since the async-publish stone still holds on its own evidence — the cubic `conj` term,
the level-triggered wakeup, `max(4)` fan-out, tick-batching. They made a buffered design drain
faster. What this finding says is that **the buffer depth, not the drain speed, was the dominant
term all along**, and none of those measurements were pointed at it.

★ **And the call was made this morning, by the builder, before any of it:** *"the chain is
backpressure ... I don't see where anything has any contention at all ... the entire system is
organically backpressured, yes?"* I agreed, then spent six stones optimising the reservoir. The
measurement that would have settled it — a `cap` sweep — costs one constant and three runs.

## Where the bound now is

661 deliveries/s over 4 lanes ≈ **6 ms per delivery per lane**, against a measured chain of ~5 ms.
The system is within ~20% of its per-message chain latency, doing one message at a time per lane.

**Wire-batching is therefore the only remaining structural lever** — not to save hops for their own
sake, but because more than one message in flight per lane is the only way past a per-message chain
bound. Decomposed estimate: the chain amortises by K, the per-message CPU (a `uuid::v4`, two
`edn::write`s, one row built) does not, so **3–5×, not 10×**.

Open, and smaller: the outbox rebuild (~0.47 ms/delivery) and the `Full`-retry spin — a `nap-ms 1`
poll in `accept!` that a shallow cap makes hot, and which the parked-reply pattern already solves
elsewhere in this tree.
