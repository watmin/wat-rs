# EXPECTATIONS — the workers stop polling

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the empty polls are gone | the circuit, printing `queue-receive-calls` | **< 20,000**, against **144,485** today. Floor is ~800 productive receives (8000 msgs at `:limit 10`) plus one park-expiry per worker per `wait-ns` |
| 2 | ★ nothing is lost | the circuit | `total=8000; distinct=8000; dup=0` (STOP-2) |
| 3 | ★ the false comment is gone | `grep -n 'never completes' wat-scripts/fanout/circuit.wat` | **zero**. It asserts a finding verified false; leaving it re-justifies the thing being deleted |
| 4 | ★ the worker actually parks | `grep -n 'wait-ns' wat-scripts/fanout/circuit.wat` | the worker's receive is **non-zero**; `:limit 10` unchanged |
| 5 | shutdown stays bounded | the circuit's `stop=` phase | on the order of one `wait-ns` plus teardown — **not** 12 × `wait-ns` (the design's measurement says one park, regardless of J) |
| 6 | `Admin::Stop` still works | the circuit completes and reports tallies | no hang (STOP-3) |
| 7 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-1) |
| 8 | the held-worker is untouched | `git diff wat-scripts/fanout/circuit.wat` | `:fanout::held-worker` unchanged; row 2 of the sane circuit still proves in-flight is load-bearing |
| 9 | the phase split | re-run | **reported**, against `setup=8353 publish=2629 drain=70421 stop=2523` |
| 10 | wall time | re-run | **reported, not promised**, against 85.5 s |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5183 tests |

**Runtime prediction:** 45–90 minutes. The edit is small; the risk is entirely in whether the queue's
wake path behaves under 12 concurrent parked waiters at process locus, which is the one thing the
verification probe did *not* cover — it parked waiters on an **empty** queue and never woke them.

## Trap doors, named in advance

- **The wake path is the untested half.** `probe-parked-waiters-stop.wat` proved parked waiters can be
  *stopped*; it never proved they are *woken* correctly when a message lands, and certainly not with
  12 of them across 4 queues. Row 2 is the guard. If messages go missing, that is a real finding
  about the queue's `ReplyTo` wake and it is worth more than this stone.
- **A park that expires is not "no work".** The producer runs alongside the consumers now. An empty
  return must re-arm and receive again — treating it as "queue drained, stop" is exactly the bug the
  old fixture had, and it will show as `distinct < 8000` **intermittently**, which is the worst way.
- **Row 1 can be gamed by raising `wait-ns` to the run length.** That drives receive-calls toward the
  floor and makes shutdown terrible. Row 5 is what catches it.
- **Firing on nothing:** rows 2, 6–11 all pass if the worker keeps polling and merely gains a
  non-zero `wait-ns` it never benefits from. **Rows 1 and 3 are what catch that**, which is why they
  are starred and why row 1 carries a number rather than a direction.
- **Row 10 is not a target.** Removing ~136,000 hops at a measured ~154 µs is on the order of 20 s of
  hop time, but contention makes that a floor rather than a prediction, and drain may re-balance
  onto the drain poller (~70,000 hops), which this stone deliberately does not touch. A disappointing
  wall time with row 1 green is a **successful** stone and an informative one.
