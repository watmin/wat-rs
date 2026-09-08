# DESIGN — a cancelled write reports what it delivered

**Measurement only.** One fact, then stop. This probe proposes no fix and writes no production code.

## Why this exists

`can an io-uring write be raced` (STRUCK, `9116db48d`) proved a parked `opcode::Write` can be
cancelled: PollAdd completes while the Write is outstanding, `AsyncCancel` retires it with
`ECANCELED(125)`, the CQ drains empty. Seven deterministic runs.

⚠ **That answer is scoped to a pipe with ZERO remaining room.** `fill_until_eagain` fills until a
1-byte write returns `EAGAIN`. `FINDING-the-writes-kept-the-1970s.md:48-53` records that the bug
lives in the *other* state:

```
send_poll_arm            completely full     POLLOUT never set    shutdown fires   ✅
partial_frame_residue    one frame of room   POLLOUT set          stuck            ⛔
```

**Every prior outcome we named assumed a full pipe.** Stone 3 would run at partial room.

## The question, and why it decides stone 3

`src/comms/process.rs:394-482` is the send loop. It is a **resume loop**:

```rust
let mut written = 0usize;
while written < framed.len() {
    …poll…
    let n = libc::write(fd, framed[written..].as_ptr(), framed.len() - written);
    …
    written += n as usize;          // :482 — the byte count is LOAD-BEARING
}
```

Under stone 1 the fd is `O_NONBLOCK`, so a short write returns `n` and the loop resumes at `written`.
Under stone 3 that `libc::write` becomes an `opcode::Write`, and the count has to come back in a CQE.

> **A Write of 8192 into a pipe with 4000 bytes free. The worker delivers 4000, blocks for the rest,
> the cancel arrives. Does the CQE carry `n=4000`, or `ECANCELED` — discarding it?**

★★ If the count is discarded the sender loses more than progress, it loses **knowledge**. It cannot
retry the frame (4000 bytes are already on the wire — the peer would see them twice) and it cannot
resume (it does not know where it stopped). The bytes are irrevocably in the pipe; only the *number*
is gone. That is an unrecoverable frame, and it is **strictly worse than what ships today**, in
exactly the regime the original bug lived in.

## What it delivers

One report naming, for a Write cancelled after a **partial** delivery:

1. the Write CQE's `result` — a partial count, `ECANCELED`, or neither
2. the `AsyncCancel` CQE's own result
3. whether the delivered bytes stayed in the pipe after the cancel

## The algorithm

1. Fill the data pipe to `EAGAIN` (the struck probe's `fill_until_eagain`, unchanged).
2. **Read back exactly `ROOM` bytes** to create a *known* amount of room. This is the one changed
   line, and it is the only read in the probe.
3. Submit `opcode::Write` of `PAYLOAD_LEN` (> `ROOM`, so it cannot complete) + a `PollAdd` on the
   stand-in broadcast. `submit()`.
4. Park 20 ms. Confirm the partial delivery: **`FIONREAD` grew by exactly `ROOM`** and **no Write
   CQE arrived**. That is the state no prior probe has built.
5. `AsyncCancel(WRITE_TOKEN)`, wait, drain, and report every CQE by `user_data`.
6. Report `FIONREAD` once more — the delivered bytes cannot be recalled.

## The one contract decision

**The probe reports; it never asserts which outcome occurred.** It asserts exactly two things: that
it *constructed* the state it claims (STOP-2), and that it *terminates* (the liveness bound). Every
outcome of the actual question is a pass. This is the same contract the struck probe held.

## Out of scope — REJECTED, not deferred

- **The migration itself.** Still a NOTE; its arc home is the builder's ruling.
- **`src/io.rs`.** It has no ring. Separate question.
- **`signalfd`.** The broadcast is already a pollable fd.
- **Revisiting stone 1.** `O_NONBLOCK` + poll is correct regardless, and is the standing answer if
  this probe says the count is lost.
- **A production Sender ring.** One fact, then stop.

## Files

One new file under `tests/`. `src/` untouched.
