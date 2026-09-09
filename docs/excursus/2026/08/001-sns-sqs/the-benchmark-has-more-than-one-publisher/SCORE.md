# SCORE — the benchmark has more than one publisher

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
`run-with` takes `p`. P publishers, own seeds, tiled ids. p=1 reproduces today.

```
Summary [ 408.062s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T23-03-26Z/`

## THE HARNESS

`Publisher` mirrors `Worker`: `start` arms `-run` (1 ms) and returns Ok; `stats`
returns `[done calls retries asleep]`. Parent spawns P in setup, starts all,
then joins. `publish` = first start → last done.

Seeds: `BACKOFF-SEED + i * 7919` → `1, 7920, 15839`. Probe:

```
seeds=1,7920,15839;distinct=yes;p3=[0,666)[666,1333)[1333,2000);p1=[0,2000);tiled=yes
```

Ids: publisher `i` owns `[ (i*n)/p , ((i+1)*n)/p )`. p=3 at n=2000 tiles
`[0,666)[666,1333)[1333,2000)`. p=1 is `[0,2000)`.

A process child is assembled from service-forms only. It cannot see
`circuit.wat` helpers (`share-stats!`, `backoff-delay`, `await-timer-ms`).
`-run` inlines stamp / drop-first / backoff / publish. Each batch restarts
from that publisher's seed — matching parent `publish-share-until-accepted!`.

`stats` is the join. `-run` is one-shot (~20 s), so the generated 10 s
deadline TimedOut. Parent `publisher-stats` uses `call-by-deadline` 120000.
First stats blocks until that publisher's `-run` returns; p=3 wall is max,
not sum.

`user::main` stays `run* 2000 4 3` (p=1). `user::run-p*` is the p=3 entry.
Policy selected by editing the one `await-ms` expression; restored to `d`.

## STOP-1 — p=1 reproduces today

Adaptive p=1 ×3, same box, quiet `ps`:

| | r1 | r2 | r3 | median | known |
|---|---|---|---|---|---|
| publish | 20409 | 20404 | 20534 | **20409** | 20379 |
| full-retries | 1389 | 1391 | 1389 | **1389** | 1386 |
| queue-receive-calls | 5066 | 4969 | 5004 | **5004** | 5032 |
| distinct | 8000 | 8000 | 8000 | 8000 | 8000 |
| setup | 10625 | 10643 | 10577 | 10625 | 10205 |
| stop | 6276 | 6041 | 6822 | 6276 | 6763 |
| asleep | 11651 | 11719 | 11672 | 11672 | 11621 |
| publish-calls | 200 | 200 | 200 | 200 | 200 |

publish +30 ms of 20379. Inside ±300. **STOP-1 did not fire.**

## THE SIX CELLS (medians)

Every completed run: `total=8000;distinct=8000;dup=0`.

```
                       p=1                         p=3
adaptive  publish      20409                       20635
          retries       1389                        2086
          receives      5004                        5062
          calls          200                         201
          setup        10625                       11288
          stop          6276                        6984

fixed 25  publish      18599                       19756
          retries        543                        1671
          receives      4792                        4971
          calls          200                         201
          setup        10614                       11300
          stop          7535                        7064

fixed  1  publish      22492                       22667 †
          retries       3778                        3937 †
          receives      5317                        5344 †
          calls          200                         201
          setup        10599                       11223 †
          stop          6019                        5979 †
```

p=3 `publish-calls=201` is the tile: 667+667+666 msgs → 67+67+67 batches.
Counts are sums across publishers. **STOP-3 did not fire. Gate 5 holds.**

## STOP-2 — concurrent, not serial

Adaptive p=3 publish median **20635**, not ~3× 20409. 25 ms p=3 **19756**,
not ~3× 18599. All P are `start`ed before any is joined. **STOP-2 did not fire.**

asleep at p=3 adaptive is ~41564 — the **sum** of three publishers' sleeps,
~3× the p=1 11672. Wall is max. That is the concurrency proof in the number.

## STOP-4 — 25 ms degrades a little, and still wins

The prediction on the record: *fixed 25 ms degrades at p=3 and adaptive does not.*

| | p=1 | p=3 | Δ |
|---|---|---|---|
| adaptive | 20409 | 20635 | **+226 ms** |
| fixed 25 | 18599 | 19756 | **+1157 ms** |
| fixed  1 | 22492 | 22667 † | +175 ms on the one survivor |

25 ms slowed 6 %. Adaptive did not. The lockstep is visible in retries
(543 → 1671, a 3.1× sum) and in e2e (p=3 25 ms has hundreds of >1000 ms
samples; p=1 25 ms has zero).

It did **not** collapse. 25 ms at p=3 (**19756**) still beats adaptive at
p=3 (**20635**) by 879 ms, and still beats adaptive at p=1. Jitter does
not overtake the swept 25 ms at N=3. The 18472 "optimum" was an N=1
artifact in its *absolute* value (p=3 25 ms is 19756, not 18472) but not
in its *ranking*.

STOP-4 as written ("does not degrade") did not fire as a binary. The
degrade is real and small. Report it that way. The correction of the last
SCORE's *framing* holds (N=1 cannot test de-correlation). The correction
of its *ranking* does not: 25 ms is still faster at the cardinality we
just became able to ask.

p=1 25 ms asleep on those three runs recorded the unused jitter draw, not
25; wall and retries are the measurement. p=3 25 ms asleep is retries×25
(41150 / 1646 = 25).

## 1 ms at p=3 — the publisher dies

† not a clean cell.

Cap-64 inner foldl: 3/3 `fanout: publisher stats lost` (child died).
Cap-256: 2/3 lost, 1 survivor at 22667 / 3937 / 5344 / distinct=8000.

Do not re-run a red. The child inlines `Topic/publish` (generated 10 s
deadline). Under a 1 ms stampede of three publishers the topic goes
silent past that deadline, `-run` asserts TimedOut, the process dies,
the parent sees Lost. 25 ms and adaptive never hit it. This is a
harness-join fact about one-shot `-run` plus a 10 s publish deadline,
not a queue bug.

The one survivor is in the table with a dagger. The two deaths are the
finding for that cell.

## BLAST

`wat-scripts/fanout/circuit.wat`, the scratch probe, this SCORE.
`git status --porcelain`: those two plus this file. **No `sqs.wat`, no
`sns-fanout.wat`, no `wat/`, no `src/`.** STOP-5 did not fire.

`sns-fanout.wat` demo still 1 ms. Force-expire `limit-ms 0` is still the
parent `publish-until-accepted!*`, not the Publisher service.

## WHAT THIS DOES TO THE LAST SCORE

Adaptive beat shipped 1 ms at p=1 (−2083 ms: 20409 vs 22492) and at p=3
on the survivor. It lost to 25 ms at p=1 (−1810 ms) and at p=3 (−879 ms).
Full jitter's de-correlation is now *askable*. At p=3 it did not buy the
wall. 25 ms still sits on the sweep's sweet spot; three clients sleeping
25 ms in parallel pay the lockstep in retries, not in a 3× wall.
