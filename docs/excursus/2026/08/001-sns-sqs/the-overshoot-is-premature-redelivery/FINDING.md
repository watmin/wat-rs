# FINDING — the overshoot is premature inbox redelivery, and my own stone caused it

Builder: *"correctness matters over speed… if we are incorrect we stop and correct ourselves. Being
faster but broken is worse."* So defect (a) — the `2010`-of-`2000` overshoot — was chased before
`rt-store`. **It is not a delivery defect. It is waste, and I introduced it.**

## The mechanism, measured — monotone in one knob

`2000 4 3 8192 true 1000`, **4-way concurrent** (the load that reproduces it), `inbox-vis-ms` swept:

| `inbox-vis-ms` | `fill-excess` across 4 runs | `dup` | `distinct` |
|---|---|---|---|
| **50** | **200 · 200 · 280 · 280** | 0 | 8000 |
| **200** ← shipped | **0 · 40 · 0 · 0** | 0 | 8000 |
| **1000** | **0 · 0 · 0 · 0** | 0 | 8000 |
| **5000** ← removed | **0 · 0 · 0 · 0** | 0 | 8000 |

★ **Delivery is correct at every value.** `dup = 0` and `distinct = 8000` throughout: at-least-once holds
and the consumer's `seen` absorbs the duplicates. **Nothing is lost and nothing is double-delivered to a
consumer.**

**The mechanism is premature inbox redelivery.** The inbox visibility expires while the topic-worker is
still fanning out and acking, so the entry becomes visible again, another worker takes it, and the batch
is fanned **twice** into the sub queues. Shorter visibility → more of it. That is the safety property
working — and paying for a timeout shorter than the work it protects.

## ⛔ AND IT IS A COST MY OWN STONE INTRODUCED

`3135df9b5` took the inbox visibility from **5 s to 200 ms**, for a 13× fault-latency win. The table above
shows 5 s had **zero** waste and 200 ms does not.

⛔ **I measured that trade and dismissed it.** The DESIGN priced *"latency against duplicate work"*, the
executor reported duplicate fan-out *"flat across a 50× range of the knob"*, and I accepted it —
**but that measurement varied `drop-ack-bp`, a SUB-QUEUE ack loss, not the inbox visibility.** Different
fault, different quantity, same-looking conclusion. **Sixth time today an instrument answered a narrower
question than the one asked.**

★★ And the instrument that would have caught it **did not exist yet**: `fill-excess` was created by
`0e309135f`, three stones later. The counter I *did* have — inbox `redeliveries` — is the one the executor
had already shown counts **one of four delivery paths**. **The constant was chosen before anything could
see its cost.**

## ⭑⭑ The trade, with both sides measured for the first time

| `inbox-vis-ms` | fault latency (slow-mode drain, `3135df9b5`) | wasted fan-out (`fill-excess`, today) |
|---|---|---|
| 5000 | 5165–5316 ms | 0 |
| **1000** | **1147–1281 ms** | **0** |
| 500 | 587–739 ms | not measured |
| **200** ← shipped | **357–405 ms** | **0–40** |
| 100 | 341–358 ms | not measured |
| 50 | ~350 ms (saturated) | 200–280 |

★★★ **1000 ms is Pareto-dominant over the 5 s we removed** — the same zero waste with **4.3× better**
fault latency. So the stone's *direction* was right and its *distance* was not.

★ **200 vs 1000 is a genuine trade**, not a defect: 3× better fault latency for occasional duplicate
fan-out. That is a ruling, and the numbers for it now exist on both axes.

## What is owed

⚠ **The lower bound is the worker's receive → fan-out → ack cycle**, and it has never been measured as a
distribution. A visibility shorter than that tail *guarantees* premature redelivery under load. Banked
histograms put `fanout` at 1–50 ms and the `inbox` handler at max 63 ms **on an idle box**; the 200 ms row
above proves the tail exceeds 200 ms under 4-way load. **That distribution is the number that should set
the constant** — which is exactly the networking-first ruling: declare a constant against what it bounds.

★ And the correctness verdict, stated plainly so it is not mistaken later: **there is no incorrectness to
stop for.** Delivery is exact at every swept value. What there is, is **a self-inflicted inefficiency
introduced by a stone that measured the wrong quantity** — which is worth correcting, and is not the same
thing.

---

# ⭑⭑ MEASURED FURTHER — two kinds of redelivery, and only one is waste

Builder asked for the worker's claim→ack cycle distribution. **It did not need a new instrument** — but
the counter I first reached for was the wrong one, and the next measurement said so.

## ⛔ First claim, made and refuted inside two minutes

From the sweep already on disk:

```
vis      inbox redeliveries    fill-excess        expired-waiters
  50ms   50  70  50  60        200 200 280 280    flat
 200ms   10   0   0   0          0  40   0   0    flat
1000ms    0   0   0   0          0   0   0   0    flat
5000ms    0   0   0   0          0   0   0   0    flat
```

`50×4=200`, `70×4=280` — so I wrote **`fill-excess = m × inbox-redeliveries`**, and concluded *"a
redelivery IS the measurement; the cycle tail is between 200 ms and 1000 ms."*

**Bracketing it refuted both:**

```
300ms   redeliveries 0  0 10  0     fill-excess 0 0 0 0
500ms   redeliveries 0 20  0  0     fill-excess 0 0 0 0
700ms   redeliveries 0 10  0  0     fill-excess 0 0 0 0
```

**20 redeliveries and zero waste at 500 ms.** The law is wrong and the counter is not the instrument.

## ⭑⭑⭑ TWO KINDS OF REDELIVERY, AND THE DIFFERENCE IS THE WHOLE POINT

| kind | what happened | cost |
|---|---|---|
| **PREMATURE** | the batch was fully fanned and its ack was in flight; the visibility expired anyway, so the re-fan **duplicates** | **waste** |
| **REPAIR** | the fan partially failed, `ok = min over subs` fell, the ack was **withheld on purpose**; the redelivery **completes the work** | **none** |

`redeliveries` counts **both**. `fill-excess` counts **only the wasteful kind.** ★ So the field created by
`0e309135f` is the only instrument that separates *the safety property working* from *the safety property
paying for a timeout that is too short* — and `expired-waiters` being flat across the whole sweep confirms
it is message-visibility expiry, not waiter timeout.

★★ **Eighth instance today of reaching for a counter whose name fits and whose semantics do not.** This
one I refuted myself, two minutes later, because I ran the bracketing sweep instead of stopping at the
tidy law.

## ⭑ The full trade, and the answer to the cycle question

| `inbox-vis-ms` | fault latency | **waste** | redeliveries (both kinds) |
|---|---|---|---|
| 50 | ~350 ms (saturated) | **200–280** | large |
| **200** ← shipped | 357–405 ms | **0–40** | 10 0 0 0 |
| **300** | ~450–500 ms (interpolated) | **0 in 4 runs** | 0 0 10 0 |
| 500 | 587–739 ms | 0 in 4 runs | 0 20 0 0 |
| 700 | not measured | 0 in 4 runs | 0 10 0 0 |
| 1000 | 1147–1281 ms | 0 | 0 0 0 0 |
| 5000 ← removed | 5165–5316 ms | 0 | 0 0 0 0 |

**The claim→ack tail that matters sits between 200 and 300 ms under 4-way load.** That is the answer to
the cycle question, and `fill-excess` crossing zero is what measures it — no histogram required.

★★★ **300 ms dominates 500, 700, 1000 and 5000** — equal (zero) waste, strictly better fault latency. And
against the shipped 200 ms it is a *small* trade: ~1.2× the latency for zero measured waste, where the
alternative on the table this morning (1000 ms) cost 3×.

## ⚠ What is NOT established

⚠ **`0 of 4` is not proof of zero.** The 200 ms cell wasted in **1 of 4** runs, so 300 ms could waste at
1-in-8 or 1-in-20 and this sample would not see it. **Any ruling on 300 ms should say that out loud**, and
a stone that lands it should widen the sample rather than inherit my four runs.
⚠ The 300 ms fault latency is **interpolated**, not measured — the sweep at `3135df9b5` covered
50/200/500/1000/5000.
⚠ And **delivery correctness is untouched throughout**: `dup = 0`, `distinct = 8000` at every value in
every run. There is still nothing here to stop for.
