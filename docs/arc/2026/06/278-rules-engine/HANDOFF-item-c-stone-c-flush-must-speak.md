# HANDOFF — item (c) stone C: a size-triggered flush must speak

Three arms in `wat/telemetry/span.wat` — `log`, `incr`, `timed` — compute a write failure and throw
it away. `pair0` holds `(state, outcome)`; they read `(first pair0)` and never `(second pair0)`. So a
sink returning `Fatal` to a size-triggered flush is swallowed and the caller is told `Ok`.

Start here, in order:

1. `DESIGN-STONE-the-size-triggered-flush-must-speak.md` — the ruling against two success values, and
   the subtlety below.
2. `BRIEF-item-c-stone-c-flush-must-speak.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat/telemetry/span.wat:170` and `:179` — `flush` and `close` already read `(second pair)` and map
   it. Copy that mapping; do not invent one.

Three things to hold:

**The flush path is already correct — do not touch it.** `flush-logs`/`flush-metrics` reset only on
success and return the original state, buffer intact, on failure. That is why no data is lost today
and why this stone is three arms and three enums.

**★ Keep the item that arrived.** The log or sample that triggered the flush is not part of the failed
batch — it is the one that made the batch too big. The failure response must be returned WITH the
accumulated state. If the failure path drops it, this fix trades a silent failure for silent data
loss, which is worse than the bug.

**No `_` wildcard on a response enum.** Adding variants makes existing matches non-exhaustive and the
checker will name each site — two `.wat` files, by census. Add real arms. A wildcard restores exactly
the swallow this removes.

`Ok` keeps meaning "accepted". A second success value (`Buffered`) was four-questioned and rejected:
a caller cannot act differently on it, and it names durability while leaving the failure on the floor.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-item-c-stone-c-flush-must-speak.md` when done. It will be graded by re-running.
