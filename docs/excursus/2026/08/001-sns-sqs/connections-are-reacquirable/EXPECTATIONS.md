# EXPECTATIONS — connections are re-acquirable

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the soul is kept | `grep -n ':durable' wat-scripts/{fanout/circuit,topic/sns-fanout,queue/sqs}.wat` | every service that dials holds its `Address` in `:durable` |
| 2 | ★ a lost pipe is no longer fatal | `grep -c 'RecvOutcome::Lost' …` then read each arm | **zero** `Lost` arms resolve to `assertion-failed!`. Today: 20 of 20 do |
| 3 | ★ no Lost arm acks | read every `Lost` arm | none acks. An unknown-outcome request that is acked is lost forever (STOP-1) |
| 4 | ★ the mechanism is gated | the redial probe wired into the floor, as `probe_queue_visibility` was | `durable-addr=ok;before=yes;redial=yes;after=yes` runs in the floor, not by hand |
| 5 | nothing is lost at weight | the circuit at `2000×4×3`, **five runs** | `total=8000; distinct=8000; dup=0` every time (STOP-5) |
| 6 | no counter, no backoff | `git diff` | no attempt state anywhere (STOP-3) |
| 7 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-4) |
| 8 | throughput | `8000 / (publish+drain)` | **reported, not chased**, against 303–325/s. Reconnect is on the failure path and should cost nothing when nothing fails |
| 9 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5191 tests |

**Runtime prediction:** 2–3 hours. Twenty arms across three files, each needing the same shape and
each needing a judgement about what "do not ack" means at that site.

## Trap doors, named in advance

- **★ This stone cannot fully prove itself, and that is by design.** Nothing today can break a pipe
  while leaving the peer alive — that is precisely what the chaos stone will inject. So rows 1–4 prove
  the recovery path is *expressible, wired and gated*; **proving it FIRES belongs to the stone after
  this one.** Do not invent a fault to prove it early; say plainly that it is unexercised.
- **The tempting wrong move is acking on `Lost`** to "clean up". It looks tidy and it silently
  destroys the message. Row 3 is the guard.
- **A `Lost` inside a `foldl` over subscribers is the hard site.** Partial progress across a batch
  is real: some subscribers took it, one did not. Not-acking the whole row is correct and will
  redeliver to subscribers that already had it — **which S13's `Seen` absorbs.** That is the pieces
  composing, not a bug.
- **Firing on nothing:** rows 5–9 all pass with no change at all, because nothing currently fails.
  **Rows 1–4 are the stone.**
