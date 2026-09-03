# SCORE — the queue is bounded

**STRUCK.** Executor: grok, 2026-09-03. `Queue/send` refuses at a depth cap. The adapter
**blocks** until accepted. Backpressure propagates. No deadlock.

```
Summary [ 353.161s] 5185 tests run: 5185 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T01-46-13Z/`

## Rows 1 and 2 — both, not either

Five runs, mem-store, topic cap 16, queue cap 16, K=10, `2000×4×3`. All
`total=8000; distinct=8000; dup=0`. `t3→t4 >1 s` count **0** every run.

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 8.073 s | **124 ms** | **991** |
| 2 | 8.045 s | **125 ms** | **994** |
| 3 | 8.332 s | **189 ms** | **960** |
| 4 | 8.354 s | **257 ms** | **958** |
| 5 | 8.440 s | **272 ms** | **948** |

Median **960/s**, e2e max **124–272 ms**. Against unbounded-batched mem (282/s, 36–42 s):
latency recovered, throughput recovered past unbatched 661/s. Two of five e2e maxes sit
just over 200 ms (257, 272); the distribution is ~200 ms, not 2.6 s / 36 s.

Landed circuit queue cap is **32** (K=10 fits with headroom). One mem run at 32: 947/s,
e2e max 153 ms, `>1 s` still 0.

## 2×2 re-run, bounded queue (row 7)

| | mem-store | sqlite-store |
|---|---|---|
| **unbatched** (FINDING) | 661/s, 200 ms | 789/s, 185 ms |
| **batched, unbounded** (FINDING) | 282/s, 36–42 s | 1568/s, 2.6 s |
| **batched, queue cap 16** | 960/s, 124–272 ms | **1240/s, 106–114 ms** |
| **batched, queue cap 32** | 947/s, 153 ms | **1383/s, 148 ms** |

Sqlite cap 32 keeps **1.75×** of unbatched 789/s (against 1.99× unbounded) and e2e **148 ms**
(against 2.6 s). Not quite ~1500/s; the 1 ms retry poll is the remaining tax. Store share
at cap 16 is 960→1240 = **1.29×**, toward the unbatched 1.19× — quadratic writes look like
an oracle concern once queues stay shallow.

## What landed

- `Queue::SendResponse` gains `:Full [depth cap]`. `queue::queue::Record` gains `cap`.
- `send` checks `pending + in-flight + n > cap` **before** the store put. All-or-nothing
  per batch. Tests use cap 1024 so lifecycle gates never Full.
- Adapter **blocks**: on Full it stashes the in-flight `msgs` (one request, not a reservoir),
  replies `None` to the topic, and 1 ms `-retry` until Ok, then Directed-replies. Drop and
  adapter-buffer were not taken.
- Circuit / sqlite probe: queue cap **32**. Topic cap stays 16.

`wat/`, `src/` empty. No deadlock (STOP-3). The 1 ms `-retry` is the named poll wart;
trigger for a parked reply is: this hypothesis held.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ e2e max ~200 ms | ✅ 124–272 ms (cap 16 five); 153 ms (cap 32 mem); 148 ms (cap 32 sqlite) |
| 2 | ★ batching's win | **1383/s sqlite cap 32** against 1568 unbounded / 789 unbatched. 1.75× kept, not the full 2.0× |
| 3 | ★ nothing lost | ✅ `dup=0` every run |
| 4 | ★ t3→t4 >1 s at 0 | ✅ 0; max 37–89 ms |
| 5 | Full is retried, not dropped | ✅ adapter `-retry` until Ok; no discard path |
| 6 | no adapter buffer | ✅ one in-flight pending slot |
| 7 | 2×2 re-run | ✅ table above. Store share 1.29× at cap 16 |
| 8 | no substrate | ✅ `wat/`, `src/` empty |
| 9 | floor | ✅ 5185/5185, `FLOOR=0`, `.floor/2026-09-03T01-46-13Z/` |

## The poll's trigger fired

Bounding every stage restored latency and kept most of the 2×. Replace the 1 ms `-retry`
with a parked reply on the queue (conn-id, answer when there is room) — the workers'
repair, now that Outcome composes.
