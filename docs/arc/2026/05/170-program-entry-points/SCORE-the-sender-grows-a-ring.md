# SCORE — the sender grows a ring

**SCORED.** Executor: grok, 2026-09-08. This stone changes `src/` on
the transport's hot path. Did not commit.

```
Summary [ 497.696s] 5226 tests run: 5226 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T09-32-15Z/`

A prior floor on the same tree, **before** the empty-wait retry, is
captured at `.floor/2026-09-08T09-19-50Z/`. It was not re-run.

## WHAT LANDED

`Sender` owns `ring: RefCell<IoUring>` (capacity 4), matching
`Receiver`. Four construction sites mint it (`pair_with_budget`,
`sender_receiver_from_fd_with_budget`, `sender_receiver_from_split_fds`)
or move it (`reinterpret`). `IoUring::new(4)` failure is a `Result` at
the three minting sites, same style as the neighbouring Receiver.

`send()` is a resume loop around one Write SQE plus, when
`SHUTDOWN_BROADCAST_READ_FD >= 0`, a PollAdd on the broadcast. Tag by
`user_data`. Write n > 0 → `written += n`, cancel a still-outstanding
PollAdd, loop. Write n < 0 → the errno mapping as it read
(`Interrupted`/`WouldBlock` retry, `BrokenPipe` → `Disconnected`, else
`Failed`). Broadcast first → AsyncCancel the Write; a completing Write
(n > 0, cancel ENOENT) is honored; `ECANCELED` is `Shutdown`. Bootstrap
(`broadcast == -1`) submits the Write alone.

`NonblockGuard` is gone from `src/comms/process.rs`. The send fd is
blocking when the Write is submitted. `try_send` still toggles
`O_NONBLOCK` itself. Header line 10 names `io_uring Write`. `Sender` is
still not `Clone`. `raw_fds` is `[write_fd, ring_fd]`.

`submit_and_wait` retries `EINTR` **and** `Ok(0)` / an empty drain.
That is the EINTR-equivalent the old `poll` loop had, which
`io_uring_enter` does not always surface as `EINTR` when a signal
arrives.

## THE FIRST FLOOR WAS RED — captured, not re-run

```
Summary [ 498.901s] 5226 tests run: 5225 passed (7 slow), 1 failed, 22 skipped
```

`.floor/2026-09-08T09-19-50Z/`

Arm, verbatim:

```
FAIL [   0.026s] wat::comms probe_arc278_send_poll_arm::probe_sender_send_wakes_on_shutdown_broadcast
expected Err(SendError::Shutdown(_)) carrying "x" — the poll arm firing on
the substrate shutdown broadcast; got "REPORT:Other:Err(Failed(\"x\",
\"io_uring wait returned no Write and no broadcast CQE\"))"
```

SIGTERM during `submit_and_wait(1)` returned a wait with no Write CQE
and no broadcast CQE. The old poll path retried `EINTR`; the ring path
treated an empty drain as `Failed`. Isolated, the same probe had passed
in 0.01 s — a 0-CQE wait, not a missing PollAdd. The empty-wait retry
is the fix. The red was not re-run; the next floor is a new run after
that named change.

## THE LOAD-BEARING ARMS

| probe | isolated | floor |
|---|---|---|
| `probe_arc278_partial_frame_residue` | **3.01 s** PASS | **3.037 s** PASS |
| `probe_arc278_send_poll_arm` | 0.01 s PASS | **0.021 s** PASS |
| `the_senders_tie_break_is_a_property` | 0.01 s PASS | **0.021 s** PASS |

Row 1 is duration, not just verdict. 3 s, not 20 s.

## THE NEW PROBE

`tests/comms/probe_arc278_sender_grows_a_ring.rs`, one `#[test]`.

- **Bootstrap** (parent, `broadcast == -1`): `pair` + send `"bootstrap"`
  + recv. No panic, no hang. Send fd blocking.
- **Speak** (child, stop already pending, empty pipe): send
  `"hello-from-ring"` returns `Ok`; FIONREAD = 16; raw read is the
  framed bytes. (`Receiver::recv` is cascade-aware and returns
  `Shutdown` under a pending stop — the bytes are proven in the pipe,
  not through recv.)
- **Stop** (child, pipe filled, stop pending): `SendError::Shutdown("x")`.
  FIONREAD unchanged. **`delivered=0`.**

Cancel on the stop half followed a Write that delivered **0** bytes.
STOP-2 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ partial_frame_residue ~3 s | ✅ isolated 3.01 s, floor 3.037 s |
| 2 | ★ send_poll_arm | ✅ floor 0.021 s PASS (after empty-wait retry; first floor red captured) |
| 3 | ★ tie-break both halves | ✅ speak=ok (bytes in pipe); stop=Shutdown |
| 4 | ⛔ NonblockGuard gone; fd blocking | ✅ no `NonblockGuard` in `process.rs`; probe asserts `O_NONBLOCK` clear before and after send |
| 5 | cancel only after 0 delivered | ✅ `delivered=0` on the Shutdown half |
| 6 | SendError unchanged | ✅ variants / EPIPE→Disconnected / Failed strings as they were; new SQE-push failures only |
| 7 | try_send untouched | ✅ `pub fn try_send` not in the diff |
| 8 | Sender not Clone | ✅ no `impl Clone for Sender` |
| 9 | four construction sites handle ring failure | ✅ three mint via `Result`; `reinterpret` moves the existing ring |
| 10 | bootstrap fallback | ✅ `broadcast == -1`, send `"bootstrap"` Ok |
| 11 | EINTR retry | ✅ `submit_and_wait_eintr` retries EINTR and Ok(0) |
| 12 | header names the ring | ✅ line 10: `io_uring Write → io_uring Read` |
| 13 | the floor | ✅ `Summary [ 497.696s] 5226 tests run: 5226 passed (7 slow), 22 skipped` |

STOP-1: send fd is blocking. STOP-2: delivered=0. STOP-3: contract unchanged.
STOP-4: minting sites return `Result`. STOP-5: first red captured, named, not re-run.

0 FAIL, 0 TIMEOUT on the green floor.

## BLAST

```
 src/comms/process.rs | 339 +++++++++++++++++++++++++++++++--------------------
?? tests/comms/probe_arc278_sender_grows_a_ring.rs
```

No `src/io.rs`. No `try_send` body. No new dependency.
