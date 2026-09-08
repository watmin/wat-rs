# DESIGN — a signal is an fd

> Added to arc 170 after its INSCRIPTION, at the builder's ruling. This finishes the one row
> `DESIGN-FD-MULTIPLEX-SHUTDOWN.md` wrote down and declined to change.

## The row arc 170 left open

That design's own table:

| Input | Source | Today | After |
|---|---|---|---|
| signal handler write byte | `substrate_on_stop_signal` writes `'!'` | wake-pipe | **wake-pipe (unchanged)** |
| lifeline pipe | parent death → POLLHUP | — | poll input |

Arc 170 turned the shutdown worker's wait into an fd multiplex and made parent-death an fd event. It
left the *signal* as the one cause that still arrives out-of-band, through a handler, and gets
translated into an fd event by hand.

## Why the self-pipe exists at all

Not because of io_uring, and not because anyone chose it. **Because a signal handler may only do
async-signal-safe work.** `child.rs:60-72` says so explicitly: each handler body is *exactly one
call*, and the wake-pipe write is justified by citing `signal-safety(7)`.

Remove the handler and the pipe has no reason to exist. `signalfd` removes the handler.

★★ And the fan-in becomes **homogeneous — every cause of shutdown is an fd event**, with no special
case for signals. `LIFELINE_FD` is already exactly that shape. This is `mora`'s doctrine one step
further: not only is *time* I/O, **signals are I/O**.

## What the disk says today

```
sigprocmask / pthread_sigmask         ZERO uses
signalfd                              ZERO uses — the single occurrence is a COMMENT at
                                      runtime.rs:421 naming "signalfd/eventfd/epoll/poll" as the
                                      load-bearing primitives, of which only poll is used
five handlers, child.rs:93-97          SIGINT, SIGTERM → substrate_on_stop_signal
                                                        → request_kernel_stop()
                                       SIGUSR1/2/HUP  → set_kernel_sigusr1/2/hup()  (atomic only)
request_kernel_stop, runtime.rs:91     KERNEL_STOPPED.store + write('!') to the wake pipe
```

★★★ **In production, `request_kernel_stop` is called from exactly one place — the handler.** Its only
other caller is inside a `#[test]` (`runtime.rs:30039`, `stopped_q_reads_kernel_flag`). So with
signalfd the wake pipe has **zero remaining production writers** and can genuinely be retired.

## ⛔ The trap that would be catastrophic

The worker **does not demultiplex.** `runtime.rs:426-431`: it polls its input set and `break`s on
*any* fd becoming ready, then runs the wake/measure path.

That is safe today only because **SIGUSR1/2/HUP never touch the pipe** — their handlers flip an atomic
that wat polls via `(sigusr1?)`, and wake nothing.

Point a signalfd at that loop unchanged and **SIGUSR1 shuts the process down.**

So the worker must become a **demultiplexer**: read `signalfd_siginfo`, dispatch on `ssi_signo`, and
only leave the loop for stop-class events.

```
signalfd readable → read siginfo(s)
    SIGINT | SIGTERM   → KERNEL_STOPPED.store(true); leave the loop → trigger_shutdown
    SIGUSR1/2/HUP      → set_kernel_sigusr1/2/hup(); KEEP POLLING
lifeline POLLHUP       → leave the loop → trigger_shutdown   (unchanged)
```

This still honours the worker's doctrine — *"the worker MEASURES and WAKES; it does not transition"*.
Setting the atomics is exactly the measurement the handlers performed.

## What lands

1. `pthread_sigmask(SIG_BLOCK, {SIGINT, SIGTERM, SIGUSR1, SIGUSR2, SIGHUP})` — **before any thread is
   spawned**, because the mask is inherited by new threads.
2. `signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK)` — atomic at creation, matching the tree's
   convention (`pipe2`, `SOCK_CLOEXEC`, `TFD_CLOEXEC`, `F_DUPFD_CLOEXEC`).
3. The worker polls `[signalfd, lifeline…]` and demultiplexes as above.
4. The five handlers, `install_substrate_signal_handlers`, and the wake pipe retire.
5. `request_kernel_stop` keeps its atomic store (a test calls it) and loses its pipe write.

## ★ A window that gets safer, not just tidier

`clone.rs:374` already passes `CLONE_CLEAR_SIGHAND`, which resets **handlers** — not the **mask**.
Today a stop signal landing in the `clone3` → `execveat` → re-install window meets a *default*
handler and **kills the child**. With the mask inherited-blocked, that signal is merely **pending**,
and is consumed the moment the child's own signalfd exists.

The exec is `execveat` on an O_PATH fd of **our own image** (`exec_plan.rs:43-49`, `:107`) — not an
arbitrary binary — so the exec'd process re-blocks and re-creates its own signalfd. The
blocked-mask-survives-exec hazard has no third-party victim here.

## Four questions

| | |
|---|---|
| Obvious? | **YES** — every shutdown cause becomes an fd; `LIFELINE_FD` is already that shape |
| Simple? | **YES** — one concept, and it *removes* machinery: five handlers, one pipe, and the async-signal-safe constraint on all of it |
| Honest? | **YES** — it retires a comment that names `signalfd` as load-bearing in a file that uses none of it |
| Good UX? | **YES** — `(:wat::kernel::stopped?)` and `(sigusr1?)` are unchanged for wat callers |

## Out of scope — REJECTED

- **`src/io.rs`.** Still cut, still blocked on ring ownership.
- **`eventfd`.** Zero uses and no caller needs one; the broadcast is a pipe and works.
- **The broadcast fan-out itself.** It stays. Parent-death and the worker's own trigger are not
  signals, so there is no signalfd for them to arrive on.

## Files

`src/runtime.rs` (mask, signalfd, worker demultiplex, wake-pipe retirement), `src/process/child.rs`
(handlers retire), `src/host/entry.rs` and `src/distribution/spawned_runtime.rs` (installer callers),
plus one new probe.
