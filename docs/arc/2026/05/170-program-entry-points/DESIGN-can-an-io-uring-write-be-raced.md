# DESIGN — can an io_uring write be raced?

**A probe: submit a `Write` that cannot complete, and find out whether a second op can win.** One
new test file. Stone 3's first step, and the fact the migration cannot be designed without.

## WHY A PROBE FIRST

The builder's ruling: *"the write path joins io_uring… this is a flaw we cannot walk past."* Agreed.
But three facts from the disk say the migration rests on an untested assumption:

```
opcode::PollAdd  ·  opcode::Read        ← the ENTIRE io_uring vocabulary in this repo
both live in comms/process.rs           ← src/io.rs has no ring at all
opcode::Write                           ← ZERO occurrences. This would be the FIRST.
the persistent ring is on the RECEIVER  ← the Sender has none (`process.rs:26`)
```

★★★ **The migration's whole promise is that an io_uring write is cancellable where a `libc::write`
is not.** That promise has never been tested here. If it does not hold — if a `Write` on a full pipe
parks in a kernel worker that a cancel cannot reach — then moving to io_uring **relocates the bug
into a place with less visibility**, which is worse than the `O_NONBLOCK` fix already shipped.

⚠ **This is exactly the assumption `examinare` says to probe before briefing.** I have been wrong
about this substrate's mechanics repeatedly in the last day — a generic local `fn`, a parametric
enum in a child, a `Peer` in a `:messages` type, four bands, one wrong probe. **A ten-line probe
costs minutes; a wrong migration of the transport costs the transport.**

## ⛔ THE QUESTION, STATED SO IT CAN COME BACK "NO"

> Submit `opcode::Write` for more bytes than a full pipe can take, plus a second op that becomes
> ready (the shutdown-broadcast fd going readable). **Does the second op complete while the write is
> outstanding — and can the write then be cancelled or abandoned without leaking the ring?**

Three outcomes, all informative:

| outcome | what it means for stone 3 |
|---|---|
| **the poll completes, the write cancels cleanly** | the migration delivers what it promises; design it |
| **the poll completes, the write cannot be cancelled** | io_uring buys *visibility* but not *cancellation*; the shape changes and must be designed around a write that outlives its caller |
| **the poll does not complete while the write is parked** | **the migration is not the fix.** io_uring would relocate the flaw, and the `O_NONBLOCK` + poll discipline already shipped is the correct answer |

★ **No prediction.** Eleven mechanisms died in this arc, several of which fitted better than this one.

## ⛔ WHAT THIS PROBE IS NOT

⚠ **Not a fix, and not a migration.** It writes no production code. It buys one fact and stops —
the same shape as `probe_arc278_partial_frame_residue`, which was also measurement-only and is what
found the original defect.

⚠ **Not a judgement on `src/io.rs`.** That file has no ring; giving it one is a larger question and
this probe does not touch it.

⚠ **Not `signalfd`.** The shutdown broadcast is already a pollable fd and `PollAdd` already consumes
it. Folding signals into the ring is a separate simplification with its own blast radius.

## SCOPE, DECIDED FROM THE DISK RATHER THAN HANDED BACK

- **`comms/process.rs` only** for the eventual migration — it already owns a persistent ring, so the
  Sender gaining one is an addition to a file that understands them. `src/io.rs` would need a ring
  built from nothing; that is its own arc-shaped question.
- **This probe touches neither** — it is a standalone test file.

⚠ **Whether stone 3 lives in arc 278 remains the builder's.** 278 is the rules engine; this is
transport substrate. The finding arrived through 278's perf line, and I am drawing here because that
is where the FINDING and both prior stones live — **not** because I have ruled on the arc.

## OUT OF SCOPE — REJECTED

- **Writing the migration.** It cannot be designed until this probe answers.
- **`src/io.rs`.** No ring; separate question.
- **`signalfd`.** Separate simplification.
- **Reverting stone 1.** `O_NONBLOCK` + poll is correct regardless of what this probe says, and is
  the fallback if the answer is the third row.
