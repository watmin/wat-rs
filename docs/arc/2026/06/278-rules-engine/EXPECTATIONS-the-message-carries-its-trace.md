# EXPECTATIONS — the message carries its trace

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the invariant is untouched | the circuit at `2000×4×3` | `total=8000; distinct=8000; dup=0` (STOP-1) |
| 2 | ★ the instrument does not perturb its subject | **five** runs, compared against the nine already on record (12472 / 12712 / 12716 / 12830 / 17076 / 17538 / 17765 / 18422 / 24827) | median and spread in the **same range**. A shifted median means the trace is changing what it measures (STOP-2) |
| 3 | ★ every stage is reported as a SHAPE | the circuit's output | a histogram line per stage with the named buckets **and** a max. **A mean alone fails this row** — the mean is what hid this for a day |
| 4 | ★ the question is answered | the `t3→t4` line across the five runs | **states plainly whether pending residency is bimodal.** A run at drain≈12 s and one at drain≈20 s should differ visibly in the 250–1000 ms bucket if the park-timeout theory holds — and if they do **not**, say so: that kills the theory and redirects the counters |
| 5 | all five stamps present | read one outcome body | `seq\|t0\|t1\|t2\|t3`, `seq` first |
| 6 | no surface change | `git diff` on the three files | `Sub::DeliverRequest` / `Queue::SendRequest` / `Queue::StatsResponse` all unchanged (STOP-3) |
| 7 | sqs.wat is one line | `git diff --stat wat-scripts/queue/sqs.wat` | the `t3` stamp only |
| 8 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-4) |
| 9 | topic-ticks unchanged | the circuit | ~200 at N=2000. This stone changes no scheduling |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5184 tests |

**Runtime prediction:** 90–150 minutes. Stamping is trivial; parsing five fields back out per
outcome and bucketing 8000 × 5 intervals is the work.

## Trap doors, named in advance

- **Row 4 is the entire point, and it can be answered "no".** If `t3→t4` is flat across a fast run
  and a slow run, the park-timeout theory is dead and the queue counters are the wrong follow-up.
  **That is a successful stone** — a measurement that redirects is worth more than one that confirms.
- **Row 2 is the one that can invalidate everything else.** A 30× payload growth touching the store
  rows and the `edn::write` in `send` is not obviously free at scale, and "payload is free" came from
  a microbenchmark, not this circuit.
- **A mean will be tempting** because it is one number and fits the summary line. Row 3 exists
  because every aggregate in this arc has hidden the thing it was summarising.
- **Firing on nothing:** rows 1, 5–10 all pass with a trace that is recorded and never analysed.
  **Rows 3 and 4 are what require it to answer something.**
- **Five runs, not one.** Row 2 cannot be satisfied by a single sample, and the last stone's
  headline number did not survive its second run.
