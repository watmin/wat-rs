# BRIEF — a short write is not retried behind our back

## The work, in one paragraph

Build one measurement probe that runs the **same** submit-peek-cancel sequence you have now written
twice, but as a **sweep over payload size** with one fresh pipe per trial, and reports a table. The
question: at what payload size, if any, does io_uring stop reporting the short count promptly and
instead park while holding delivered bytes? Every row is a pass; the table is the deliverable.

## Read in order

1. **`tests/comms/probe_arc278_cancelled_partial_write.rs`** — your v2 file. Its sequence is exactly
   what one trial does: fill → page-aligned readback → `POLLOUT` gate → submit Write+PollAdd → peek at
   20 ms → `AsyncCancel` → drain → report. Lift it into a `fn trial(payload_len: usize) -> Vec<String>`
   and call it four times.
2. **`tests/comms/probe_arc278_io_uring_write_race.rs`** — `fill_until_eagain`, `fionread`, `drain_cq`,
   `errno_name`, `fmt_result`, `token_name`, `pipe2_cloexec`, and the worker-thread + `recv_timeout`
   wrapper. Unchanged.
3. **`docs/arc/2026/06/278-rules-engine/SCORE-a-cancelled-write-reports-what-it-delivered-v2.md`** —
   the 8192 row you are reproducing as the control, and the report shape to match.
4. **`src/comms/process.rs:482`** — `written += n as usize;`. This is the loop the answer decides.
   You are not editing it.

## Implementation sketch

```rust
const ROOM: usize = 4096;                                   // one slot; POLLOUT set
const PAYLOADS: [usize; 4] = [8192, 16384, 65536, 131072];  // control, 4x, capacity, 2x capacity

fn trial(payload_len: usize) -> Vec<String> {
    // fresh pipe, fresh ring, every time — nothing inherited
    let (data_r, data_w) = pipe2_cloexec();
    let filled = fill_until_eagain(data_w_fd);
    let got    = /* read exactly ROOM */;
    // GATE: poll(write_fd, POLLOUT, 0) must be SET, else STOP-1
    // …push Write(payload_len) + PollAdd, submit(), sleep 20ms, drain_cq()…
    let delivered = fionread(data_r_fd) - fion_after_readback;
    // …AsyncCancel, wait, drain, drain again…
    // row: payload, filled, got, POLLOUT revents, CQE-at-peek?, its result,
    //      delivered, AsyncCancel result, Write final result, final drain
}

for p in PAYLOADS { report.extend(trial(p)); }   // ALL FOUR — see STOP-5
```

## Blast radius

**One new file: `tests/comms/probe_arc278_short_write_retry_sweep.rs`.** No `src/` changes. No new
dependencies. `build.rs` picks it up.

## STOP triggers

**STOP-1** — in any trial, if `POLLOUT` on the write fd is not set after the readback, STOP and
surface which payload's trial it was. That trial's state does not exist and its row would be a lie.

**STOP-2** — if answering the question appears to need a change under `src/`, STOP.

**STOP-3** — if you need to read the data pipe after a Write is submitted, STOP and report why.

**STOP-4** — if a trial cannot terminate under its liveness bound, STOP and report the parked shape
for that payload rather than raising the bound.

**STOP-5** — **do not stop the sweep early.** If an early row looks conclusive, the remaining rows are
still run and still reported. A truncated sweep is the exact failure this stone exists to end.

## What "done" looks like

The probe prints four rows, each naming its payload, its measured `filled`, its `POLLOUT` `revents`,
whether a Write CQE arrived at peek and with what `result`, its `FIONREAD` growth, and its cancel
outcome. Floor Summary reads 5224 passed / 22 skipped / 0 FAIL / 0 TIMEOUT, run on a quiet box.
Write the SCORE in the shape of the v2 score, with the table reproduced verbatim and a section
naming **which sizes, if any, parked while holding bytes**.
