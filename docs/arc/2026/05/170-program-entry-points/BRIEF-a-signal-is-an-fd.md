# BRIEF — a signal is an fd

## The work, in one paragraph

Signals currently arrive out-of-band, through five handlers, and one of them translates itself into an
fd event by writing a byte down a self-pipe. Block the five signals process-wide, create a `signalfd`,
and hand it to the shutdown worker as just another input — next to the lifeline fd that is already
exactly that shape. The handlers, the pipe, and the async-signal-safe constraint on all of them go
away. **The worker must gain a demultiplex**, because it currently wakes on *any* input becoming
ready and a signalfd carries all five signals on one fd.

## Read in order

1. **`src/runtime.rs:412-431`** — the worker's poll loop. It builds `pollfds` from `input_fds` and
   `break`s on the first ready fd. **This is the loop that must change shape**: from *break-on-any* to
   *read, dispatch, and keep polling unless the event is stop-class*.
2. **`src/process/child.rs:37-97`** — the five handlers and `install_substrate_signal_handlers`. Note
   `:60-72`: the async-signal-safe justification is the whole reason the pipe exists. Note also that
   **SIGUSR1/2/HUP set an atomic and wake nothing** — that asymmetry is what row 1 protects.
3. **`src/runtime.rs:91-102`** — `request_kernel_stop`: `KERNEL_STOPPED.store` + the wake-pipe write.
   The store stays (a `#[test]` at `:30039` calls it); the write goes.
4. **`src/runtime.rs:307-330`** — `init_shutdown_signal_with_inputs`, including the **fork-aware
   rebuild guard**. A fork child needs its own signalfd exactly as it needs its own wake pipe; follow
   this guard rather than inventing one.
5. **`src/comms/process.rs:1400-1412`** — `timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK|TFD_CLOEXEC)`,
   *"atomic at creation"*. **This is the exemplar** for how a new fd type is created in this tree.

## Implementation sketch

```rust
// 1. BEFORE any thread is spawned — the mask is inherited by new threads.
let mut mask: libc::sigset_t = std::mem::zeroed();
libc::sigemptyset(&mut mask);
for sig in [libc::SIGINT, libc::SIGTERM, libc::SIGUSR1, libc::SIGUSR2, libc::SIGHUP] {
    libc::sigaddset(&mut mask, sig);
}
libc::pthread_sigmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut());

// 2. Atomic at creation, matching timerfd's shape.
let sfd = libc::signalfd(-1, &mask, libc::SFD_CLOEXEC | libc::SFD_NONBLOCK);

// 3. The worker: signalfd joins the poll set beside the lifeline fds,
//    and the loop DEMULTIPLEXES instead of breaking on any ready fd.
loop {
    poll(&mut pollfds, -1);                     // EINTR retries, as today
    if signalfd_is_ready {
        // read in a loop until EAGAIN — siginfo is 128 bytes and several may queue
        for si in read_siginfos(sfd) {
            match si.ssi_signo as i32 {
                libc::SIGINT | libc::SIGTERM => stop_class = true,   // KERNEL_STOPPED.store
                libc::SIGUSR1 => set_kernel_sigusr1(),               // measure only
                libc::SIGUSR2 => set_kernel_sigusr2(),
                libc::SIGHUP  => set_kernel_sighup(),
                _ => {}
            }
        }
    }
    if lifeline_ready { stop_class = true; }     // POLLHUP — unchanged
    if stop_class { break; }                    // only then leave → trigger_shutdown
}
```

## Blast radius

`src/runtime.rs` (mask, signalfd, worker loop, wake-pipe retirement), `src/process/child.rs` (handlers
retire), `src/host/entry.rs` and `src/distribution/spawned_runtime.rs` (the installer's two callers),
one new probe under `tests/`. No `src/io.rs`. No `eventfd`. No change to the wat-visible surface.

## STOP triggers

**STOP-1** — if `SIGUSR1` can reach the shutdown path, STOP. That is a process death where the
contract promises a polled flag, and **no existing test sends SIGUSR1 to a live server**, so the floor
will not catch it for you. The demultiplex is the stone, not a detail of it.

**STOP-2** — if the mask cannot be established before every thread spawn, STOP and name the thread
that starts first. A thread that misses the mask can take an unhandled signal and die by default
action — intermittent, load-dependent, and it would present as a mystery death rather than a bug.

**STOP-3** — if handlers must remain installed alongside the signalfd, STOP. Two mechanisms competing
for one signal is not a migration; surface the reason instead.

**STOP-4** — if a spawned child or an exec'd runtime cannot re-establish its own mask and signalfd,
STOP and report where the chain breaks. Do not leave a child relying on inherited state.

**STOP-5** — on any red floor arm: capture it whole, name the exact arm, do not re-run. As you did on
the ring stone — that captured red is the only reason its live-lock was ever found.

## What "done" looks like

The new probe proves both directions: **SIGUSR1 sets `(sigusr1?)` and leaves the process alive and
`(stopped?)` false**; SIGTERM stops it and a send blocked on a full pipe returns `Shutdown`. No handler
remains for the five and each is blocked. `partial_frame_residue` passes in ~3 s (duration, not
verdict), `send_poll_arm` and `the_senders_tie_break_is_a_property` pass. Floor Summary reads 5227
passed / 22 skipped / 0 FAIL / 0 TIMEOUT on a quiet box.

The SCORE should state what the clone3→`execveat` window now does: `CLONE_CLEAR_SIGHAND` resets
handlers but not the mask, so a stop signal arriving there is **pending rather than fatal** — a
safety improvement this stone gets for free, and worth recording as a fact you observed rather than a
claim inherited from the design.
