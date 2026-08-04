# BRIEF — #71 STEP (a): PROBE the partial frame. Measure only; design nothing.

> **This brief buys ONE fact and stops.** Task #71 says `⛔ GROUND FIRST, DO NOT DESIGN FIRST`.
> Nothing here proposes a fix. If the probe shows the hazard is unreachable, that is a result and
> #71 closes. If it shows a hang, that is a different and worse finding than the task predicted.

## What is already ground — do not re-derive it

**The exact site.** `comms::process::Sender::send`, `src/comms/process.rs`. The frame is built once
(`edn_bytes + '\n'`) then pushed through `while written < framed.len()`. Inside that loop, when the
fd is NOT writable and the shutdown broadcast is readable, it does:

```rust
return Err(SendError::Shutdown(value));      // process.rs:401, INSIDE the write loop
```

`value` goes back to the caller. **The bytes already written do not.** If `written > 0` at that
moment the pipe holds `framed[0..written]` — a frame with no trailing newline and no owner.

**The tie-break matters and it narrows reachability.** The poll prefers writable: `if fds[0].revents
!= 0 { break; }` — a stop only wins when the write WOULD have blocked. So the hazard needs the pipe
to be FULL at a moment when `written > 0`, i.e. the frame did not fit in the free space.

**⚠ THE EXISTING PROBE CANNOT SHOW THIS, and that is the whole reason this one exists.**
`tests/comms/probe_arc278_send_poll_arm.rs` fills the pipe to capacity and *then* sends. Its first
poll finds the pipe already full, so it returns `Shutdown` with **`written == 0`** — no partial
frame. It proves liveness (the blocked send wakes) and is silent about framing. Copy its SHAPE; do
not copy its fill step.

**Grounding question (b) is ANSWERED — the thread tier has NO exposure, and by shape not by grep.**
`comms::thread::Sender::send` (`src/comms/thread.rs:78`) hands a whole `T` to a crossbeam channel.
There is no byte framing, no write loop, and no partial to leave behind — a value transfers or it
does not. It cannot even return `Shutdown`; its only error is `Disconnected`. **The hazard is
process-tier only.** Do not spend a probe on the thread tier.

## The one thing to build

A child-process probe, modelled on `tests/comms/probe_arc278_send_poll_arm.rs` (re-exec via
`current_exe()`, `CHILD_ENV` selector, READY/REPORT protocol on **stderr**, parent collects through
a bounded channel `recv_timeout` so the probe can never hang even when the subject does).

**The fill step is the one real difference — you must engineer `written > 0`:**

1. Fill the pipe to capacity with **valid line-framed EDN filler** (the existing probe fills with
   `0u8` bytes, which are not lines and which a `Receiver` cannot consume — you need filler the
   reader can drain).
2. Free a KNOWN, SMALL amount of space by consuming whole frames back through the `Receiver`
   (never by reading raw bytes — a raw read can split a line and manufacture the very corruption
   under test, which would be the instrument supplying the result).
3. `sender.send(payload)` where `payload` is comfortably LARGER than the space you freed, so the
   first `write(2)` lands SHORT and `written > 0`.
4. Parent fires a real SIGTERM. The send returns `Shutdown`.

## What to REPORT — this is the deliverable, not a pass/fail

Report each, as measured values, with no interpretation:

1. Did `send` return `SendError::Shutdown`? (If not, say what it returned — the setup failed.)
2. **Is there a partial frame in the pipe?** After the `Shutdown`, drain and report: total bytes
   present, and whether the bytes after the final `\n` are non-empty. Non-empty = the hazard is
   REAL and reachable.
3. **★ WHAT DOES THE READER DO?** This is the question #71 actually asks, and there are two very
   different answers — distinguish them:
   - **the writer's fd is still OPEN** (the realistic case: the sender got `Shutdown` and is
     unwinding, not dropping). Call `receiver.recv()` and see. It may **BLOCK FOREVER** waiting for
     a newline that is never coming. **If it hangs, that is the finding — and it is worse than the
     "silent mis-parse" the task predicted.** Use a timeout; report "blocked past N seconds",
     never let the probe hang.
   - **the writer's fd is DROPPED** → EOF mid-line. Report exactly what `recv()` yields: a named
     `RecvError`, a decode failure, a silently truncated value, or a swallow.
4. Anything you had to assume.

## ⛔ STOPs

- **⛔ Design nothing.** No fix, no proposal, no "we could…". Measure and report. #71 rules the cure.
- **⛔ Never let the probe hang.** Bounded `recv_timeout` on every collection; the child is
  disposable; the parent must always terminate. The subject may hang — the instrument may not.
- **⛔ Do not fill or drain with raw byte reads/writes on the DATA path.** Frames go through
  `Sender`/`Receiver`. An instrument that splits a line itself proves nothing about the subject.
- **⛔ Do not touch `F_SETPIPE_SZ`** — the coupling the design deliberately avoids; withdrawn twice
  already, and it is not yours to reopen.
- **⛔ Do not call the blocking a bug.** It is the intended backpressure (task #71 STOP-1, the
  builder's ruling). The framing residue is the subject; the block is the design.
- **⛔ Do not self-signal.** The stop arrives as a real SIGTERM from the PARENT, never
  `libc::raise` in the measuring process — the whole `85b789ac` reckoning was about this.

## Verify

`cargo nextest run --release -E 'test(<your test name>)'`, foreground. You may `cargo build
--release`. Do NOT run the full unfiltered suite; do not commit, stash, or touch git.
