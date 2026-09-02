# EXPECTATIONS — accept, then fan out

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ publish returns before delivery | publish into a topic with a deliberately slow subscriber | returns promptly, not at subscriber speed. **RED today** |
| 2 | ★ nothing is lost | the circuit | `total=8000; distinct=8000; dup=0` |
| 3 | ★ the outbox term is load-bearing | remove it from the drain condition | that variant **FAILS** with `distinct < 8000`. If it still passes, the term is not doing the work (STOP-1) |
| 4 | refusal, not drop | fill the outbox bound | a distinct refusal reaches the caller (STOP-2) |
| 5 | an idle topic never ticks | a topic with nothing published | no ticks |
| 6 | outbox depth is observable | the circuit's drain | reads it |
| 7 | no substrate change | `git diff wat/ src/` | empty (STOP-4) |
| 8 | the phase split | re-run with instrumentation | **reported**, against `publish=24.3 s, drain=0.02 s` |
| 9 | wall time | re-run the circuit | **reported, not promised**, against 35.7 s |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, FLOOR=0 |

**Runtime prediction:** 90–150 minutes. The outbox and tick are transcriptions of the queue's; the
drain term and the bound carry the risk.

## Trap doors, named in advance

- **Forgetting the drain term.** The circuit stops while messages sit in the outbox and `distinct`
  comes back short — *sometimes*, depending on timing. Row 3 catches it by removing the term and
  requiring a failure; row 2 alone would pass on a lucky run.
- **Dropping on a full outbox** instead of refusing. Silent data loss with a caller standing right
  there who could have handled it.
- **A zero-duration timer.** The sane circuit found that a duration-0 `after` **never fires at
  process tier**. Use a non-zero delay; 1 µs works.
- **Firing on nothing:** rows 2, 4–10 all pass if `publish` still delivers synchronously and merely
  gains an outbox it never uses. Rows 1 and 8 are what catch it.
