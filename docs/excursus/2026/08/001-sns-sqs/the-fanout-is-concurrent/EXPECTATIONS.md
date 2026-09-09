# EXPECTATIONS — the fan-out is concurrent

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ it is ACTUALLY concurrent, proved by construction | a probe: one topic, **four subscribers that each sleep 200 ms**, publish ONE message, time the delivery | **~200 ms, not ~800 ms.** Sequential is sum, concurrent is max. This is the acceptance criterion; everything else can pass without it |
| 2 | ★ the drain drops | the circuit's `drain=` | **reported.** ~4.7 ms of the 5.18 ms per delivery is the chain, so order-of-4× is the shape — a **measurement, not a target** (STOP-4) |
| 3 | ★ nothing is lost | the circuit at `2000×4×3` | `total=8000; distinct=8000; dup=0` (STOP-1) |
| 4 | ★ every outcome is faced | read the diff | each `SendOutcome` arm named (`Sent`/`Closed`/`Stopped`/`Lost`); no `_`-swallowed send. `recv` may use `_` for the non-Message arms as the old code did |
| 5 | the sends precede the recvs | read the diff | **two separate folds.** A fused send-then-recv per peer is the old behaviour wearing new verbs — row 1 catches it, row 5 explains it |
| 6 | no surface change | `git diff wat-scripts/topic/sns-fanout.wat` | `:demo::Sub` and `:demo::Topic` surfaces unchanged; no new messages, no new features |
| 7 | blast radius held | `git diff --stat` | `sns-fanout.wat` only. `wat/`, `src/`, `sqs.wat`, `circuit.wat` empty (STOP-3) |
| 8 | per-delivery slope | `500×4×3`, `1000×4×3`, `2000×4×3` | all complete, all lossless; per-delivery still ~flat, now at a lower level. A slope that **reappears** means something new is superlinear |
| 9 | receive calls unchanged | `queue-receive-calls` | ~8,048. This stone touches neither the park nor the wakeup |
| 10 | wall time | re-run | **reported, not promised**, against 55.7 s |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5183 tests |

**Runtime prediction:** 60–120 minutes. The edit is ~20 lines; writing the tree's **first** raw
send/recv to a service peer, and facing four `SendOutcome`s correctly, is the work.

## Trap doors, named in advance

- **Row 1 exists because rows 2–11 all pass on a sequential implementation.** A fused
  send-then-recv per peer is exactly what the generated client already does, and it would look like
  a successful refactor. Four deliberately-slow subscribers and one message separate sum from max
  unambiguously; nothing else does.
- **The client-side cap guard is gone at this call site** — accepted in the DESIGN, with the
  server-side guard as the backstop. If the executor finds a reason it matters, that is STOP-2, and
  the fix is a generated send-only client method rather than a hand-rolled check.
- **Four in-flight replies mean four ways to hang.** If one subscriber never answers, the topic
  blocks in `recv` with three replies already queued. That is the same exposure as today (the
  sequential version blocks too), but it now happens with sends already committed — so a `Lost`
  peer must be faced, not assumed.
- **Row 2 is not a target.** If drain barely moves, the four chains were contending on something
  shared rather than running independently — a finding about the topology, and STOP-4 says report it.
