# EXPECTATIONS — the server drops a reply

Written **before** the strike.

| # | what | expected |
|---|---|---|
| 0 | ⛔ **the first act** | is `state` in scope at the five send sites? **Answer before building** |
| 1 | ★★ **`seen-dups > 0`** | the number that has never moved outside a deterministic gate. Any non-zero |
| 2 | ★★ **the placement discriminates** | drop **before** the ledger write → `seen-dups = 0`; **after** → `> 0`. Same rate, same seed, one variable |
| 3 | ★★ **`distinct = 8000`** | ⛔ below is **loss** — the only genuine failure here, and the finding worth the stone |
| 4 | ★ rate 0 unchanged | floor `5214/5214`, `seen-dups=0`, no draw taken |
| 5 | the seed replays | two runs, same seed, same `seen-dups` |
| 6 | scope | the drop only on `Seen/claim`; no eviction; no `src/` |

## ⛔ ROW 2 IS THE TABLE, MEASURED

The tracker has asserted since the arc opened that *before dispatch* produces no duplicate and
*after the arm* produces one. 3c measured the first half: **`seen-dups = 0` under 24 severs**, with
the mechanism named in advance (*arms run to completion; an alarm fires between them*).

Row 2 measures the second half **with one variable**. If both cells agree, the placement was never
the variable and the table is unproven — that is a finding, not a pass.

## ⛔ AND ROW 3 IS WHERE I EXPECT TROUBLE

T1's retry lands on a **fresh peer** and re-sends the claim. If the ledger already recorded the seq,
the retry gets `Dup` → `first? = false` → **no outcome emitted by either attempt**, and `distinct`
drops.

That is the stranding I predicted for 3d and never got to run. **If it appears here, it is the
finding** — a real defect in claim-before-emit, surfaced by the first fault that can reach it.
Report it with the mechanism. **Do not repair it in this stone**; a fix whose failure was never
observed is what this arc has spent eleven stones learning not to ship.

If `distinct` holds at 8000, say what absorbed it — that is more interesting than the prediction
holding.

## RUNTIME PREDICTION

**60–120 minutes**, most of it the first act and the seed threading. The drop itself is a few lines.

## TRAP-DOOR RISKS

1. **The drop must return `true`.** `false` is the world stopping.
2. **Only `Seen/claim` has a deadline.** A drop on any other call is a hang, not a fault.
3. **Threading the seed through the drop branch** — 3c's trap: dropping the advanced seed makes every
   draw identical and the rate wrong, while row 5 passes for the wrong reason.
4. **`wat/service.wat` is stdlib and every service expands through it.** Rate 0 must draw nothing.
5. **The four `peers_bijection` goldens** snapshot lines 896-934. Edit inside the helper at `:3108`
   and nothing shifts; insert above and they red.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 0 unanswered, or answered by assumption.
- Row 2 with one placement, or with the two cells at different seeds.
- `:wat::rand::int` used anywhere — replay is the requirement.
- `distinct < 8000` repaired rather than reported.
- Rate 0 drawing.
