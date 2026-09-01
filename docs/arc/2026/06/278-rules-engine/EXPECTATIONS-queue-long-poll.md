# EXPECTATIONS — long polling in `wat-queue`

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ a send wakes a parked receive | park with `wait-ns > 0`, then send | the waiter gets the message **with the visibility re-put applied** — identical to an immediate receive. **RED today** |
| 2 | ★ a parked receive times out | park, let `wait-ns` elapse | empty reply, queue keeps serving |
| 3 | ★ one receive path | `git diff` — find the shared fn | `receive`, `send` and the tick all call it. **Two builders is this stone's double-count** (STOP-1) |
| 4 | `wait-ns = 0` unchanged | every existing queue gate | passes **unedited** (STOP-2) |
| 5 | empty round-trips fall | count receive calls in a drain | materially fewer than the spin count |
| 6 | an idle queue is silent | a queue with no waiters | no ticks (STOP-4) |
| 7 | waiters are `:ephemeral` | `git diff` on the Record | no waiter state in `:durable` (STOP-3) |
| 8 | FIFO among waiters | two parked, one message | the first parked is served |
| 9 | the circuit | re-run it | `total=8000; distinct=8000; dup=0`; wall time **reported** against 88.6 s, not promised |
| 10 | substrate untouched | `git diff wat/ src/` | empty — the change is in `wat-scripts/` only |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, FLOOR=0 |

**Runtime prediction:** 90–150 minutes. Extracting the shared path and the waiter bookkeeping carry
it; the timer is a transcription of the span's.

## Trap doors, named in advance

- **Two receive paths.** The wake path is covered by no existing gate, so it can diverge for a long
  time silently. Row 3 and row 1 together are the only guard.
- **Waking a waiter without the visibility re-put** — it would receive a message that is still
  visible to another worker, and the circuit's `dup=0` would eventually break. Row 1 checks the
  re-put, not just delivery.
- **A tick that re-arms unconditionally** — every queue wakes forever. Row 6.
- **Editing an existing queue gate** to accommodate `wait-ns`. Row 4: it must be additive.
- **Firing on nothing:** rows 2, 4, 6–11 all pass if `wait-ns` is accepted and ignored. Rows 1 and 5
  are what catch it.
