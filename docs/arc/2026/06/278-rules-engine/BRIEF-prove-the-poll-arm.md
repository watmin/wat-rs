# BRIEF — cut the cap, PROVE the poll arm

**Stones:** `DESIGN-STONE-send-mirrors-recv.md` (as amended) and
`DESIGN-STONE-the-client-validates-locally.md` (why the cap left).
**Starting tree:** uncommitted work from a prior strike, two-thirds correct. **Do not revert it
wholesale** — you are cutting one part and proving another.
**Floor:** `4343 tests run: 4343 passed, 262 skipped`. The tree is currently RED with exactly one
regression (`probe_arc278_service_max_frame_bytes::large_foo_accepts_a_600kib_request`, a hang).
**Part 1 removes that regression.**

## Part 1 — cut the cap check (mechanical, ~15 min)

The transport cannot know which *op* is being sent, so it can never hold the right budget. The
check moves to the generated client method in a later strike; it does not belong here.

Delete, in `src/comms/process.rs`:
- the `max_frame_bytes` field on `Sender`
- `if framed.len() > self.max_frame_bytes { return Err(SendError::FrameTooLarge(value)); }`
- the field's threading at every construction site (`pair_with_budget`,
  `sender_receiver_from_fd_with_budget`, `sender_receiver_from_split_fds`, and the clone site)

And in `src/comms/mod.rs`, **delete the `FrameTooLarge` variant from `SendError`**. Nothing can
produce it once the check is gone, and an arm with no producer accumulates lies. `SendError` ends
as `Disconnected(T) | Shutdown(T) | Failed(T, String)`.

Keep `Receiver::max_frame_bytes` exactly as it is — the receiver's dismissal is the backstop and is
correct.

That alone should return the floor to `4343/4343/0/262`. Confirm it before Part 2.

## Part 2 — PROVE the poll arm (the substance)

**The poll arm has never been exercised.** The prior strike's only attempt reused a test that never
reaches the write loop, so it is evidence of nothing. Right now we have a mechanism in the tree that
nobody has shown works.

Write a probe that proves: **a `comms::process::Sender::send` blocked on a full pipe wakes and
returns `SendError::Shutdown` when the substrate shutdown broadcast fires.**

### ⛔ THE TEST SHAPE IS THE HARD PART — copy it, do not invent it

`tests/channel/probe_arc170_writer_joins_lockstep.rs` already solves this exact problem for
`PipeWriter::write`. **Read its module header and its body before writing a line.** It does, and you
must do, all of:

- **Fill the pipe deliberately**: toggle `O_NONBLOCK` on the write fd, write 4 KiB chunks until
  `EAGAIN`, restore blocking mode. The kernel buffer is then exactly full.
- **Block on a BACKGROUND thread**, never the test's main thread.
- **Rendezvous** on a `bounded(0)` channel — the writer signals readiness immediately before it
  calls `send`.
- **Collect the outcome through a BOUNDED `recv_timeout`, NEVER a raw `.join()`.** Its own words:
  *"so this probe itself can never hang even while the defect is present."*

**A probe for a hang must be structurally incapable of hanging.** A raw `.join()` on a thread that
may never return is the same defect you are testing for, relocated into the test. The prior strike
deadlocked; do not repeat it.

- **Arm the broadcast**: `init_shutdown_signal()` must run, or `SHUTDOWN_BROADCAST_READ_FD` is `< 0`
  and your poll degenerates to `POLLOUT`-only — which waits forever with nothing able to wake it.
  That is a *different* bug and would make your probe prove the opposite of what it claims. Assert
  the fd is armed before you rely on it.
- **Signal a CHILD, never `raise`.** `libc::raise` is condemned in this project (a self-directed
  signal makes the measurer and the measured one process). Use the shape at `tests/cli/wat_cli.rs:799`
  — `libc::kill(child.id(), SIGTERM)` — or trigger the broadcast directly if that reaches the same
  arm; say which you used and why.

## ★ THE DELIBERATE BREAK

Remove the broadcast arm from `Sender::send`'s poll (leave `POLLOUT`), rebuild, run your new probe,
confirm **RED**. Restore byte-exact, confirm green. Report both with actual output.

**If it does NOT go red, your probe does not depend on the mechanism** — say so plainly. That is a
finding about the probe, not a pass, and it is more valuable than a green you cannot trust.

## Done means

- Cap check, `Sender` budget field, and `SendError::FrameTooLarge` all gone; the regression is gone.
- A probe that proves a blocked `Sender::send` wakes on the broadcast — and that **cannot itself
  hang**.
- The break went RED and the restore went green, both with real output.
- `cargo nextest run --release` Summary verbatim against `4343/4343/0/262`, arithmetic explained.
- `cargo clippy --release --all-targets` clean.
- Every STOP hit; if none, say so.

## STOPs

- **⛔ Do not put the cap back anywhere in `comms/`.** It belongs in codegen, later.
- **⛔ Do not use a raw `.join()`** waiting on the blocked writer thread.
- **⛔ Do not use `libc::raise`.**
- **⛔ Do not claim the poll arm works because the suite is green.** Only the deliberate break earns
  that claim.
- **⛔ If the probe deadlocks, STOP and report its shape** — where both sides parked, and whether
  the broadcast fd was armed. Do not iterate blindly against a hang.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the FOREGROUND and block on it. Do not commit,
push, or stash. If a verification is incomplete when you finish writing, say so plainly.
