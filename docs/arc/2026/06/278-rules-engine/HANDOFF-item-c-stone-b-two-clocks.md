# HANDOFF — item (c) stone B: two clocks

You are giving the buffered span **two independent cadences** — logs fast, counters and durations on
a slow beat — each timer re-arming itself. Stone A's flush path is **split, not rewritten**.

Start here, in order:

1. `DESIGN-STONE-the-buffered-span-timers.md` — the split's invariant and the arming answer.
2. `BRIEF-item-c-stone-b-two-clocks.md` — the rooms as exact `file:line`, four STOP triggers.
3. `SCORE-item-c-stone-a-buffered-span.md` — the stone you are building on.

Three things carry the risk:

**One emit-and-reset path PER ACCUMULATOR.** Stone A's guarantee was "one path, so a mid-life flush
and `close` cannot disagree". Splitting keeps it only if each group still has exactly one builder,
called by its timer, its size trigger, and `close`. A timer that builds its own metrics is stone A's
double-count coming back through the split — and it passes every gate that does not flush that group
twice.

**Arm on the empty→non-empty transition; store no flag.** `:init` cannot arm (it returns a `State`,
not an `Outcome`), and an armed-flag has no honest home — `:durable` survives hibernation and would
resurrect a span believing in a timer that does not exist. A flush is pending exactly when its
accumulator is non-empty, so arm when it becomes non-empty. The timer resets it, so the next
accumulation re-arms. An idle span arms nothing.

**Time arrives as I/O.** Bound the new gates on the OBSERVED count, never a sleep-then-assert — see
`probe_arc278_self_scheduling.wat`'s `poll-until`, which polls until the observed value reaches
target and whose `nap` is a `select'` on a one-shot `after`. A sleep is a guess, guesses race, and a
flaky floor arm is the worst thing you can ship here.

Do not edit stone A's gates. If one needs editing to go green, the split changed behaviour it was
meant to preserve — STOP and report which.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-item-c-stone-b-two-clocks.md` when done. It will be graded by re-running.
