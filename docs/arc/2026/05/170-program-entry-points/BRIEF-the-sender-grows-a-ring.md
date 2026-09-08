# BRIEF — the sender grows a ring

## The work, in one paragraph

`Sender` owns a raw `write_fd` and nothing else; `Receiver` owns a persistent `IoUring`. Give the
`Sender` a ring of its own and move its wait onto it: submit the `Write` and a `PollAdd` on the
shutdown broadcast together, and let whichever completes decide. The write's own tie-break —
*writable wins, so a dying process can still speak* — stops being a hand-written branch and becomes a
property of the multiplexer. Four measurement stones already establish that this is safe.

## Read in order

1. **`src/comms/process.rs:1288-1330`** — `uring_read_into_acc`. **This is the exemplar**: the
   submission shape, the `user_data` tagging, the buffer-lifetime SAFETY comment, and the `EINTR`
   retry on `submit_and_wait` you are mirroring.
2. **`src/comms/process.rs:1014-1040`** and **`:2044-2052`, `:2100-2106`, `:2124-2130`** — how the
   Receiver builds `IoUring::new(4)` at each of its construction sites, and how each site's error
   style matches its own signature. The Sender's four sites (`:614`, `:2055`, `:2108`, `:2132`) need
   the same treatment.
3. **`src/comms/process.rs:384-482`** — the send loop you are replacing. Note `:394` `let mut written
   = 0usize;` and `:482` `written += n as usize;` — **the resume loop stays**; only how it waits
   changes.
4. **`src/comms/process.rs:322-347`** — `NonblockGuard`. **It leaves with the wait.** See STOP-1.
5. **`tests/comms/probe_arc278_cancelled_partial_write.rs`** and
   **`probe_arc278_short_write_retry_sweep.rs`** — the measured behaviour this design rests on, and
   the shape for the new probe.

## Implementation sketch

```rust
// Sender gains one field, mirroring Receiver:
ring: RefCell<IoUring>,          // IoUring::new(4) at each of the four construction sites

// send(), per attempt — the resume loop is unchanged around it:
let write_e = opcode::Write::new(types::Fd(fd),
                                 framed[written..].as_ptr(),
                                 (framed.len() - written) as u32)
    .build().user_data(WRITE_TOKEN);
let poll_e  = opcode::PollAdd::new(types::Fd(broadcast_fd),
                                   (libc::POLLIN | libc::POLLHUP) as u32)
    .build().user_data(BROADCAST_TOKEN);
// broadcast_fd == -1 (bootstrap) → submit the Write alone.

// SAFETY: `framed` is a live Vec on send()'s stack and outlives every wait below.
unsafe { ring.submission().push(&write_e)?; ring.submission().push(&poll_e)?; }
loop { match ring.submit_and_wait(1) { Ok(_) => break,
       Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue, Err(e) => …, } }

// Whichever completed decides. Tag by user_data, never by order.
//   WRITE, n > 0  → written += n; drain the PollAdd; loop
//   WRITE, n < 0  → the errno mapping EXACTLY as it reads today
//   BROADCAST     → AsyncCancel(WRITE_TOKEN), drain, Err(SendError::Shutdown(value))
```

## Blast radius

`src/comms/process.rs` only — the `Sender` struct, its four construction sites, `send`, the removed
`NonblockGuard` use, and the module header at line 10. Plus one new probe under `tests/comms/`.
**No `src/io.rs`. No `try_send`. No new dependency** (`io_uring` is already imported here).

## STOP triggers

**STOP-1** — the send fd must be **blocking** when the Write is submitted. Every measurement behind
this stone was made on a blocking fd; with `O_NONBLOCK` set, io_uring returns `-EAGAIN` instead of
parking and the design's premise evaporates. If removing `NonblockGuard` from this path looks unsafe
for a reason the design did not anticipate, **STOP and surface that reason.**

**STOP-2** — if a cancel ever follows a Write that reported a **non-zero** delivered count, STOP.
The premise (measured: room `> 0` completes immediately; room `== 0` delivers nothing) is broken, and
the whole design needs re-deciding rather than patching.

**STOP-3** — if `SendError`'s variants, errno mapping, or message strings need to change, STOP and
surface it. The wait is in scope; the contract is not.

**STOP-4** — if a construction site cannot report an `IoUring::new(4)` failure through its own
signature, STOP and name the site. Do not silently `unwrap` where the Receiver's neighbour returns a
`Result`.

**STOP-5** — on any red floor arm, STOP, capture it whole, name the exact arm, and surface it. No
test here is pre-blessed; a red is a red; do not re-run it.

## What "done" looks like

`probe_arc278_partial_frame_residue` passes in **~3 s** (read the duration, not just the verdict — it
is the arm that hung), `probe_arc278_send_poll_arm` passes, and a new probe shows both halves of the
tie-break: with room available and a stop pending the send **still writes**; with the pipe full and a
stop pending it returns `Shutdown`. The floor's Summary line reads 5226 passed / 22 skipped / 0 FAIL
/ 0 TIMEOUT on a quiet box. The SCORE names what the new probe observed on the cancel path — in
particular that no cancel ever followed a delivered count.
