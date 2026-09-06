# DESIGN — a single value is a batch of one

**The queue and seen surfaces become batch-only.** `wat-scripts/` — `sqs.wat`, `circuit.wat`,
`sns-fanout.wat`, two probes. Architectural perf. Correctness preserved and measured.

## WHY — 30 round trips where 3 would do

The worker receives **10 envelopes** (`:limit 10`, which is also SQS's batch maximum) and then
processes them **one at a time**:

```
per envelope:  Seen/check  →  emit  →  Seen/mark  →  Queue/ack
per batch:     10 checks + 10 marks + 10 acks  =  30 round trips
```

The topic worker adds **10 more acks** per batch — its send is already bucketed and batched, its
ack is a `foldl`.

★ Three surfaces are singular, and they are the only ones left:

```wat
Queue::AckRequest   [queue  id  <- String]      ← ONE
Seen::CheckRequest  [queue  seq <- String]      ← ONE
Seen::MarkRequest   [queue  seq <- String]      ← ONE
Queue::SendRequest  [queue  bodies <- Vector]   ← already batched
Queue::ReceiveRequest [… limit …]               ← already batched
```

**The batch shape was built on `send` and `receive` and never finished.** Across a run that is
~32 000 singular round trips (8000 × 3, plus 8000 topic acks) where ~3200 would do — at a
measured **179 µs of dispatch plus a store verb** each, most of it serialized through the queue
and seen actors.

## ⛔ THE ONE CONTRACT DECISION

**There is no singular form. A caller with one item passes a batch of one.**

Not `:ids` *beside* `:id` — `:id` **ceases to exist**. The single-item call becomes
unrepresentable, which is the only version of this that cannot rot: an optional batch API grows
a singular caller the first tired afternoon.

Batch cap **10**, matching `receive`'s existing `:limit 10` and SQS's batch maximum.

## ⛔ PER-ENTRY RESULTS ONLY WHERE OUTCOMES ACTUALLY DIFFER

SQS's `DeleteMessageBatch` returns `Successful[]` / `Failed[]` because its backend can partially
fail. **Ours provably cannot**, and we measured it: a second ack of a deleted row is a **no-op
returning `Success`** (`SCORE-the-queue-can-drop-too`, row 4, answered from a run).

| verb | response | why |
|---|---|---|
| `check` | **a vector of `Recorded`/`Absent`, aligned to input order** | each seq genuinely has its own answer |
| `mark` | `:Ok []` | idempotent hashmap write; it cannot partially fail |
| `ack` | `:Ok []` | a missing row deletes as a no-op `Success` — measured |

★ **A `Failed[]` we can never populate would be a lying surface.** Copying SQS's shape past the
point where our semantics match it is the collapse this arc keeps refusing.

⚠ **Expiry condition:** if the store ever becomes remote or fallible, `mark` and `ack` gain a
per-entry result list *then*, on evidence. Written down so a later reader knows the omission was
reasoned, not overlooked.

## ⚠ THE CORRECTNESS WINDOW WIDENS, AND THAT MUST BE MEASURED

The receipt discipline survives — the order is still **check-all → emit the absent → mark those
→ ack all**, so the receipt is still written *after* the work.

★★ But between `check` and `mark`, **all ten** are now unmarked instead of one. A concurrent
worker checking the same seqs sees `Absent` for the whole batch. **That is the s3 window
widening from 1 message to `batch-size`** — the same window
`redelivery_mid_processing_never_loses` already gates deterministically.

**`distinct` must still hold. `dup` may rise, and if it does that is reported, not hidden.**

## FILES

`wat-scripts/queue/sqs.wat`, `wat-scripts/fanout/circuit.wat`, `wat-scripts/topic/sns-fanout.wat`,
`wat-scripts/scratch-pad/probe-three-waiters-wake.wat`,
`wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat`.

11 `AckRequest` sites, 7 `Queue/ack` calls, 2 `Seen/check`, 3 `Seen/mark`. **No codemod** — the
per-row folds become batch calls, which is semantic restructuring, not a mechanical rewrite.

## OUT OF SCOPE = REJECTED

- **`src/`, `wat/`.** This is a userland surface change.
- **Batching `Store` verbs.** A separate layer and a separate stone; mixing them destroys
  attribution.
- **Raising the inbox cap.** Measured last stone as pure displacement: `publish` falls, `drain`
  rises, throughput unchanged.
- **Compiled wat.**
