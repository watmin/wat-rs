# HANDOFF — long polling in `wat-queue`

You are letting `receive` **wait** for a message instead of returning empty. A worker currently
spends a full network round-trip (154 µs at process locus, measured) to be told "nothing yet", and
`circuit.wat` does that on every iteration of a fixed-size fold.

Start here, in order:

1. `DESIGN-STONE-the-queue-long-poll.md` — why now, and the contract decision.
2. `BRIEF-queue-long-poll.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat-tests/service-deferred-reply.wat` — the worked exemplar for `ReplyTo`, including a **timer**
   waking a client at both loci. That capability landed this evening; the queue could not have had
   long polling before today.

Three things to hold:

**★ ONE receive path.** The `send` arm and the expiry tick must call the *same function* the `receive`
arm calls — same `scan-index`, same visibility re-put, same envelope shape. If the wake path builds
its own reply the two drift, and they drift silently, because every existing queue gate covers the
immediate path and none covers the wake path. This is the same discipline that made the telemetry
stones hold: one path, so the two callers cannot disagree.

**`wait-ns = 0` is byte-identical to today.** Every existing queue gate must pass unedited. That is
the evidence long polling is additive rather than a rewrite — if a gate needs editing, something
moved that should not have.

**Waiters are `:ephemeral`, and an idle queue never wakes.** A `conn-id` does not survive a fork or a
resume, so a persisted waiter is a promise to a client that cannot exist. Arm the expiry tick on the
**empty→non-empty transition** of the waiter set and re-arm only while waiters remain — the span's
pattern (`wat/telemetry/span.wat`), with no stored flag. One scanning tick, not one timer per waiter:
an internal arm is `[s ctx]` with no request, so a fired timer cannot say which waiter expired. It
does not need to — `ctx`'s `start-ns` is a fresh clock read stamped by the serve loop.

Then have the circuit's worker ask for `wait-ns > 0` and `limit > 1`, re-run it, and **report the
wall time — do not promise one.** The stone two back predicted a circuit number from the wrong
measurement and was wrong.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-queue-long-poll.md` when done. It will be graded by re-running, and by re-running the
circuit.
