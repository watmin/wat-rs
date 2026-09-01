# DESIGN STONE — the sane circuit

**Commissioned 2026-09-01.** `circuit.wat` is both a capability proof and a perf check. It is sound
as the first and unusable as the second, because its consumers are not consumers.

## What it does today, verified by line

```
:320   Topic/publish        all 2000 messages, FIRST
:337   worker/start         workers spawned AFTER
:363   kick-drain
:138   (range 0 cap)        with :cap n = 2000
```

Twelve workers × 2000 fixed receive attempts = **24,000 polls for 8000 messages**, ~16,000 empty **by
construction**. A worker does not stop when the queue is empty and does not stop when the app says
so — it stops when it has counted to 2000.

Nothing operates this way. You do not fill a queue with no consumers, and a consumer does not have a
hardcoded iteration budget.

★ **It also makes the perf number meaningless in a specific way**: the run is dominated by empty
polls that no queue improvement can remove. Long polling measured as a *loss* here — 88.6 s → ~106 s
— because a fixture with no waiting in it has no waiting to make cheap. Three stones used this as
their witness (perf-2, perf-3, long polling); the first two got away with it only because store cost
dominated.

## The shape it should have

```
start workers        →  they begin consuming immediately, on empty queues
publish              →  concurrently with consumption, as a producer actually behaves
drain to completion  →  the app watches depth, not a counter it invented
stop the workers     →  Admin::Stop; each returns its tally via Status::Stopped
```

Every piece exists:

- **a consumer that runs until told to stop** — the worker becomes **self-scheduling**: each tick is
  one long-polled `receive`, process, ack, re-arm. Landed this morning, green at both loci.
- **a wait that is not a spin** — long polling. Landed tonight.
- **interruptibility** — because the work is a *tick*, the serve loop regains control between
  messages and can take `Admin::Stop`. A worker that looped internally could not be stopped; **the
  tick shape is what makes shutdown possible**, not a stylistic choice.
- **shutdown and tallies** — `Admin::Stop` and `Status::Stopped` carrying projected state (arc 291).

## ★ THE CONTRACT DECISION: the app stops on DEPTH, and depth needs both numbers

`stats` reports `receive-calls` and `ticks` — instrumentation, not depth. The app cannot ask *"is it
drained?"*, so it cannot know when stopping is safe.

> `stats` gains **pending** (visible) and **in-flight** (received, not yet acked).

Both, and the second is the load-bearing one. **Stopping a worker with a message in flight loses an
outcome**: the message is invisible until its visibility timeout, the run ends before that, and
`distinct` comes back under 8000. A drain condition of *pending = 0* alone is wrong; it must be
**pending = 0 AND in-flight = 0**.

This is exactly why SQS exposes `ApproximateNumberOfMessages` *and* `...NotVisible`. We arrive at the
same pair for the same reason.

## What must not change

`total=8000; distinct=8000; dup=0` is the proof, and it stays the proof. **A perf rewrite that
weakens the invariant has destroyed the thing it was speeding up.** The output string keeps its
shape so every earlier measurement remains comparable.

## Out of scope = REJECTED

- Any change to `Outcome`, `service.wat`, or the runtime.
- Queue semantics beyond adding depth to `stats`.
- Tuning wait durations or worker counts for a good number. Make it *sane*; chase perf after, against
  a fixture that can measure.
