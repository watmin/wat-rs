# EXPECTATIONS — a cancelled write reports what it delivered

Written **before** the strike. Measurement only: **the report is the deliverable**, not a verdict.

Rows state **what must be true**, not where to look — the invariant finds what an instruction misses.
(That rule cost this arc a stone: `"no libc::signal( in src/"` found a second installer that
`"change child.rs"` could not.)

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the probe names the Write CQE's `result`** | run the probe | a partial count, `ECANCELED`, or "no Write CQE" — stated explicitly, whichever it is |
| 2 | **a PARTIAL delivery was really constructed** | the probe's own output | `FIONREAD` grew by **exactly `ROOM`** after submit **and** no Write CQE had arrived |
| 3 | **the room was a known quantity, not an approximation** | same | `FIONREAD` after the readback == `filled - ROOM`, both numbers printed |
| 4 | **the cancel is reported on both sides** | same | the `AsyncCancel` CQE result **and** the Write CQE result, each named by `user_data` |
| 5 | **the delivered bytes could not be recalled** | same | `FIONREAD` after the cancel is reported; the bytes that landed are still there |
| 6 | **nothing drained the data pipe after submit** | rows 2+5 agree arithmetically | the only read is the deliberate readback; no unexplained `FIONREAD` loss |
| 7 | **the ring is left clean** | same | final drain named — empty, or every leftover CQE listed |
| 8 | **it terminates** | timing | well under 1 s typical, with a liveness bound whose miss prints a diagnostic |
| 9 | **measurement only** | `git diff --stat -- src/` | **empty** — one new file under `tests/`, nothing else |
| 10 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5223 passed (5222 + this probe), 22 skipped, 0 FAIL, 0 TIMEOUT |

## The rows that matter most

⚠ **Row 1 is the stone, and every outcome passes it.** `n=4000` says stone 3 can preserve the resume
loop. `ECANCELED` with no count says the migration **relocates the flaw into stream corruption** —
the most valuable of the answers, and the one that would stop the migration.

⚠ **Row 2 is what stops a wrong probe.** A full pipe would give a parked write that delivered
*nothing* — that is the state the struck probe already measured, and re-measuring it here would look
like a result and be a repeat. **The growth must be exactly `ROOM`.**

⚠ **Row 3 exists because "roughly some room" is not a measurement.** Print both numbers.

⚠ **Row 8 protects the floor.** Three tests already sit at 19–25 s against a 30 s wall; the floor
reddens under load without a line of code changing.

## A third outcome, named in advance so it is not a surprise

The kernel may deliver **zero** bytes and wait for the full 8192 to fit. That is neither a partial
count nor a corruption risk — it would mean an io_uring `Write` does **not** short-write the way
`libc::write` does, which changes stone 3's shape entirely. **Report it plainly; it is a pass.**

## Runtime prediction

**30–50 minutes.** One file, cloned from a probe struck an hour ago; the only new mechanic is the
readback. The floor is the long pole.

## Trap-doors named in advance

- **`ROOM` must be smaller than the payload**, or the write completes and there is no parked state.
- **Pipe capacity is not a constant.** Use the measured `filled`, never a literal 65536.
- **The readback is the ONLY read.** After the Write is submitted, a read changes the state under
  measurement — the struck probe's `_keep_data_r` comment says why that is load-bearing.
- **`AsyncCancel` may race a completion.** Nothing drains the pipe after the readback, so the Write
  cannot finish; if a Write CQE appears before the cancel anyway, that is a result — report it.
- **Buffer lifetime across `submit_and_wait`.** The payload and the `Timespec` must outlive every
  wait; the struck probe ends with `let _keep = (payload, ts, data_w);` for exactly this.
- **`ECANCELED` may arrive as a second CQE** after the cancel's own. Drain twice, as the struck probe
  does.

## What this stone does NOT claim

⚠ It is **not** stone 3, and does not decide stone 3's arc home — that is the builder's ruling.
⚠ It does **not** touch `src/io.rs`, fold in `signalfd`, or revisit stone 1.
