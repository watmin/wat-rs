# DESIGN — the topic is durable

Drawn 2026-09-02. **Not struck.** Main line item 2.

## Why

`publish` returns `Ok` — which this arc has defined as **accepted** — for a message held in
`:ephemeral` state:

```wat
;; wat-scripts/topic/sns-fanout.wat
:durable   [cap, delay-ns]
:ephemeral [subs, outbox, ticks, deliver-armed?, arm-deliver]
```

If the topic dies, every accepted-but-undelivered message is gone. Meanwhile the **queue** puts to
its store *before* replying `Ok`. Two services in the same fixture disagree about what acceptance
means, and only one of them is telling the truth. That asymmetry has been there since the
async-publish stone and nothing pointed at it.

This is a **correctness** stone. Any throughput change is a side effect to be reported, not a goal.

## What it delivers

A topic whose `Ok` is true, and whose delivery is at-least-once **per subscription**.

## The shape — composed, not invented

```
topic-service  =  publish surface  +  ONE queue-service instance  +  J internal workers
```

- `publish` writes **N rows, one per subscription**, into the topic's own internal queue, then
  replies `Ok`. The write is the durability; the `Ok` follows it.
- internal workers consume the internal queue and call `Queue/send` on the subscriber's queue,
  **acking only on success**.
- subscriber returns `Full` (or anything but Ok) → **do not ack** → the row's visibility expires →
  it is retried.

★ **The unit is `(message, subscriber)`, not `message`.** SNS is **at-least-once per subscription**
— retry policies and DLQs attach per subscription — and a message-level row would re-deliver to
*every* subscriber when one of them needed a retry. Per-subscription rows are also what make one
stalled subscriber hold up only its own backlog.

★ **Do not write a second delivery engine.** A durable topic with background retry *is* a queue
whose consumers are subscribers. The queue already has the level-triggered wakeup, the depth bound,
parked waiters, batching and — as of `PROBE(278)` — a **proven** redelivery path. Reimplementing an
outbox inside the topic would be a second copy of all of it.

## The one contract decision: retry comes from visibility, not from a retry counter

The internal worker's failure handling is **"do not ack"**. Nothing counts attempts, nothing
schedules a backoff, nothing tracks per-row state beyond what the queue already tracks.

This is proven, not assumed — `wat-scripts/scratch-pad/probe-visibility-redelivers.wat`, gated by
`tests/services/probe_queue_visibility.rs`:

```
first=got;while-inflight=none;after-expiry=got;same=yes
```

An in-flight row is **invisible** to other workers, and an unacked row comes back **as the same
message**. That is the entire retry mechanism, and it already exists.

## What this deletes

The topic's `:ephemeral outbox`, its `-deliver` tick, `deliver-armed?`, and the `arm-deliver`
helper — all of it becomes the internal queue's job. The topic's own cap/`Full` backpressure is
replaced by the internal queue's depth bound.

**Deleting the outbox is part of the stone, not a follow-up.** Leaving both would mean two buffers
in series, which is how this arc got into trouble in the first place.

## Out of scope = REJECTED

- **Endpoint subscribers** (HTTP, lambda — things that can simply be *down*). Queue subscribers only,
  as the builder ruled. The design generalises to them; building it does not belong here.
- **Packet loss.** Main line item 3, and it is what will force `dup=0` to become `dup ≥ 0`.
- **Per-subscription retry policy / DLQ / backoff.** Real SNS has them; visibility alone is the
  mechanism here, and a counter is a different stone.
- **`wat/`, `src/`.** Neither changes.
