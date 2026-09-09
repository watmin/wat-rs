# DESIGN — a write never blocks outside the multiplexer

**`send`'s write becomes non-blocking; the `poll` owns all waiting.** `src/io.rs` +
`src/comms/process.rs`. Stone 1 of 3 from `FINDING-the-writes-kept-the-1970s.md`.

## WHY

Both files poll `[fd → POLLOUT, broadcast → POLLIN]`, and on `POLLOUT` issue
`libc::write(fd, buf, FULL remaining length)` on a **blocking** fd.

★★★ **`POLLOUT` promises one byte; the code demands all of them.** The kernel writes what fits and
blocks for the rest — and the shutdown broadcast is an **fd**, which cannot interrupt a write already
inside the kernel. `EINTR` could, but the handler is installed with `libc::signal()`, whose glibc
semantics auto-restart.

The RED: `probe_arc278_partial_frame_residue`, 20 s, `.floor/2026-09-08T04-10-37Z/`. Its sibling
`probe_arc278_send_poll_arm` **passed in 34 ms** in the same run, because it fills the pipe
*completely* — `POLLOUT` never fires and the shutdown arm works. **Partial room is the precondition
no other test builds.**

## ⛔ THE ONE CONTRACT DECISION — the multiplexer owns every wait

> **No unmultiplexed syscall may block.** The `poll` is the multiplexer; the write must not be a
> second, invisible wait outside it.

The write goes non-blocking. A short count returns to the loop, the loop returns to `poll`, and the
broadcast is watched across every wait.

★ **The resume machinery already exists.** `comms/process.rs` has `while written < framed.len()`;
`io.rs` returns `Ok(ret)` and its caller loops `while !remaining.is_empty()`. **Both files are
already prepared to resume from a partial write.** Blocking inside `write` buys nothing.

★★ **`try_send` in `comms/process.rs:476` already toggles `O_NONBLOCK`** for exactly this reason.
The pattern is proven, local, and in the same file.

## ⛔ WHY NOT EINTR

The instinct is to clear `SA_RESTART` via `sigaction` so the write returns `EINTR`. **That makes it
*usually* wake, which is worse than always failing:** the signal can land between the check and the
syscall, and the write blocks anyway. That race is the entire historical motivation for self-pipes
and `signalfd`.

⚠ `sigaction` is still worth doing — **stone 2** — but on its own merits, not as this fix.

## WHAT THIS IS AND IS NOT

⚠ **It is not the architectural fix.** The write path staying on `libc` while the read path is
`io_uring` is **stone 3**, and it removes the pattern rather than patching it.

⚠ **`O_NONBLOCK` must be restored**, and restored on every exit path including the error returns.
`try_send` shows the shape; a leaked `O_NONBLOCK` on a shared fd is a worse bug than this one.

## OUT OF SCOPE — REJECTED

- **`sigaction`** — stone 2.
- **`io_uring` for the write** — stone 3, and possibly its own arc: **the builder's ruling.**
- **The other blocking `libc` sites** (`runtime.rs` 5, `process/boot/` 5). Not on this path; census
  recorded in the FINDING.
