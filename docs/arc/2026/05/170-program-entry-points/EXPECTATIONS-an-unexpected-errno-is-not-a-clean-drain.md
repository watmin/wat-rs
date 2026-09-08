# EXPECTATIONS — an unexpected errno is not a clean drain

Written **before** the strike. Small stone; `src/runtime.rs` only.

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the errno decision is unit-testable without a broken fd** | new unit tests | a pure function maps errno → `Done` / `Retry` / `Fatal`, called directly by tests |
| 2 | ★ **the three outcomes are distinct** | those tests | `EAGAIN`/`EWOULDBLOCK` → Done; `EINTR` → Retry; `EBADF`, `EINVAL`, `EIO`, and `n == 0` → **Fatal** |
| 3 | ★ **Fatal is not "return false"** | read the diff | diagnostic to fd 2 then `_exit(1)`, matching `runtime.rs:432-436` and `:446-451` |
| 4 | **the normal drain is unchanged** | the existing signal probe | `a_signal_is_an_fd` PASS — SIGUSR1 flips its flag and does not stop; SIGTERM stops |
| 5 | **no spin remains representable** | read the loop | the worker can no longer observe *readable → nothing drained → poll again* without a report |
| 6 | **no retry budget, backoff, or counter was added** | `git diff` | none — the wait is honest or fatal |
| 7 | **the transport arms still hold** | `partial_frame_residue`, `send_poll_arm`, `the_senders_tie_break_is_a_property` | PASS; residue in **~3 s** — read the duration |
| 8 | **blast radius** | `git diff --stat` | `src/runtime.rs` only |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5227 **+ the unit tests you added** (state the number), 22 skipped, 0 FAIL, 0 TIMEOUT, quiet box |

## The rows that carry it

★ **Row 1 is the stone.** The reason this defect survived review is that its error path is
unreachable from a test — private function, broken fd required. Extracting the decision is what makes
the invariant *gateable* rather than merely *stated*. A fix that leaves the decision inline and
untested has changed the code without changing what can be proven about it.

★ **Row 2 is the collapse, undone.** Today four outcomes share two silent exits. Name three and the
two worlds stop printing the same line.

⚠ **Row 6 exists because the tempting fix is the wrong one.** A retry cap or a backoff would convert a
spin into a slow spin and keep the failure silent. The signalfd is how this process learns to stop; if
it cannot be read, there is no honest way to continue.

## Runtime prediction

**20–40 minutes.** One extracted function, a handful of unit tests, and a loop that gets shorter. The
floor is the long pole.

## Trap-doors

- **`_exit`, not `exit`** — the drain runs on the worker thread and the tree's two precedents both use
  `_exit` for fork safety.
- **`write(2, …)` before exiting**, not `eprintln!` — same async-signal-safe discipline the neighbours
  carry.
- **`EWOULDBLOCK` and `EAGAIN` are the same value on Linux** but both spellings appear in Rust code;
  match on `ErrorKind::WouldBlock` as the existing code does.
- **`EINTR` must still retry**, not become Fatal — it is a normal interruption.
- **Do not widen the Fatal set to `EINTR` or `EAGAIN` by accident** when restructuring the match.

## What this stone does NOT claim

⚠ It does **not** claim a reachable trigger exists today. It removes a state whose only honest
outcome is a report, and makes the decision testable.
⚠ It does **not** touch the demux, the mask, the ctor, or anything else in `a signal is an fd`.
