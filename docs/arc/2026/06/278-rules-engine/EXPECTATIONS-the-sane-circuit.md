# EXPECTATIONS — the sane circuit

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the invariant holds | run the circuit | `total=8000; distinct=8000; dup=0` — unchanged (STOP-1) |
| 2 | ★ nothing lost at shutdown | the in-flight term removed from the drain condition | that variant **FAILS** with `distinct < 8000`. If it still passes, the condition is not doing the work (STOP-2) |
| 3 | `stats` reports depth | `pending` and `in-flight` | both present and accurate |
| 4 | no fixed iteration counts | `grep -n 'range 0 cap' circuit.wat` | gone; no replacement bound (STOP-3) |
| 5 | the worker is interruptible | `Admin::Stop` mid-run | stops promptly; no hang (STOP-4) |
| 6 | producer and consumers overlap | workers start before publish | yes |
| 7 | empty polls are gone | count receive calls | approaching the message count, not ~3× it |
| 8 | participation is healthy | the `workers=` field | not the 4-of-12 collapse |
| 9 | tallies come back via `Status::Stopped` | the collection path | no invented side channel |
| 10 | substrate untouched | `git diff wat/ src/` | empty |
| 11 | the wall time | re-run | **reported, not promised**, against 88.6 s |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, FLOOR=0 |

**Runtime prediction:** 120–180 minutes. The worker's restructure and the drain condition carry it.

## Trap doors, named in advance

- **`pending = 0` as the whole drain condition.** Loses in-flight messages silently, and only
  sometimes — the kind of bug that passes ten runs and fails the eleventh. Row 2 is the guard, and it
  works by *removing* the term and requiring a failure.
- **A "safety" iteration bound** to stop a hang. That converts a shutdown bug into a flaky
  under-count. Row 4.
- **A worker that loops internally** — faster to write, cannot be stopped. Row 5.
- **Tuning for a good number.** Row 11 says report it; a slower sane program is a better deliverable
  than a fast nonsensical one.
- **Firing on nothing:** rows 1, 3, 6, 9–12 all pass if the worker keeps its fixed count and merely
  gains a wait. Rows 4, 7, 8 are what catch it.
