# BRIEF — the topic is durable

`publish` returns `Ok` for a message that exists only in `:ephemeral` state — if the topic dies it
is gone, while the queue beside it puts to a store before replying `Ok`. Give the topic its **own
queue-service instance**, write one row per subscription before replying, and let internal workers
drain it to the subscribers.

## Read in order

1. **`DESIGN-STONE-the-topic-is-durable.md`** — the shape, the `(message, subscriber)` unit, and the
   one contract decision (retry is visibility expiry, not a counter).
2. **`wat-scripts/scratch-pad/probe-visibility-redelivers.wat`** and
   **`tests/services/probe_queue_visibility.rs`** — the retry mechanism, **already proven and
   gated**. `first=got;while-inflight=none;after-expiry=got;same=yes`. Do not re-derive it; this is
   your worked reference for what "do not ack" buys you.
3. **`wat-scripts/queue/sqs.wat`** — the whole service you are composing with. Note especially its
   `:durable [cap]`, the `send` cap check, and `SendResponse::Full`.
4. **`wat-scripts/fanout/circuit.wat`**, `:fanout::worker` — the internal workers are **this
   service**, or its shape: park on the internal queue, take a batch, act, ack. Do not write a new
   one from scratch.
5. **`wat-scripts/topic/sns-fanout.wat`** — everything being deleted: `outbox`, `-deliver`,
   `deliver-armed?`, `arm-deliver`, and the topic's own `cap`/`Full`.

## The sketch

Load-bearing: the store write precedes the `Ok`, the row is per-subscription, and the failure path
is *not acking*. Illustrative: naming.

```
publish  ->  for each subscription s:  row(pk = topic/s, body = msg)
             ONE Queue/send carrying N bodies to the internal queue
             then reply Ok                     <-- Ok AFTER the write, never before

worker   ->  receive from the internal queue (batch)
             for each row: Queue/send to that subscriber's queue
               Ok    -> ack
               Full  -> DO NOT ACK  (visibility expires; retried)
```

## Blast radius

`wat-scripts/topic/sns-fanout.wat` (mostly deletion), `wat-scripts/fanout/circuit.wat` (wiring the
topic's internal queue + store), and any `Topic::PublishResponse::Full` match site.
**`wat/` and `src/` untouched.**

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Still the guard. Note that
   at-least-once *permits* duplicates in principle; with reliable IPC and no loss injected there is
   no source for one, so a `dup > 0` here is a real bug, not the semantics showing through.
2. **If you find yourself keeping the old `:ephemeral` outbox as well — STOP.** Two buffers in
   series is the shape this arc spent a day escaping.
3. **If you find yourself writing a retry counter, a backoff, or per-row attempt state — STOP.**
   Visibility expiry is the mechanism and it is proven; anything more is a different stone.
4. **If `wat/` or `src/` need to change — STOP and surface it.**
5. **If the topic's internal queue needs a different Store than the subscribers use — STOP and say
   why.** It should be an ordinary `mem-store` like every other.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` before any timing.

**Five runs, report the spread.** Throughput is a **side effect** here, not the goal — report it,
do not chase it.

Write `SCORE-the-topic-is-durable.md` when done. It will be graded by re-running.
