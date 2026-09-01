# HANDOFF — item (c) stone D: the bounded buffer

You are closing the last unbounded path in the telemetry chain. Today a persistently failing sink
makes `flush-logs` return `written = 0` every time, the buffer retains everything, and it grows with
every call — until the service being observed dies because its observability backed up.

Start here, in order:

1. `DESIGN-STONE-the-bounded-buffer.md` — what is already bounded (measured), why backpressure is not
   the missing piece, and the three-part contract decision.
2. `BRIEF-item-c-stone-d-bounded-buffer.md` — the rooms as exact `file:line`, four STOP triggers.
3. `SCORE-item-b-batched-writer.md` — the stone before this, and the one with no delta; its note on
   why is worth two minutes.

Four things to hold:

**Bound `logs` AND `durations`.** Both grow per call — `logs` by a `conj` per `log`, `durations` by a
`conj` per `timed` (`span.wat:135`). `counters` is one `i64` per key and is already bounded: leave it
alone.

**★ Drop the OLDEST, and count every drop.** The counter is an ORDINARY counter in the existing
`counters` map, so it emits through the existing metrics path on the existing clock and costs no new
machinery. It is `O(1)` in space, so it survives the very condition it reports — which a dropped Log
would not.

**Report on BOTH channels.** The counter reaches the operator; `:Dropped{buffered, cap}` reaches the
caller. Neither alone is enough, and a drop reported on neither is exactly the silent loss this
campaign has spent the day removing. `Ok` keeps meaning **accepted** — a dropped record was not.

**Never block the producer.** A full buffer must not make `log` wait. Backpressure already exists
where it belongs (the span's loop blocks its clients while it flushes); adding it here would stall a
service because its log sink is down, which is the failure this stone prevents, arrived at from the
other side.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-item-c-stone-d-bounded-buffer.md` when done. It will be graded by re-running.
