# DESIGN STONE — long polling in `wat-queue`

**Commissioned 2026-09-01.** The consumer of the deferred reply. Real SQS has `WaitTimeSeconds` for
one reason and we now have the same reason, measured.

## Why

At process locus a service call costs **154 µs** — the fast end of a realistic network hop, and it
only rises when the transport becomes TCP. `circuit.wat`'s worker asks `:limit 1` and its fold runs
`cap` times regardless, so **every empty poll spends a network round-trip to be told "nothing yet."**
The circuit's 88 s is a round-trip budget.

Long polling removes the empty round-trip; batching divides the non-empty ones.

## Everything needed now exists, and did not this morning

- **wait** — `poll'` over a real selectable set; timer-as-peer (arc 292); timerfd at process tier.
- **the deadline** — `Alarm` + `NoReplyAndArm`, green at both loci as of today.
- **naming the waiter** — `ctx`'s `conn-id`, *"the name that outlives the round"*.
- **delivering to it** — `Outcome::ReplyTo`, landed this evening, and proven from a **timer** arm at
  both loci.

## ★ THE CONTRACT DECISION: one receive path, not two

A woken waiter must get **exactly** what an immediate `receive` would have got — the same
`scan-index`, the same visibility re-put, the same envelope shape. If the wake path builds its own
reply, the two drift, and they drift *silently*: the immediate path is covered by every existing
queue gate and the wake path by none of them.

> The `send` arm and the expiry tick must call **the same receive function** the `receive` arm calls.

This is the discipline that made the telemetry stones hold — *one emit-and-reset path per
accumulator*, so a mid-life flush and `close` could not disagree. Same shape, different service.
**Two receive paths is the double-count of this stone.**

## The shape

`ReceiveRequest` gains `wait-ns <- i64`.

```
receive, messages available      → reply now                       (today's path, untouched)
receive, none, wait-ns = 0       → reply empty                     (today's path, untouched)
receive, none, wait-ns > 0       → park: store the waiter, NoReplyAndArm the expiry tick
send arrives                     → put, then run the receive path for parked waiters, ReplyTo them
-expire-waiters fires            → ReplyTo empty to any past deadline, drop them, re-arm if any remain
```

★ **`wait-ns = 0` must be byte-identical to today.** Every existing queue gate then passes untouched,
and that is the evidence that long polling is additive rather than a rewrite.

## Waiters live in `:ephemeral`

A waiter is `{conn-id, queue, limit, visibility-ns, deadline-ns}`. It belongs in `:ephemeral`, and the
reason is not convenience: `:durable` *"crosses the wire, survives hibernation"*, and **a conn-id does
not survive either**. The connection it names is gone on the far side of a fork and gone after a
resume. Persisting a waiter would be persisting a promise to a client that cannot exist.

## Arming without a flag

**One recurring expiry tick, not one timer per waiter** — because an internal arm is `[s ctx]` with
**no request payload**, so a fired timer cannot say *which* waiter expired. The tick scans instead.

Armed on the **empty→non-empty transition** of the waiter set, exactly as the span's flush timers are
(item (c) stone B): the tick resets nothing, so it re-arms only while waiters remain, and **a queue
with no waiters never wakes**. No `armed?` field, which has no honest home.

And the tick needs no clock read: `ctx`'s `start-ns` is *"one clock read, stamped fresh per call in
the serve loop"* — the substrate already stamps the moment the tick fired.

## Ordering

FIFO among waiters, matching `ReplyTo`'s own delivery order (the vector's). First parked, first
served — the obvious policy, stated so a second consumer can argue for another with evidence.

## Out of scope = REJECTED

- Any change to `Outcome`, `service.wat`, or the runtime. The substrate is done.
- Per-waiter timers (impossible without an internal-op payload, and unnecessary).
- A max-waiters bound. Real, but it is stone D's argument in a new place and wants its own
  measurement.
- Changing `limit`'s existing meaning. It is already honoured, and `limit 1` stays valid for a client
  that wants exactly one.
