# SCORE v2 — a cancelled write reports what it delivered

**SCORED.** Executor: grok, 2026-09-08. Measurement only. v1 is not
struck; v1's brief was wrong. Same file, `ROOM=4096`, gate is
`POLLOUT`. No production code.

```
Summary [ 493.811s] 5223 tests run: 5223 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T07-07-58Z/`

## THE FACT

**With a real writer slot (`POLLOUT` set), `opcode::Write` of 8192
short-writes `n=4096` and completes before cancel.** `AsyncCancel`
returns `ENOENT(2)` — the Write was already done. The resume loop at
`src/comms/process.rs:482` keeps working. There is no
delivered-but-uncounted state.

Isolated (`--nocapture`), 20.5 ms:

```
ROOM=4096; PAYLOAD_LEN=8192
fill: libc write-until-EAGAIN accepted 65536 bytes; FIONREAD=65536
readback: got=4096; FIONREAD after=61440 (was 65536); ROOM=4096 is a multiple of 4096
poll write_fd POLLOUT after readback: np=1 revents=0x4 POLLOUT_SET=true
peek 20ms after submit: 1 CQE(s); FIONREAD=65536 (after-readback 61440); delivered=4096 (ROOM=4096)
  peek CQE user_data=1 (Write) result=ok n=4096
delivery check: Write CQE present at peek result=ok n=4096
AsyncCancel CQE result: ENOENT(2)
Write CQE result (the question): ok n=4096 — arrived at peek, before cancel
FIONREAD after cancel=65536 (after-readback 61440, after-submit 65536, delivered=4096)
final drain: CQ empty
```

Floor: `PASS [ 0.049s] probe_cancelled_partial_write_reports_delivery`.

v1's `ECANCELED` + `delivered=0` was the full-pipe state again: 4000
readable bytes released no writer slot.

## How I know the pipe had writable room

After fill, `fill_until_eagain`'s 1-byte write is `EAGAIN` — full to
the writer. Readback of **4096** (one slot on this box): `got=4096`,
`FIONREAD` 65536 → 61440. Then `poll(write_fd, POLLOUT, timeout=0)`:
`np=1`, `revents=0x4`, **`POLLOUT_SET=true`**. That is FINDING:51's
regime (POLLOUT set, room smaller than the frame). STOP-1 did not fire.

`FIONREAD` is reported beside every step; it is not the proof of room.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ POLLOUT set after readback | ✅ `np=1 revents=0x4 POLLOUT_SET=true` |
| 2 | ★ Write CQE `result` named | ✅ `ok n=4096` at peek, before cancel |
| 3 | readback page-aligned | ✅ `ROOM=4096`, `got=4096` |
| 4 | delivery measured | ✅ FIONREAD 61440 → 65536, growth 4096 |
| 5 | cancel on both sides | ✅ AsyncCancel `ENOENT(2)`; Write `n=4096` |
| 6 | no unexplained FIONREAD loss | ✅ 65536 → 61440 (readback) → 65536 (delivery) → 65536 |
| 7 | ring left clean | ✅ final drain CQ empty |
| 8 | terminates | ✅ 20.5 ms isolated; 49 ms on the floor. Bound 2 s |
| 9 | measurement only | ✅ `git diff --stat -- src/` empty |
| 10 | the floor | ✅ `Summary [ 493.811s] 5223 tests run: 5223 passed (7 slow), 22 skipped` |

Did not migrate. Did not touch `src/`. Did not tune `ROOM` past the
briefed 4096.

## BLAST

Same uncommitted file: `tests/comms/probe_arc278_cancelled_partial_write.rs`.
No `src/` changes.
