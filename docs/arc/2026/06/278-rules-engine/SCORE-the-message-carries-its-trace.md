# SCORE — the message carries its trace

**STRUCK.** Executor: grok, 2026-09-02. In-band, five stamps, histogram per stage. No mean.

```
Summary [ 352.277s] 5184 tests run: 5184 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T23-24-21Z/`

First floor was red — kept, not re-run. `.floor/2026-09-02T23-17-03Z/`:

```
Summary [ 348.923s] 5184 tests run: 5182 passed (3 slow), 2 failed, 15 skipped
```

Arms: `probe_ex001_fanout::fanout_compute_is_complete_and_lossless`, `probe_arc278_sane_circuit::receive_calls_are_not_triple_the_messages`. Both raised `ServiceNotRunning` on `:wat::kernel::println` inside `run*` — rust tests apply `:user::compute` / `:user::compute-calls` with no stdio. Traces now ride in the returned phases string. Named, fixed, new floor.

## Row 4 — pending residency is not the 250 ms park

Five runs. Fast (drain=11867) against slow (drain=18686):

```
t3->t4   <1ms  1-10   10-50   50-250  250-1000  >1000   max
12 s        0  1944    5187      694        175      0    654ms
19 s        0  2800    4152      332        105    611   6619ms
```

The 250–1000 ms bucket — where a waited-out park would land — is **small and does not track drain** (175 vs 105). The bucket that tracks drain is **>1000 ms**: 0 on the fast run, 611 on the slow run, max 6.6 s.

**The park-timeout theory is dead.** A measurement that redirects. Queue counters on served / woken / expired-empty would count the 250 ms cases; they would not explain a 4–6 s wait. The variance is not in the queue's 250 ms wake path.

## Five drains (row 2) — same range, not a shifted median

```
this stone     11867  15615  15784  16250  18686     median 15784   1.6×
on record      12472  12712  12716  12830  17076
               17538  17765  18422  24827            median 17076   ~2×
```

Median and spread in the same range. Payload ~4 → ~120 bytes did not move the distribution. STOP-2 does not fire. All five: `total=8000; distinct=8000; dup=0`. topic-ticks=200.

## The shape (row 3) — one line per stage, no mean

Run 1 (drain=11867), buckets sum to 8000 on every line:

```
sample=0|1788390801381279641|1788390802178224596|1788390802178530583|1788390802179471363|1788390802183278362
outbox   <1ms=0 1-10=0 10-50=0 50-250=0 250-1000=160 >1000=7840 max=11355ms
t1->t2   <1ms=7977 1-10=23 10-50=0 50-250=0 250-1000=0 >1000=0 max=2ms
t2->t3   <1ms=2075 1-10=5901 10-50=24 50-250=0 250-1000=0 >1000=0 max=43ms
t3->t4   <1ms=0 1-10=1944 10-50=5187 50-250=694 250-1000=175 >1000=0 max=654ms
e2e      <1ms=0 1-10=0 10-50=0 50-250=0 250-1000=151 >1000=7849 max=11809ms
```

Outbox residency **is** the drain for most messages: ~7840 of 8000 sit >1 s in the topic FIFO waiting their turn (K=10, 200 ticks). t1→t2 is free. t2→t3 is the send hop (mostly 1–10 ms). e2e max ≈ drain.

## What landed

- `circuit.wat` — `t0` at `accept!` (retries keep the same stamp); `t2` at adapter entry; `t4` at worker receive, after `Queue/receive` returns; histogram over the 8000 outcome bodies.
- `sns-fanout.wat` `-deliver` — `t1` at dequeue, per message inside the K-loop.
- `sqs.wat` `send` — `t3` on entry. Guard: stamp only when the body already carries `|` (the in-band prefix). Unconditional stamp made mem vs sqlite **diverge** (`DIFFERENTIAL-MISMATCH` on timestamps) and broke `probe_ex001_queue` / `probe_queue_long_poll`. Circuit always arrives as `seq|t0|t1|t2`. Queue lifecycle fixtures send bare strings and stay byte-identical.

`seq` is first. Sample body above. `wat/`, `src/` empty. `Queue::StatsResponse` unchanged.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ invariant | ✅ `total=8000; distinct=8000; dup=0` on all five |
| 2 | ★ instrument does not perturb | ✅ median 15.8 s against 17.1 s; spread 1.6× against ~2× |
| 3 | ★ shape, not a mean | ✅ six buckets + max per stage; 8000 parsed |
| 4 | ★ t3→t4 bimodal? | **no, not at 250–1000.** Slow runs grow a **>1 s** tail (0 → 611). Park theory dead. Counters on the 250 ms wake path would be work in the wrong interval |
| 5 | five stamps, seq first | ✅ `0\|t0\|t1\|t2\|t3\|t4` |
| 6 | no surface change | ✅ Sub / SendRequest / StatsResponse unchanged |
| 7 | sqs.wat is the t3 stamp | ✅ send only; `contains? "\|"` guard so lifecycle gates stay byte-identical |
| 8 | no substrate | ✅ `wat/`, `src/` empty |
| 9 | topic-ticks ~200 | ✅ 200 on all five |
| 10 | floor | ✅ 5184/5184, `FLOOR=0`, `.floor/2026-09-02T23-24-21Z/` (red predecessor named above) |

## Counters still deferred

The trigger was "if t3→receive is bimodal." Slow runs have a second population, and it is **seconds**, not a park. Expired-empty is 250 ms. The next stone is not three fields on `StatsResponse`.
