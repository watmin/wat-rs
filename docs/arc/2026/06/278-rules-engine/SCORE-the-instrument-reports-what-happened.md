# SCORE — the instrument reports what happened

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. `stop` is teardown. `publish-attempts`
counts every Topic/publish. Delay not pinned.

```
Summary [ 479.678s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T04-42-23Z/`

## THE SPLIT

`t-collect0` after drain, before any stats call. Collect is
`sum-calls` / `sum-ticks` / `topic-ticks` / `sum-disrupts` / `seen-stats` /
`collect-stop` / `empty-flags` / summary. `t-stop0` then `topic-worker/stop`.
`t-end`.

`empty-flags` is a `receive` limit 1 per queue. It touches the system. It
belongs in **collect** — BRIEF sketch; it is a post-drain emptiness check,
not a process reap. SCORE says so because EXPECTATIONS asked.

Phases line: `collect={c}` beside `stop={s}`; `total` is `t-setup0`→`t-end`
so gate 4 is visible.

## THE COUNTER

Live path is Publisher `-run` inline. Inner `st` is
`(left right att)`. `att+1` on full accept, Accepted 0, **and** partial.
Outer acc first is `(calls retries attempts)`. `StatsResponse::Ok` is
`[done calls retries asleep attempts]`.

`join-publishers` returns `((calls retries) (asleep attempts))` — Tuple
has no fourth accessor.

Parent `publish-until-accepted!*` (share-stats / `publish-n-until-accepted!`)
threads `attempts` the same way: +1 on both recurse arms and the success
return. `full-retries` still counts bounces only.

## CIRCUIT ×3, shipped config, unpinned

`ps` before: grok 11.6 %, claude 4.6 %. Delay not pinned.

Every run: `total=8000;distinct=8000;dup=0`.

```
                    r1     r2     r3    med    DESIGN today
setup            12368  12392  12276  12368           12297
publish          20484  20893  20795  20795           20812
drain              212    231    213    213             186
collect           5939   6221   5423   5939            ~4800
stop               131    584    631    584            ~1800
total            39136  40323  39340  39340
publish-calls      200    200    200    200             200
full-retries      1370   1381   1375   1375            (adaptive ~1389)
asleep           11044  11413  11271  11271
publish-attempts  1570   1581   1575   1575
receives          5050   5019   5070   5050
```

Before (DESIGN): `setup 12297  publish 20812  drain 186  stop 6797`.

After (median): `setup 12368  publish 20795  drain 213  collect 5939  stop 584`.

## STOP-1 — publish and drain did not move

publish median **20795** vs 20812 (−17). Inside ±300. drain 213 vs 186 is
the same poll; the drain code was not touched. distinct=8000, dup=0.
**STOP-1 did not fire.**

## PHASES SUM (gate 4)

Each run, `setup+publish+drain+collect+stop` is `total − 2`. Integer-ms
truncation across five intervals vs one. **STOP-2 did not fire.**

## COLLECT vs STOP — where the 1.8 s went

DESIGN's old `stop=6797` was collect ~4823 + "teardown" ~1790. Their
teardown chunk was `_stoptw` + `empty-flags` + leftover, attributed as
"26 process reaps". Measured after the split:

- **collect ~5939** = DESIGN's 4823 + `empty-flags` (~1.1 s of four
  Immediate receives). Internal split not re-timed; DESIGN's
  `sum-disrupts 1262` / `collect-stop 3509` still describe the rest.
- **stop ~584** is `topic-worker/stop` only (131 / 584 / 631 — noisy).
- Remaining process reaps happen when the `let` drops, **after** `t-end`.
  They were never inside the wall. `total` does not include them.

collect+stop median 6523 vs old stop 6797. Same work, labelled. stop is
not ~1800 because empty-flags is collect and the 26 reaps are outside
the timer. That is the finding, not a STOP.

## publish-attempts (gate 5)

Median **1575**. Always `publish-calls + full-retries` (200+1375).
Zero partials at this load: inbox cap 64, batch 10, workers drain in
chunks that land on Accepted 0 or Accepted 10.

The partial arm increments `att`. Equality is "the topic did not
partial", not "the branch is still invisible". Per-call figure is now
computable: `publish / publish-attempts` = 20795/1575 ≈ **13.2 ms**.
The 5.7 ms that misled DESIGN used `retries+200` as if it were the
call count *and* attributed blocked time to calls; with no partials
the denominator matches, and the honest quotient is still not 5.7
because asleep is drawn delay, not per-call overhead.

## full-retries (gate 6)

Median **1375**. Unpinned adaptive, matching SCORE-the-benchmark's
1389, not the pinned-25 ms 520–600. DESIGN forbade pinning. Bounce-only
meaning held: attempts − calls = retries on all three runs.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔⛔ publish and drain do not move | ✅ publish 20795 vs 20812; drain 213 vs 186 |
| 2 | ★★ stop is teardown only | ✅ `_stoptw` only; ~584 not ~1800 (see above) |
| 3 | ★★ collect appears | ✅ median 5939 (includes empty-flags) |
| 4 | ⛔ phases sum to the wall | ✅ total − 2 ms each run |
| 5 | ★★ publish-attempts counts partials | ✅ arm increments; this load had zero partials so attempts = calls + retries |
| 6 | ⛔ full-retries keeps bounce meaning | ✅ 1375 unpinned; attempts − calls = retries |
| 7 | ⛔ delivery exact | ✅ ×3 `distinct=8000;dup=0` |
| 8 | ⛔ the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 9 | ⛔ blast | ✅ `circuit.wat` + SCORE. No `wat/`, no `src/`, no `sqs.wat`, no `sns-fanout.wat` |

**STOP-3 did not fire.**
