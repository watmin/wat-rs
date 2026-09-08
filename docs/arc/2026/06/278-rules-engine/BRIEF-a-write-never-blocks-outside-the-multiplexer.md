# BRIEF — a write never blocks outside the multiplexer

Make the polled write non-blocking so the `poll` owns every wait. `src/io.rs` +
`src/comms/process.rs`.

## Read in order

1. **`FINDING-the-writes-kept-the-1970s.md`** — the mechanism, why it hid, and both files' own
   comments reasoning to the edge of it.
2. **`src/comms/process.rs:348-430`** — the `while written < framed.len()` loop: the poll, the
   `"writable wins ties"` break at `:393`, and the full-length `libc::write` at `:410`.
3. **`src/comms/process.rs:476`** — **`try_send`'s `O_NONBLOCK` toggle. This is the shape to copy**,
   including how it restores the original flags.
4. **`src/io.rs:665-724`** — the same pattern in the origin file: poll at `:669`, tie-break, and the
   full-length `libc::write` at `:719`.
5. **`src/io.rs:735-750`** — the caller's `while !remaining.is_empty()` loop, which already resumes
   from short writes.

## The work

**1. The write is non-blocking** in both polled paths. Either toggle `O_NONBLOCK` for the duration
(as `try_send` does) or set it once at fd construction if every user of that fd is poll-driven —
**state which you chose and why.**

**2. A short count is not an error.** `written += n` and loop; the next iteration re-polls with the
broadcast armed.

**3. `EWOULDBLOCK`/`EAGAIN` loops back to the poll**, never to a blind retry and never to an error.

**4. Restore the flags on every exit path** — success, `Shutdown`, `Disconnected`, `Failed`. A leaked
`O_NONBLOCK` on a shared fd is worse than the bug being fixed.

**5. Retire the two false comments.** `comms/process.rs:353` (*"THE BLOCKING IS NOT THE BUG"*) and
`io.rs:670` (*"deliberately unresolved… correct either way"*) both assert something that is only true
at full-pipe. Replace them with what is now true.

## Blast radius

`src/io.rs` and `src/comms/process.rs`. **Rust substrate — this is the first substrate stone of the
arc's perf line**, so a rebuild is in play and the floor is the real gate.

## STOP triggers

- **STOP-1** — if any fd made non-blocking is **shared** with a path that assumes blocking
  semantics, **STOP and name it.** That is a different design.
- **STOP-2** — **do not use `sigaction` or touch signal handling.** Stone 2.
- **STOP-3** — **do not introduce `io_uring` on the write path.** Stone 3, and possibly its own arc.
- **STOP-4** — floor red on **any** arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-5** — if `probe_arc278_partial_frame_residue` still hangs, **STOP and report its output.**
  The probe is the gate; a passing floor with that probe still red is not a pass.
- **STOP-6** — if the fix requires changing what `send` returns to callers, **STOP.** The typed
  outcomes (`Shutdown` / `Disconnected` / `Failed`) are a contract.
