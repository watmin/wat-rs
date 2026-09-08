# BRIEF — an unexpected errno is not a clean drain

## The work, in one paragraph

`drain_signalfd` has four distinguishable outcomes and two silent exits: `EAGAIN` (done) and *any
other errno* (the fd cannot be read) both `break` and return `false`, and so does `n == 0`. The
worker reacts to `false` by polling again — so an unreadable-but-readable signalfd is a 100 % CPU spin
in which signals are never processed. Split the decision out as a pure function, name three outcomes,
and make the fatal one report and exit the way this file already does twice.

## Read in order

1. **`src/runtime.rs:305-343`** — `drain_signalfd`. The two `break`s at the end of the `n < 0` arm and
   the `n == 0` arm are the collapse.
2. **`src/runtime.rs:484-505`** — the worker loop. It polls again on `false`; this is why the collapse
   is a spin rather than a cosmetic issue.
3. **`src/runtime.rs:432-436`** and **`:446-451`** — **the exemplars.** `signalfd(2)` and `pipe2(2)`
   failures each write a raw diagnostic to fd 2 and `_exit(1)`; the second says *"_exit(2): fork-safe;
   same rationale as signalfd failure above."* Copy this handling exactly.

## Implementation sketch

```rust
/// What a signalfd read outcome means. Pure — no syscalls, so tests can
/// reach every arm without a broken fd.
#[derive(Debug, PartialEq, Eq)]
enum DrainStep { Done, Retry, Fatal }

fn classify_signalfd_read(n: isize, errno_kind: std::io::ErrorKind) -> DrainStep {
    // n > 0  -> caller dispatches the siginfo
    // EAGAIN/EWOULDBLOCK -> Done ; EINTR -> Retry ; anything else, and n == 0 -> Fatal
}

// in drain_signalfd, the fatal arm — the shape runtime.rs:432-436 already uses:
let msg = b"substrate: signalfd(2) read failed; the process can no longer observe stop signals\n";
unsafe { libc::write(2, msg.as_ptr() as *const _, msg.len()) };
unsafe { libc::_exit(1) };
```

## Blast radius

`src/runtime.rs` only — the new function, its `#[cfg(test)]` unit tests, and the drain loop. No new
files, no new dependency, no change to the demux or the mask.

## STOP triggers

**STOP-1** — if the errno decision cannot be made reachable by a unit test without constructing a
broken fd, STOP and say what blocks it. Making the invariant *gateable* is the point of the stone; a
fix that leaves it unprovable has changed the code without changing what can be proven.

**STOP-2** — do **not** add a retry budget, backoff, sleep, or spin counter. That converts a spin into
a slow spin and keeps the failure silent. If honest-or-fatal seems wrong for a case you find, STOP and
name the case.

**STOP-3** — if `EINTR` or `EAGAIN` end up in the Fatal set while restructuring the match, STOP. Those
are normal outcomes and turning them fatal would kill healthy processes.

**STOP-4** — on any red floor arm: capture it whole, name the arm, do not re-run.

## What "done" looks like

Unit tests call the classifier directly and pin all three outcomes, including `n == 0` → Fatal.
`a_signal_is_an_fd` still passes (SIGUSR1 flips its flag and does not stop; SIGTERM stops).
`partial_frame_residue` passes in ~3 s — read the duration. The floor Summary reads 5227 plus the unit
tests you added — **state that number explicitly** — with 22 skipped, 0 FAIL, 0 TIMEOUT, on a quiet
box. The SCORE should say plainly which errnos are now fatal and quote the diagnostic text.
