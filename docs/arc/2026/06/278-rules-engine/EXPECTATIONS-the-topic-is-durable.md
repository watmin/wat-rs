# EXPECTATIONS — the topic is durable

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ `Ok` means durable | a probe: publish, then read the topic's internal queue depth **before any delivery** | the message is **in the store**. Today it is in `:ephemeral` state and a crash loses it |
| 2 | ★ the unit is per-subscription | the internal queue after one publish to N subscribers | **N rows**, not 1. A single row re-delivers to everyone on one subscriber's retry |
| 3 | ★ a refused subscriber is retried, not dropped | a probe: one subscriber queue at `cap 0`/full, publish, then free it | the message **arrives after** the queue drains — via visibility expiry, with **no retry counter** in the diff (STOP-3) |
| 4 | ★ one stalled subscriber does not stall the others | same probe, N=2, one full | the healthy subscriber receives **immediately**; publish does not block |
| 5 | ★ the old outbox is gone | `grep -n 'outbox\|deliver-armed?\|arm-deliver\|-deliver' wat-scripts/topic/sns-fanout.wat` | **zero** hits. Two buffers in series is STOP-2 |
| 6 | ★ nothing is lost | the circuit at `2000×4×3`, **five runs** | `total=8000; distinct=8000; dup=0` every time (STOP-1) |
| 7 | the internal queue is an ordinary queue-service | read the wiring | a `queue::queue/start` instance with a `mem-store`, not a bespoke outbox (STOP-5) |
| 8 | throughput, as a side effect | `8000 / (publish+drain)` | **reported, not chased**, against 921–954/s. A durable write on the publish path is expected to cost; say how much |
| 9 | latency | the `e2e` histogram | **reported**, against max 152–197 ms |
| 10 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-4) |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5186 tests |

**Runtime prediction:** 2–4 hours. The deletion is large and mechanical; the wiring of a
second queue-service instance into the circuit, and rows 3–4, are the work.

## Trap doors, named in advance

- **Rows 3 and 4 are the ones that cannot be faked.** Everything else passes on a topic that writes
  durably and then delivers exactly as before. Retry-on-refusal and per-subscriber independence are
  what the `(message, subscriber)` row is *for*, and they need a subscriber that actually refuses.
- **`dup=0` still holds here and that is not evidence of correctness.** At-least-once permits
  duplicates; reliable IPC means nothing generates one. **When main-line item 3 injects loss this
  invariant must change** — do not treat today's `dup=0` as proof the design is exactly-once.
- **A durable write on the publish path will cost throughput.** That is expected and row 8 is
  reported, not chased. **Do not batch the publish write to recover it** — that is a separate
  decision with its own latency trade, and this arc has already been burned once by optimising
  before the design settled.
- **Firing on nothing:** rows 1, 6–11 all pass on a topic that keeps its outbox *and* adds a store
  write. Rows 2, 3, 4 and 5 are what require the design to actually change.
