# DESIGN STONE — item (c) stone B: two clocks

Stone A built the flush path and proved it cannot double-count. Stone B gives it **two independent
cadences**: logs flushed fast, counters and durations flushed on a slow beat (30s/60s), each
re-arming itself.

## What stone A left, exactly

`flush-accumulators` (`span.wat:322`) flushes **logs AND metrics together** and returns
`(State, CloseResponse)`. **Both** size triggers call it — so crossing the *logs* cap today forces a
*metrics* flush too, and vice versa. That coupling is invisible while there is one cadence. It
becomes wrong the moment there are two.

## The split, and the invariant it must preserve

`flush-accumulators` becomes **two paths, one per accumulator group**:

```
flush-logs      logs                      → write-logs,    reset logs
flush-metrics   counters + durations      → write-metrics, reset both
```

★ **Stone A's guarantee generalises rather than weakens.** It was *"one emit-and-reset path, so a
mid-life flush and `close` cannot disagree."* It becomes:

> **One emit-and-reset path PER ACCUMULATOR.** Each group's timer, its size trigger, and `close` all
> call that group's single path.

`close` calls both. Each size trigger calls only its own — which also fixes the pre-existing
cross-coupling above. **Two paths for the same accumulator is the double-count returning**, and it
would pass every row that does not flush that accumulator twice.

## ★ THE ARMING PROBLEM, and the answer that needs no new state

Who arms the first timer? Nothing can:

- `:init` returns a `State`, not an `Outcome` — it has no way to emit an `Alarm`.
- there is no "start" op on `Span`; the service is born from `span/start`, the constructor.

The proven pattern is in `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat`: *"if no flush
armed, ARM one."* That implies an armed-flag — and there is nowhere clean to put one. `:durable`
*"crosses the wire, survives hibernation"*, which a live timer does not; `:ephemeral` holds live
handles (`Peer`, `Lru` — `cache.wat:198`) and has no plain-value precedent in the corpus.

**So do not store it — derive it.** A flush is pending exactly when its accumulator is non-empty. So:

> **Arm on the empty→non-empty transition.** In `log`, if `logs` was empty *before* the `conj`, arm
> `-flush-logs`. In `incr`/`timed`, if both counters and durations were empty before, arm
> `-flush-metrics`.

Self-consistent by construction: the timer flushes and resets → the accumulator is empty again → the
next accumulation re-arms. **An idle span arms nothing and no timer ever fires for it.**

The one edge, and it is benign: a *size*-triggered flush empties the accumulator while its timer is
still armed, so the next accumulation arms a second one and the first fires on an empty accumulator.
That is a no-op — **stone A already gates it** (`second_flush_and_empty_close_emit_nothing`). A
spurious tick costs one wake; a stored flag costs a state field with no honest home and a hibernation
bug waiting in it.

## Cadence is configuration, not a constant

The two intervals live on the span's `Record` (`:durable`) beside `namespace` — they are part of what
the span IS, they should survive hibernation, and the builder asked for "30s or 60s or whatever".
Defaults chosen once, overridable per span at `span/start`.

## What is NOT changing

- `flush-accumulators`' emission and reset logic — it is **split**, not rewritten. Stone A's gates
  must pass untouched.
- The `flush` op stays: a forced flush of everything, which is what an operator wants.
- `close` stays flush-the-remainder — now of both groups.
- No new surface op, no `Journal` change, no runtime change.

## Out of scope = REJECTED

- Backpressure, bounded buffers, or a drop policy when a flush fails. Stone A left
  `LogResponse::Ok` meaning "buffered" and that contract question is the builder's, unruled.
- Jitter/backoff on the cadence. A fixed interval first; a proven need before a knob.
