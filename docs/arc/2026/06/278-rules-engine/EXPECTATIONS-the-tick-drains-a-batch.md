# EXPECTATIONS — the tick drains a batch

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the topic actually batches | the circuit's `topic-ticks=` | **~200**, against **2000**. At K=10 and N=2000 the floor is 200 (STOP-4) |
| 2 | ★ nothing is lost | the circuit at `2000×4×3` | `total=8000; distinct=8000; dup=0` (STOP-1) |
| 3 | ★ the rebuild is amortised, not just the timer | `500×4×3`, `1000×4×3`, `2000×4×3` per-delivery | the slope must **nearly vanish**: today 1.75 → 2.32 → 2.40 ms (+0.65). The rebuild is the only known superlinear term, so amortising it by 10 should leave ≲ +0.1 ms (STOP-5) |
| 4 | ★ the topic stays interruptible | the circuit completes; `stop=` phase | `Admin::Stop` reached, `stop=` in the same range as today (~4.3 s), **not** growing with N. A drain-until-empty tick would show here (STOP-2) |
| 5 | one rebuild per tick | read the diff | the rebuild is **outside** the per-message loop, dropping K at once. Row 1 passes even if it is inside — row 3 is what catches that, and this row is why |
| 6 | the fan-out is still concurrent | `probe_async_publish::fanout_is_max_not_sum` | passes. The per-message four-send/four-recv shape is unchanged |
| 7 | drain drops | the circuit's `drain=` | **reported, not promised**, against 19.2 s. ~4 ms of the 9.57 ms/message is what is being amortised |
| 8 | no surface change | `git diff wat-scripts/topic/sns-fanout.wat` | `:demo::Sub` and `:demo::Topic` surfaces unchanged |
| 9 | blast radius held | `git diff --stat` | `sns-fanout.wat` only; `wat/`, `src/`, `sqs.wat`, `circuit.wat` empty (STOP-3) |
| 10 | receive calls unchanged | `queue-receive-calls` | ~7,100. This stone touches neither the park nor the wakeup |
| 11 | wall time | re-run | **reported, not promised**, against 33.3 s |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5184 tests |

**Runtime prediction:** 60–90 minutes. The loop is small; keeping the rebuild outside it, and the
`Option` facing on every indexed read, is the work.

## Trap doors, named in advance

- **Row 1 passes with the rebuild still inside the loop.** Ten deliveries per tick drops
  `topic-ticks` to 200 whether or not the rebuild moved — and the rebuild is ~1.87 ms of the ~4 ms.
  **Row 3 is the one that proves the expensive half**, and row 5 says where to look.
- **Drain-until-empty is the tempting simplification** and it will look *better* on rows 1, 3 and 7
  while making the topic deaf for the entire run. Row 4 is the guard, and it is weak — the circuit
  does not directly measure responsiveness — so STOP-2 asks for judgement, not just a green row.
- **K larger than the outbox.** At the tail the outbox holds fewer than K; the bound must be
  `min(K, length)` or the indexed reads run off the end. The `Option` from `vector::get` is the
  backstop, but an `Option/expect` that fires is a crash, not a guard.
- **Firing on nothing:** rows 2, 4, 6, 8–12 all pass with no change at all. **Rows 1 and 3 are the
  two that must move**, and they measure different halves.
