# HANDOFF — the sane circuit

`circuit.wat` publishes all 2000 messages *before* any worker starts (`:320` vs `:337`), and each of
the twelve workers then does exactly 2000 receive attempts (`:138`, `(range 0 cap)`, `:cap n`). That
is 24,000 polls for 8000 messages, ~16,000 empty by construction. A worker does not stop when the
queue is empty and does not stop when the app says so — it stops when it has counted to 2000.

You are making it a program that could exist.

Start here, in order:

1. `DESIGN-STONE-the-sane-circuit.md` — the shape it should have, and the contract decision.
2. `BRIEF-the-sane-circuit.md` — the rooms as exact `file:line`, four STOP triggers.
3. `tests/services/probe_arc278_self_scheduling.wat` — the worker's new shape: arm a tick, do one
   unit, re-arm.

Three things to hold:

**★ The drain condition needs BOTH depth numbers.** `pending = 0` alone is wrong: stopping a worker
that holds an unacked message loses that outcome, because the message stays invisible until its
visibility timeout and the run ends first. It must be `pending = 0 AND in-flight = 0`. This is why
SQS exposes two counters, and row 2 proves the term is load-bearing by *removing* it and requiring a
failure.

**The tick shape is what makes shutdown possible.** One long-polled receive per tick, then re-arm.
A worker that loops internally is faster to write and cannot take `Admin::Stop` — it will hang the
shutdown. Returning control to the serve loop between messages is the point, not a style.

**No fixed iteration counts, and no "safety" bound to stop a hang.** If something hangs, the
shutdown condition is wrong and that is the bug to fix. A bound converts it into a flaky
under-count, which is worse because it passes most of the time.

The invariant is not negotiable: `total=8000; distinct=8000; dup=0`. A perf rewrite that weakens the
proof has destroyed the thing it was speeding up.

Report the wall time; do not promise one. **A slower sane program is the deliverable** — perf gets
chased after, against a fixture that can actually measure.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-the-sane-circuit.md` when done. It will be graded by re-running.
