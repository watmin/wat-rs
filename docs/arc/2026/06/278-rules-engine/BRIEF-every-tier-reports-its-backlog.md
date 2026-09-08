# BRIEF — every tier reports its backlog

## The work, in one paragraph

Every question the perf drive asked needed a counter that did not exist, and each answer cost a stone.
Give the queue five free counters — `sends-accepted`, `sends-refused`, `acks`, `redeliveries`,
`expired-waiters` — carried in a named `Stats` record, and have the circuit report them **per tier**:
the topic's inbox and each subscriber queue. No sampling, no behaviour change, no discard path.

## Read in order

1. **`wat-scripts/queue/sqs.wat:130-152`** — the `:durable` / `:ephemeral` split. **The counters go in
   `:ephemeral`**, beside `receive-calls` / `ticks` / `store-calls` / `store-ns` / `handler-ns`, so the
   `Record` is untouched and no constructor ripples.
2. **`wat-scripts/queue/sqs.wat:73-79`** — `StatsResponse::Ok`, currently 7 positional fields. It gains
   a `:queue::Stats` record instead. **`TakeAcc` and `RetryAcc` in this same file are the exemplar** for
   a named aggregate replacing a wide positional shape.
3. **`wat-scripts/queue/sqs.wat:392-425`** — the admission block. `take == 0` is `sends-refused`;
   `take > 0` is `sends-accepted`. Both arms already exist.
4. **`wat-scripts/queue/sqs.wat:1001`** — *"expire past-deadline waiters"*. This is `expired-waiters`,
   and it is the **only** timeout-driven drop in the file. **Nothing discards messages** — see STOP-3.
5. **`wat-scripts/fanout/circuit.wat`** — the report line, and the two `Queue::StatsResponse::Ok`
   consumers. The circuit already dials every queue; it needs to read stats per tier and print per tier.

## Implementation sketch

```wat
;; :ephemeral gains five i64 counters (NOT :durable — the Record must not change)
sends-accepted <- :wat::core::i64
sends-refused  <- :wat::core::i64
acks           <- :wat::core::i64
redeliveries   <- :wat::core::i64
expired-waiters <- :wat::core::i64

;; the reply carries a named aggregate instead of widening a positional variant
(:wat::core::defrecord :queue::Stats
  [receive-calls <- :wat::core::i64  ticks <- :wat::core::i64
   visible <- :wat::core::i64  unacked <- :wat::core::i64
   store-calls <- :wat::core::i64  store-ns <- :wat::core::i64  handler-ns <- :wat::core::i64
   sends-accepted <- :wat::core::i64  sends-refused <- :wat::core::i64  acks <- :wat::core::i64
   redeliveries <- :wat::core::i64  expired-waiters <- :wat::core::i64])

:Ok [stats <- :queue::Stats]
```

Increment at the arms that already branch. The circuit reads stats once per phase boundary and prints
`tier=inbox` and `tier=sub[i]` lines; rates are `count ÷ phase duration`, computed at print time.

## Blast radius

**8 files, 17 sites** — counted: `sqs.wat`, `sns-fanout.wat`, `circuit.wat`, and the five scratch-pad
probes that match on `Queue::StatsResponse::Ok` (`probe-the-server-manages-its-own-capacity`,
`probe-three-waiters-wake`, `probe-depth-derived-from-the-index`, `probe-stats-sees-an-expired-unacked`,
`probe-does-anyone-hold-a-service-name`). The compiler names any I missed — **trust it over this list.**

## STOP triggers

**STOP-1** — if a counter increment requires a store call, **STOP**. The instrument must be free; a
sampled version would be 39 % of store traffic (2 store calls per `stats`, five queues, measured).

**STOP-2** — if the counters must go in `:durable`, **STOP and say why**. `:ephemeral` is what keeps the
`Record` — and therefore every constructor and every scratch-pad probe — out of the diff.

**STOP-3** — do **not** add a discard, retention, or dead-letter path so that something can be counted.
Nothing discards messages today; that absence is a finding to report, not a gap to fill.

**STOP-4** — do **not** change admission, visibility, ack semantics, or the cap. Counting only.

**STOP-5** — if `sends-refused` does not account for the publisher's `full-retries`, **STOP and report
the gap.** It means another rejection path exists, which is worth more than the counters.

**STOP-6** — on any red floor arm: capture whole, name the arm, do not re-run.

## What "done" looks like

One n=2000 `vis-ms=1000` run on a quiet box, printing per-tier lines for the inbox and each of the m
subscriber queues, each carrying arrival, service, refusals, redeliveries, expired waiters and depth —
with `store-calls` within a few of the 9731 baseline, proving the instrument is free. The no-arg run and
every wrapper report identical completeness fields. Floor Summary reads 5235 / 22 skipped / 0 FAIL /
0 TIMEOUT.

The SCORE should state **which tier is the constraint** (arrival ≈ service, depth growing), and record
plainly that **no message-discard counter was added because no discard path exists.**
