# FINDING — where the time actually goes, and it is not the failure modes

Builder asked whether there are known perf/chaos angles left, or whether we need to measure the time
sinks. Measured. **The failure modes are largely squeezed; the time is somewhere nobody has looked.**

## 1. Failure-mode cost is now indistinguishable from the happy path

Same size, same box, clean vs 10 % ack-reply loss (`200 2 2 32 false 0` vs `… 0 1000 42`), after
`3135df9b5` set the inbox visibility to 200 ms:

```
            pub-work        inbox              worker   fanout          subq            drain
clean       max=53ms        max=330ms          max=0ms  max=55ms        max=74ms        397
chaos       max=35ms        max=344ms          max=0ms  max=28ms        max=44ms        375
```

Every one of the six histograms matches within noise. **At this size a 10 % reply-loss regime costs
nothing measurable.** What failure-mode latency remains is fully accounted for — 200 ms visibility plus
up to 250 ms re-claim — and it only fires on a rare stall.

★ The one *measured* failure-mode inefficiency left is `ok = min over subs` wasting 28–48 fan-out
bodies per 11 runs (`the-inbox-visibility-stops-being-an-outlier/SCORE.md`). That is **work, not
latency**, and changing it changes the safety semantics — a ruling, not a stone.

## 2. ⭑⭑⭑ `setup` is 40–66 % of every run, and it is linear in PROCESS COUNT

```
setup vs n   (m=2, j=2):   6595   6650   6629      ← FLAT across 4× in n
```

It is not a per-message cost at all. It scales with the topology:

| | measured | fit |
|---|---|---|
| m=1 | 4844 | `4844 + 1800·(m−1)` → 4844 |
| m=2 | 6652 | → 6644 |
| m=4 | 10257 | → 10244 |
| m=8 | 17463 | → 17444 |
| j=1 | 5285 | `5285 + 1350·(j−1)` → 5285 |
| j=2 | 6698 | → 6635 |
| j=4 | 9328 | → 9335 |

**Linear to within 20–60 ms across an 8× range in `m`.** So:

```
setup ≈ 3 s fixed + ~1.8 s per subscriber + ~1.35 s per consumer tier
```

At the standard sweep config `m=4 j=3` that is **~12.5 s before a single message moves** — 40 % of a
32 s run at n=4000, and 66 % of a 10 s run at n=200.

`collect` scales identically (m: 768 / 1604 / 2677 / 4501; j: 538 / 1020 / 2441), so it is the same
per-process cost paid on the way down.

★ Working the arithmetic back through what each increment spawns (a sub store + sub queue per `m`,
`m` consumers per `j`) puts it at roughly **450–670 ms per spawned process.** ⚠ **That number is
inferred from the fit, not measured directly** — I have not timed a bare spawn, and saying "process
spawn costs 500 ms" would be exactly the substitution this campaign keeps catching.

## What this changes about the perf list

- **Failure-mode perf: essentially done.** Nothing left that a knob can reach.
- ⭑ **The dominant cost in the system is fixed setup, and it is invisible in every per-message
  metric we have been optimising.** `drain` — the thing five stones chased — is **397 ms** against
  `setup`'s **6595 ms** at n=200. The residual drain slope (4.44 vs 4.0), `scan-index`'s +45 %, and
  `count-index`'s 1.27× are all rounding errors beside this.
- ⭑ **And it compounds.** Every sweep cell, every chaos cell, every floor circuit test pays it. Cutting
  it makes all future measurement cheaper, which is the rarest kind of perf win.

## The next probe, named and not yet run

**Isolate bare process-spawn cost**, outside the circuit: spawn N trivial processes, time it, and see
whether ~450–670 ms per process reproduces. That separates *spawn* from *schema creation*, *grant*, and
*connect* — the three other candidates inside `setup` — and it must be a measurement, not a reading.

⚠ The precedent for why: `wat-scripts/scratch-pad/probe-derive-decomposition.wat`'s own header records
*"I twice attributed the remaining ~5 s to a component by READING the code — first to seeding
(disproved: ~13 ms), then to `vec->pvec` (disproved: … moved the wall clock by a median +0.06 s).
Both were reasoned, neither measured."* Same trap, same file, one arc earlier.
