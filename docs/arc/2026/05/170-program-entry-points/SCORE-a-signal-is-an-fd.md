# SCORE — a signal is an fd

**SCORED.** Executor: grok, 2026-09-08. Did not commit.

```
Summary [ 499.089s] 5227 tests run: 5227 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T17-29-09Z/`

A prior floor on this stone is captured at `.floor/2026-09-08T17-17-36Z/`.
It was not re-run.

## WHAT LANDED

The five signals are blocked process-wide and delivered on a `signalfd`
(`SFD_CLOEXEC | SFD_NONBLOCK`). The shutdown worker polls
`[signalfd, extra…]` and **demultiplexes**: SIGINT/SIGTERM store
`KERNEL_STOPPED` and leave the loop; SIGUSR1/2/HUP only flip their
atomics and keep polling; a lifeline POLLHUP is stop-class. Then the
worker writes the broadcast as before.

The five `sigaction` handlers are gone. The wake pipe is gone.
`request_kernel_stop` keeps the atomic store (a unit test calls it)
and no longer writes a pipe. The broadcast fan-out stays.

`pthread_sigmask` is per-thread. Blocking only inside `init` left
libtest's extra threads unblocked; `kill(pid, SIGUSR1)` died by
default action (signal 10) before the probe could assert. A
`#[ctor::ctor]` blocks the five **before `main`**, so every later
thread inherits the mask — including the shutdown worker and libtest
workers. `init` still blocks (idempotent) so an exec'd image that
re-runs ctors and then inits is covered twice.

## THE CLONE3 WINDOW — observed, not inherited

`CLONE_CLEAR_SIGHAND` resets handlers, not the mask. A stop signal in
the clone3→exec window is **pending rather than fatal**.

Observed on the first floor of this stone, not as a claim from the
design:

```
TIMEOUT [  40.214s] wat::process pidfd_primitive::pidfd_observes_signal_exit
```

`.floor/2026-09-08T17-17-36Z/` ARM: `(test timed out)`. That child
`pause()`s and never execs, never creates a signalfd. It inherited the
blocked SIGTERM mask; `pidfd_send_signal(SIGTERM)` left the signal
pending; `pause()` never returned. That is the window, live.

The test child now unblocks SIGTERM before `pause()` so the default
action can fire — the pidfd path it exists to observe. An exec'd
runtime (`spawned_runtime`) blocks again (ctor + init) and creates its
own signalfd; pending signals are consumed there.

## THE FIRST FLOOR WAS RED — captured, not re-run

```
Summary [ 497.511s] 5227 tests run: 5226 passed (7 slow), 1 timed out, 22 skipped
```

Arm: `pidfd_observes_signal_exit` TIMEOUT 40.214s. Mechanism: inherited
blocked mask + `pause()` in a non-exec clone3 child. Not re-run.

## THE LOAD-BEARING ARMS

| probe | isolated | floor |
|---|---|---|
| `a_signal_is_an_fd` | 0.01 s PASS | **0.023 s** PASS |
| `partial_frame_residue` | **3.02 s** PASS | **3.027 s** PASS |
| `send_poll_arm` | 0.01 s PASS | **0.076 s** PASS |
| `the_senders_tie_break_is_a_property` | 0.01 s PASS | **0.026 s** PASS |

## THE NEW PROBE

- SIGUSR1 → `KERNEL_SIGUSR1` true, `KERNEL_STOPPED` false, pair
  send/recv still works.
- SIGTERM to a child blocked on a full pipe → `SendError::Shutdown`.
- `sigaction` query: `SIG_DFL` for all five; all five in the blocked
  mask.
- A thread spawned after the mask is set reports all five blocked.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ SIGUSR1 does not stop | ✅ flag true, stopped? false, still serving |
| 2 | ⛔ SIGINT/SIGTERM still stop | ✅ child `Shutdown` on full-pipe send |
| 3 | ★ no handler, five blocked | ✅ SIG_DFL + `sigismember` |
| 4 | ★ mask before any thread | ✅ ctor before `main`; spawned thread inherits |
| 5 | signalfd atomic | ✅ `SFD_CLOEXEC \| SFD_NONBLOCK` in the call |
| 6 | wake pipe retired | ✅ no `SHUTDOWN_WAKE_WRITE_FD`; store-only `request_kernel_stop` |
| 7 | lifeline still cascades | ✅ existing spawn/process probes on the floor |
| 8 | spawned child still stops | ✅ send_poll_arm / residue / this probe's SIGTERM child |
| 9 | transport stop arms | ✅ residue **3.027 s** |
| 10 | tie-break | ✅ 0.026 s PASS |
| 11 | comment uses signalfd | ✅ worker docs name it as the primitive |
| 12 | the floor | ✅ `Summary [ 499.089s] 5227 tests run: 5227 passed (7 slow), 22 skipped` |

STOP-1: SIGUSR1 does not take the shutdown path. STOP-2: ctor is
before any thread; the first probe death (signal 10) named libtest's
siblings. STOP-3: no handlers remain (`sigaction` query SIG_DFL).
STOP-4: exec'd runtime re-blocks (ctor) and creates its own signalfd;
a non-exec clone3 child must unblock to die by default (pidfd timeout).
STOP-5: first red captured, named, not re-run.

0 FAIL, 0 TIMEOUT on the green floor.

## BLAST

```
 src/distribution/mod.rs                            |  18 +-
 src/distribution/spawned_runtime.rs                |   2 +-
 src/host/entry.rs                                  |  19 +-
 src/process/child.rs                               |  90 +------
 src/runtime.rs                                     | 276 +++++++++++++--------
 ...probe_arc278_shutdown_priority_is_the_ruling.rs |  24 +-
 tests/process/pidfd_primitive.rs                   |  21 +-
?? tests/comms/probe_arc278_a_signal_is_an_fd.rs
```

No `src/io.rs`. No `eventfd`. Broadcast fan-out stays. Wat-visible
`stopped?` / `sigusr1?` unchanged.
