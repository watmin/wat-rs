# DESIGN — the server manages its own capacity

**Partial admission.** `Queue::send` accepts what it has room for and says how many; `Topic::publish`
does the same in messages. `SendResponse`/`PublishResponse` stop leaking depth and cap.
`wat-scripts/queue/sqs.wat` + `wat-scripts/topic/sns-fanout.wat` + `wat-scripts/fanout/circuit.wat`.

## ⛔ FIRST — A RULING OF MINE IS REVERSED, WITH THE REASON

`TRACKER … WHERE WE ARE NOW` says *"per-entry outcomes on the batch surfaces — **RULED OUT**
2026-09-06."* **That ruling is wrong for `Queue::send` and `Topic::publish`.** It stands for
`Store::put`.

The evidence behind it was that a **store write** cannot partially fail — measured, still true:
`put-one-row` is DELETE → clear-index → INSERT → insert-projections, every step on a table `:init`
created, and a missing projection is skipped, not an error.

★★ But that is the **storage** layer. `Queue::send` has an **admission** gate *before* the write
(`sqs.wat:353`), and admission is partial by nature: *"I have room for 4 of your 10."* Carrying a
storage-atomicity result to an admission decision is the error, and it is mine.

★ There is no conflict between them: the queue admits 4, writes **those 4 atomically**, rejects 6.
Atomic storage, partial admission.

## WHY — the all-or-nothing gate manufactures three defects

`sqs.wat:353`:

```wat
(:wat::core::if (:wat::i64::> (:wat::i64::+ depth n0) cap)
  … (:queue::Queue::SendResponse::Full depth cap) …
```

1. **A batch too large for `cap` can NEVER be admitted** — not a capacity condition, a permanent
   livelock. At `nsubs ≥ 7` a legal 10-message publish makes 70 bodies against `cap 64` and can
   never land. Today that is masked by an `assertion-failed!` — the service dies on well-formed
   input.
2. **Whole batches bounce off a queue that had room.** Measured: **3770 `Full` retries per run**,
   ~19 per accepted batch, ≈3.8 s of the 22.6 s publish window spent sleeping on a queue that
   usually had *some* space.
3. **`Full [depth cap]` hands the caller our internal pressure state.** Every client that matches
   it is coupled to our sizing. A client should learn what happened to *its* request and nothing
   about our internals.

## ⛔ THE ONE CONTRACT DECISION — `Accepted [count]`, a PREFIX, no internals

`:Ok` and `:Full` collapse into one arm:

```wat
:Accepted [count <- :wat::core::i64]     ;; the first `count` entries landed
```

- `count == n` — all of it
- `0 < count < n` — the queue took what it had room for; **resend from `count`**
- `count == 0` — no room now; retry unchanged. This is the 429.

★★ **Prefix, not a per-id list.** A queue is ordered and `send` already staggers `isk` by index
(`sqs.wat:371`), so "the first N landed" is the natural unit and the caller's retry is a drop.
SQS returns `Successful`/`Failed` id lists because its entries carry client-chosen ids; ours do
not, and inventing them to imitate the shape would be cargo-culting.

⚠ **`depth` and `cap` appear in no response.** That is the point of the stone as much as the
partial admission is.

## THE TOPIC, IN MESSAGE UNITS

`publish` fans **msg-major** (`sns-fanout.wat:84-99`: for each msg, all subs), so a prefix of
pairs is a whole number of messages plus possibly a partially-fanned tail message.

```wat
:Accepted [count <- :wat::core::i64]     ;; messages FULLY fanned = floor(pairs-accepted / nsubs)
```

★ A partially-fanned tail message reports as **not accepted**; the client re-publishes it, and the
subscribers that already received it get a duplicate. **That is exactly what at-least-once means**,
and `Seen` is the dedupe that already exists for it. The `(message, subscriber)` pair remains the
delivery unit, as the tracker rules.

⚠ So **`dup` may become non-zero** and must stay a REPORT. `distinct` remains the gated invariant.

## `:max-entries` COMES OFF `Queue::send` — it is not optional

An entry cap **rejects outright what admission would take partially**, so it directly defeats this
stone. It also never belonged: I added it modelling SQS's `SendMessageBatch = 10`, a *public HTTP
API* limit, on an **internal** queue whose only writer is the topic — and `:max-request-bytes`
already carries the real wire constraint (51 KB at 256 subscribers against a 512 KB cap).

★ It **stays** on `Topic::publish`, which is the client-facing surface where 10 is SNS's own
number and a publisher genuinely should be bounded.

## OUT OF SCOPE — REJECTED

- **Moving the fanout to the drain worker.** A topology change, and partial admission removes the
  reason for it. If capacity ever needs decoupling from `nsubs` for another reason, that is its own
  stone.
- **Raising any `cap`.** The pressure system is the feature.
- **`Store::put` per-entry outcomes.** The ruling stands there; storage really is all-or-nothing.
