# DESIGN — the message carries its trace

Drawn 2026-09-02. **Not struck.** Instrumentation that **stays**.

## Why

Drain is 12.5–24.8 s — a 2× spread with no identified cause
(`FINDING-the-drain-variance.md`). Two hypotheses are dead: the park duration, and the obvious
lost-wakeup window. What is known is that `drain` tracks `qticks` and `receive-calls` almost
linearly, so the variance is in **how a message spends its time**, not in how much work there is —
the work is fixed at 8000 deliveries.

Every number this arc has attacked was an aggregate. Aggregates are what hid the cubic term for
three stones and what let a bimodal distribution read as a 35% win. **The circuit cannot say which
component is slow, because nothing measures a single message's journey.**

## What it delivers

Each message carries its own timestamps, so the run reports **where time goes, per stage, as a
distribution**.

```
body = "<seq>|<t0>|<t1>|<t2>|<t3>"
```

| stamp | taken where | the interval it opens |
|---|---|---|
| `t0` | publisher, before `Topic/publish` | outbox residency |
| `t1` | topic `-deliver`, at dequeue | topic → adapter |
| `t2` | adapter `deliver`, on entry | adapter → queue |
| `t3` | queue `send`, on entry | **pending residency** |
| — | worker `-tick`, at receive | end-to-end |

★ **`t3` → receive is the interval to watch.** That is how long a message sat in the queue waiting
for a consumer, and it is exactly where "woken by a send" differs from "waited out a 250 ms park".
If the bimodality is where it is suspected, that interval is bimodal and the other four are flat.

## The one contract decision: in-band, not through the telemetry service

The trace rides **in the payload**, so it costs **no extra hops**. Routing it through the built
telemetry service would add ~8000 log calls to a system whose cost is dominated by hops — the
instrument would change the thing it measures. In-band is the only shape that does not.

`seq` stays the **first** field so body identity survives on the prefix.

## Reporting: the shape, not the average

Report a **histogram per stage** — log-scale buckets, `<1 ms · 1–10 · 10–50 · 50–250 · 250–1000 ·
>1000` — plus the max. `sort`/`sort-by` exist so percentiles are available too, but buckets are
what make bimodality visible at a glance and they need no sort.

**A mean is what has been hiding this all day.** The 250–1000 ms bucket is the one that answers the
question: a message that waited out a park lands there and nowhere else.

## The counters are DEFERRED, with a trigger — not cut

Three counters on the queue (`served immediately` / `woken by a send` / `expired empty`) would say
*which mechanism* produced a slow message, where the trace says only *that* it was slow and where.

They are deferred because they cost a **surface change**: `Queue::StatsResponse::Ok` gains three
fields, and that is **11 match sites across 4 files**, including `sqs.wat` — which five consecutive
stones have deliberately left untouched under STOP-1.

**The trigger:** if the trace shows `t3`→receive is bimodal, the counters are the next stone and
they are cheap to justify. If it shows that interval is *flat*, the variance is not in the queue's
wake path at all and counters there would have been work in the wrong service. **Measure, then
decide** — the sequencing that has worked every time this arc it was followed, and failed every time
it was not.

## Out of scope = REJECTED

- **The queue counters.** Above, with their trigger.
- **The telemetry service.** Above — it would perturb what it measures. This does not retire it; it
  is the wrong instrument for *this* question.
- **Acting on what the trace shows.** This stone measures. The next one attacks.
- **`wat/`, `src/`.** Neither changes.
