# HANDOFF — excursus 002 stone 3

You are closing the last road: a handle created by the CALLER as a temporary argument, whose CALLEE
takes it as a param and tail-escapes a peer of it. Measured open — it type-checks clean under both
existing walls and severs at runtime.

Start here, in order:

1. `DESIGN-stone-3-param-ownership.md` — the direction table, the contract decision, its honest cost.
2. `BRIEF-stone-3-param-ownership.md` — the rooms, the rule, four STOP triggers.
3. `SCORE-stone-2-tail-escape.md` — the previous stone. Its "Not done" section names this road.

The one thing that carries the risk: **direction.** A `Handle` param is a BORROW when a peer is
returned upward (`conn(h)` — the caller owns the handle and outlives the call) and an OWNER when a
peer goes downward into a tail call (this frame dies before the callee runs). Widen the downward
direction only. Widening both rejects every `conn` helper in the corpus, three of them in the
stdlib — that error was caught by the corpus before stone 1 shipped and must not come back.

This stone is deliberately conservative: it will reject a callee that tail-escapes a peer of a param
handle even where the caller still holds it and the program is safe. That trade is stated on
purpose — but **if live code hits it, STOP and report rather than rune.** If the shape is common the
trade is wrong and this stone should not ship as drawn. A wall that makes correct programs
unwriteable is worse than the hole it closes.

Write `probes/red-param-tail-escape.wat` FIRST, with all three shapes — the rejection and the two
that must keep compiling. In `probes/`, never `wat-scripts/`, and no rune on it.

Do not touch `src/runtime.rs`. TCO is not the defect: the same severing reproduces with no tail call
at all, through an ordinary function return.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-stone-3-param-ownership.md` when done. It will be graded by re-running.
