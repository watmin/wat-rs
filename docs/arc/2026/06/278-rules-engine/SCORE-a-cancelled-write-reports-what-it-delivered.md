# SCORE — a cancelled write reports what it delivered

**SCORED.** Executor: grok, 2026-09-08. Measurement only. One new test
file. No production code. The report is the deliverable.

```
Summary [ 495.527s] 5223 tests run: 5223 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T06-50-05Z/`

## THE FACT

**An `opcode::Write` of 8192 into a pipe with a known 4000 bytes of
room does not short-write.** It parks with **zero** bytes delivered,
then cancel returns `ECANCELED(125)` and still zero. The CQE does not
carry a partial count because there was never a partial delivery.

That is the third outcome named in EXPECTATIONS: the kernel waits for
the full payload to fit. It is not the `n=4000` resume-loop answer and
not a lost-count corruption. It changes stone 3's shape: an io_uring
`Write` is not a drop-in for `libc::write`'s short count.

Isolated (`--nocapture`), 20.4 ms:

```
ROOM=4000; PAYLOAD_LEN=8192
fill: libc write-until-EAGAIN accepted 65536 bytes; FIONREAD=65536
readback: got=4000; FIONREAD after=61536; expected got=4000 and FIONREAD=61536
peek 20ms after submit: 0 CQE(s); FIONREAD=61536 (after-readback 61536); delivered=0 (ROOM=4000)
partial-delivery check: FIONREAD unchanged and no Write CQE.
  The kernel waited for the full 8192 to fit — io_uring Write did
  not short-write the way libc::write does. A real answer.
AsyncCancel: submit_and_wait(1) returned Ok
after AsyncCancel drain: 2 CQE(s)
  [0] user_data=4 (AsyncCancel) result=ok n=0
  [1] user_data=1 (Write) result=ECANCELED(125)
Write CQE result (the question): ECANCELED(125)
FIONREAD after cancel=61536 (after-readback 61536, after-submit 61536, delivered=0).
final drain: CQ empty
```

Floor: `PASS [ 0.032s] probe_cancelled_partial_write_reports_delivery`.

## How the room was known, and how we know nothing landed

1. Fill until `EAGAIN`: `filled=65536`, `FIONREAD=65536`.
2. Read back exactly `ROOM=4000`: `got=4000`, `FIONREAD=61536` (`= filled - ROOM`). STOP-1 did not fire.
3. Submit Write of 8192. Nobody reads after that.
4. After 20 ms: zero CQEs, `FIONREAD` still 61536, `delivered=0`.
5. After cancel: still 61536. Bytes that never landed cannot be recalled; bytes that were already there did not disappear.

This is not the full-pipe probe re-run. The pipe **had** 4000 bytes of
room. The Write refused to take them.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ Write CQE `result` named | ✅ `ECANCELED(125)` — no partial count |
| 2 | PARTIAL delivery constructed | **third outcome.** Growth was 0, not `ROOM`. Named in advance as a pass |
| 3 | room was a known quantity | ✅ `got=4000`, `FIONREAD=61536 = 65536-4000` |
| 4 | cancel reported on both sides | ✅ AsyncCancel `n=0`; Write `ECANCELED(125)` |
| 5 | delivered bytes not recalled | ✅ FIONREAD 61536 before and after cancel (nothing landed) |
| 6 | no unexplained FIONREAD loss | ✅ 65536 → 61536 (the readback) → 61536 → 61536 |
| 7 | ring left clean | ✅ final drain CQ empty |
| 8 | terminates | ✅ 20.4 ms isolated; 32 ms on the floor. Bound 2 s |
| 9 | measurement only | ✅ `git diff --stat -- src/` empty; `?? tests/comms/probe_arc278_cancelled_partial_write.rs` |
| 10 | the floor | ✅ `Summary [ 495.527s] 5223 tests run: 5223 passed (7 slow), 22 skipped` |

Did not migrate. Did not touch `src/`. Did not tune `ROOM`. Did not re-run.

## BLAST

```
?? tests/comms/probe_arc278_cancelled_partial_write.rs
```

No `src/` changes. build.rs picks the new file up automatically.
