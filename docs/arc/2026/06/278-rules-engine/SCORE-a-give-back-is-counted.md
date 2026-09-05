# SCORE — a give-back is counted

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat-scripts/fanout/circuit.wat` only (`69/33`). No `.rs`.

```
Summary [ 363.920s] 5214 tests run: 5214 passed (3 slow), 20 skipped
```

`.floor/2026-09-05T09-53-56Z/`

## THE COUNTER

`gave-back` is a fourth field on `DisruptsResponse::Ok`, a durable on `:fanout::worker`,
init 0 via `mk-worker`. Incremented in the give-back arm **and only there**. Surfaced on
the existing `disrupts` channel. Printed on the tiny summary and beside `disrupts=` on
phases. `held-worker`'s stub is `(Ok 0 0 "" 0)` — no give-back path, so zero.

There is no `:wat::core::fourth`. The fold accumulator was widened from the 3-tuple
`(q, seen, outs)` to `((q, seen, outs), delta)`. `+ 1` lives in the exhausted-`a3` arm;
`PeerGone` and `Answered` pass `delta` through. The Record is written after the fold
when `delta > 0`.

## ROW 1 — twelve runs, every one

check-drop (`:user::drop-check-tiny` / `drop_check_tiny`) ×12:

| run | gave-back | total | distinct | dup |
|---|---|---|---|---|
| 1 | 0 | 100 | 100 | 0 |
| 2 | 0 | 100 | 100 | 0 |
| 3 | 0 | 100 | 100 | 0 |
| 4 | 0 | 100 | 100 | 0 |
| 5 | 0 | 100 | 100 | 0 |
| 6 | 0 | 100 | 100 | 0 |
| 7 | 0 | 100 | 100 | 0 |
| **8** | **1** | **100** | **100** | **0** |
| 9 | 0 | 100 | 100 | 0 |
| 10 | 0 | 100 | 100 | 0 |
| 11 | 0 | 100 | 100 | 0 |
| 12 | 0 | 100 | 100 | 0 |

**1/12** fired. Same rarity as the crash (~1 in 6). Rate not tuned.

## ROW 2 — the row we could not write before

Run 8: `gave-back=1` and `total=100; distinct=100; dup=0`. The path ran, and it lost
nothing.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ counter fires, ×12 | ✅ **1/12** `gave-back=1` (run 8); the rest 0 |
| 2 | ★★ when `gave-back > 0`, lossless | ✅ run 8: `total=100; distinct=100; dup=0` |
| 3 | ⛔ rate-0, no give-backs | ✅ `gave-back=0` ×5 |
| 4 | mark-drop unaffected | ✅ 6/6, `total=100; distinct=100; dup=0; gave-back=0` |
| 5 | rate-0 invariant | ✅ `total=8000; distinct=8000; dup=0; seen-recorded=8000` ×5 |
| 6 | the floor | ✅ **5214/5214, 20 skipped** |
| 7 | blast radius | ✅ `circuit.wat` only |
| 8 | timings | report only: publish **46161 45912 46569 47257 45426** (before `45771–47039`) |

Mark-drop `gave-back=0` ×6 is the STOP-2 control: dropped marks are not give-backs.

## NOT TOUCHED

Drop rate, retry budget, queue-side knobs, redelivery fixture, perf.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me, and rows 1–2 landed **stronger** on my runs
than on the executor's.

| # | my result | |
|---|---|---|
| 1 | check-drop ×12: **3/12 fired** — runs 04, 07, 12 at `gave-back` = 1, 1, **2**. Four events | ✅ |
| 2 | **all three of those runs**: `total=100; distinct=100; dup=0` | ✅ |
| 3 | rate-0 ×5: `gave-back=0` all five | ✅ |
| 4 | mark-drop ×6: 6/6, `total=100; distinct=100; dup=0`, **`gave-back=0` ×6** | ✅ |
| 5 | rate-0 ×5: `total=8000; distinct=8000; dup=0` | ✅ |
| 6 | `Summary [ 360.684s] 5214 passed, 20 skipped` — `.floor/2026-09-05T10-09-46Z/` | ✅ |
| 7 | `circuit.wat` only | ✅ |
| 8 | publish `45984 46063 46171 46298 46672` vs before `45771–47039` — reported, **not gated** | ✅ |

★★ **Row 2 is the row this arc has been unable to write, and it is now verified on four
give-back events across three independent runs of mine** — where the executor caught one. *A
give-back loses nothing* has stopped being an argument and become a measurement.

Combined across both of us: **5 events in 24 runs**, consistent with the ~1-in-6 exhaustion rate
measured before the fix. The rate was not tuned (STOP-3 held).

## ROW 4 IS THE NON-VACUITY CONTROL, AND IT DISCRIMINATES

`gave-back = 0` on **all six** mark-drop runs. If the counter counted more than its name —
`PeerGone`, a clean check, a retry — dropped marks would move it. They do not. **STOP-2 holds by
measurement, not by inspection**, and the counter means exactly the one thing it is named for.

Verified structurally too: `(:wat::i64::+ gb0 1)` appears **exactly once** in the file, in the
exhausted-`a3` arm (`circuit.wat:488`); every other path threads `gb0` unchanged.

## THE DEVIATION WAS ANTICIPATED AND IS REAL

**There is no `:wat::core::fourth`** — I checked rather than repeat it: `grep` for it across
`wat/` returns nothing. So the fold's 3-tuple could not simply grow, and the accumulator became
`((q, seen, outs), delta)`. The BRIEF allowed this and asked for it to be said; it was.

★ Minor substrate observation, not a stone: tuple accessors stop at `third`, so any fold needing
a fourth carried value nests instead of widening. Recorded in passing — it cost this stone
nothing, and it will cost the next one the same nesting.

## ★ WHAT THIS CLOSES

**The consumer path now has no unobservable events.** Every fault the circuit can suffer is
counted and printed: severs (`disrupts`), redeliveries absorbed (`seen-skipped`), receipts
written (`seen-recorded`), give-backs (`gave-back`), and the delivery invariants
(`total`/`distinct`/`dup`).

Three stones ago the crash read `0/6` because the injector looked away. Two stones ago the fix
read green because nothing counted it. **Neither can happen again in this path** — which is the
only durable defence against the failure mode that has cost this campaign the most.

## NEXT, IN ORDER

1. Queue-side drop knobs (`ack`, `receive`) — the rest of the coverage gap.
2. The redelivery fixture that kept its name and lost its meaning.
3. Rung 3: an undeadlined generated client method should have no form.
4. Then perf, starting with the send-path double scan.
