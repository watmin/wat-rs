# EXPECTATIONS — the store injector has teeth

**Written BEFORE the strike**, from `be945ff12`.

## What this stone is graded on

That **`Accepted n == real-rows` is now held by the floor** — the invariant this session's biggest stone rests
on, currently guarded by nothing — and that no test sits near the wall.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ a floor test drives the proxy | `grep -rln faulting-store tests/**/*.rs` | **≥1** file — currently **0** |
| 2 | ⭑⭑ `n == real-rows` on drop-reply | read the harness + run it | asserted as an **equality**, and passing |
| 3 | `n == real-rows` on passthrough | same | asserted and passing |
| 4 | the fault provably fired | `drops-fired > 0` | present on the drop-reply test |
| 5 | die path | `TimedOut` **and** `real-rows == 1` | both asserted |
| 6 | ⛔ three tests, not one | `nextest -E 'test(store_injector)'` or equivalent | **3** tests |
| 7 | ⭑ each runtime STATED vs the 40 s wall | the SCORE | headroom per test, not bare seconds. ⚠ The requirement is the **statement**, not a line: the drop-reply test is predicted ~17 s and **≥1.6× headroom is acceptable**. Flag anything under 1.5× |
| 8 | ⛔ no count assertions | read the harness | only `drops-fired > 0`; everything else an equality |
| 9 | ⛔ probe and proxy unchanged | `git diff --stat -- wat-scripts/` | **EMPTY** |
| 10 | ⛔ no `src/` or `wat/` | `git diff --stat -- src/ wat/` | **EMPTY** |
| 11 | ⭑ 3× identical relations | three runs | relations identical (counts may move) |
| 12 | floor green, new count stated | Summary line | green; the count **will move off 5241** and must be said |
| 13 | clippy | `-D warnings` | exit 0 |
| 14 | ⭑ floor-time delta reported | vs **528.833 s** (my guarded run) | how much of the guard's ~30 s this spends |
| 15 | happy path | `2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |

## ⭑ Row 2 is the stone

Everything else is hygiene. `Accepted n == real-rows` is the entire content of `04726854e`, it is an equality
between what the queue **says** and what the store **holds**, and **nothing automated holds it today.** A green
floor that does not assert it is the same silence this campaign has now paid for four times.

## ⚠ What I will reject

- **One combined test** (row 6 / STOP-2) — 20.7 s measured against a 40 s wall this floor has already hit.
- **A count pinned** beyond `drops-fired > 0` (row 8 / STOP-4).
- **The probe or proxy edited** (row 9 / STOP-1).
- **Runtimes quoted as bare seconds** (row 7). The wall is the unit that matters here. ⚠ But I will **not** reject a test at 17–20 s — that was my own prediction; the row asks for the headroom to be *stated*, not for a number to be beaten.
- **`5241` reported as the final count** (row 12). This stone moves it.
- **Row 2 asserted as `n > 0`.** It is an equality or it is nothing.

## Runtime prediction

**30–50 minutes.** One harness file, three thin tests, no wat changes; the long poles are three runs for row 11
(~30 s each) and a floor.

## Trap-doors, ranked

1. ⛔ **One test at 20.7 s** (row 6).
2. ⛔ **`n > 0` instead of the equality** (row 2) — passes while wrong.
3. **Pinning `drops-fired`** (row 8).
4. **Editing the probe to make harnessing easier** (row 9) — it already returns Strings.
5. **Reporting the old floor count** (row 12).
