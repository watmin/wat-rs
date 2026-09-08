# BRIEF — a cancelled write reports what it delivered

## The work, in one paragraph

Build one measurement probe that answers a single question: **when an `opcode::Write` has already
delivered part of its payload into a pipe and is then cancelled, does the completion carry the count
of bytes it delivered?** The struck probe next door proved a *fully blocked* Write can be cancelled;
this one constructs the state where the Write has **partially succeeded**, cancels it, and reports
what the CQE says. Every outcome is a pass — the report is the deliverable.

## Read in order

1. **`tests/comms/probe_arc278_io_uring_write_race.rs`** — the exemplar; clone its shape. Take
   `fill_until_eagain`, `fionread`, `drain_cq`, `errno_name`, `fmt_result`, `token_name`,
   `pipe2_cloexec`, and the worker-thread + `recv_timeout` wrapper as they stand. Its `run_probe`
   is the sequence you are modifying, and its closing `let _keep = (payload, ts, data_w);` is the
   buffer-lifetime discipline to keep.
2. **`docs/arc/2026/06/278-rules-engine/SCORE-can-an-io-uring-write-be-raced.md`** — the shape of the
   result you are producing, and the numbers your run will sit beside.
3. **`src/comms/process.rs:394-482`** — the send loop this decides. `:482` is `written += n as
   usize;`: the byte count is what the loop resumes from. This is *why* the question matters; you
   are not editing it.
4. **`docs/arc/2026/06/278-rules-engine/FINDING-the-writes-kept-the-1970s.md:48-53`** — the table
   showing that a full pipe and a partly-full pipe are different regimes, and the bug lives in the
   second.

## Implementation sketch

The struck probe's `run_probe`, with one insertion after the fill and the assertions retargeted:

```rust
const ROOM: usize = 4000;            // < PAYLOAD_LEN so the Write cannot complete
const PAYLOAD_LEN: usize = 8192;     // > PIPE_BUF, as before

let filled = fill_until_eagain(data_w_fd);
let fion_full = fionread(data_r_fd);

// THE ONE NEW MECHANIC — create a KNOWN amount of room. The only read in
// this probe; after this, nothing reads the data pipe again.
let mut sink = vec![0u8; ROOM];
let got = unsafe { libc::read(data_r_fd, sink.as_mut_ptr() as *mut _, ROOM) };
let fion_after_readback = fionread(data_r_fd);
// report: filled, fion_full, got, fion_after_readback

// …push Write(PAYLOAD_LEN) + PollAdd, submit(), sleep 20ms, drain_cq()…

let fion_after_submit = fionread(data_r_fd);
let delivered = fion_after_submit - fion_after_readback;
// The three branches, ALL of them reported:
//   delivered == ROOM  && no Write CQE  → parked after a PARTIAL delivery  ← the target state
//   delivered == 0     && no Write CQE  → the kernel waited for full room  ← a real answer
//   a Write CQE present                 → it completed; state not constructed
```

Then `AsyncCancel(WRITE_TOKEN)` exactly as the exemplar does, drain, report every CQE by
`user_data`, and print `FIONREAD` once more so the delivered bytes are shown to be still in the pipe.

## Blast radius

**One new file: `tests/comms/probe_arc278_cancelled_partial_write.rs`.** No `src/` changes. No new
dependencies — `io_uring` and `libc` are already in the test tree. `build.rs` picks the file up.

## STOP triggers

**STOP-1** — if the readback does not leave exactly `ROOM` bytes of room (`fion_after_readback !=
fion_full - ROOM`, or `got != ROOM`), **STOP and surface it.** Do not proceed with an approximate
pipe state and do not tune `ROOM` until it looks right; an unknown state measures nothing.

**STOP-2** — if answering the question appears to require a change under `src/`, **STOP.** This is a
measurement; the migration is a separate, unstarted stone.

**STOP-3** — if you find yourself needing to read the data pipe *after* the Write is submitted,
**STOP and report why.** That read changes the state being measured; the exemplar's `_keep_data_r`
comment records that nothing may drain it.

**STOP-4** — if the probe cannot be made to terminate under its own liveness bound, **STOP** and
report the parked shape. Do not raise the bound to make it pass.

## What "done" looks like

The probe runs isolated in well under a second, prints a report naming every CQE by `user_data`, and
the floor's Summary line reads 5223 passed / 22 skipped / 0 FAIL / 0 TIMEOUT. Write the SCORE in the
shape of `SCORE-can-an-io-uring-write-be-raced.md`, including a section stating **how you know the
delivery was partial**. Run the floor when the box is quiet; nothing else may be running beside it.
