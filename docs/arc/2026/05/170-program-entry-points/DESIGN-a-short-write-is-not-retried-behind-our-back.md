# DESIGN — a short write is not retried behind our back

**Measurement only.** The table is the deliverable. No production code.

## Why this exists

`a cancelled write reports what it delivered` (v2, SCORED) measured **one point**:

```
ROOM=4096  PAYLOAD_LEN=8192  →  Write CQE at peek, result ok n=4096, AsyncCancel ENOENT
```

io_uring `Write` short-wrote and completed before there was anything to cancel. On that evidence the
resume loop at `src/comms/process.rs:482` keeps working and no delivered-but-uncounted state exists.

⚠ **That is one payload size.** io_uring carries re-issue logic for short I/O: a request that
transfers part of its buffer can be resubmitted for the remainder rather than completed. If that
engages at larger payloads, the CQE does **not** arrive at peek — the request parks **while holding
delivered bytes**. Cancel it and the count may come back as `ECANCELED`, and *that* is the corruption
state this line of work has been circling.

★★ **Two stones in a row were correct answers to questions whose boundary sat elsewhere.** The full
pipe hid the partial case; `ROOM=4000` hid that a sub-page readback frees no writer slot. The fix is
not more care in picking the point — it is **to stop picking a point.**

## What it delivers

One table. `ROOM` fixed at 4096 (one slot, `POLLOUT` set — the regime v2 established), `PAYLOAD_LEN`
**swept**, one fresh pipe per trial:

| payload | rationale |
|---|---|
| 8192 | v2's measured point — the control. It must reproduce, or the instrument moved. |
| 16384 | still small; 4× the room |
| 65536 | the whole pipe capacity on this box — cannot fit even into an empty pipe |
| 131072 | 2× capacity — well past any plausible single-shot transfer |

Each row reports: `POLLOUT` state, whether a Write CQE arrived at the 20 ms peek and with what
`result`, the `FIONREAD` growth, and — where the write parked — what the cancel returned.

## The three shapes, per row

- **CQE at peek, `result = 4096`** — a faithful short write at this size. Safe.
- **No CQE, `FIONREAD` grew by 4096** — ⛔ **parked while holding delivered bytes.** Then the cancel
  decides it: `4096` means the count survives; `ECANCELED` means the sender has bytes on the wire and
  no idea how many. That is the migration relocating the flaw.
- **No CQE, `FIONREAD` unchanged** — took nothing despite room. Report it.

## The one contract decision

**Every size in the sweep is run and reported, even if the first row looks conclusive.** A sweep that
stops early is a point measurement wearing a table's clothes — which is the failure this stone exists
to end.

## Out of scope — REJECTED

- The migration itself; `src/io.rs`; `signalfd`; revisiting stone 1. Unchanged from the prior stones.
- Sweeping `ROOM` as a second axis. One axis, so the table stays readable and the runtime stays small.

## Files

One new file under `tests/`. `src/` untouched.
