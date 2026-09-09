# SCORE — the fan-out is concurrent

**STRUCK.** Executor: grok, 2026-09-02. Two folds, no select, no surface change.

```
Summary [ 352.122s] 5184 tests run: 5184 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T09-19-25Z/`

## Row 1 — the constructed proof

One topic, four subscribers that each sleep 200 ms, one message:

```
dt-ms=204;shape=max
```

Sequential is ~800 ms. Concurrent is max. `tests/services/probe_async_publish.rs::fanout_is_max_not_sum` and
`wat-scripts/scratch-pad/probe-fanout-is-max.wat` both print it. This is the only row that fails
on a fused send-then-recv.

## What landed

`wat-scripts/topic/sns-fanout.wat` `-deliver` only: send all four, then recv all four.
Every `SendOutcome` arm is named (`Sent`/`Closed`/`Stopped`/`Lost`). Recv uses `_` for
non-Message, as the old code did. First raw `kernel::send` to a defservice client in the
tree — no exemplar; the generator at `service.wat:2197-2302` is what was mirrored.

Client-side `:max-request-bytes` is skipped at this call site; the server-side guard still
fires. Accepted (DESIGN).

`circuit.wat`, `sqs.wat`, `wat/`, `src/` empty.

## Drain (row 2) — it dropped, not 4×

| N×M×J | drain before | drain after | per-delivery | was |
|---|---|---|---|---|
| 500×4×3 | 9611 | 3501 | **1.75 ms** | 4.81 |
| 1000×4×3 | 19622 | 9268 | **2.32 ms** | 4.91 |
| 2000×4×3 | 41429 | 19205 | **2.40 ms** | 5.18 |

~2.2× at weight, not 4×. The four chains are independent enough to drop drain from 41.4 s
to 19.2 s, and not independent enough for max(4). Topology finding, not a miss: STOP-4
is "if the drain does not drop." It dropped. Per-delivery stays ~flat at the new level
(1.75 → 2.32 → 2.40 against the old 4.9 → 7.5 → 9.2).

## The circuit at weight

```
queue-receive-calls=7238
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
setup=8351;publish=791;drain=19205;stop=4325;ticks=744
WALL_SEC=33.309
```

Receive calls 7238 against 8048 — concurrent bursts let `:limit 10` batch more. Park and
wakeup untouched. Invariant holds.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ actually concurrent | ✅ `dt-ms=204;shape=max` (four 200 ms sleepers) |
| 2 | ★ drain drops | ✅ **19.2 s** against 41.4 s |
| 3 | ★ nothing lost | ✅ `total=8000; distinct=8000; dup=0` |
| 4 | ★ every SendOutcome faced | ✅ Sent/Closed/Stopped/Lost; no `_` on send |
| 5 | two separate folds | ✅ send fold, then recv fold |
| 6 | no surface change | ✅ `:demo::Sub` / `:demo::Topic` messages and features unchanged |
| 7 | blast radius | ✅ `sns-fanout.wat` (+ the row-1 rust test and scratch-pad). circuit/sqs/wat/src empty |
| 8 | per-delivery slope | ✅ 1.75 / 2.32 / 2.40 ms, still ~flat, lower |
| 9 | receive calls | 7238 against ~8048 — batching from concurrent arrival, reported |
| 10 | wall time | **33.3 s** against 55.7 s — reported |
| 11 | floor | ✅ 5184/5184, `FLOOR=0`, `.floor/2026-09-02T09-19-25Z/` |

## Batching still cut

8,000 chain traversals → ~800 still wants two surface changes. Size it now that the
chain is no longer sequential.
