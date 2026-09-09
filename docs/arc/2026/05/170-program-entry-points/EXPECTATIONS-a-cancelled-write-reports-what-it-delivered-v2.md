# EXPECTATIONS v2 — a cancelled write reports what it delivered

Written **before** the re-strike. Measurement only: the report is the deliverable.

⚠ **v1's rows passed while the measured state was wrong.** Row 3 asked that the room be "a known
quantity" and accepted `FIONREAD` arithmetic as the proof. `FIONREAD` counts *readable* bytes; the
writer's room is a different quantity, and at `ROOM=4000` it was **zero**. The rows below gate the
**regime**, not a proxy for it.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the pipe really had writable room** | the probe's own output | `POLLOUT` on the write fd is **SET** after the readback, and the probe prints `revents` |
| 2 | ★ **the Write CQE's `result` is named** | run the probe | `n=4096`, `ECANCELED`, or "no Write CQE" — whichever, stated |
| 3 | **the readback was page-aligned** | same | `ROOM` is a multiple of 4096, and `got == ROOM` |
| 4 | **delivery is measured, not assumed** | same | `FIONREAD` growth after submit printed with the before-value beside it |
| 5 | **the cancel is reported on both sides** | same | `AsyncCancel` result and Write result, each by `user_data` |
| 6 | **no unexplained `FIONREAD` movement** | rows 4+7 agree arithmetically | every change traces to the readback or a delivery |
| 7 | **the ring is left clean** | same | final drain named — empty, or every leftover listed |
| 8 | **it terminates** | timing | well under 1 s typical; a liveness bound whose miss prints a diagnostic |
| 9 | **measurement only** | `git diff --stat -- src/` | **empty** |
| 10 | **the floor holds** | `scripts/floor.sh` | Summary line: 5223 passed, 22 skipped, 0 FAIL, 0 TIMEOUT |

## The row that carries v1's lesson

⚠ **Row 1 is the one that would have caught v1**, and it is phrased as the regime marker rather than
as a measurement to be satisfied. `FINDING:51` defines the stuck case as *`POLLOUT` **set**, one
frame of room* — so `POLLOUT` set is the state, and anything that does not have it is a different
experiment wearing this one's name.

⚠ **Row 2 passes on every outcome**, including the one that makes the whole corruption worry moot:
a prompt `n=4096` means io_uring `Write` short-writes like `libc::write`, the resume loop at
`src/comms/process.rs:482` keeps working unchanged, and no delivered-but-uncounted state exists.

## Runtime prediction

**15–30 minutes.** Two constants and one gate change in a file already written and passing. The floor
is the long pole.

## Trap-doors

- **4096 is this box's slot size**, not a universal constant. Print what the readback actually bought.
- **`POLLOUT` set does not promise the whole payload fits** — that asymmetry is the original bug, and
  here it is the point rather than a hazard.
- **The readback stays the only read.** Nothing may drain the pipe after submit.
- **`ECANCELED` may arrive as a second CQE.** Drain twice, as both prior probes do.
- **Buffer lifetime across `submit_and_wait`** — payload and `Timespec` outlive every wait.

## What this stone does NOT claim

⚠ Not stone 3, and it does not decide stone 3's arc home — the builder's ruling.
⚠ Does not touch `src/io.rs`, fold in `signalfd`, or revisit stone 1.
