# SCORE — the topic names which send failed

**SCORED. STOP-5.** Executor: grok, 2026-09-08. Did not commit.
The three arms were counted separately. They do not account for the
publisher's rejections. Did not fix the gap.

```
Summary [ 496.104s] 5235 tests run: 5235 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T21-10-48Z/`

No tests added. **5235**. 0 FAIL, 0 TIMEOUT.

## WHAT LANDED

`:demo::Topic::StatsResponse::Ok` is arity 5: `depth ticks inbox-lost
inbox-closed inbox-timedout`. Three `i64` counters on `:durable`,
incremented in the existing `Lost` / `Closed` / `TimedOut` arms of the
inbox send. Reply stays `Accepted 0`. Redial stays. `:max-entries [msgs
10]` untouched. Stats sentinels are `Ok -1 -1 -1 -1 -1`.

The circuit report line carries all three beside `full-retries`.

## THE MEASUREMENT

n=2000, `vis-ms=1000`, load at start **0.87**. Quiet box.

```
publish-calls=200  publish-attempts=1438  full-retries=1238  asleep=8324
fill=15910  drain=2550  total=37289
distinct=8000  dup=0
inbox-lost=0  inbox-closed=0  inbox-timedout=0
```

## STOP-5 — the three do not account for the rejections

`0 + 0 + 0 = 0`. `full-retries = 1238`. **Gap = 1238.**

No arm dominates. Every arm is **zero**. A zero is a result: `Lost`,
`Closed` and `TimedOut` of the topic's inbox send did not fire in this
run.

The publisher retries on `PublishResponse::Accepted c` with `c <= 0`,
and its own `Lost`/`Closed`/`TimedOut` of `Topic/publish` **assertion-fail**
— they are not the retry path. So the 1238 `Accepted 0` replies were
`RecvOutcome::Message`. The path that produces that without touching
the three arms is already in the file, unread by the DESIGN:

```
SendResponse::Accepted pairs=0  →  floor = 0/nsubs = 0, rem = 0
  →  the rem==0 branch returns Accepted floor   (= 0)
```

A successful inbox send that accepted **zero pairs**. Not a transport
failure. That is the fourth source. Not counted, not fixed.

## IDENTITY

`12 2 2 32 false`: `total=24;distinct=24;dup=0` and
`full-retries=0;inbox-lost=0;inbox-closed=0;inbox-timedout=0`.
Floor `:user::compute` PASS 20.972 s.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ three arms counted separately | ✅ three `:durable` counters, never summed |
| 2 | ★ report line carries all three | ✅ beside `full-retries` |
| 3 | ★ they account for the rejections | **STOP-5.** gap=1238. Named: `Accepted pairs=0` on the Message path |
| 4 | no behaviour changed | ✅ `Accepted 0`, redial, `:max-entries [msgs 10]` |
| 5 | existing entries unaffected | ✅ completeness identical at n=12; floor compute PASS |
| 6 | blast | ✅ two files + Record ctors the compiler named (scratch-pad) |
| 7 | stats sentinels left alone | ✅ `-1` values kept; arity 5 |
| 8 | box quiet | ✅ load 0.87 at start |
| 9 | the floor | ✅ `Summary [ 496.104s] 5235 tests run: 5235 passed (7 slow), 22 skipped` |

## BLAST

```
 wat-scripts/topic/sns-fanout.wat                   | 80
 wat-scripts/fanout/circuit.wat                     | 28
 wat-scripts/scratch-pad/*.wat                      | Record ctors (compiler-named)
```
