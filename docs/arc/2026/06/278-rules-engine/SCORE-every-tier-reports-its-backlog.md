# SCORE — every tier reports its backlog

**SCORED.** Executor: grok, 2026-09-09. Did not commit.

```
Summary [ 503.173s] 5235 tests run: 5235 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T00-00-39Z/`

No tests added. **5235**. 0 FAIL, 0 TIMEOUT.

## WHAT LANDED

Five counters on the queue's **`:ephemeral`**: `sends-accepted`, `sends-refused`,
`acks`, `redeliveries`, `expired-waiters`. `:queue::queue::Record` is untouched.

`StatsResponse::Ok` now carries a `:queue::Stats` record (the TakeAcc/RetryAcc
shape) instead of seven positional fields. Increments are field stores — no
store call.

`seen-ids` is a sixth ephemeral HashSet used only to distinguish a first
delivery from a redelivery. It is not on `Stats`.

No discard / retention / dead-letter path was added. None exists.

## THE MEASUREMENT

n=2000, `vis-ms=1000`, load at start **0.83**. Quiet box.

```
full-retries=1205  store-calls=9755  fill=15542  drain=2932  total=37062
distinct=8000  dup=0
```

| tier | accepted | refused | acks | redeliveries | expired-waiters | visible/unacked | store-calls |
|---|---|---|---|---|---|---|---|
| inbox | 8000 | **1205** | 8000 | 0 | 170 | 0/0 | 8558 |
| sub[0] | 2000 | 0 | 2000 | 0 | 29 | 0/0 | 2661 |
| sub[1] | 2000 | 0 | 2010 | 20 | 33 | 0/0 | 2465 |
| sub[2] | 2000 | 0 | 2000 | 20 | 42 | 0/0 | 2493 |
| sub[3] | 2000 | 0 | 2040 | 0 | 48 | 0/0 | 2496 |

## WHICH TIER IS THE CONSTRAINT

**The inbox.** It is the only tier with refusals. Arrival 8000 equals service
8000 by the end; depth is zero because the run drained. During fill the cap-64
inbox is what refuses — 1205 times — and that is the publisher's wait.

Subscribers refuse nothing. Their extra acks / redeliveries (sub[1] 20, sub[3]
acks 2040) are visibility expiry on the worker side, not admission.

## ROW 5 — refusals reconcile

Inbox `sends-refused=1205`. Publisher `full-retries=1205`. **Equal. No gap.**
A refused batch is one refusal, as specified.

## ROW 3 — the instrument is free

`store-calls=9755` vs pre-stone **9731** (+24). That is the five end-of-run
`stats` reads (2 store calls each, plus a couple already in collect), not the
counters. Increments do not touch the store. Not 39 %.

## THE ABSENCE

No message-discard counter. `retention|expire|discard|evict|reap` still hits
only waiter expiry. `expired-waiters` counts that. Messages become visible
again; they are not dropped.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ five counters distinct | ✅ never summed, never reusing ticks |
| 2 | ★ live on `:ephemeral` | ✅ Record constructors unchanged |
| 3 | ★ no store-call increment | ✅ 9755 vs 9731 |
| 4 | ★ every tier reports | ✅ inbox + sub[0..3] |
| 5 | refusals reconcile | ✅ 1205 = 1205 |
| 6 | absence recorded | ✅ no discard path, none added |
| 7 | existing entries | ✅ n=12 completeness identical; floor compute PASS |
| 8 | blast | ✅ 8 files |
| 9 | box quiet | ✅ load 0.83 |
| 10 | the floor | ✅ `Summary [ 503.173s] 5235 tests run: 5235 passed (7 slow), 22 skipped` |

## BLAST

```
 wat-scripts/queue/sqs.wat                          | 314
 wat-scripts/fanout/circuit.wat                     |  61
 wat-scripts/topic/sns-fanout.wat                   |   9
 wat-scripts/scratch-pad/probe-*.wat                |  20
```
