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

---

# ⭑⭑ `collect` MEASURED — it is neither cold boot nor record interpretation

Builder's ruling: *"bringing new wat procs online is a slow boot — that's setup — then, nearly all of
wat is still interpreted. Neither of these two we are going to address here."* Then the question:
**is `collect` either of those?** Measured, not reasoned.

## ROBUST 1 — payload is not the driver

12 workers fixed (`m=4 j=3`), records varied 100×:

| records | `collect` | `collect-busy-ms` |
|---|---|---|
| 8000 | 5210 | 235 |
| 1600 | 4628 | 222 |
| 400 | 3744 | 143 |
| 80 | **3526** | 172 |

**Records fell 100×; `collect` fell 1.48×.** So at 12 workers, **~3500 ms of `collect` is
payload-independent** and only ~1700 ms of the 5210 at the standard size is record-related.

## ROBUST 2 — at negligible payload it is still expensive, and topology-dependent

At `n=20` (80 records), `collect` still costs **524 ms at 1 worker up to 4315 ms at 12.**

## ROBUST 3 — the sub-queues are barely involved either way

`collect-busy-ms` is **143–235 ms** across every cell above — ~4 % of the phase, whatever the payload
or topology.

## ⛔ NOT ROBUST — the functional form, and I nearly shipped an over-fit

A `524 + 520·log₂(w)` model fits the **m-sweep** almost exactly (524/1037/1549/2090 measured against
524/1044/1564/2084) — and **fails the j-sweep by 1.8× at 12 workers** (measured 4315, model 2388).

★ **So worker count alone does not determine `collect`.** Varying `m` adds sub-queues and stores as
well as workers; varying `j` adds only consumers. A single-variable law is not available from this
data, and I was one paragraph from asserting a logarithm. **Two sweeps, one beautiful fit, and the
other sweep is what killed it.**

## So: the answer to the builder's question

| part of `collect` | share at the standard config | class |
|---|---|---|
| record-dependent (the interpreted `conj` fold over `Outcome`s, plus serialization) | ~1700 ms, **⅓** | **interpretation — excluded** |
| payload-independent per-process teardown | ~3500 ms, **⅔** | **neither. Not boot, not record interpretation** |

**`collect` is not a slow boot** — it is teardown, not bringing procs online. **Two thirds of it is
neither of the excluded classes**, so on the builder's own reasoning that two thirds is in scope.

## ⭑⭑⭑ And the finding inside it: a stop round-trip costs ~100× a stats round-trip

`:fanout::worker/stop` is stdlib-generated. Read at `wat/service.wat:2928-2968`, it is **one
`send Admin::Stop` plus one `recv`** — **no timeout, no poll, no reap.**

```
Queue/stats   1 send + 1 recv, process peer   ≈ 2.8 ms   (calibrated via `arm`, 202763705)
worker/stop   1 send + 1 recv, process peer   ≈ 250–520 ms at negligible payload
```

**~100× for the same shape of exchange, with no wait in the path** — and it is not payload, because
ROBUST 1 drove payload down 100× and this cost stayed. That gap is the in-scope item, it lives in
`wat/service.wat`, and **nothing here explains it yet.**

⚠ What I will not do is name the mechanism. Today's record on that is five for five against me. The
next step is a probe that times the two halves of the stop exchange separately — the `send` to the
service's acknowledgement, and the service's own shutdown-to-`Status::Stopped` — because those are
different systems and only one of them can be at fault.

---

# ⛔ PROBE — A STOP IS FREE. MY MECHANISM WAS WRONG.

`wat-scripts/scratch-pad/probe-what-does-a-stop-cost.wat` — a minimal `defservice` whose whole state
is **one i64**, so payload is provably negligible. Spawned and stopped five times on each locus, with
the thread/process pair as a free control (identical but for the locus token):

```
thread   start=0ms  stop=0ms      ×5
process  start=408  stop=0ms
process  start=397  stop=0ms
process  start=406  stop=0ms
process  start=401  stop=0ms
process  start=398  stop=0ms
```

★★★ **`<svc>/stop` costs 0 ms — on a thread and on a process alike. The entire process-lifecycle cost
is in SPAWN (~400 ms), and none of it is in teardown.**

## ⛔ Which corrects the commit above

`1196720c3` concluded *"`collect` is two-thirds process teardown"* and attributed the payload-independent
per-worker cost to `:fanout::worker/stop`. **The measurements in that commit stand** — payload-independent
(records ↓100×, collect ↓1.48×), topology-dependent, sub-queues ~4 % — **but the mechanism does not.**
Stop is free, so whatever costs ~376 ms per worker inside `collect` **is not the stop.**

★ **I never verified that attribution.** `collect` spans `t-collect0 → t-stop0` and contains
`sum-*`, `topic-ticks`, `topic-inbox-fails`, `sum-disrupts` (per worker), `seen-stats`, `collect-stop`,
an `empty-flags` fold doing a `Queue/receive` per queue, and `summarize`. I picked `collect-stop` out of
that list because it was the one that mentioned workers, and asserted it. **Sixth time today that a
mechanism I named from reading died on measurement** — and this time the probe I wrote to confirm it is
what killed it.

★★ The one thing the probe does establish, which is worth as much as the refutation: **teardown is not
the mirror of boot.** Spawn is ~400 ms and stop is ~0 ms, so the builder's exclusion of `setup` as cold
boot does **not** carry over to `collect` by symmetry. `collect`'s cost has to be explained on its own
terms, and it is still unexplained.

## What is now known, and what the next measurement must be

**Known:** at `m=4` fixed (queues fixed), workers 4 → 12 moves `collect` 1308 → 4315 ms at negligible
payload — **+376 ms per worker.** And a stop is free. So the per-worker cost is one of the other
per-worker things inside the phase, or an interaction none of the sweeps isolates.

**Next:** ⛔ **instrument `collect`'s own steps** — a timestamp between each of its ~10 bindings — and
read which one carries the 376 ms. The infrastructure for exactly this landed at `202763705`
(per-phase boundary samples); this is the same move one level down.

⚠ **No candidate is named here on purpose.** `sum-disrupts` is per-worker and would be ~34 ms at the
calibrated 2.8 ms round-trip, so it does not fit; `empty-flags` is per-queue, not per-worker; and
`summarize` folds records, which ROBUST 1 excluded. Every one of those is a *reading*, and readings are
0 for 6 today. The instrument decides.
