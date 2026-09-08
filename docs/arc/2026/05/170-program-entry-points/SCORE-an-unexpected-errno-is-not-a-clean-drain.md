# SCORE — an unexpected errno is not a clean drain

**SCORED.** Executor: grok, 2026-09-08. Did not commit.

```
Summary [ 498.739s] 5232 tests run: 5232 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T17-56-50Z/`

5227 + **5** unit tests = **5232**.

## WHAT LANDED

`classify_signalfd_read(n, errno_kind) -> DrainStep` is a pure function.
`drain_signalfd` dispatches `n > 0` as a siginfo; everything else goes
through the classifier. Three named outcomes, not two silent `break`s.

| outcome | mapping |
|---|---|
| `EAGAIN` / `EWOULDBLOCK` (`ErrorKind::WouldBlock`) | **Done** |
| `EINTR` (`ErrorKind::Interrupted`) | **Retry** |
| `EBADF`, `EINVAL`, `EIO`, any other errno, **`n == 0`** | **Fatal** |

Fatal:

```
substrate: signalfd(2) read failed; the process can no longer observe stop signals
```

then `_exit(1)` — the same `write(2)` / `_exit(1)` shape as
`signalfd(2)` and `pipe2(2)` init failure.

## THE TESTS

Called the classifier directly. No broken fd.

- `eagain_is_done`
- `ewouldblock_is_done`
- `eintr_is_retry`
- `unexpected_errno_is_fatal` — EBADF, EINVAL, EIO
- `n_zero_is_fatal` — `n == 0` is Fatal even if the stale errno is EAGAIN

## THE LOAD-BEARING ARMS

| probe | floor |
|---|---|
| `a_signal_is_an_fd` | **0.030 s** PASS |
| `partial_frame_residue` | **3.029 s** PASS |
| `send_poll_arm` | PASS |
| `the_senders_tie_break_is_a_property` | PASS |

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ decision unit-testable without a broken fd | ✅ `classify_signalfd_read` |
| 2 | ★ three outcomes distinct | ✅ Done / Retry / Fatal as tabled |
| 3 | ★ Fatal is not return false | ✅ diagnostic + `_exit(1)` |
| 4 | normal drain unchanged | ✅ `a_signal_is_an_fd` PASS |
| 5 | no silent spin remain | ✅ Fatal never returns to the poll loop |
| 6 | no retry budget / backoff / counter | ✅ none |
| 7 | transport arms | ✅ residue **3.029 s** |
| 8 | blast | ✅ `src/runtime.rs` only |
| 9 | the floor | ✅ `Summary [ 498.739s] 5232 tests run: 5232 passed (7 slow), 22 skipped` |

0 FAIL, 0 TIMEOUT.

## BLAST

```
 src/runtime.rs | 117 +++++++++++++++++++++++++++++++++++++++++++++++----------
```
