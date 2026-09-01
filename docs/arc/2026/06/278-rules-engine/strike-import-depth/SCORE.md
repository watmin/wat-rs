# SCORE — A6, weighed against the orchestrator's own re-run

> Re-run here at `bb0256e38`. The rider's report is cited only where it reports something I
> cannot reconstruct.

## The scorecard, re-run

| # | expected | actual |
|---|---|---|
| 1 | controls green before | ✅ the rider's equivalent: threading present, wall inert at `u32::MAX - 1` → 423/423. Behaviourally HEAD, and it isolates the threading from the refusal |
| 2 | `:and` probe RED before | ✅ RED **by acceptance** — "IMPORT ACCEPTED A 308-DEEP :and TOWER" |
| 3 | `:user`-cycle probe RED before | ✅ RED by acceptance |
| 4 | both GREEN after | ✅ — and **four** probes, not two (see below) |
| 5 | refusal is a value, not a death | ✅ each probe matches the `malformed` refusal naming `MAX_IMPORT_DEPTH` |
| 6 | the bound was measured | ✅ **3** (corpus max, instrumented) and **3,000–5,000** (the abort window) are both written at the constant, with the caveat that the corpus is a floor, not a ceiling |
| 7 | the budget is shared | ✅ **five** functions each open with `deeper(depth, span)?` — `unpack_expr`, `unpack_prog`, `unpack_pat`, `unpack_cond_op`, `unpack_driver` |
| 8 | no instrument left behind | ✅ `git diff` mentions instrumenting only in the constant's doc |
| 9 | blast radius | ✅ exactly the two named files |
| 10 | floor ≥ 5,191 + every arm | ✅ `Summary [ 405.409s] 5195 tests run: 5195 passed (1 slow), 21 skipped`, **zero FAIL rows** |
| 11 | clippy | ✅ rc=0, zero warnings |

## The mutation I re-drove myself

**Budget in `unpack_expr` only** — removed `deeper` from the other four, rebuilt:

```
Summary  4 tests run: 1 passed, 3 failed
  PASS  import_refuses_an_and_tower_past_the_depth_bound          ← the naive fix looks correct
  FAIL  import_refuses_a_user_prog_cycle_tower_past_the_depth_bound
  FAIL  import_refuses_a_driver_tower_past_the_depth_bound
  FAIL  import_refuses_a_pattern_tower_past_the_depth_bound
```

**This is the whole strike in one table.** The obvious fix — a counter on the function the finding
names — passes the obvious probe and leaves three towers open. The `:user` probe is sized for
exactly this: 158 layers = 159 expr frames (under 300, so an expr-only budget accepts) but 318
total frames (over 300, so only the shared budget refuses).

The rider's other mutation, **bound to 1 → 20 of 427 fail**, confirms the wall is on the path every
import takes rather than passing on a technicality.

## ⛔ Where MY brief was thin — and one of these is the strike's best finding

- **A. ★ I named ONE tower and there were THREE.** My BRIEF item 4 listed `:1357/:1401/:1452/:1512`
  as "four more *entries into* `unpack_prog`". Two of those live inside functions that are
  **themselves self-recursive**, which I never checked: `unpack_driver` (`:and`/`:or`/`:not`/
  `:exists`) and `unpack_cond_op` (`:or-c`/`:not-c`). A budget threaded only through the functions
  I named would have left both alive — and my own `:and` probe would have gone green over them.
- **B. ★★ `unpack_driver`'s doc comment stated the defect as a feature, and that is why I walked
  past it.** Verbatim at HEAD: *"a driver tree of any depth round-trips **without a depth
  parameter** — the wire's nesting IS the recursion."* Every word true; it is also the exact
  statement of the vulnerability. An accurate comment in the wrong register is a defect's alibi —
  nothing drifts, so nothing checks it, and it reads as settled. Promoted to memory.
- **C. Only one of my four "entries" genuinely threads.** `:1401` (fold), `:1452` (rhs-op) and
  `:1512` (rhs) are not reachable from inside the descent, so a fresh `0` is correct there. My
  phrase "need the same budget" implied they should continue an existing one. Wrong.
- **D. Trap 4 and rows 2/3 are in tension, and I did not sequence them.** The probes need concrete
  depths derived from a bound that does not exist until the measurement is done — but the rows need
  those probes RED *before* the wall exists. The rider resolved it: instrument build (wall inert) →
  measure → add probes (wall still inert) → capture RED → turn the wall on. **Put that ordering in
  the next brief that measures its own constant**, or the executor measures, fixes, and then has no
  honest RED to show.
- **E. Row 1 as written could not be run.** "Controls green before" at pristine HEAD is not the
  interesting control; the useful one is threading-present-wall-inert, which also proves the
  threading alone broke nothing. The rider substituted it and said so.

## The finding I did not ask for

**The corpus max nesting depth is 3.** Every packed program in 423 tests bottoms out at
`unpack_prog` → `unpack_expr` → one operand. So trap 6 ("if the bound breaks a test, the bound is
wrong") could not have fired at any bound above 3 — **the corpus was never going to constrain this
number**, and the only real constraint is the 3,000 abort floor. The export/import corpus is broad
in *variants* and flat in *depth*; that is a coverage finding about export/import, standing open.

## Arms not driven, named

None. All four probes were driven RED→GREEN, each RED by *acceptance* rather than by some other
refusal answering in the depth wall's place — the rider checked each message.

`check_expr_slots` / `check_pat_slots` (STOP-3): **verified, not assumed** — private to
`export.rs`, reachable only from `check_program_slots` (sole caller `unpack_prog`) and
`check_cond_ops` (sole caller `unpack_compiled_cond`). Every tree they walk came through the
bounded unpack. **No second door.**
