# BRIEF v2 — a cancelled write reports what it delivered

**v1 is NOT STRUCK, and the fault is in v1's brief, not in the strike.** v1 executed exactly what it
was told, its STOP-1 checked exactly what it was given to check, and the state it constructed was
still wrong. Read this section before the work.

## What v1 measured, and why it is not the answer

v1 reported: *"the kernel waited for the full 8192 to fit — io_uring Write did not short-write."*
That conclusion is unsupported. **The pipe had no writable room at all.**

Linux pipes allocate capacity in **4096-byte slots**. Reading 4000 bytes out of a full pipe frees
4000 *readable* bytes but releases **no slot** — the page is still partly occupied, so the writer
gains nothing. Measured on this box, directly:

```
readback=4000   FIONREAD=61536  readable-freed=4000   POLLOUT=clear   TRUE writable room=0
readback=4095   …                                     POLLOUT=clear   TRUE writable room=0
readback=4096   FIONREAD=61440  readable-freed=4096   POLLOUT=SET     TRUE writable room=4096
readback=8192   FIONREAD=57344  readable-freed=8192   POLLOUT=SET     TRUE writable room=8192
```

So v1's Write parked with zero bytes delivered **because the pipe was still completely full to the
writer** — the same state the previous stone already measured. `FIONREAD` is a count of *readable*
bytes; `filled - FIONREAD` is not writable room, and v1's brief used it as though it were.

★ **The correct instrument was one function away in the file you already had.** `fill_until_eagain`
ends by proving fullness with a 1-byte write that must `EAGAIN`. The same question asked *after* the
readback would have caught this immediately.

## The corrected construction

1. `ROOM` becomes **4096** — page-aligned, so the readback actually releases a slot. `PAYLOAD_LEN`
   stays 8192, so the payload still cannot fit.
2. **The gate becomes `POLLOUT`, not `FIONREAD` arithmetic.** `poll(write_fd, POLLOUT)` must come
   back **SET** after the readback. That is the regime marker from
   `FINDING-the-writes-kept-the-1970s.md:51` — the stuck case is the one where *`POLLOUT` is set and
   the room is smaller than the frame*. Gate on the property that defines the regime.
3. Keep `FIONREAD` as *reporting*, since delivery is still measured by its growth. It is no longer
   the proof that room exists.

## Read in order

1. **`tests/comms/probe_arc278_cancelled_partial_write.rs`** — your v1 file, uncommitted and on the
   tree. It is right except for `ROOM` and the room-proof; keep everything else.
2. **`tests/comms/probe_arc278_io_uring_write_race.rs`** — `fill_until_eagain`'s closing 1-byte
   `EAGAIN` check is the shape of a real state proof.
3. **`docs/arc/2026/06/278-rules-engine/FINDING-the-writes-kept-the-1970s.md:48-53`** — the table
   whose second row (`POLLOUT` **set**, one frame of room) is the state to construct.

## Implementation sketch

```rust
const ROOM: usize = 4096;          // page-aligned: releases a slot. 4000 releases nothing.
const PAYLOAD_LEN: usize = 8192;   // > ROOM, so the Write still cannot complete

let filled = fill_until_eagain(data_w_fd);
let got = /* read exactly ROOM */;
let fion_after_readback = fionread(data_r_fd);

// THE GATE — the regime marker, not an arithmetic proxy.
let mut pfd = libc::pollfd { fd: data_w_fd, events: libc::POLLOUT, revents: 0 };
let np = unsafe { libc::poll(&mut pfd, 1, 0) };
// report np and pfd.revents; POLLOUT must be set, else STOP-1.

// …Write(PAYLOAD_LEN) + PollAdd, submit(), sleep 20 ms, drain_cq()…
```

## The three outcomes, all of them passes

- **A Write CQE arrives promptly with `result = 4096`** — io_uring `Write` short-writes the way
  `libc::write` does. The resume loop at `src/comms/process.rs:482` keeps working, and there is never
  a delivered-but-uncounted state to lose. Report it as the clean answer it is.
- **No Write CQE, `FIONREAD` grew by exactly `ROOM`** — parked *after* a partial delivery. Now the
  cancel matters: does the Write CQE carry `4096` or `ECANCELED`?
- **No Write CQE, `FIONREAD` unchanged, `POLLOUT` was SET** — the kernel took nothing despite room.
  A genuine and surprising result; report it plainly.

## Blast radius

The one file already on the tree. No `src/` changes. No new dependencies.

## STOP triggers

**STOP-1** — after the readback, if `POLLOUT` on the write fd is **not set**, STOP and surface it.
The parked-with-room state does not exist and nothing downstream means anything. Do not adjust
`ROOM` to make a number look right.

**STOP-2** — if answering the question appears to need a change under `src/`, STOP.

**STOP-3** — if you need to read the data pipe after the Write is submitted, STOP and report why.

**STOP-4** — if the probe cannot terminate under its own liveness bound, STOP and report the parked
shape rather than raising the bound.

## What "done" looks like

The probe states, in its own output: the true writable room it constructed, that `POLLOUT` was set,
which CQE arrived and with what `result`, and what `FIONREAD` did at each step. Floor Summary reads
5223 passed / 22 skipped / 0 FAIL / 0 TIMEOUT, run on a quiet box. Write `SCORE-…-v2.md` including a
section titled **how I know the pipe had writable room**.
