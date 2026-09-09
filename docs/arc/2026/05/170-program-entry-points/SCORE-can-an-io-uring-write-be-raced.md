# SCORE — can an io_uring write be raced?

**SCORED.** Executor: grok, 2026-09-08. Measurement only. One new test
file. No production code. The report is the deliverable.

```
Summary [ 495.321s] 5222 tests run: 5222 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T06-19-37Z/`

## THE FACT

**The poll completes while the write is outstanding, and the write
cancels cleanly.** DESIGN row 1: the migration can deliver what it
promises. This stone does not write that migration.

Isolated (`--nocapture`), 20.4 ms:

```
fill: libc write-until-EAGAIN accepted 65536 bytes; FIONREAD=65536;
      1-byte nonblock write then EAGAIN (pipe is full for the writer)
peek 20ms after submit, before broadcast: 0 CQE(s); FIONREAD=65536
blocked-write check: Write CQE absent after 20 ms, FIONREAD unchanged,
      payload=8192 > PIPE_BUF, pipe proven full by EAGAIN.
      The Write is outstanding — that is the parked state.
after broadcast + Timeout(500ms) + submit_and_wait(1): 1 CQE(s)
  [0] user_data=2 (PollAdd) result=ok n=1
race: PollAdd completed while Write still outstanding — second op CAN win.
AsyncCancel: submit_and_wait(1) returned Ok
after AsyncCancel drain: 2 CQE(s)
  [0] user_data=4 (AsyncCancel) result=ok n=0
  [1] user_data=1 (Write) result=ECANCELED(125)
final drain: CQ empty
```

Floor: `PASS [ 0.032s] probe_io_uring_write_raced_against_polladd`.

## How the write was known to be blocked

1. Fill until `EAGAIN` (65536 bytes); a further 1-byte nonblock write
   is `EAGAIN`.
2. `FIONREAD` = 65536.
3. Submit `opcode::Write` of 8192 (`> PIPE_BUF`). Nobody reads.
4. After 20 ms: zero CQEs, `FIONREAD` still 65536. Bytes did not land.
   That is the parked state, not an immediate `-EAGAIN` CQE.

## Adjustments from the suggested shape

- Fill is raw `O_NONBLOCK` `libc::write` until `EAGAIN` (`try_send`-style
  toggle), not `Sender::try_send`. This probe is not the framed send path.
- Stand-in broadcast is a second `pipe2(O_CLOEXEC)`, not eventfd.
- A `Timeout` SQE (500 ms, `user_data=3`) rides with the wait so a parked
  Write cannot hang `submit_and_wait`. The test thread also
  `recv_timeout`s at 2 s with a diagnostic.
- `opcode::Write` **is** in the pinned `io-uring` 0.7.14 (`CODE=23`).
  `AsyncCancel` is present. STOP-1 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ the probe reports | ✅ PollAdd first (`user_data=2`, `n=1`); Write later `ECANCELED(125)` |
| 2 | the write really was blocked | ✅ EAGAIN + FIONREAD unchanged + no Write CQE for 20 ms |
| 3 | cancellation attempted and reported | ✅ `AsyncCancel` `n=0`; Write CQE `ECANCELED(125)` |
| 4 | ring drained | ✅ final drain CQ empty |
| 5 | terminates | ✅ 20.4 ms isolated; 32 ms on the floor. Bound 2 s |
| 6 | measurement only | ✅ `?? tests/comms/probe_arc278_io_uring_write_race.rs` only; `src/` untouched |
| 7 | the floor | ✅ `Summary [ 495.321s] 5222 tests run: 5222 passed (7 slow), 22 skipped` |

Did not migrate. Did not touch `src/io.rs`. Did not fold `signalfd`.
Did not revisit stone 1.

## BLAST

```
?? tests/comms/probe_arc278_io_uring_write_race.rs
```

No `src/` changes. build.rs picks the new file up automatically.
