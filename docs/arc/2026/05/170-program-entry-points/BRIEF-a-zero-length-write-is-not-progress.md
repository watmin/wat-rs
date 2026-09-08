# BRIEF — a zero-length write is not progress

## The work, in one paragraph

`write_once` returns `Wrote(0)` when the Write CQE is exactly `0`, and `send` does `written += n` — so
a zero re-enters the loop and resubmits the identical Write, forever, reporting nothing. Make the state
unrepresentable: `WriteWait::Wrote` carries a `NonZeroUsize`, so the compiler forces `write_once` to
decide what `n == 0` means, and the decision is a typed error rather than a silent retry.

## Read in order

1. **`src/comms/process.rs:459-465`** — the three branches. `:465` is `return Ok(WriteWait::Wrote(0))`,
   the state to eliminate.
2. **`src/comms/process.rs:523-525`** — `while written < framed.len() { … Wrote(n) => written += n … }`.
   The loop condition guarantees a non-empty buffer, so `written += 0` is a pure spin.
3. **`src/comms/process.rs`, `try_send`'s short-write branch** (search `never torn frames on the wire`)
   — **the exemplar for the reasoning.** It refuses to loop on a short write and says why: *"rather
   than loop (which could spin against a still-full pipe), treat it as best-effort failure."* `send`
   should be no more willing to spin than `try_send` is.
4. **`src/comms/process.rs`, `enum WriteWait`** — the type that gains `NonZeroUsize`.

## Implementation sketch

```rust
enum WriteWait { Wrote(std::num::NonZeroUsize), Errno(i32), Shutdown }

// write_once, replacing the three-way split's zero case:
if n > 0 {
    // NonZeroUsize::new returns Option; the None case IS the decision below.
    if let Some(nz) = std::num::NonZeroUsize::new(n as usize) {
        return Ok(WriteWait::Wrote(nz));
    }
}
if n < 0 { return Ok(WriteWait::Errno(-n)); }
// n == 0 on a non-empty buffer: the kernel does not define this. Report it.
Err("io_uring Write returned 0 for a non-empty buffer — zero is not progress".into())

// send:
Ok(WriteWait::Wrote(n)) => written += n.get(),
```

The `Err(String)` arm already exists in `write_once`'s signature and already maps to
`SendError::Failed(value, reason)` in `send`, so no new variant and no new arm are needed.

## Blast radius

`src/comms/process.rs` only — the `WriteWait` enum, `write_once`'s three-way split, and `send`'s match
arm. The compiler will name every other site that needs `.get()`. No new dependency.

## STOP triggers

**STOP-1** — do **not** use `unreachable!()` or a panic for `n == 0`. POSIX not defining an outcome is
not a guarantee the kernel never produces one, and a panic on the transport's hot path is worse than a
typed error.

**STOP-2** — do **not** add a retry, budget, or backoff. A zero is not progress; waiting longer does
not make it progress. That is the whole content of the stone.

**STOP-3** — if the change requires a new `SendError` variant or touching `Disconnected` / `Shutdown` /
the existing `Failed` strings, STOP and surface it. Only the zero case is in scope.

**STOP-4** — if `try_send` appears to need changing, STOP. It is already correct and its comment is the
reasoning this stone adopts.

**STOP-5** — on any red floor arm: capture whole, name the exact arm, do not re-run.

## What "done" looks like

A literal `WriteWait::Wrote(0)` does not compile. `n == 0` produces `SendError::Failed` carrying a
reason that names the zero-length write. `send` adds only non-zero counts. `try_send` is untouched.
`partial_frame_residue` passes in ~3 s (read the duration), with `send_poll_arm`,
`the_senders_tie_break_is_a_property` and `a_signal_is_an_fd` green. Floor Summary reads 5235 plus any
tests you add — **state that number** — 22 skipped, 0 FAIL, 0 TIMEOUT, on a quiet box.

The SCORE should say which rung landed: the state is unrepresentable, not merely asserted against.
