# SCORE — the circuit runs on sqlite

**STRUCK.** Executor: grok, 2026-09-03. The fixture itself moved. mem-store remains
the differential oracle in `wat-scripts/queue/sqs.wat`. The sqlite probe copy
was the same transform and is deleted.

```
Summary [ 354.058s] 5190 tests run: 5190 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T06-37-46Z/`

## The decision

Move. Durability made the store hot (FINDING 1.68×); S14's batch put is O(table)
on mem and linear on sqlite. After S14:

| | deliveries/s | e2e max |
|---|---|---|
| mem (S14) | 184–193 | **5.6–5.8 s** |
| **sqlite** | **311–325** | **651–748 ms** |

**1.65×** throughput, and the 5.6 s mem tail is gone. Same 2×2 the
wire-batching stone named: batching is correct on sqlite and harmful on mem.
The fixture should demonstrate the linear store.

Codemod: `fix-circuit-to-sqlite.wat`, applied to `circuit.wat`. Idempotent.
`probe-circuit-sqlite.wat` git-rm'd — it was the same file.

## Five runs, sqlite, `2000×4×3`

All `total=8000; distinct=8000; dup=0`.

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 24.613 s | 729 ms | **325** |
| 2 | 25.781 s | 748 ms | **310** |
| 3 | 25.807 s | 702 ms | **310** |
| 4 | 25.636 s | 707 ms | **312** |
| 5 | 25.814 s | 651 ms | **310** |

Median **312/s** against S14-mem 189/s.

## Floor red on the way in

`.floor/2026-09-03T06-31-36Z/`. Arm: `every_tracked_wat_parses` —
`probe-circuit-sqlite.wat — could not read: No such file`. Deleted from disk
without `git rm`; `git ls-files` still listed it. `git rm` is the rest of
"the probe is redundant". Not a flake.

## What landed

`circuit.wat` is sqlite-store throughout (inbox and subscriber queues).
`sns-fanout.wat` standalone gates stay mem. `sqs.wat` `:user::compute` is
still the mem/sqlite oracle. `wat/`, `src/` empty.
