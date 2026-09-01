# BRIEF — item (c) stone D: the bounded buffer

Give `logs` and `durations` a bound, drop the oldest on overflow, count every drop, and tell the
caller. Closes the last unbounded path: today a persistently failing sink grows the buffer until the
observed service dies.

Read `DESIGN-STONE-the-bounded-buffer.md` beside this first — it carries the three-part contract
decision and why a drop must be reported on two channels rather than one.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat`, the `log` arm** — the `conj` onto `logs`, and the size trigger above
   it. The bound check goes beside the size trigger, not inside `flush-logs`: this is a *memory*
   bound on accumulation, not a *wire* cap on submission. They are different questions.
2. **`wat/telemetry/span.wat:135`** — `(hashmap::assoc ds1 name (conj samples1 nanos))`. `durations`
   grows per `timed` call exactly as `logs` grows per `log`. Same treatment, its own bound and its
   own counter.
3. **`wat/telemetry/span.wat:64`** — `(hashmap::assoc cs name next-would)`. Counters are one `i64`
   per key: already bounded, **do not touch them**.
4. **`wat/telemetry/span.wat`, `incr`** — the exemplar for incrementing a counter in the existing
   `counters` map. The drop counters are ordinary counters; that is the whole point.
5. **`wat/telemetry.wat`** — the Record's cadence fields (stone B) show where a configurable bound
   goes and how it is defaulted.

## The work

**1. Two bounds on the Record**, beside the cadences: `logs-max`, `duration-samples-max`, defaulted,
overridable at `span/start`. Counted in ITEMS.

**2. Drop the oldest on overflow.** When appending would exceed the bound, drop from the FRONT until
it fits, then append. The arriving record is kept — it is the freshest.

**3. Count every drop** into the existing `counters` map as `:logs-dropped` / `:samples-dropped`.
Ordinary counters: they emit through the existing metrics path, on the existing clock, as deltas.

**4. Tell the caller.** `LogResponse` and `TimedResponse` gain `:Dropped [buffered <- i64  cap <- i64]`.
`Ok` continues to mean **accepted**; a dropped record was not accepted and must not be called `Ok`.

## Blast radius

`wat/telemetry.wat` (Record fields + two response variants), `wat/telemetry/span.wat` (two arms).
**No `Journal` change. No new surface op. No runtime change. Do not touch `flush-*` or the batched
writers.**

## STOP triggers

**STOP-1 — a drop must never be silent.** Both channels or neither: the counter AND the response. A
drop that only increments a counter is invisible at the call site; one that only returns `Dropped` is
invisible to the operator. If either is impractical, STOP and report — a half-reported drop is the
silent loss this campaign exists to remove.

**STOP-2 — do not bound `counters`.** One `i64` per key is already bounded. A key-cardinality limit is
a different concern with no evidence behind it.

**STOP-3 — do not block the producer.** A full buffer must not make `log` wait. A service must not
stall because its log sink is down; that is the failure mode this stone exists to prevent, arrived at
from the other side.

**STOP-4 — the drop counter must survive the condition it reports.** It lives in `counters`
(`O(1)` per key), never as a Log or a duration sample — those are the things being dropped.

## The gates to write

- **★ the bound holds:** with a failing sink, log far past `logs-max` — the buffer never exceeds it.
  **RED today; unbounded growth is the finding this closes.**
- **★ the drop is counted:** after that overflow, drain against a working sink — a `:logs-dropped`
  metric appears with the exact number dropped. Not approximate.
- **the caller is told:** the `log` that overflows returns `:Dropped`, not `Ok`.
- **the oldest go:** after overflow the buffer holds the MOST RECENT `logs-max` records, in order.
- **samples too:** the same four against `timed`/`duration-samples-max`.
- **nothing changes when under the bound:** every stone A/B/C and item-(b) gate still passes.

## Prior comparable result

`SCORE-item-b-batched-writer.md` beside this — the first stone in this campaign with no delta, and
its note on why: the rulings were made against measurements taken first, not from the design's prose.
