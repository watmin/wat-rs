# EXPECTATIONS — a short write is not retried behind our back

Written **before** the strike. Measurement only: **the table is the deliverable.**

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **every swept size has a row** | the probe's own output | 4 rows: 8192, 16384, 65536, 131072 — none skipped, none merged |
| 2 | ★ **each row names whether a Write CQE arrived at peek, and its `result`** | same | `ok n=…`, or "no Write CQE", stated per row |
| 3 | **the control reproduces** | the 8192 row | Write CQE at peek, `ok n=4096` — v2's measured point, unchanged |
| 4 | **each trial really had a writer slot** | same | `POLLOUT` **SET** after the readback, `revents` printed, in **every** row |
| 5 | **delivery is measured per trial** | same | `FIONREAD` growth printed beside its before-value, per row |
| 6 | **where the write parked, the cancel is reported** | same | `AsyncCancel` result and the Write's final `result`, both named |
| 7 | **each trial gets a fresh pipe** | same | `filled` printed per row; no trial inherits another's state |
| 8 | **the ring is clean per trial** | same | final drain named per row |
| 9 | **it terminates** | timing | 4 trials × ~20 ms; well under 1 s total, with a liveness bound that prints a diagnostic |
| 10 | **measurement only** | `git diff --stat -- src/` | **empty** |
| 11 | **the floor holds** | `scripts/floor.sh` | Summary line: **5224** passed (5223 + this probe), 22 skipped, 0 FAIL, 0 TIMEOUT |

## The rows that carry the lesson

⚠ **Row 1 is the stone.** Two stones in a row measured a single point and answered a question whose
boundary was elsewhere. A sweep that stops at the first conclusive-looking row repeats that exactly.

⚠ **Row 3 is the instrument check.** If 8192 no longer reports `ok n=4096`, something about the
measurement moved and nothing else in the table can be trusted.

⚠ **Row 4 is v2's lesson kept.** `POLLOUT` set is the regime; `FIONREAD` arithmetic is not a proof of
writable room, and at `ROOM=4000` it read plausibly while the true room was zero.

## The outcome that stops the migration

⛔ Any row showing **no Write CQE at peek with `FIONREAD` grown by 4096** is a request parked while
holding delivered bytes. If its cancel then returns `ECANCELED` rather than a count, the sender would
have bytes on the wire and no way to learn how many — it can neither resume nor retry. **Report it
loudly; it is the most valuable answer in the table and it stops stone 3.**

## Runtime prediction

**25–45 minutes.** The sequence exists twice already; this wraps it in a loop over four sizes with a
fresh pipe per trial. The floor is the long pole.

## Trap-doors

- **A fresh pipe per trial is mandatory.** A trial that reuses a drained pipe measures a different
  regime than the one its row claims.
- **65536 is this box's capacity, not a constant.** Print the measured `filled` per trial; if it is
  not 65536, say so and keep the sweep's intent (a size at capacity and one past it).
- **4096 is this box's slot size.** Print what the readback actually bought.
- **A parked trial must still terminate** — cancel it, drain it, and move to the next size.
- **Buffer lifetime across `submit_and_wait`** — each trial's payload and `Timespec` outlive its waits.
- **`ECANCELED` may arrive as a second CQE.** Drain twice per trial, as all three prior probes do.

## What this stone does NOT claim

⚠ Not stone 3, and it does not decide stone 3's arc home — the builder's ruling.
⚠ Does not touch `src/io.rs`, fold in `signalfd`, or revisit stone 1.
⚠ Does not sweep `ROOM`. One axis.
