# SCORE — the trace stamp stops round-tripping

**STRUCK.** Executor: grok, 2026-09-03. `Queue/send` no longer inspects the body.
The topic-worker stamps t1/t2/t3 itself and strips only the idx prefix. No
production code adds a field that other production code removes.

```
Summary [ 354.353s] 5190 tests run: 5190 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T06-49-06Z/`

## The wart

`sqs.wat` used to `contains? b "|"` and append a timestamp. The topic-worker
then split on `|` and re-joined minus that segment, because the inbox send
had just added one. Token-vs-form hid the first half (`contains? b`, not
`contains? body`). Last, deliberately, because S13/S14 could have moved the
format.

## Trace not lost (STOP)

`traces-of` still expects 6 parts. Sample is `seq|t0|t1|t2|t3|t4` on every
run. t3 now sits on the topic-worker, just before `Queue/send`, instead of
inside the queue. `t2→t3` collapses to <1 ms; `t3→t4` carries the send+wait.
All five histogram stages still populate. STOP did not fire.

`grep -n 'contains?' wat-scripts/queue/sqs.wat` — zero hits.

## Five runs, sqlite (the fixture), `2000×4×3`

All `total=8000; distinct=8000; dup=0`.

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 24.790 s | 704 ms | **323** |
| 2 | 25.072 s | 640 ms | **319** |
| 3 | 25.367 s | 581 ms | **315** |
| 4 | 25.221 s | 632 ms | **317** |
| 5 | 25.096 s | 700 ms | **319** |

Median **319/s** against S2's 310–325/s. Within noise. The round-trip was
string surgery, not a throughput term.

## What landed

- `sqs.wat` `send` writes `SendRequest/bodies` as given.
- `sns-fanout.wat` topic-worker: rest = everything after idx; stamps
  `{rest}|{t1}|{t2}|{t3}`. Circuit worker still appends t4.
- `wat/`, `src/` empty.
