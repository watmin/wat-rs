# DESIGN STONE — item (c) stone D: the bounded buffer

**Commissioned 2026-09-01.** Closes the last unbounded path in the telemetry chain, and with it the
ruling that has been open since stone A.

## What is already bounded, and what is not — measured

| accumulator | shape | bounded? |
|---|---|---|
| `counters` | one `i64` per key (`hashmap::assoc cs name`) | ✅ by key cardinality, not by call count |
| `durations` | `Samples` = `(Vector :- [i64])`, one `conj` per `timed` (`span.wat:135`) | ⛔ grows per call |
| `logs` | `(Vector :- [Log])`, one `conj` per `log` | ⛔ grows per call |

And `flush-logs` post-item-(b) always returns `suffix = drop(logs, written)`. A persistently failing
sink gives `written = 0` every time, so the buffer retains everything and grows with each call.
**I grepped for a bound: there is none.**

So the chain is bounded *while the sink accepts* — size trigger, two clocks, and fragmentation all
guarantee a drain — and **unbounded exactly when the sink is down.**

## Backpressure is NOT the missing piece — it is already there

Every hop is `send'` then `recv'`: blocking request/reply. And the span is a **serializing actor**, so
while its loop is inside a flush waiting on the journal it is not serving anyone — every `log` caller
queues behind it. A slow sink transitively slows its producers, for free, through the one loop.

The only unbackpressured window is accumulation *between* flushes, which is what a buffer is for.
Backpressure is correctly absent in the one place it should be.

## The failure this stone prevents

Unbounded growth on a failing sink means **the service being observed dies because its observability
backed up.** Telemetry must never kill its host. That outranks keeping every log.

## ★ THE CONTRACT DECISION: bound each accumulator, drop the OLDEST, and count it

Three parts, and the third is what makes the first two honest:

**1. A bound per accumulator**, on the Record beside the cadences: `logs-max` and
`duration-samples-max`. Counted in ITEMS, not bytes — `O(1)` to check, the same unit the drop counter
reports in, and the unit an operator actually reasons about. (Bytes are already the *wire* cap's job;
this is a *memory* bound. Different questions, different units, both needed.)

**2. Overflow drops the OLDEST.** When the sink returns, the freshest context is what is worth
having; a buffer stuck for an hour holding hour-old records is the less useful half. This is the
ring-buffer intuition and what log agents do.

*The alternative considered:* drop the newest (reject the arrival) is simpler and avoids rewriting the
buffer, and it keeps the records closest to the onset of the failure. Rejected because the recovery
case — sink returns, what do I still have? — is the one that matters, and it wants recency.

**3. Every drop increments an ordinary counter** — `:logs-dropped`, `:samples-dropped` — in the
existing `counters` map. This costs no new machinery: it emits through the existing metrics path, on
the existing clock, as a delta per period, and it is `O(1)` in space so it survives the very
condition it reports.

★ **A drop that nothing records is exactly the silent loss this campaign has spent all day removing.**
The counter is not a nicety; it is what makes a bounded buffer honest rather than a quiet data hole.

## `log` must not answer `Ok` when it dropped

`Ok` means **accepted**. A dropped record was not accepted, so saying `Ok` re-creates stone C's lie
one layer up.

This does need a new variant — and it is a different case from the `Buffered` one I ruled against in
stone C. That was two names for one outcome (accepted). This is a *second outcome*: not accepted.

`LogResponse` / `TimedResponse` gain `:Dropped [buffered <- i64  cap <- i64]` — the caller learns its
record did not make it, and the numbers say why. Not `RequestTooLarge` (the item is not too large,
the buffer is full) and not a sink failure (the sink refused nothing; we did).

So a drop is reported twice, deliberately: **immediately** to the caller as a matchable value, and
**in aggregate** through the counter to the backend. Neither channel alone is enough — the caller
usually cannot act, and the operator is not watching the call site.

## Out of scope = REJECTED

- **Blocking the producer when the buffer is full.** Coherent (backpressure already exists during a
  flush) but wrong: a service must not stall because its log sink is down.
- **Bounding `counters`.** Already `O(1)` per key; a key-cardinality bound is a different concern with
  no evidence behind it.
- **`write-*-stream` / item (a).** Unchanged ruling: it waits for a consumer that actually streams.
- **Persisting a dropped record anywhere.** A bound that spills to disk is a different system.
