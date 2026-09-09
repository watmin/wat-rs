# DESIGN — every tier reports its backlog

**Measurement only.** The precondition for every decision behind it: removing the cap, restructuring
the topic, and choosing pool sizes are all unjudgeable without this.

## Why now

The perf drive walked three claims and refuted each one:

```
"the drain is superlinear"      → drain is 6.4% of the run
"fill is slow"                  → 49% of fill is one publisher waiting
"the transport fails"           → transport clean; the QUEUE accepts zero
```

Each refutation needed a new counter or a new knob. **Three numbers per tier, sampled once per phase,
would have answered all of it in one run** — and none of them exist.

## ⛔ Two things the disk says, before any design

**1. Nothing is ever discarded.** `retention|expire|discard|evict|reap` across `sqs.wat` yields exactly
one hit: `:1001`, *"expire past-deadline **waiters**"*. Waiters, not messages. Visibility expiry makes
a message **visible again**; no path drops one.

★ So a *"discarded from timeout"* counter would read **zero forever**. That is an **absence, recorded**
— not a metric to add. The two real timeout-driven events are **redeliveries** (visibility expired,
message re-visible) and **expired waiters** (a receiver polled and got nothing).

**2. Sampling would corrupt what it measures.** `Queue/stats` costs **2 store calls**. Five queues at
10 Hz over a 38 s run = 380 × 5 × 2 = **3800 store calls**, against a measured total of **9731**.
**39 % of store traffic would be the instrument.**

★★ So the design is **counters, not sampling.** The queue accumulates in `:ephemeral` — no store call,
no extra work — and the circuit reads stats a handful of times at phase boundaries. Rates are
`count ÷ phase duration`.

## What lands

Five counters on the queue's **`:ephemeral`** state, beside the `receive-calls` / `ticks` /
`store-calls` / `store-ns` / `handler-ns` that already live there:

| counter | answers |
|---|---|
| `sends-accepted` | **arrival** — what got in |
| `sends-refused` | **backpressure** — admissions that took zero |
| `acks` | **service** — what completed |
| `redeliveries` | visibility expired; a worker did not ack in time |
| `expired-waiters` | a receiver waited and got nothing |

★★★ **`:ephemeral`, not `:durable`** — so `:queue::queue::Record` is untouched and **no constructor
ripples.** That is the exact trap that made the last stone seven files instead of two.

With the existing `visible` / `unacked`, each tier then reports arrival, service, refusal, redelivery
and depth. The constraint tier is the one where **arrival ≈ service and depth grows.**

## The one contract decision

**`StatsResponse::Ok` carries a `:queue::Stats` record instead of positional fields.**

Twelve positional fields is unreadable at a call site, and this arc already established the shape:
`sqs.wat` gained **`TakeAcc`** and **`RetryAcc`** as named aggregates rather than nested tuples, under
the builder's rule that `defrecord` is for pure, EDN-expressible data. Stats is exactly that.

The 17 sites are being touched either way; doing it now makes every future counter free.

## OUT OF SCOPE — REJECTED

- **A dead-letter destination.** The builder's ruling: metric only.
- **A retention/discard mechanism.** It does not exist; inventing one to have something to count is
  backwards. The absence is the finding.
- **Removing the cap.** This stone makes that judgeable; it does not do it.
- **The topic restructure.** Behind this, and blocked on it.
- **Per-sample time series.** Rejected on measured cost — see above.

## Files

`wat-scripts/queue/sqs.wat` (counters, the `Stats` record, the reply), `wat-scripts/topic/sns-fanout.wat`
and `wat-scripts/fanout/circuit.wat` (consumers + the report line), and **five scratch-pad probes** that
match on `StatsResponse::Ok`. **8 files, 17 sites** — counted, not guessed.
