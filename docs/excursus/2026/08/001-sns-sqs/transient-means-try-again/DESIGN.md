# DESIGN — transient means try again

**A retryable store error must not kill the queue.** `wat-scripts/queue/sqs.wat`.
Resilience. No surface change.

## WHY — the most recoverable error in the surface is treated as fatal

Three sites in `sqs.wat` match a store response and collapse **everything that is not
`:Success`** into one assertion:

```wat
:217  (_ (assertion-failed! "queue.take: scan-index failed" …))
:472  (_ (assertion-failed! "queue.send: store put failed"  …))
:728  (_ (assertion-failed! "queue.ack: store delete failed" …))
```

Those `_` arms swallow **four distinct outcomes**:

| store says | means | today |
|---|---|---|
| **`:Transient`** | **"busy — retry me"** | ⛔ **the queue dies** |
| `:Constraint` | schema / uniqueness violation | dies (correct) |
| `:Fatal` | do not retry | dies (correct) |
| `:RequestTooLarge` / `:RequestMalformed` | the caller's fault | dies (arguably correct) |

★ **This is live code, not injection-only.** `sqlite-store.wat:57` and `:70` map
`sqlite::Error::Transient` → `PutResponse::Transient` / `DeleteResponse::Transient`. SQLITE_BUSY
is an ordinary condition on a contended or file-backed store.

★★ **A "try again in a moment" from the store kills the queue service — and with it every client
of that queue**, not merely the one whose batch failed. `probe-entry-three-of-ten.wat` measured
it: `store=Transient; send=Lost`, and a message silently lost.

## ⛔ THE ONE CONTRACT DECISION

**`:Transient` is retried inside the queue, with a bounded budget. Everything else keeps dying —
but says which one it was.**

- **`:Transient`** → bounded retry (budget + a short wait between attempts). On success, the arm
  proceeds as if the first attempt had worked.
- **Budget exhausted** → assert, naming *transient, exhausted after N*. A store that is still
  unavailable after N tries is genuinely gone, and dying is honest then.
- **`:Constraint` / `:Fatal` / `:RequestTooLarge` / `:RequestMalformed`** → assert **with a
  message naming the variant.** Today all four produce the same string, so the operator cannot
  tell a schema violation from a dead disk.

★ **No surface change.** `SendResponse`/`AckResponse` gain nothing; no caller moves. Telling the
*caller* about an exhausted store is step 2's question, and it lands on a queue that no longer
dies for the wrong reason.

★★ The idiom already exists in this file — `sqs.wat:493`, `:523`: *"Do not claim Ok — the put is
unknowable. Full is the caller's retry."* An unknowable store outcome already hands the retry
somewhere. `:Transient` is the one that never got it.

## ⚠ WHAT A RETRY MUST NOT DO

A retried `put` or `delete` **re-sends the same batch**. `delete` is idempotent — measured, a
second delete of a missing row is a no-op `Success`. **`put` is not.** If a partial put landed
k of n and we retry the whole batch, the k already written are written **again**.

★★★ So the retry is honest for `:Transient` **only because `:Transient` from a real store means
the operation did not commit.** ⛔ **The wrapper in `probe-entry-three-of-ten.wat` deliberately
violates that** — it applies k of n *and* reports `:Transient`, which no honest store does.

**That probe models a lying store, and this stone does not fix lying stores.** It fixes the
queue's response to an honest one. The partial-commit case is step 2, and conflating them would
make this stone unmeasurable.

## FILES

`wat-scripts/queue/sqs.wat` only.

## OUT OF SCOPE = REJECTED

- **Per-entry outcomes** (step 2). This stone is in front of it, not instead of it.
- **Telling the caller about an exhausted store.** A surface change; step 2.
- **The topic batch** (step 3), `setup`/`stop` (step 4), compiled wat.
