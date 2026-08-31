# BRIEF — excursus 002 stone 3: a param is an owning binding, downward

Close the last road: a handle created by the CALLER as a temporary argument, whose CALLEE takes it
as a param and tail-escapes a peer of it. This is a **widening of stone 2**, not a third wall — the
one change is that a `Handle`-typed parameter counts as an owning binding for the DOWNWARD (tail
call) escape.

Read `DESIGN-stone-3-param-ownership.md` beside this first. It carries the direction table, the one
contract decision, and the honest cost of the trade.

## Read in order, and why

1. **`DESIGN-stone-3-param-ownership.md`** — specifically the direction table. Upward and downward
   have DIFFERENT answers for a param, and getting that backwards breaks either `conn(h)` (if you
   widen upward too) or road 3 (if you widen neither).
2. **`src/check.rs`** — stone 2's `HandleTailEscape` and its "let that creates a Handle of S" test.
   That predicate is what widens. Everything else about stone 2 stays as it is.
3. **`SCORE-stone-2-tail-escape.md`** beside this — its "Not done" section names exactly this road
   and why neither wall reaches it.
4. **`probes/red-tail-escape.wat`** — the shape and register for the acceptance criterion you will
   write, and where it must live.

## The rule

At a scope that OWNS a `Handle` of service S — owning means **either** it called `/start` **or** it
has a `Handle`-typed parameter — if that scope is in tail position and its tail expression is a
user-function call taking a `Peer` of S, reject.

Stone 1 is **not** touched. Widening the upward direction rejects every `conn` helper in the corpus,
including three in the stdlib.

## Write the acceptance criterion FIRST

`probes/red-param-tail-escape.wat`, modelled on `red-tail-escape.wat` beside it. Self-contained, and
it must carry **three** shapes, because two of them prove the rule did not over-fire:

- `:red::drive-param` — handle as a param, tail-escapes a peer → **must be REJECTED**
- `:red::conn` — handle as a param, RETURNS a peer (upward) → **must KEEP compiling**
- `:red::held-param` — handle as a param, drive in a BINDING not the tail → **must KEEP compiling**

`probes/`, never `wat-scripts/` — that tree's loader gate type-checks every file. No rune on it.

## Blast radius

`src/check.rs` only, plus the `CheckErrorKind` file if you mint a distinct variant (reusing
`HandleTailEscape` is acceptable — state which and why). **No runtime change. No change to stone 1.**

## The census must be RUN, and it is the row that can STOP this stone

Build, then `--check` the corpus. Expected rejections: the new red probe, plus the already-runed
instruments. **This stone is deliberately conservative** — it rejects a callee that tail-escapes a
peer of a param handle even when the caller still holds it and the program is safe.

**If live code hits that false-positive shape, STOP and report it.** If it is common, the trade is
wrong and this stone should not ship as drawn. A wall that makes correct programs unwriteable is
worse than the hole it closes.

## STOP triggers

**STOP-1** — if widening the ownership predicate also fires on stone 1's upward direction, STOP.
`conn(h)` must keep compiling; if it does not, the widening leaked across directions.

**STOP-2** — if any live corpus code (not a probe, not an instrument) is rejected, STOP and report
the site and its shape. That is the finding that decides whether this stone ships.

**STOP-3** — rune the INSTRUMENT, never the ACCEPTANCE CRITERION. `probe-self-sched-bisect.wat`
contains `:sched::hold-as-param`, which is this exact road and must keep RUNNING; it gets a rune.
`probes/red-param-tail-escape.wat` must stay rejected.

**STOP-4** — no runtime change. TCO is not the defect: the same severing reproduces with no tail
call at all (`:sev::dial-and-drop`, an ordinary return). If you find yourself in `src/runtime.rs`,
STOP.

## Prior comparable result

`SCORE-stone-2-tail-escape.md` — same excursus, same machinery, and its Row 7 shows the standard for
a contract decision that could rot silently.
