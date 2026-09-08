# BRIEF v2 — the sender grows a ring

**v1 is NOT STRUCK.** The design landed, the load-bearing arms passed, and the red you captured was
handled exactly right — caught, named, not re-run, fixed by a named change. **The fix for that red
introduced a live-lock**, on the very path the red exercised. One arm has to go.

## The defect

`submit_and_wait_eintr` (`src/comms/process.rs:349-362`):

```rust
Ok(0) => continue,   // "0 CQEs: the wait returned without a completion"
```

**`submit_and_wait` does not return a CQE count. It returns the number of SQEs SUBMITTED** — it is
`enter(len, want, …)` with `len = self.sq_len()`, straight from `io_uring_enter`'s return value
(`io-uring-0.7.14/src/submit.rs`).

Measured on this box, against the pinned crate:

```
call 1 (SQ had 1 SQE)           -> Ok(1)     <- submitted count, not completions
call 2 (SQ empty, CQE waiting)  -> Ok(0)
call 3 (same)                   -> Ok(0)
CQEs still undrained: 1
```

So once the submission queue is empty, `submit_and_wait(1)` returns **`Ok(0)` immediately and
forever** while a completion sits undrained. `Ok(0) => continue` then spins at 100% CPU and never
reaches `drain_cqes`.

## Why it is reachable — it is the red's own path

`write_once`'s loop (`:425-451`) is correct: on an empty drain it `continue`s with the SQEs still in
flight. That is the right guard for the red. But the next iteration calls `submit_and_wait_eintr`
with an **empty SQ** — and that is exactly the state above.

```
iter 1: SQ has 2 SQEs -> Ok(2) -> return -> drain EMPTY (the red) -> continue
iter 2: SQ empty      -> Ok(0) -> continue -> Ok(0) -> continue -> ...  never returns
```

Two paths reach it: the empty drain at `:428`, and the "a CQE arrived but it was not ours" fall-through
at `:449`.

★ **The green floor does not disprove this.** A live-lock would have hit the 30 s wall as a TIMEOUT;
the floor had none. The empty-drain case is rare — that is why it took a signal landing inside
`io_uring_enter` to produce it once. **This is a green floor over a path that was not exercised**, and
it is the same shape as the original finding: the state that disproves the code is one no test builds.

## The change

**Delete the `Ok(0) => continue` arm.** `Ok(_) => return Ok(())` already covers it.

```rust
fn submit_and_wait_eintr(ring: &mut IoUring) -> std::io::Result<()> {
    loop {
        match ring.submit_and_wait(1) {
            Ok(_) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(e) => return Err(e),
        }
    }
}
```

This is correct because `submit_and_wait(1)` carries `min_complete = 1`: with an empty SQ it **blocks
until a completion is available**, then returns `Ok(0)`. The helper returns, the caller drains, and
`write_once`'s own empty-drain `continue` remains the guard for the signal case — now terminating,
because each iteration's wait genuinely blocks.

Keep the comment's insight; it was right about the mechanism and wrong about the number: a signal
interrupting `io_uring_enter` can present as a **wait that yields no completion**, which is why
`write_once:428` retries. That guard stays.

## Read in order

1. **`src/comms/process.rs:349-362`** — the arm to delete.
2. **`src/comms/process.rs:425-451`** — `write_once`'s loop; unchanged, and the reason the deletion is
   safe.
3. **`io-uring-0.7.14/src/submit.rs`, `submit_and_wait`** — `let len = self.sq_len();` … `enter(len as _, want as _, …)`. The return is `len`, not completions.

## Blast radius

`src/comms/process.rs`, one match arm. Everything else in v1 stands.

## STOP triggers

**STOP-1** — if deleting the arm makes any probe hang, STOP and surface it. That would mean
`submit_and_wait(1)` is returning without blocking on an empty SQ, and the reasoning above is wrong.

**STOP-2** — do **not** add a spin-guard, a retry budget, or a timeout to paper over a wait. The wait
must block; if it does not, that is the finding.

**STOP-3** — on any red floor arm: capture, name the arm, do not re-run. As you did.

## What "done" looks like

`probe_arc278_partial_frame_residue` ~3 s (read the duration), `send_poll_arm` PASS,
`the_senders_tie_break_is_a_property` PASS, floor 5226 / 22 skipped / 0 FAIL / 0 TIMEOUT on a quiet
box. The SCORE should state plainly that the v1 arm could not terminate, and what replaced it.

## One secondary observation, not a blocker

`write_once` returns `Ok(WriteWait::Wrote(0))` when the Write CQE is `0` (`:463`), and `send` then
does `written += 0` and resubmits — an unguarded loop if a pipe write ever reports 0 for a non-empty
buffer. It should not happen; nothing currently stops it if it does. Name a decision either way.
