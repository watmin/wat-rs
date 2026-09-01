# BRIEF — item (c) stone B: two clocks

Give the buffered span **two independent cadences** — logs flushed fast, counters and durations on a
slow beat — each timer re-arming itself. Stone A's flush path is split, not rewritten.

Read `DESIGN-STONE-the-buffered-span-timers.md` beside this first: it carries the split's invariant
and the arming answer, which is the part you would otherwise have to invent.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat:322`** — `flush-accumulators`. It flushes logs AND metrics together and
   returns `(State, CloseResponse)`. **This is what you split.** Its emission and reset logic is
   correct and gated; move it, do not rewrite it.
2. **`wat/telemetry/span.wat:94-98` and `:140-144`** — the two size triggers. Both call
   `flush-accumulators` today, so each currently flushes the OTHER group too. After the split each
   calls only its own — that is a bug fix riding along, not a scope creep.
3. **`wat-scripts/scratch-pad/probe-self-scheduling-loop.wat`** — the proven hand-rolled sink loop:
   buffer, arm a flush timer on first item, flush on tick. Green. The exemplar this transcribes.
4. **`tests/services/probe_arc278_self_scheduling.{rs,wat}`** — a real `defservice` arming a `-tick`
   that re-arms itself, green at BOTH loci. Copy its shape for the internal ops: leading dash =
   reactor-internal, `Outcome::NoReplyAndArm` to re-arm, `NoReply` to stop.
5. **`wat/service.wat:50-64`** — `Alarm` and `Outcome`. `arms` is a `Vector` and each `Alarm` carries
   its own `after` AND its own `op`, so two cadences need no new machinery.

## The work

**1. Split the flush.** `flush-logs` (logs) and `flush-metrics` (counters + durations). Each is the
ONE emit-and-reset path for its group. `close` calls both; each size trigger calls only its own.

**2. Two internal ops.** `-flush-logs` and `-flush-metrics`. Each flushes its own group and
re-arms itself at its own interval — unless its accumulator is empty, in which case it does NOT
re-arm (an idle span must go quiet, not tick forever).

**3. Arm on the empty→non-empty transition.** No armed-flag anywhere. In `log`: if `logs` was empty
BEFORE the `conj`, return `Outcome::ReplyAndArm` with a `-flush-logs` alarm. In `incr`/`timed`: if
counters AND durations were both empty before, arm `-flush-metrics`. See DESIGN for why a stored flag
has no honest home.

**4. Cadence on the Record.** Two `:durable` fields beside `namespace`, with defaults, overridable at
`span/start`.

## Blast radius

`wat/telemetry/span.wat`, and `wat/telemetry.wat` only if the Record's new fields force it. **No new
surface op. No `Journal` change. No runtime change.**

## STOP triggers

**STOP-1 — one path per accumulator.** If a group ends up with two emit-and-reset paths (say the
timer building its own metrics instead of calling `flush-metrics`), STOP. That is stone A's
double-count returning, and it passes every row that does not flush that group twice.

**STOP-2 — an idle span must arm nothing.** If a span that never logs and never counts still arms a
timer, STOP: that is a wake per span per interval for nothing, and it scales with span count.

**STOP-3 — no armed-flag in `:durable`.** `:durable` survives hibernation; a resurrected span would
believe a timer is armed that does not exist, and never re-arm. If you conclude the transition test
cannot work and a flag is required, STOP and report rather than putting it in `:durable`.

**STOP-4 — stone A's gates are not to be edited.** `probe_arc278_span_buffered`'s three tests and the
span surface/service/macros probes must pass UNCHANGED. If a stone-A gate needs editing to go green,
the split changed behaviour it was supposed to preserve — STOP and report which.

## The gates to write

- **two cadences, independently:** a span that only logs must flush logs and NOT metrics; a span that
  only counts must flush metrics and NOT logs. This is the whole point of the stone and it is what a
  single shared timer would silently pass today.
- **an idle span is silent:** no writes at all over several intervals.
- **the tick re-arms:** logs flushed at least twice from one span without a client-triggered flush.

Time-dependent tests: bound them on the OBSERVED count, never a sleep-then-assert. See
`probe_arc278_self_scheduling.wat`'s `poll-until` — it polls until the observed value reaches target
with a bounded attempt count, and its `nap` is a `select'` on a one-shot `after`, never a sleep.

## Prior comparable result

`SCORE-item-c-stone-a-buffered-span.md` beside this — the stone this builds on, and its delta section
records a specification error of mine that the strike was right to break.
