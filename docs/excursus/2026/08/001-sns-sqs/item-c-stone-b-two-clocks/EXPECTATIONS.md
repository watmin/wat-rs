# EXPECTATIONS — item (c) stone B

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **the two clocks are independent** | a span that ONLY logs, over several intervals | logs flushed; **zero** metric writes. A shared timer passes every other row and fails only this |
| 2 | ★ **and the other way** | a span that ONLY counts | metrics flushed; **zero** log writes |
| 3 | the tick re-arms | one span, no client flush, observe log writes | **≥ 2** flushes — one tick proves arming, two proves re-arming |
| 4 | an idle span is silent | a span that never logs or counts, several intervals | **zero** writes, and no timer armed (STOP-2) |
| 5 | ★ no double-count survives the split | stone A's `incr ×3 → flush → incr ×2 → close` | **exactly 5**. The split is where this regresses |
| 6 | one path per accumulator | `grep -n 'flush-logs\|flush-metrics' wat/telemetry/span.wat` | each defined ONCE; called by its timer, its size trigger, and `close`. Two builders for one group is a FAIL |
| 7 | size triggers decoupled | crossing the logs cap | flushes logs only — metrics untouched. Today it flushes both |
| 8 | stone A's gates UNEDITED | `git diff tests/services/probe_arc278_span_buffered.*` | empty. If they needed editing, behaviour changed (STOP-4) |
| 9 | no armed flag in `:durable` | `git diff wat/telemetry.wat` | cadence fields yes; an `armed?` bool NO (STOP-3) |
| 10 | cadence is configurable | `span/start` with a non-default interval | honoured |
| 11 | no new surface op | `grep -cE '^   \([a-z-]+ \[self' wat/telemetry.wat` for Span | still **5** (`incr`/`timed`/`log`/`flush`/`close`); `-flush-*` are internal, not on the surface |
| 12 | no runtime change | `git diff --stat src/runtime.rs` | empty |
| 13 | time is I/O, not a sleep | read the new gates | bounded observe-until loops; `nap` via `select'` on a one-shot `after`. **A bare sleep-then-assert is a FAIL** — it is a guess, and guesses race |
| 14 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5143+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 60–120 minutes. The split is mechanical; the arming transition and the
time-bounded gates carry the cost.

## Trap doors, named in advance

- **One timer flushing both groups.** Simplest thing that passes rows 3–14. Rows 1 and 2 exist only
  for this, and they are the stone.
- **A second emit path for a group** — the timer building metrics itself instead of calling
  `flush-metrics`. Stone A's double-count, returning through the split. Rows 5 and 6.
- **Arming unconditionally** rather than on the transition — every span wakes forever. Row 4.
- **A sleep in a gate.** Passes locally, flakes on a loaded machine, and a flaky floor arm is the
  worst thing to ship here. Row 13.
- **Firing on nothing** — timers armed but never flushing, with `close` doing all the work. Rows 3
  and 1 catch it; nothing else does.
