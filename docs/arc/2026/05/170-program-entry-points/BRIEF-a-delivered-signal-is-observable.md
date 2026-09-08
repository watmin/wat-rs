# BRIEF — a delivered signal is observable

## The work, in one paragraph

`a signal is an fd` (STRUCK, `5094221aa`) moved flag-setting from a synchronous handler to the
shutdown-worker thread, so a service can now answer a request **before** the worker has drained the
signalfd — and `(:wat::kernel::sighup?)` reads false right after a SIGHUP was delivered. Make the flag
readers authoritative: answer from the atomic when it is latched, and from the kernel's **pending set**
when it is not. The signals are blocked, so the pending set is updated synchronously with the sender's
syscall — earlier than any handler could have run.

## The regression, verbatim

```
.floor/2026-09-08T18-08-14Z/   (captured, not re-run)
Summary [ 477.344s] 5232 tests run: 5231 passed (7 slow), 1 failed, 22 skipped
FAIL wat::kernel test::deftest_wat_tests_service_signal_observer_measures_itself
  wat-tests/service-signal-observer.wat:84:3  assert-eq failed
  actual:   [true, true, true, true, true, FALSE, true, …]   (index 5 = `hup` from after-hangup)
```

## Read in order

1. **`src/intrinsic/kernel/ambient.rs`** — its module doc names the seven verbs (`stopped?`,
   `sigusr1?`, `sigusr2?`, `sighup?`, `reset-sigusr1!`, `reset-sigusr2!`, `reset-sighup!`). **Establish
   which file holds the live read bodies** — `src/runtime.rs:19634` also carries a
   `KERNEL_STOPPED.load`. Confirm before editing; do not assume.
2. **`src/runtime.rs:102-127`** — the atomics and their setters, plus the `#[cfg(test)]` reset. Note
   there is **no production `reset-stopped!`**: that is what makes the asymmetry below safe.
3. **`src/runtime.rs:331-352`** — the worker's demux, which sets the same atomics. Both paths must
   converge on the atomic; you are not changing this.
4. **`wat-tests/service-signal-observer.wat:95-150`** — the failing test. Its own comment,
   *"sighup: a bitflip only, no wake — drive `observe` to see it"*, is the contract being repaired.

## Implementation sketch

Two families, and the difference is derived — see the DESIGN's table.

```rust
// Resettable: sighup? / sigusr1? / sigusr2?  — CONSUME selectively, then latch.
fn observed(flag: &AtomicBool, sig: libc::c_int) -> bool {
    if flag.load(Ordering::SeqCst) { return true; }        // latched: no syscall
    let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
    unsafe { libc::sigemptyset(&mut set); libc::sigaddset(&mut set, sig); }
    let zero = libc::timespec { tv_sec: 0, tv_nsec: 0 };   // zero, NOT null — null blocks forever
    let rc = unsafe { libc::sigtimedwait(&set, std::ptr::null_mut(), &zero) };
    if rc == sig { flag.store(true, Ordering::SeqCst); return true; }
    false                                                   // EAGAIN = nothing pending; EINTR = same
}

// stopped?  — NON-DESTRUCTIVE. The worker must still see SIGTERM.
fn stopped() -> bool {
    if KERNEL_STOPPED.load(Ordering::SeqCst) { return true; }
    let mut pending: libc::sigset_t = unsafe { std::mem::zeroed() };
    unsafe { libc::sigemptyset(&mut pending); libc::sigpending(&mut pending) };
    unsafe { libc::sigismember(&pending, libc::SIGTERM) == 1
          || libc::sigismember(&pending, libc::SIGINT)  == 1 }
}
```

Measured on this box before drawing: `sigtimedwait({SIGHUP}, 0)` consumed SIGHUP and **left SIGTERM
pending**; with nothing pending it returned in **2 µs**; and a process-directed pending signal is
visible from a **different thread** via `sigpending()`.

## Blast radius

The reader path only — `src/intrinsic/kernel/ambient.rs` and/or `src/runtime.rs` — plus new tests.
**Not** the mask, **not** the signalfd, **not** the worker demux, **not** the drain classifier.

## STOP triggers

**STOP-1** — if any reader path would dequeue `SIGINT` or `SIGTERM`, STOP. The worker must see them or
the process becomes unstoppable: a worse defect than the one being fixed, and one no existing test
catches.

**STOP-2** — if the fix appears to need a change under `wat-tests/`, STOP and surface why. The
contract is what is broken; a more patient test would pass while leaving the defect in place.

**STOP-3** — if `reset-sighup!` cannot be made to clear the flag durably (a still-pending signal
re-reporting through the reader), STOP and name it. That is the reason the resettable family consumes.

**STOP-4** — if 20 consecutive runs of the failing arm are impractical, STOP short of claiming the race
is fixed: report the number you actually ran.

**STOP-5** — on any red floor arm: capture whole, name the exact arm, do not re-run.

## What "done" looks like

`deftest_wat_tests_service_signal_observer_measures_itself` passes **20 consecutive runs**. A new test
shows `SIGHUP` → `(sighup?)` true → `(reset-sighup!)` → `(sighup?)` false with no drain between, and
another shows a reader consuming `SIGHUP` while a pending `SIGTERM` survives untouched. `wat-tests/`
is unchanged. `a_signal_is_an_fd`, `send_poll_arm`, `partial_frame_residue` (~3 s) and
`shutdown_priority_is_the_ruling` pass. Floor Summary reads 5232 plus the tests you add — **state that
number** — 22 skipped, 0 FAIL, 0 TIMEOUT, on a quiet box.

The SCORE should say plainly which reader consumes and which does not, and why the two differ.
