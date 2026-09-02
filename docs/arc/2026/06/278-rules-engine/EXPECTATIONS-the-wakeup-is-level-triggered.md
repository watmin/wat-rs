# EXPECTATIONS — the wakeup is level-triggered

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the deadlock is gone | the circuit at `1000×4×3` **and** `2000×4×3`, park adopted | both **complete**. These are the exact sizes that hung (STOP-1) |
| 2 | ★ nothing is lost | the circuit | `total=8000; distinct=8000; dup=0` (STOP-5) |
| 3 | ★ the park is actually on | `grep -n 'wait-ns' wat-scripts/fanout/circuit.wat` | the worker's receive is **non-zero** (STOP-4) |
| 4 | ★ the invariant is asserted, not trusted | a probe that drives the queue through every arm and checks *waiters non-empty ⟹ an alarm is outstanding* | it **holds**, and the probe **fails** if the flag is forced wrong. Row 4 is why the flag is allowed to exist |
| 5 | ★ edge-triggering is gone | `grep -n 'was-empty?' wat-scripts/queue/sqs.wat wat-scripts/topic/sns-fanout.wat` | **zero**. Today: 2 sites, one per service |
| 6 | ★ one place decides | every `Outcome`/`SelfOutcome` in the two services takes its `arms` from the helper | no arm builds an `Alarm` inline. This is what makes row 5 stay true |
| 7 | the empty polls collapse | the circuit, `queue-receive-calls` | **< 20,000** against 141,297. `500×4×3` was 2012 for 2000, so the floor is ~1 per message |
| 8 | ticks do not amplify | the queue's `ticks` counter at `2000×4×3` | bounded — same order as messages, **not** as requests. The DESIGN's amplification risk, measured |
| 9 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-3) |
| 10 | held-worker untouched | `git diff wat-scripts/fanout/circuit.wat` | `:fanout::held-worker` unchanged; the sane circuit's row 2 still proves in-flight is load-bearing |
| 11 | the scale matrix | `12×4×3`, `500×4×3`, `1000×2×3`, `1000×4×2`, `1000×4×3`, `2000×4×3` | **all complete, all lossless.** The four that passed before must not regress |
| 12 | wall time | re-run | **reported, not promised**, against 85.3 s polling |
| 13 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5183 tests |

**Runtime prediction:** 2–4 hours. The helper is small; finding **every** return path in both
services is the work, and row 1 is a slow gate (two runs that previously hung).

## Trap doors, named in advance

- **A missed return path is the entire bug again**, and it will look like success at `500×4×3`. Rows
  1 and 11 exist because only weight exposes it. Row 6 is the structural guard: if any arm still
  builds an `Alarm` inline, a future edit re-opens the class.
- **The flag is the new hand-maintained invariant.** If it is ever wrongly `true`, the deadlock
  returns and looks identical. **Row 4 is not optional** — it is the price of being allowed to use a
  flag instead of the substrate repair.
- **Tick amplification** is the design's own named risk and row 8 measures it. Unbounded alarms would
  trade a deadlock for a leak.
- **Row 1 cannot be satisfied by luck.** These sizes hung repeatedly with named drivers, so a single
  green run is weak evidence — run each twice and say so. If one passes and one hangs, that is a
  **finding**, not a flake, and STOP-1 applies.
- **Firing on nothing:** rows 2, 9–13 all pass with `wait-ns 0` still in place. **Rows 1, 3 and 7 are
  what catch that**, which is why the park being on is its own starred row.
- **Row 12 is not a target.** The prize is ~58% of all hops, but contention and the untouched drain
  poller (~70,000 hops) both re-balance. A modest wall time with rows 1 and 7 green is a **successful**
  stone.
