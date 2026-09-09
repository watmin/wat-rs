# SCORE — vis is swept, not chosen (v2)

**SCORED.** Executor: grok, 2026-09-08. Did not commit.

```
Summary [ 505.820s] 5235 tests run: 5235 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T20-20-46Z/`

No tests added. **5235**. 0 FAIL, 0 TIMEOUT.

The sweep ran **before** the floor, not beside it.

## WHAT LANDED

`run-with` takes a 15th parameter `vis-ms`. `0` reaches the existing
drop?/non-drop conditional unchanged (200 ms if any drop rate, else
1 000 000 ms). A positive value is milliseconds × 1 000 000 → vis-ns.

The six internal callers pass `0`. The argv entry gains an optional sixth
slot; the five-slot command line is unchanged.

`limit-ms (:wat::i64::/ vis 1000000)` at the coupling site is **not** in
the diff.

## THE FORK — branch one

**A `vis` exists that completes n=2000. Every swept value does.** The
coupling `limit-ms = vis / 1e6` was fine; the 1 000 000 ms default was
wrong. That default is 25× the run, so a stuck message cannot recover
in-run. At 1 s through 100 s, redelivery happens inside the drain's
patience and the point completes.

This stone does not pick a point in the band. The default is still
today's 1 000 000 ms.

## THE SWEEP

`2000 4 3 8192 true`. Geometric across the derived band. No point skipped.
Quiet box (load at each start stated).

| vis-ms | completed | distinct | dup | drain-ms | check-ex | mark-ex | ack-retries | ack-exhausted | wall-s | load |
|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | **yes** | 8000 | 0 | 2629 | 0 | 0 | 5 | 1 | 39.6 | 0.74 |
| 3000 | **yes** | 8000 | 0 | 4110 | 0 | 0 | 11 | 0 | 42.1 | 1.91 |
| 10000 | **yes** | 8000 | 0 | 10148 | 0 | 0 | 1 | 0 | 47.8 | 2.31 |
| 30000 | **yes** | 8000 | 0 | 31179 | 0 | 0 | 12 | 0 | 69.6 | 2.12 |
| 100000 | **yes** | 8000 | 0 | 100177 | 0 | 0 | 2 | 0 | 138.3 | 1.99 |

Drain tracks vis: once vis is shorter than the drain's ~215 s patience,
the run finishes, and drain-ms ≈ vis-ms for the larger values (the retry
bound *is* vis, so a stuck envelope becomes visible again at vis and the
drain waits that long). `ack-exhausted` is 0 except one exhausted ack at
1 s.

## IDENTITY

5-slot `12 2 2 32 false` completeness fields byte-identical to the
pre-knob baseline. Explicit `vis-ms=0` sixth slot matches the five-slot
line. `--check` clean. Floor `:user::compute` PASS 21.084 s.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ existing entries identical | ✅ 5-slot fields identical; wrappers pass 0; floor compute/sane PASS |
| 2 | ★ sweep table, every value | ✅ five rows, none skipped |
| 3 | ★ the fork is named | ✅ **branch one** — vis completes n=2000; coupling was fine, value was wrong |
| 4 | `vis-ms=0` is today | ✅ existing drop?/non-drop conditional under the `> 0` test |
| 5 | sixth argv slot optional | ✅ 5-slot works; usage names `[vis-ms]` |
| 6 | coupling untouched | ✅ `limit-ms` not in the diff |
| 7 | box quiet per timed run | ✅ load 0.74 / 1.91 / 2.31 / 2.12 / 1.99; floor after the sweep |
| 8 | blast | ✅ `wat-scripts/fanout/circuit.wat` only |
| 9 | the floor | ✅ `Summary [ 505.820s] 5235 tests run: 5235 passed (7 slow), 22 skipped` |

## BLAST

```
 wat-scripts/fanout/circuit.wat | 34 +++++++++++++++++++++-------------
```

Seven call sites plus the usage string. No `sqs.wat`, no `wat/`, no
`StatsResponse`.
