# FINDING — the drain variance, and what it is not

Recorded 2026-09-02, after `ARC(278): the tick drains a batch`. **Open — mechanism unidentified.**

## The fact

Tick-batching made the circuit's drain **non-deterministic**. Same box, same session, one variable
(`git stash` on `sns-fanout.wat`):

```
PREVIOUS stone (one message per tick)   drain 19508 / 19554 / 19741     ±1.2%    qticks ~800
THIS     stone (K=10 per tick)          drain 12472 .. 24827            ~2x      qticks 696..5064
```

Median improves 19.55 s → 17.54 s (~10%). **The worst case, 24.8 s, is worse than the stable thing
it replaced.** Determinism was a property the sane-circuit stone explicitly bought
(`SCORE-the-sane-circuit.md`: *"2.5× faster and deterministic — the old one's variance was its
consumers guessing when to stop"*), and it was traded away without anyone deciding to.

## What the variance correlates with

`drain` tracks `qticks` almost linearly, and `queue-receive-calls` moves with it:

```
qticks   696    896   1058   1084   3066   3224   3472   3768   5064
drain  12472  12830  12716  12712  17538  17076  17765  18422  24827
```

So the variance lives in **how often a worker falls back to the tick/deadline path instead of being
woken by a `send`** — not in the work itself, whose volume is fixed at 8000 deliveries.

## What it is NOT — two hypotheses killed

**1. The park duration.** Six runs at `wait-ns 50 ms` against three at 250 ms:

```
 50 ms:  12895  15353  15390  16700  17076  20396     1.6x spread
250 ms:  12712  12716  17765                          1.4x spread
```

Both variable. Shortening the park does not stabilise it.

⚠ **This nearly shipped as "50 ms fixes it".** The first three 50 ms runs came back tight
(15353/15390/16700) and looked like a result. They were not — the next three were 12895/20396/17076.
Three samples of a bimodal distribution can look unimodal. The only reason the wrong conclusion was
not published is that the edit which was supposed to move to 1000 ms **failed its
`assert s.count(old)==1` guard** (`:wait-ns 50000000` matched the held-worker too), so those runs
were still at 50 ms and exposed the spread by accident.

**2. The obvious lost-wakeup window** — a `send` landing between a worker deciding to receive and
its waiter being registered. Ruled out by reading: `receive` checks for available messages *before*
parking (`sqs.wat`, the `(if (not (empty? envs)) …)` branch), so a late-registering waiter finds a
pending message immediately.

## The measurement that would find it

`receive-calls` and `ticks` are aggregates that cannot separate the cases. The queue needs to count
**why each receive ended**:

- served immediately (messages were already there)
- parked, then woken by a `send`
- parked, then expired empty on its deadline

The variance is entirely in the ratio between the second and third. Three counters on the queue's
`:ephemeral` state, reported through `stats`, localise it in a single run.

## Process note — this applies to every perf number in this arc

**Every drain figure graded before this one was a single sample**, including the ones used to size
three stones and to date the cursor ruling twice. The previous stone happened to be stable, so it
never bit.

Re-running a **timing** measurement to establish a distribution is not the flake re-run the floor
doctrine forbids. That rule protects **red assertions**, where a green re-run destroys the only
evidence. For a timing row the spread *is* the evidence, and one sample is not a measurement.
