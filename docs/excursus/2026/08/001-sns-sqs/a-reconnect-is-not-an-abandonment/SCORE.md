# SCORE — a reconnect is not an abandonment

**SCORED. STOP-7.** Executor: grok, 2026-09-08. Tree dirty, uncommitted.
`Lost`/`Closed` in all three ladders redial and retry on the same
elapsed bound as `DeadlineFired`. `ack-exhausted` is on the Record,
the summary, the phases line, and the failure string. n=2000 still
did not drain. Did not re-run. Did not change `vis`.

```
Summary [ 498.810s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T03-55-17Z/`

## WHAT LANDED

`:fanout::worker` only, `circuit.wat` only.

- First-attempt and inner-fold `Lost`/`Closed` arms in `seen-until`
  (check + mark) and ack collapsed onto `_`, which is the old
  `DeadlineFired` path: elapsed from `now` / `ack-start-ns`, bound
  `vis/1e6`, backoff, recurse with the redialed peer.
- After-fold `Got` only when `done` and still inside the bound.
  `Exhausted` otherwise — from the bound, not from a transport reason.
- `ack-exhausted` last on `worker::Record` and `DisruptsResponse::Ok`.
  Tick pair nested `(Tuple (Tuple ce me) (Tuple ar ae))`. `sum-disrupts`
  third is `(Tuple ars ae)`. Failure / summary / phases all carry it.

`:2075` untouched. `vis` untouched. PersistentMap untouched.

## STOP-7 — n=2000 did not drain

```
drained-never: last=[0/0][0/20][0/20][0/10] outbox=0 attempts=8000 elapsed=214907;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0
```

`circuit.wat:2293` `require!`. Visible 0 on every queue; **50 unacked**
stuck (was 40). 8000 poll attempts, 214.9 s. Did not re-run.

All four counters are **0**. The previous stone reported `ack-retries=6`
with no `ack-exhausted`; this stone reports zeros on the failure path.
The tick updates the Record only after the ladder returns. A worker
still inside the retry fold (bound = 1 000 000 ms) has not Exhausted
and has not written the counters. Drain gave up at 215 s. That is the
named result STOP-7 asked for: the failure can now say it did not
abandon.

## WHAT DID COMPLETE

| n | distinct | dup | seen-recorded | drain ms | ack-exhausted |
|---|---|---|---|---|---|
| 12 | 48 | 0 | 48 | 78 | 0 |
| no-args | 8000 | 0 | 8000 | 213 | 0 |
| 500 | 2000 | 0 | 2000 | 559 | 0 |
| 1000 | 4000 | 0 | 4000 | **1131** | 0 |
| 2000 fill-first | — | — | — | **drained-never** | **0** (with ce=0 me=0 ar=0) |

n=500 drain 559 vs 561 last stone. n=1000 drain **1131** in the
1110–1170 band (was 1165; 1365 before PersistentMap). Row 6 held.

no-args: `seen-recorded=8000;seen-skipped=0`. collect=16190 (untouched
`:2075`). New field `ack-exhausted=0`; every pre-existing field
identical in kind.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ n=2000 completes | **STOP-7.** 50 unacked, counters 0. Allowed to fail |
| 2 | Exhausted only from the bound | ✅ no `Exhausted` in a `Lost` or `Closed` arm; `_` checks `elapsed >= limit-ms` |
| 3 | the retry is bounded | ✅ same `now` / `ack-start-ns` origin as `DeadlineFired`. STOP-1 did not fire |
| 4 | `ack-exhausted` reported | ✅ summary, phases, failure string. Failure path showed `ack-exhausted=0` |
| 5 | distinct = n×m, dup=0 | ✅ at n=12 / 500 / 1000 / no-args. n=2000 did not finish |
| 6 | PersistentMap gain kept | ✅ n=1000 drain 1131 (band 1110–1170) |
| 7 | no-args unchanged | ✅ identity on pre-existing fields; `ack-exhausted=0` additive |
| 8 | nothing else moved | ✅ `circuit.wat` only; `:2075` and `vis` untouched |
| 9 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 10 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 11 | the floor | ✅ `Summary [ 498.810s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 did not fire** (n=12 typed and ran; new arms ride `now` /
`ack-start-ns`). **STOP-7 fired** from n=2000. STOP-2 / STOP-4 / STOP-5
/ STOP-6 held. Did not restore: n=500/1000/no-args are green and the
ladders no longer discard after redial.

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## BLAST

```
 wat-scripts/fanout/circuit.wat | 123 +++++++++++++++++++----------------------
 1 file changed, 58 insertions(+), 65 deletions(-)
```

No `sqs.wat`. No `wat/`. No ripple. PersistentMap stays.
