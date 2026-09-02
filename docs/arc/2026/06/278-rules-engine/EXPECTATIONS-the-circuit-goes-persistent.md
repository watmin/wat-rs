# EXPECTATIONS — the circuit goes persistent

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ nothing observable changed | the circuit at `2000×4×3` | `total=8000; distinct=8000; dup=0` (STOP-1) |
| 2 | ★ the drain drops | the circuit's `drain=` | **reported.** Isolated says 30.7 s → 3.7 s of a 73.4 s drain, so ~46 s is the shape to expect — but this row is a **measurement, not a target** (STOP-4) |
| 3 | ★ both accumulators moved | `grep -n 'outbox <-\|outcomes <-' wat-scripts/topic/sns-fanout.wat wat-scripts/fanout/circuit.wat` | both `PersistentVector`; **zero** `:wat::core::Vector` on either |
| 4 | the `Option` is faced | `grep -n 'vector::get' wat-scripts/` | every site matched or `Option/expect`ed with a located message — none `_`-swallowed |
| 5 | the queue is untouched | `git diff --stat wat-scripts/queue/sqs.wat` | **empty** |
| 6 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-3) |
| 7 | held-worker untouched | `git diff wat-scripts/fanout/circuit.wat` | `:fanout::held-worker` unchanged |
| 8 | the scale matrix | `500×4×3`, `1000×4×3`, `2000×4×3` | all complete, all lossless. The per-delivery cost should now be **flat across N** rather than 4.9 → 7.5 → 9.2 ms — that is the real proof the cubic term is gone |
| 9 | receive calls unchanged | `queue-receive-calls` | ~8,052. This stone touches neither the park nor the wakeup |
| 10 | wall time | re-run | **reported, not promised**, against 87.3 s |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5183 tests |

**Runtime prediction:** 45–90 minutes. The change is mechanical; threading the type through every
`State` construction is the work.

## Trap doors, named in advance

- **Row 8 is the real acceptance criterion, not row 2.** A wall-time drop could come from anything;
  **per-delivery cost going flat across N** is what proves the cubic term specifically is gone. If
  drain falls but the slope stays, something else is also superlinear and that is the next stone.
- **A missed `State` construction** still type-checks if it happens to build the old container in a
  place the checker can unify — row 3 greps declarations, so also read the diff.
- **The isolated gain may not transfer.** 30.7 s was measured with no I/O interleaved. If drain
  barely moves, **that is a finding, not a failure** — it would mean the rebuild was overlapping
  with I/O rather than adding to it, and STOP-4 says report it rather than chase it.
- **Firing on nothing:** rows 1, 5–7, 9, 11 all pass if nothing is swapped at all. **Rows 3 and 8
  are what catch that.**
