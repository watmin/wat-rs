# EXPECTATIONS — what does publish actually cost

Written before the strike. Every "before" is a run of mine on `30fe0b9f0`, quiet box.

| # | what | expected |
|---|---|---|
| 1 | ★★ **five unit costs, one run** | µs/op for: round-trip thread, round-trip process, `Store/put`, `count-index`, `scan-index limit 1` |
| 2 | ★★ **the arithmetic, stated** | `predicted`, `actual = 37300 ms`, and **`gap`** — reported whatever it is |
| 3 | ★ **the interpretation floor** | the **thread**-locus round trip, called out explicitly: this is the number the arc's terminal condition is measured against |
| 4 | ⛔ blast radius | `git diff --stat` → `wat-scripts/scratch-pad/` **only** |
| 5 | ⛔ the box was quiet | `ps` checked and reported before each timing loop |
| 6 | floor untouched | `5215 passed`, 22 skipped — a probe cannot change it, and the run proves the tree was not disturbed |
| 7 | ★ **the next target, named** | from the units and the gap: **which single term should the next stone attack**, with its share of publish |

### Before-state, recorded verbatim

```
row 2   publish median 37318 ms (mine: 37318 37769 37409 37095 36364)
        count-index 467 us  ->  8000 x 467us = 3.7 s  = 11% of publish
        UNATTRIBUTED ~33.6 s
row 1   LIMIT65 1350 us | LIMIT1 504 us | COUNT 467 us   (put: NEVER MEASURED)
row 6   Summary [375.530s] 5215 passed, 22 skipped  .floor/2026-09-05T23-43-25Z/
```

## ⛔ ROW 2 IS THE STONE, AND THE GAP IS THE DELIVERABLE

This stone does not optimize. **It either closes the arithmetic or names where the model is
blind**, and both are wins. A SCORE that reports five numbers without attempting the sum has not
struck.

## ⛔ ROW 3 IS WHY THIS MATTERS BEYOND ONE STAGE

The builder's terminal condition is *stop when wat's interpretation overhead is the dominant
term*. **A bare thread-locus round trip is the closest thing we have to that number.** Every
client call in the tree pays it. Report it plainly — it bounds every optimization that follows.

## RUNTIME PREDICTION

45–75 min. Five loops in one harness, plus the two-loci split. No rebuild needed for a
scratch-pad probe.

## TRAP DOORS

1. **A "bare" round trip that isn't.** If the arm allocates, formats, or touches state, the
   number is not the floor. STOP-1.
2. **Timing a cold path.** Warm up; the existing harness already does.
3. **Fitting the model to the answer.** STOP-3 — if the call count per publish is not 2, say so
   and use the real one.
4. **Reporting units without the sum.** Row 2.
