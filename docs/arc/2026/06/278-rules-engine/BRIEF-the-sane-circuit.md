# BRIEF — the sane circuit

Make `circuit.wat` a program that could exist: consumers that consume until stopped, a producer that
runs alongside them, and a shutdown driven by queue depth rather than a hardcoded count. It stays the
composition proof; it becomes usable as a perf witness.

Read `DESIGN-STONE-the-sane-circuit.md` beside this first — the contract decision is that the drain
condition needs **two** depth numbers, and getting it wrong loses messages rather than time.

## Read in order, and why you are being sent there

1. **`wat-scripts/fanout/circuit.wat:138`** — `(range 0 cap)` with `:cap n = 2000`, and `:320`/`:337`
   (publish before workers start). **This is what you are replacing**, and the line numbers are the
   evidence it is a batch job rather than a consumer.
2. **`tests/services/probe_arc278_self_scheduling.wat`** — a service that arms `-tick`, does one
   unit, and re-arms. **The worker's new shape.** Note it is interruptible *because* it is a tick.
3. **`wat-scripts/queue/sqs.wat`** — `receive` with `wait-ns` (tonight), and `stats`, which reports
   `receive-calls`/`ticks` and must gain depth.
4. **`wat/service.wat`, the Admin arms** — `Admin::Stop` → `Status::Stopped` with the projected
   state. **That is how a worker returns its tally**; you are not inventing a reporting channel.

## The work

**1. `stats` gains `pending` and `in-flight`.** Visible messages, and received-but-unacked.

**2. The worker becomes self-scheduling.** One tick = one long-polled `receive` (limit > 1), process,
ack, re-arm. No `(range 0 cap)`, no `done` flag, no fixed budget.

**3. Main reorders**: start workers → publish → poll `stats` until **pending = 0 AND in-flight = 0**
across every queue → `Admin::Stop` each worker → collect tallies from `Status::Stopped`.

**4. The output string keeps its shape** so every earlier measurement stays comparable.

## Blast radius

`wat-scripts/fanout/circuit.wat` and `wat-scripts/queue/sqs.wat` (stats only). **No `service.wat`, no
`Outcome`, no runtime, no stdlib.**

## STOP triggers

**STOP-1 — the invariant is not negotiable.** `total=8000; distinct=8000; dup=0`. A perf rewrite that
weakens the proof has destroyed the thing it was speeding up. If you cannot keep it, STOP.

**STOP-2 — in-flight must be in the drain condition.** Stopping a worker holding an unacked message
loses an outcome: it stays invisible until its visibility timeout and the run ends first. `pending =
0` alone is a silent under-count. Both numbers, or STOP.

**STOP-3 — no fixed iteration counts anywhere.** Not in the worker, not in the drain poll, not as a
"safety" bound. If you need a bound to stop something hanging, the shutdown condition is wrong and
that is the bug.

**STOP-4 — the worker must be interruptible.** One unit per tick. A worker that loops internally
cannot take `Admin::Stop` and will hang the shutdown; a tick returns control to the serve loop
between messages.

## The gates to write

- **★ the invariant holds** — `total=8000; distinct=8000; dup=0`, unchanged.
- **★ no message is lost at shutdown** — stop while work is in flight and prove the drain condition
  prevented it: with the in-flight term removed, this must FAIL.
- **workers consume concurrently with the producer** — participation is not the 4-of-12 collapse the
  `done` flag caused.
- **empty polls are gone** — count receive calls; it should approach the message count, not 3× it.
- **the wall time** — **reported, not promised**, against 88.6 s. It may be worse and still be right:
  a sane program is the deliverable, and perf is chased after.

## Prior comparable result

`SCORE-queue-long-poll.md` — where using this fixture as a perf witness produced a measured "loss"
for a feature that was working correctly, and the decomposition that showed why.
