# EXPECTATIONS — can an io_uring write be raced?

Written **before** the strike. A measurement probe: **the report is the deliverable**, not a verdict.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the probe reports** | `cargo nextest run --release <probe>` | names which `user_data` completed, in what order, with what results |
| 2 | the write really was blocked | read the probe's own output | bytes written < payload len, **and it says how it verified that** |
| 3 | cancellation is attempted and reported | same | `AsyncCancel` result stated, success or failure |
| 4 | the ring is drained and reported | same | what remained in the CQ is named, not discarded |
| 5 | it terminates | timing | **well under 1 s typical**; a liveness bound with a diagnostic message |
| 6 | measurement only | `git diff --stat` | **one new file under `tests/`**; `src/` untouched |
| 7 | the floor | `scripts/floor.sh` | **read the Summary line**: 5222 passed (5221 + this probe), 22 skipped |

⚠ **Row 1 is the stone, and every outcome passes it.** The probe is not required to show that
io_uring can race a write — it is required to **find out**. "The poll never completes while the write
is parked" is a full pass and the most valuable of the three answers, because it would mean the
migration relocates the flaw.

⚠ **Row 2 is what stops a wrong probe.** This arc has already committed one probe that measured its
own syntax error and reasoned from it across two artifacts. **State how you know the write blocked.**

⚠ **Row 5 protects the floor.** Three tests already sit at 19–25 s against a 30 s wall; the floor
goes red under load without a line of code changing. A slow probe makes that worse for everyone.

## Runtime prediction

**45–75 minutes.** One file, an unfamiliar opcode, and pipe-state engineering that the residue probe
shows is fiddly. The floor is the long pole.

## Trap-doors named in advance

- **`PIPE_BUF` is 4096.** A write at or under it is POSIX-atomic and will behave differently from one
  above it. The payload must exceed it or the probe measures the wrong thing.
- **Buffer lifetime across `submit_and_wait`.** The Read site's SAFETY comment says exactly why its
  buffer outlives the wait; a stack buffer freed early is undefined behaviour, not a failed test.
- **`user_data` is the only way to tell completions apart.** Two ops, two tags, checked — not assumed
  by order.
- **io_uring may run a blocking write on a kernel worker thread.** If so the CQE may simply never
  arrive; that is a *result*, and the bound is what turns it into a report rather than a hang.
- **A pipe filled with `EAGAIN` is full for the writer**, but a reader draining it changes that. The
  probe owns both ends; **do not let anything read.**

## What this stone does NOT claim

⚠ **It is not stone 3.** It buys the fact stone 3 cannot be designed without.

⚠ **It does not touch `src/io.rs`** — no ring there; separate question.

⚠ **It does not fold in `signalfd`** — the broadcast is already a pollable fd.

⚠ **It does not revisit stone 1.** `O_NONBLOCK` + poll is correct regardless, and is the standing
answer if this probe says io_uring cannot race a parked write.
