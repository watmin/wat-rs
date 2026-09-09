# EXPECTATIONS — stop fetching rows to get a number

Written before the strike. Every "before" is a run of mine on `c78c5463e`, quiet box.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the send path makes ONE store call** | `sqs.wat:293` calls `total`, not `depth`; same at `:472`, `:502`. `stats` still calls `depth` |
| 2 | ★★ **the call returns no rows** | `count-index` in both stores returns a count; **no row vector is built** |
| 3 | ⛔ **correctness holds** | floor **`5215 passed`, 22 skipped** |
| 4 | ⛔ **the numbers still agree** | `total == visible + unacked` — `probe-depth-derived-from-the-index.wat` still `agree=yes`; `probe-stats-sees-an-expired-unacked.wat` unchanged |
| 5 | ★★ **publish improves, measured in two steps** | after change 1: report. after change 2: report. **Median ≤ 42000 ms** (before: median **51300**) |
| 6 | unit cost | re-run `probe-what-a-scan-costs.wat`: report `count-index` µs vs the `1336` / `448` baselines |
| 7 | chaos unaffected | check/mark/recv/ack-drop ×3 each: `total=100; distinct=100; dup=0` |
| 8 | rate-0 invariant | circuit ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000` |
| 9 | ★ **the new dominant term** | from the stage histograms, **name what is now the largest contributor**, with its numbers |

### Before-state, recorded verbatim

```
row 3  Summary [366.205s] 5215 tests run: 5215 passed, 22 skipped  .floor/2026-09-05T22-25-51Z/
row 5  publish 49739 50975 51300 52904 53458   (quiet, x5; median 51300)
row 6  LIMIT65 rows=60 us_per_scan=1336  |  LIMIT1 rows=1 us_per_scan=448
row 7  distinct=100 on all twelve chaos runs
row 8  total=8000; distinct=8000; dup=0 x5
row 9  outbox 250-1000=7739 max=5693ms; t1->t2 <1ms=8000; t2->t3 <1ms=8000;
       t3->t4 1-10=991 10-50=3544 50-250=3405 >1000=40 max=4333ms
```

## ⛔ ROW 5 IS A DIRECTION, NOT A POINT

`≤ 42000 ms` is an **18% improvement on the median**, against a projection of ~33 s. It is
deliberately wide: it cannot red on noise, but it **does** fail if nothing actually improved.

★ I have three times in this arc turned an observation into a gate and had it fire wrongly. This
row gates *movement*, which is what the stone controls, and reports the value.

## ⛔ ROW 9 IS THE ROW THAT MAKES THIS A LOOP

**Name the new leader.** If `outbox` is no longer dominant, say what is — `t3->t4`, `setup`,
`stop`, the store, the interpreter. **A perf stone that reports only its own saving cannot tell
us when to stop**, and the arc's terminal condition is *interpretation overhead becomes dominant*.

## RUNTIME PREDICTION

90–150 min. The surface feature plus two store impls is most of it; `total` is small. Budget a
rebuild per measurement, and two measurement passes for row 5.

## TRAP DOORS

1. **A `count` that fetches and counts in wat.** Buys nothing; STOP-2. The saving is the rows
   never being built.
2. **Pointing `stats` at `total`.** It needs the split; row 4 catches it.
3. **Measuring on a loaded box.** Check `ps` first — a contended run has invalidated a perf claim
   in this arc before.
4. **Reporting only the saving.** Row 9. The leader is the deliverable that makes the next stone
   drawable.
