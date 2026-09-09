# DESIGN — the inbox visibility stops being an outlier

## Why

`18a86fe27` identified the single largest latency term in this system under fault, and it is one
hardcoded number. Every construction of the **same** worker, across the whole corpus:

```
wat-scripts/topic/sns-fanout.wat:888   (:demo::mk-tw  200000000 …)     200 ms
                            :969                     200000000        200 ms
                           :1140                     200000000        200 ms
                           :1182                     200000000        200 ms
scratch-pad/probe-refused-retry-self-consumes.wat ×2  200000000        200 ms
wat-scripts/fanout/circuit.wat:2323    (:demo::mk-tw 5000000000 …)     5 s   ← the ONLY outlier, 25×
```

⛔ **5 s is not a considered value. It is one site disagreeing with six others by 25×**, undocumented,
and it is the term that made a *correct* system look broken: the drain's slow mode is exactly one of
these timeouts.

Measured on this box (`18a86fe27`, `50 2 2 32 false 0 0 1000 42`):

```
alone   drain =   78   5190     71
8-way   drain =  149  159  165  5255  5283  5300  5305  5313
```

Two clusters, nothing between; six slow samples across both load regimes span **123 ms**. A **67×**
latency term (5190 vs 78 ms) that a constant, not queueing, produces.

## What it delivers

The inbox visibility becomes a **named, reachable parameter** — `inbox-vis-ms`, symmetric with the
existing `vis-ms` that already controls the *subscriber* queues — and the sweep says what its default
should be. Nothing else on the perf or reliability list is a 67× term.

## The one contract decision

**Parameterise and sweep; do not simply edit the literal.** A shorter visibility means *earlier*
redelivery, which means the worker's fan-out can be re-issued while the first attempt is still in
flight — duplicate work, absorbed by the consumer's `seen` dedupe but not free. That trade is real and
now measurable, so the number is chosen by data:

| what to watch | what it means |
|---|---|
| `dup` | ⛔ correctness. Must be **0** at every value. Non-zero ends the sweep |
| inbox-tier `redeliveries` | the **cost** — premature redelivery caused by too short a visibility |
| slow-mode incidence and `drain` | the **payoff** |

**The prior is 200 ms**, because six sites already use it — but the sweep decides, and if the data is
ambiguous the default is the builder's ruling, not mine.

## ⚠ The hazard, named because it is the reason to sweep rather than assume

The worker claims a batch with `:visibility-ns vis :limit 10 :wait (UpTo (Milliseconds 250))`
(`sns-fanout.wat:417`). **At 200 ms the visibility is SHORTER than the receive's own 250 ms wait**, so
an entry can expire while the worker that claimed it is still working. Whether that matters depends
on how long a receive→fan→ack cycle actually takes.

From a banked run's histograms: `fanout` is 1–50 ms (`max=14ms`) and the `inbox` handler 10–250 ms
(`max=63ms`) — comfortably inside 200 ms **on an idle box**. ⚠ Under 8-way contention those inflate,
and a cycle exceeding the visibility produces exactly the duplicate fan-out above. **That is why the
sweep must run under load as well as idle**, and why `redeliveries` is a row rather than a footnote.

★ Note the six precedent sites already run 200 ms under the same 250 ms wait, so the configuration is
not novel — it is merely unmeasured at this scale.

## Out of scope = rejected

- **Wiring the inbox into *fault* injection** (`circuit.wat:2178`'s three hardcoded zeros). Still the
  highest-value correctness item, and materially cheaper to build once a fault recovery costs 200 ms
  instead of 5 s. **This stone is its prerequisite, not its substitute.**
- **The two chaos cells crossing nextest's 15 s SLOW warning.** If the sweep lands short, they fall
  back under it on their own. `.config/nextest.toml` is not touched either way — the builder has
  said nextest is a later matter.
- **`run-with`'s parameter list.** It is at 15 and this makes 16, which is a smell. Collapsing it into
  a record would touch every caller and every fixture — a corpus migration, a different stone.
  **Named, not done.**
- The residual drain slope, `scan-index`, `count-index`, `TakeAcc`'s per-waiter-per-fold allocation,
  and the poller's own 300 round-trips.

## ⛔ What this stone must not do

**Do not change the constant to make a red green.** The two chaos reds were already fixed at
`18a86fe27` without touching any rate, cap, or timeout, and they must stay fixed *by that mechanism*.
This stone asks a different question: **was 5 s ever the right number?** If the sweep says the
outlier was correct and the six other sites are wrong, that is a finding and the stone lands as a
comment explaining why, not as an edit.

## Baseline to beat

```
50 2 2 32 false 0 0 1000 42     alone: drain 78 / 5190 / 71     8-way: 149…5313, 5 of 8 slow-mode
2000 4 3 8192 true 1000         distinct=8000 dup=0, inbox accepted=2000
```
