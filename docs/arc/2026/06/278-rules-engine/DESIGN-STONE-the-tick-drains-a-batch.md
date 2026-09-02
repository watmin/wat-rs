# DESIGN — the tick drains a batch

Drawn 2026-09-02. **Not struck.**

## Why

`topic-ticks=2000` for 2000 messages — **one `-deliver` tick per message**, measured. Every one of
them pays a timer arm, a timerfd fire and a select wake. The four queues together tick **654** times
across the same run, because they arm from state; the topic does not batch at all.

Per message, derived from two measured points (before/after concurrent fan-out):

```
9.57 ms/message
  ~3.72   the chain      max(4) — already fixed, and not addressed here
  ~1.87   outbox rebuild  paid ONCE PER MESSAGE
  ~2.11   tick overhead + per-tick state construction
```

**Both of the last two are per-tick costs, not per-message costs.** They are per-message only
because the tick handles one message.

## What it delivers

`-deliver` drains up to **K** messages per tick. One timer round trip per K, one state construction
per K, and — the part that is easy to miss — **one outbox rebuild per K** instead of one per
message.

## The one contract decision: K is bounded, not drain-until-empty

Draining until empty would be simpler and is **wrong**. The topic is a serializing actor: while
`-deliver` runs, it accepts nothing — not `publish`, not `stats`, not `Admin::Stop`. Draining 2000
messages in one arm would make the topic unresponsive for the whole run and undo the
interruptibility the sane-circuit stone bought (`SCORE-the-sane-circuit.md` row 5).

So K is a bound, and the trade is throughput against how long the topic is deaf. **K = 10**, which
is not a new number: it is the worker's existing `:limit 10` on `Queue/receive`, chosen for exactly
this reason. Reusing it keeps one tunable in the fixture instead of two.

## What is NOT batched

The fan-out stays **per message**: K rounds of (four sends, four recvs). Issuing all K×4 sends
before collecting would be wire-level pipelining — 40 replies outstanding, ordering and mailbox
depth both in play — and that belongs to wire-batching, not here.

★ **Tick-batching is not wire-batching**, and conflating them is why "batching" was cut wholesale
from the concurrent fan-out stone when only half of it deserved cutting:

| | what it amortises | cost |
|---|---|---|
| **tick** (this stone) | timer, state construction, outbox rebuild | **no surface change** |
| **wire** (still cut) | the chain itself | `Sub::DeliverRequest` **and** `Queue::SendRequest` both carry many — two contracts, two new places to lose a message |

## Out of scope = REJECTED

- **Wire-batching.** Above, with its cost. Size it after this lands.
- **The cursor.** Now ~20% of per-delivery and its expiry has arrived — but tick-batching amortises
  the rebuild by K, which changes that share again. **Re-measure after this, then rule.**
- **Tuning K.** Report what 10 does. A tuned constant with no stated trade is a guess wearing a
  number.
- **`wat/`, `src/`, `sqs.wat`, `circuit.wat`.** None of them change.
