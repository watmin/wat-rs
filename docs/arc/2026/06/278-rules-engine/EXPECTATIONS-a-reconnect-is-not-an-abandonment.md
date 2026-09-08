# EXPECTATIONS — a reconnect is not an abandonment

Written **before** the strike. A correctness fix; the gate is that n=2000 completes.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **n=2000 completes** | `… circuit.wat 2000 4 3 8192 true` | `total=8000;distinct=8000;dup=0` — no `drained-never` |
| 2 | `Exhausted` only from the bound | read all three ladders | **no** `Exhausted` in a `Lost` or `Closed` arm |
| 3 | the retry is bounded | read the new arms | rides the **same** elapsed bound as `DeadlineFired` — STOP-1 |
| 4 | `ack-exhausted` exists and is reported | summary, phases, failure string | present in all three, beside its two siblings |
| 5 | correctness untouched | n=500 / 1000 / 2000 | `distinct` = n×m, `dup=0` at every depth |
| 6 | the PersistentMap gain is kept | n=1000 | `drain` ≈ **1110–1170**, not back at 1365 |
| 7 | no-args unchanged | `… circuit.wat` | every pre-existing field identical |
| 8 | nothing else moved | `git diff` | `:fanout::worker` only; `:2075` and `vis` untouched |
| 9 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 10 | scripts load | `every_wat_scripts_file_loads` | green |
| 11 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Row 1 is the stone and it is allowed to fail.** If n=2000 still strands, row 4's counter is what
makes the next diagnosis possible — that is STOP-7, and it is a result, not a failure.

⚠ **Row 6 guards the previous stone.** The `PersistentMap` change is in the tree, uncommitted, and
its n=1000 gain (1365 → ~1115, **−18 %**) is real. This stone builds on it.

⚠ **Row 3 is the one that could go wrong quietly.** An unbounded reconnect-retry loop would pass row
1 by hiding a genuine peer death as an infinite retry. The bound is what makes the fix safe.

## Runtime prediction

**40–60 minutes.** Three ladders, two arms each, one counter threaded through the existing groove,
one file. n=2000 is ~1 minute when it passes and ~3.5 when it strands. The floor is the long pole.

## Trap-doors named in advance

- **Both `Lost` and `Closed`, in all three ladders — six arms.** Fixing the ack path and forgetting
  `seen-until` leaves check and mark still discarding on a reconnect.
- **The redialed peer must be the one retried with.** The current arms already produce a fresh peer
  and drop it; the fix is to use it.
- **`ack-exhausted` counts abandonments, `ack-retries` counts attempts.** Two different numbers;
  incrementing the wrong one type-checks.
- **Do not let the retry restart the elapsed clock.** The bound is measured from the first attempt,
  as `DeadlineFired` already does with `ack-start-ns`.

## What this stone does NOT claim

⚠ **It does not touch `vis`.** 1000 s on a ~40 s run is 25× the run and is separately illogical —
its own stone, deliberately after this one so this fix is judged on the bug and not on a masking
constant.

⚠ **It does not explain the slope.** It unblocks the n=2000 measurement that `the dedupe map stops
cloning itself` could not complete.

⚠ **It does not fix the map-clone class** — `circuit.wat:2075` and `wat/query/mem.wat` still carry it.
