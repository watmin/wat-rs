# HANDOFF — excursus 002 stone 2

You are striking one stone: **the tail escape** — a `let` that creates a `Handle`, whose tail call
carries that service's `Peer` out while the scope, and the handle, die underneath it. This is the
shape that kept two tests `#[ignore]`d for 38 days under a diagnosis that was never measured.

Start here, in order:

1. `DESIGN-stone-2-tail-escape.md` — the closed seven-form tail table, the one contract decision,
   and a collision whose answer is the OPPOSITE of stone 1's.
2. `BRIEF-stone-2-tail-escape.md` — rooms as exact `file:line`, the rule, four STOP triggers.
3. `SCORE-stone-1-creation-scope-escape.md` — the previous stone in this excursus. Its Row 9
   section records a specification error in the brief that you were right to refuse; this brief is
   written to avoid repeating it.

Three things carry most of the risk:

**The runtime already defines tail position.** `eval_tail` (`src/runtime.rs:4360-4438`) is the
authority: seven forms, `if` `match` `let` `do` `and` `or` `ann-form`. Do not write a second list —
one shared constant, or a drift gate that fails when two disagree. A duplicated list is the one
failure here that nothing goes red for.

**Both conditions matter.** The scope dies only if the let's tail expression is a *user-function*
call AND the let is itself in tail position. Skipping the second is cheap and makes every non-tail
let a false positive.

**Rune the instrument, never the acceptance criterion.** `probe-self-sched-bisect.wat` must keep
RUNNING — it prints the discrimination table that is this excursus's evidence — so its three
deliberate escapes get runes. `probes/red-tail-escape.wat`, which you write first, must stay
rejected: runing it produces a green floor from a wall that catches nothing.

The census here cannot be a grep — this shape lives in the AST, not the text. Build the wall, run
`--check` across the corpus, and read what it rejects. Anything outside the expected list is a
finding to report, not a nuisance to silence.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run it, name the exact arm, surface it.

When you are done, write `SCORE-stone-2-tail-escape.md` beside these: each EXPECTATIONS row's real
result, the honest deltas, the line counts. It will be graded by re-running, not by reading.
