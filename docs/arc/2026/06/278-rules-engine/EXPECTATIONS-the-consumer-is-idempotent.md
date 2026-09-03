# EXPECTATIONS — the consumer is idempotent

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the detector can SEE a message duplicate | a probe that **forces** redelivery (tight visibility, slow ack) with dedupe **disabled** | `distinct` is **short** of the expected count — i.e. the duplicate is visible as a duplicate. Today both counters rise together and `dup` stays 0 |
| 2 | ★ the consumer absorbs it | the same probe with dedupe **on** | exactly the expected outcomes; the redelivered message produces **one** |
| 3 | ★ loss is still detected | a probe that drops a message | `distinct` short. **Row 1 must not be bought by making the invariant blind to loss** (STOP-2) |
| 4 | ★ identity comes from the publisher | read the diff | the id is the published message's, stable across retries and across subscribers — **not** the envelope's `sk` |
| 5 | dedupe is in the consumer only | `git diff wat-scripts/queue/sqs.wat` | **empty** (STOP-1) |
| 6 | nothing is lost at weight | the circuit at `2000×4×3`, five runs | `total=8000; distinct=8000` every time |
| 7 | the window was not widened | `grep -n 'vis-ns\|visibility-ns' wat-scripts/fanout/circuit.wat` | unchanged from the current values (STOP-3) |
| 8 | throughput | `8000 / (publish+drain)` | **reported, not chased**, against 149–161/s |
| 9 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-4) |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5189 tests |

**Runtime prediction:** 90–150 minutes. The dedupe is small; **rows 1–3 are the work**, because each
needs a fixture that deliberately produces the failure it is checking for.

## Trap doors, named in advance

- **Rows 1 and 3 are a pair and can be gamed individually.** An invariant that counts nothing
  detects no duplicates; an invariant that counts everything detects no loss. **Both must hold**, and
  each needs its own deliberately-broken fixture — the same discipline as the sane circuit's row 2,
  which proved a term load-bearing by removing it.
- **`dup=0` today is not a passing grade, it is a blind spot.** Do not report it as evidence.
- **The redelivery must be forced, not waited for.** A test that hopes for a race is the flake this
  stone exists to remove. Tight visibility plus a deliberately slow ack makes it deterministic.
- **Firing on nothing:** rows 5–10 all pass with no dedupe at all. Rows 1–4 are the stone.
