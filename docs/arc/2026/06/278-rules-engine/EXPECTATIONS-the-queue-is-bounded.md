# EXPECTATIONS — the queue is bounded

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ latency recovers | the `e2e` histogram, **five runs** | max back to **~200 ms**, from 2.6 s (sqlite) / 36–42 s (mem). This is the whole point (STOP-2) |
| 2 | ★ batching's win survives | `8000 / (publish+drain) seconds` | **reported**, against 1568/s batched-sqlite and 789/s unbatched-sqlite. Keeping ~1500/s **and** 200 ms is the win; either alone is not |
| 3 | ★ nothing is lost | five runs at `2000×4×3` | `total=8000; distinct=8000; dup=0` **every time** (STOP-1) |
| 4 | ★ the queue depth is actually bounded | the `t3→t4` histogram | the `>1 s` bucket at **0**, and max small. Today ~7000 of 8000 sit `>1 s` |
| 5 | the refusal is faced, not dropped | read the adapter's `Full` arm | retries until accepted; **no path discards a message** |
| 6 | no buffering was added | `git diff` | no new accumulator in the adapter (STOP-2) |
| 7 | the 2×2 is re-run | mem vs sqlite, bounded queue | **both cells reported.** If the store's share falls back toward 1.19×, its quadratic writes are an oracle concern rather than a perf one |
| 8 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-4) |
| 9 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5185 tests |

**Runtime prediction:** 90–150 minutes. The cap check is small; the adapter's retry and the cap
sweep are the work.

## Trap doors, named in advance

- **Deadlock is the real risk and it is new.** Every stage blocking on the next is precisely the
  shape that can wedge — a worker waiting on a queue that is waiting on a store, with the adapter
  blocking the topic behind it. STOP-3 says capture and name it rather than re-running; the last
  deadlock in this arc took two stones to understand because the first report was a single hang.
- **A too-small cap turns the retry poll hot** and will show as throughput loss with good latency.
  A too-large cap is the reservoir again with good throughput and bad latency. **Rows 1 and 2 must
  both pass** — either alone is achievable by picking a bad cap.
- **Firing on nothing:** rows 3, 5, 6, 8, 9 all pass if the cap is set so high it never trips. Rows
  1, 2 and 4 are what require it to actually bind.
- **The retry poll is a known wart, not a finding.** Do not spend the stone replacing it; its
  replacement has a trigger and is a follow-up.
