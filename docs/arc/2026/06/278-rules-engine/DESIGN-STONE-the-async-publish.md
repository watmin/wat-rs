# DESIGN STONE — accept, then fan out

**Commissioned 2026-09-01.** The circuit's latency, located by phase instrumentation:

```
setup (spawn ~22 processes)   ~10.4 s
publish (2000 messages)        24.3 s      ← 12.2 ms each
drain                           0.02 s
stop + collect                  1.1 s
```

**The consumers were never the problem.** Drain is 21 ms — the workers keep up with the producer in
real time and then wait. Every perf stone before this one aimed at the consumer side.

## The defect

`sns-fanout.wat`'s `publish` is a `foldl` over subscriber peers calling `Sub/deliver` **synchronously**,
one at a time. Each `deliver` blocks on the adapter, which blocks on `Queue/send`, which blocks on
`Store/put`. One publish is **13 sequential blocking round-trips and the publisher waits for all of
them.**

Two things make that 12.2 ms rather than ~2 ms: the chain is **serial** (nothing pipelines), and each
hop costs ~**6×** its idle price (~900 µs against a measured 154 µs) because those queues and stores
are single-threaded actors *simultaneously* serving the consumer side.

**Real SNS returns once the message is accepted.** Delivery to subscribers is its own concern. We
made the publisher wait for every subscriber's disk write.

## The shape

```
publish  → append to the outbox, reply ACCEPTED, arm the drain tick
-deliver → take the head, fan out to subscribers, re-arm while the outbox is non-empty
```

The publisher's critical path becomes **one hop instead of thirteen**, and the fan-out overlaps
instead of serialising.

Both pieces landed today and neither existed when this circuit was written: **`Outcome::ReplyTo`**
(reply to a client that is not the invoker) and **self-scheduling** (`Alarm` + `NoReplyAndArm`, green
at both loci). Arm on the **empty→non-empty transition**, no stored flag — the pattern from the span
and the queue.

## ★ THE CONTRACT DECISION: `publish` means ACCEPTED, and the drain condition grows a term

`publish` today returns the count it delivered. It cannot any more — it has not delivered anything
yet. So `Ok` means **accepted for delivery**, exactly as `LogResponse::Ok` came to mean *buffered*.

And that has a consequence the circuit must respect:

> **An empty queue no longer means the work is done.** A message can be accepted, sitting in the
> topic's outbox, not yet delivered to any queue. Stopping on `pending = 0 AND in-flight = 0` would
> stop before it arrives.

So the drain condition gains **the topic's outbox depth**. This is the same lesson as the sane
circuit's in-flight term, one layer upstream: **a completion check must cover every place a message
can be resting**, and async delivery just created a new one. Getting this wrong loses messages and
the invariant catches it — `distinct < 8000`.

## ★ Async publish CREATES the unbounded-buffer risk, so the bound ships with it

The whole point is that the publisher no longer blocks. Which means it can now outrun the fan-out,
and the outbox grows without limit — the exact failure item (c) stone D fixed for the span's logs,
reachable here for the first time **because of this change**.

So the outbox is bounded, and a `publish` that cannot be accepted says so:

- **not dropped.** A dropped log is a lost line; a dropped publish is lost *data the caller could
  have handled*. The caller is right there, holding the message, and it is synchronous with them.
- **a distinct response** — accepted vs refused-full. Refusal is backpressure, which is what a real
  broker does when a publisher outruns it (SNS throttles; it does not silently discard).

Stone D learned this for logs by shipping the hole first. Here it is known before the change lands,
so it lands with the change.

## Out of scope = REJECTED

- Any substrate change. `ReplyTo` and self-scheduling are done.
- Delivery retry policy, DLQs, per-subscriber failure isolation. Real, and each wants its own
  evidence.
- The ~10.4 s setup cost (process spawn). Separately measured, separately drawn; it does not scale
  with `n` and is a different investigation.
