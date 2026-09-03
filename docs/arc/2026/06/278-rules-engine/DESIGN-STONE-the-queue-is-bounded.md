# DESIGN — the queue is bounded

Drawn 2026-09-02. **Not struck.**

## Why

Bounding the topic's outbox did not create backpressure. It moved the reservoir one stage
downstream, to the only stage with no bound at all: **`Queue/send` always accepts.**

```
                        mem            sqlite
unbatched (cap 16)   661/s  200ms   789/s  185ms
batched   (K=10)     282/s  36-42s  1568/s  2.6s
```

Batching works — 2.0× on a linear store — and costs **13× latency**, because the queues absorb what
the topic no longer does (`t3→t4 >1 s` on ~7000 of 8000 messages).

The model this arc is chasing is the builder's: *"one function call, lockstep all the way down,
organic backpressure."* That requires **every** stage bounded. We bounded one.

## What it delivers

`Queue/send` refuses at a depth cap, and the refusal **propagates**: adapter → topic → producer. The
reservoir then has nowhere to move to, because every stage has a bound.

## The one contract decision: the adapter BLOCKS rather than dropping or buffering

When the queue refuses, the adapter has three options and only one is backpressure:

- **drop** — data loss, and the invariant is the whole point of this fixture
- **buffer in the adapter** — the reservoir moves *again*, one stage further up. This is the mistake
  this stone exists to stop repeating
- **block until accepted** — the adapter's serve loop stalls, so the topic's `Sub/deliver` stalls,
  so the topic's outbox fills, so `publish` refuses, so the producer paces

**Block.** It is the only one that makes the chain behave like one function call, and it is exactly
what the builder described.

## Known wart, named up front: the retry is a poll

The adapter retries on `Full` with a small nap, mirroring `accept!` in `circuit.wat`. **That is a
poll, and this arc has spent the day deleting polls.** It is accepted here because this stone is a
hypothesis test — *does bounding every stage restore latency while keeping the 2×?* — and the poll
is the smallest mechanism that answers it.

If the answer is yes, the poll is replaced by a parked reply (the queue holds the sender's `conn-id`
and answers when there is room), which is the same repair already made for the workers and now
expressible because `Outcome` composes. **That is a follow-up with a trigger, not a deferral.**

## The surface

```
Queue::SendResponse   :Ok []   ->   :Ok [] | :Full [depth <- i64  cap <- i64]
queue::queue::Record  gains    ->   cap <- i64
```

One variant on a response enum and one durable field — mirroring `Topic::PublishResponse::Full`,
which already exists and already works. The adapter must face `Full`; nothing else changes shape.

## Out of scope = REJECTED

- **Replacing the retry poll with a parked reply.** Above, with its trigger.
- **Partial batch acceptance** (the adapter takes 7 of 10 and reports it). Correct in principle,
  and it needs per-subscriber cursors in the topic — a much larger change. All-or-nothing per batch
  is wasteful and simple; measure first.
- **`mem-store`'s quadratic writes.** Real, and if the queues stay shallow its share should fall
  back toward the 1.19× it shows unbatched. **Re-measure the 2×2 after this** before deciding
  whether it is a perf problem or only an oracle problem.
- **`wat/`, `src/`.** Neither changes.
