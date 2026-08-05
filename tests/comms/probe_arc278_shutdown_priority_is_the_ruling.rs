//! CHARACTERIZATION GATE — arc 278. **Shutdown priority is a RULING, not an accident.**
//!
//! Builder, 2026-08-05, shown `wait_for_data_or_cascade`'s tie-break directly:
//!
//! > *"if we're going down — no one else matters."*
//!
//! This file pins that. When a `recv` wakes with **both** arms ready — a value readable on the data
//! fd *and* the substrate cascade fired — the stop wins and the value is not read. That is the
//! intended contract.
//!
//! ```ignore
//! if got_broadcast { Ok(PollOutcome::Shutdown) }      // the ruling
//! else if got_data { Ok(PollOutcome::DataReady) }
//! ```
//!
//! ## ⛔ WHAT THIS FILE REPLACES, AND WHY THE REPLACEMENT MATTERS
//!
//! It was `probe_arc278_a_wake_is_not_a_preemption.rs`, and it asserted the **opposite** — that
//! returning `Shutdown` here was a defect ("a delivered frame is discarded"). It was `#[ignore]`d and
//! red, and its red was written into a design stone and a seam as a *root cause*.
//!
//! **It proved nothing.** It fires the cascade on purpose and then expresses surprise that a shutdown
//! is reported. A gate that asks for X and objects to receiving X measures only itself.
//! `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`
//!
//! Two further corrections carried here so they are not re-derived:
//!
//! 1. **"discard" was never a real mechanism.** A `PollAdd` completion reports *readiness*; it reads
//!    nothing. No bytes are read and dropped — they stay in the kernel pipe buffer, untouched, and
//!    die later at fd close. The honest sentence is *"the poll saw a value was readable, the caller
//!    declined to read it, and reported the stop."*
//! 2. **The drain is not greedy.** io_uring is a *completion queue*, not `select`: `submit_and_wait(1)`
//!    blocks for **at least** one completion and the kernel is free to complete both polls first. Both
//!    CQEs are genuinely handed to us, and draining the CQ is mandatory hygiene, not over-harvesting.
//!
//! ## If this test goes RED
//!
//! Someone changed shutdown priority. That is a **ruling change and it is the builder's**, not a
//! refactor — do not "fix" this test to match new behaviour. Either the change is authorized (then
//! this file is rewritten deliberately, with the new ruling quoted) or the change is the bug.
//!
//! ## Firing the REAL cascade — the one part of the old gate worth keeping
//!
//! `runtime::trigger_shutdown()` drops the crossbeam Sender (the **thread**-tier sever) and **never
//! writes the broadcast byte** — the shutdown worker alone writes it (`runtime.rs:477`). A gate that
//! called it would leave `got_broadcast` false, produce no tie, and test nothing. (That is exactly
//! how an earlier version of this probe "passed" while measuring the harness.)
//!
//! Production's path is: **write the wake pipe → the worker wakes → the worker writes the broadcast.**
//! That is what the helper below does, and it then **waits on the wire** — `poll()` until the
//! broadcast fd is genuinely readable — rather than sleeping. `mora`: a sleep is a guess, and guesses
//! race. That wait is also the honest non-vacuity guard: it cannot pass unless the cascade really
//! fired. Asserting `broadcast_fd >= 0` would guard the *apparatus*, not the *condition*.
//!
//! ## Isolation
//!
//! Shutdown infra is process-global and one-way. `.config/nextest.toml` forks every test into its own
//! process with `retries = 0`, so this cannot leak sideways and a red cannot be retried away.

use std::sync::atomic::Ordering;
use wat::comms::{process, RecvError};

/// Fire the cascade the way production fires it, and return only once the broadcast arm is
/// genuinely readable. Returns the broadcast fd so the caller can report it.
fn fire_real_cascade_and_wait_until_broadcast_is_ready() -> i32 {
    wat::runtime::init_shutdown_signal();

    let broadcast_fd = wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(Ordering::SeqCst);
    let wake_fd = wat::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);

    // Without an armed broadcast fd, `recv` takes a BOOTSTRAP FALLBACK that polls the data fd
    // alone — one arm, no tie, nothing under test.
    assert!(broadcast_fd >= 0, "broadcast fd absent ({broadcast_fd}) — recv would single-arm poll");
    assert!(wake_fd >= 0, "wake-pipe write fd absent ({wake_fd}) — cannot fire the real cascade");

    // The signal handler's move: one byte on the wake pipe. The worker owns the broadcast write-end
    // and writes it on wake — writing the broadcast ourselves would simulate the mechanism instead
    // of driving it.
    let byte = b"!";
    let n = unsafe { libc::write(wake_fd, byte.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "failed to write the wake byte");

    // Wait ON THE WIRE for the worker to have written the broadcast. This is what makes the tie
    // deterministic instead of a race, and it is the guard that cannot pass vacuously.
    let mut pfd = libc::pollfd { fd: broadcast_fd, events: libc::POLLIN | libc::POLLHUP, revents: 0 };
    loop {
        let r = unsafe { libc::poll(&mut pfd, 1, 5_000) };
        if r > 0 {
            break;
        }
        assert!(r != 0, "the shutdown worker never wrote the broadcast within 5s — the cascade did not fire");
        let err = std::io::Error::last_os_error();
        assert!(err.raw_os_error() == Some(libc::EINTR), "poll on the broadcast fd failed: {err}");
    }

    eprintln!("[gate] cascade fired for real: broadcast fd {broadcast_fd} is READY (worker wrote it)");
    broadcast_fd
}

/// ★ THE RULING: both arms ready ⇒ the stop wins.
///
/// Three values are sitting readable in the pipe when the cascade lands. Every `recv` reports the
/// stop; none of them reads a value. *If we're going down, no one else matters.*
#[test]
fn a_fired_cascade_outranks_readable_data() {
    let (tx, rx) = process::pair::<String>().expect("process::pair");

    for frame in ["one", "two", "three"] {
        tx.send(frame.to_string()).expect("send must succeed before the cascade fires");
    }

    fire_real_cascade_and_wait_until_broadcast_is_ready();

    // Both arms ready, by construction, every run.
    for attempt in 1..=3 {
        eprintln!("[gate] -> recv #{attempt} with BOTH arms ready");
        let got = rx.recv();
        eprintln!("[gate] <- recv #{attempt} returned {got:?}");
        match got {
            Err(RecvError::Shutdown) => {}
            Ok(v) => panic!(
                "SHUTDOWN PRIORITY WAS CHANGED. recv #{attempt} returned the value {v:?} while the \
                 substrate cascade was fired. The ruling is \"if we're going down, no one else \
                 matters\" — a stop outranks readable data. If this change is intended it is the \
                 BUILDER'S ruling to make, and this file must be rewritten deliberately with the new \
                 ruling quoted. Do not adjust this assertion to match the code."
            ),
            Err(other) => panic!("expected Shutdown, got {other:?}"),
        }
    }
}
