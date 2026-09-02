# SCORE — D4, weighed against the orchestrator's own re-run

> Re-run at `78cc04b71` + the rider's uncommitted rune block. **The cure is not the one the strike
> prescribed — it is better, and the rider argued it rather than taking the sketch.**

## The scorecard

| # | pre-value at `35e0938cb` | after, MY re-run |
|---|---|---|
| 1 | ★ **(8,8) → (16,16) → (24,24)** across three panics | ✅ **arena 8 → 8 → 8 → 8**; probe green |
| 2 | nested: outer `sp=4`, inner observed 4 — the heap arm | ✅ `Err` arm still taken; a 64-slot inner frame does not grow a 4-slot arena |
| 3 | the false `:96` doc *"nested calls therefore stack"* | ✅ **gone**; replaced by `⛔ THERE IS NO STACK POINTER` stating what never happened |
| 4 | ⚠ `EXEC_SP`'s fate — inert, `start` always 0 | ✅ **DECIDED: DELETED**, with the induction written down — see A |
| 5 | TLS teardown | ✅ **no site** — no `Drop` was added, so `try_with` never arose. Said, not assumed |
| 6 | hot path | ✅ **strictly less work**: two TLS `Cell` accesses and one add REMOVED per call; nothing added |
| 7 | radius `eval.rs` + probe | ⚠ **+1 line of `mod.rs`** — see D |
| 8 | lints 196/196 | ✅ 196/196 |
| 9 | floor 5310/5310 | ✅ **`5312 tests run: 5312 passed, 21 skipped`**, exit=0 (5310 + the two probes) |
| 10 | clippy rc=0 | ✅ rc=0, silent |

## ⭐ A — THE CURE IS A DELETION, AND THE ARGUMENT IS THE DELIVERABLE

The strike's ★ said *"the arena pointer is restored by a `Drop`."* The rider did not build that. It
observed that a `SpGuard` would restore a `start` that is **provably `0`** — the only route to the
`Ok` arm is an unborrowed arena, and with the cursor always restored that means a cursor of `0` by
induction — so the guard would guard a constant, and the mechanism it guards computes nothing.

So the cursor is **deleted**. That is a higher rung of the same ladder: the guard *cures* the strand,
the deletion makes it **structurally impossible** — with no state carried across `f`, an unwind has
nothing to strand. It also dissolves the strike's own trap 2 (no `Drop`, therefore no destructor
touching a thread-local at teardown, therefore no `try_with` question) and is *less* work on the hot
path, not more.

**I weighed this by driving it, not by reading the argument.** Mutation 3 — reinstate the cursor
WITH a correct guard and `assert_eq!(start, 0)` on every entry to the `Ok` arm — held over
**1508/1508** across the whole rete surface. Then, because a passing assert can be an assert that
never runs, I flipped it to `assert_eq!(start, 999)`: **RED immediately, `left: 0, right: 999`, from
two different tests.** The assert executes and `start` is genuinely `0` every time. The deletion's
licence is measured, not argued.

## ⭐ B — THE FAILURE-EVEN-IF-GREEN TRAP WAS REAL, AND I DROVE IT

EXPECTATIONS named the trap: *a single-panic probe passes a partial fix.* Under **mutation 1** (the
defect restored verbatim) my own run REDs — and it REDs at **round 2**, not round 1:

```
round 2: a panic through `f` must strand nothing; arena went 8 -> 16
  left: 16
 right: 8
```

Round 1 passes on the bug. With the arena warmed to 8, the first panic strands *inside* a buffer
already big enough; only the second forces the resize. **A one-panic probe would have certified this
defect green.** The three-round requirement was the difference between a proof and a stamp.

Mutation 1 also shows what the second probe is worth: `a_nested_frame_takes_the_heap_arm…`
**PASSED under the defect.** It is a regression guard for the `Err` arm, not evidence of the cure —
and the rider said so itself rather than counting it as a discriminator.

## ⭐ C — A GATE FIRED FOR REAL, AND I CONFIRMED IT IS LOAD-BEARING

`wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves` caught the new
doc citing the now-deleted `EXEC_SP` — the gate this arc built, catching this arc's own strike. The
rider closed it with `rune:lint(cited-name-absent)` (option 4: the absence is the point), which is
the established form — ten prior uses across `src/rete/`.

**A suppression is a claim, so I tested it.** Removing the rune and running the lint binary:

```
FAIL (132/196) wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves
Summary [136.270s] 196 tests run: 195 passed, 1 failed, 0 skipped
🔥 1 name(s) cited in a comment under src/rete resolve to NOTHING
```

The rune is doing work. It is not decoration.

## ⛔ D — WHERE MY BRIEF WAS WRONG, and the sharpest one is a citation

The rider returned six thin spots. Three are mine to own:

1. **★ and ⚠ were mutually exclusive and my brief read them as sequential.** The ★ prescribed the
   guard; the ⚠ said the guard makes the cursor provably dead and listed deletion FIRST. Take
   deletion and the ★ *dissolves*. A rider treating ★ as the contract lands the **weaker** cure. The
   fix the rider proposes is right: the ★ should have been *"the frame carries no state across
   `f`"* — satisfied by either cure, pre-committing to neither.

2. **⛔ MY READ-LIST ITEM 3 CITED THE WRONG FILE.** I sent the rider to
   `src/rete/kernel/fire/rules.rs` for B1's `ArmLease` — *the worked reference I told it to copy*.
   That file contains **zero** occurrences of `ArmLease` and **zero** `impl Drop`. The real site is
   `src/rete/kernel/arm.rs:829` (`struct ArmLease`) and `:840` (`impl Drop`). Verified by grep just
   now. I named a real symbol at an address I never checked.

3. **Row 7's radius was short by one file, and it was a clippy hazard.** `EXEC_SP` was the only
   `Cell` user in `expr_ir`, so deleting it leaves `use std::cell::{Cell, RefCell}` unused in
   `mod.rs` — RED under `-D warnings`. True radius: `eval.rs` + one line of `mod.rs` + probe.

The other three are consequences of #1: traps 1 and 2 have no site under a deletion, and
EXPECTATIONS' mutation 2 and row 1 are written in guard-vocabulary (`SpGuard(end)`, "the pointer")
for a tree that now has no pointer. The rider ran mutation 2 by reintroducing the whole cursor, and
noted honestly that it therefore proves the probe reads **the leaked resource** (arena length), not
"the pointer".

## ⛔ E — THE RIDER'S OWN DISCLOSED MISTAKE

It lost the first `rete_citation_resolves` RED to a `| tail` and had to re-run the single test to
capture it whole. It reported this unprompted and named it as the truncating-pager failure
`CLAUDE.md` forbids. Disclosed, not discovered — which is the behaviour the tier depends on.

## Line counts

`eval.rs` **+124 / −15**, `mod.rs` **+1 / −1** (in `073546093`), plus the 6-line rune block. Of the
124, an 81-line probe module and a ~30-line doc section: **the mechanism itself got smaller.**

## Per-arm status

| arm | status |
|---|---|
| `Ok` / normal return | **proven** |
| `Ok` / unwind through `f` | **proven** — RED→green, three panics, driven by me under mutation 1 |
| `Err` heap arm | **proven driven, NOT a discriminator** — passes under the defect too |
| `resize` growth branch / no-resize branch | **proven** |
| TLS-teardown drop | **not reachable, and why**: no `Drop` was added; the only TLS destructor is the arena `Vec`'s own, which predates this strike |
