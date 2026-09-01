# EXPECTATIONS — item (c) stone D

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **the bound holds** | failing sink, log far past `logs-max` | buffer never exceeds `logs-max`. **RED today** |
| 2 | ★ **the drop count is exact** | after overflow, drain against a working sink | a `:logs-dropped` metric with **exactly** the number dropped. An approximate count is a FAIL — it is the same class as item (b)'s off-by-one |
| 3 | the caller is told | the overflowing `log` | `:Dropped{buffered, cap}`, **never `Ok`** |
| 4 | the OLDEST go | after overflow, drain | the buffer held the most recent `logs-max`, in order |
| 5 | samples too | the same four against `timed` | `duration-samples-max`, `:samples-dropped` |
| 6 | counters untouched | `git diff` | no bound on `counters` (STOP-2) |
| 7 | the producer never blocks | a full buffer with a dead sink | `log` returns promptly; no stall (STOP-3) |
| 8 | the counter survives the condition | where the drop count lives | in `counters`, `O(1)` per key — never a Log or a sample (STOP-4) |
| 9 | under the bound, nothing changes | a small buffer | identical to today |
| 10 | prior gates hold | `cargo nextest run --release -E 'test(probe_arc278_span)'` | all pass; **assertions unedited** (mechanical Record-construction edits are fine) |
| 11 | no new surface op, no runtime change | op counts; `git diff --stat src/runtime.rs` | 5 and 6; empty |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5159+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 45–90 minutes. Two arms, two Record fields, two response variants; the
exact-count gate carries the cost.

## Trap doors, named in advance

- **A drop reported on one channel only.** Counter without response = invisible at the call site;
  response without counter = invisible to the operator. Rows 2 and 3 are both required, and neither
  substitutes for the other.
- **An inexact drop count.** Item (b)'s off-by-one wearing a different hat, and just as silent.
- **Dropping the arriving record instead of the oldest** — passes rows 1, 2, 3 and 7; only row 4
  catches it.
- **Bounding by bytes** — re-measures the whole buffer per call and answers the wire cap's question,
  not the memory one.
- **Firing on nothing:** rows 3–12 all pass if the bound is never reached because it was set
  enormous. Row 1 must drive genuinely past it.
