//! RED GATE — arc 278 task #79. PROCESS TIER.
//!
//! The failing test this exists for is `deftest_wat_tests_test_test_run_string_entry_path`
//! (`wat-tests/test.wat:290`) — a `deftest-hermetic` that forks a child, spawns a **process** peer
//! inside it, and `recv`s one frame. Process tier at both levels. No crossbeam anywhere.
//!
//! ## The rule under test
//!
//! `comms::process::wait_for_data_or_cascade` polls two arms — the data fd and the shutdown
//! broadcast fd — collects both readiness flags, and then:
//!
//! ```ignore
//! // Broadcast wins ties — substrate is going down; honest reporting
//! if got_broadcast { Ok(PollOutcome::Shutdown) }   // <- got_data collected, then DISCARDED
//! else if got_data { Ok(PollOutcome::DataReady) }
//! ```
//!
//! `RecvError::Shutdown` surfaces to wat as `RecvOutcome::Stopped`, and the failing test's own
//! `Stopped` arm reads:
//!
//! > *"stopped before the child sent its value — **the child was ALIVE**"*
//!
//! Which is exactly the state this rule produces: the child sent, the frame is in the pipe, a stop
//! arrives, and the parent is handed a stop instead of its frame.
//!
//! ## ⛔ FIRING THE **REAL** CASCADE — the mistake this file is a rewrite of
//!
//! A previous version of this gate called `runtime::trigger_shutdown()` directly. **That only drops
//! the crossbeam Sender (the THREAD-tier sever) and never writes the broadcast byte** — the
//! broadcast is written solely by the shutdown worker (`runtime.rs:477`). So `got_broadcast` was
//! false, there was no tie at all, and the gate "passed" while testing nothing.
//!
//! The real path is: **write the wake pipe → worker wakes → worker writes the broadcast.** That is
//! what a signal handler does, and it is what this gate does.
//!
//! ## Determinism — no sleeps, no load, no race to lose
//!
//! 1. `tx.send(..)` returns only once the bytes are framed and written ⇒ the **data arm is ready**.
//! 2. write one byte to `SHUTDOWN_WAKE_WRITE_FD` ⇒ the worker will write the broadcast.
//! 3. **`poll()` the broadcast fd until it is actually readable** — we wait on the wire for the
//!    worker to have done it. Not a sleep: `mora` — a sleep is a guess, and guesses race.
//! 4. only then `rx.recv()` ⇒ **both arms ready, by construction, every run.**
//!
//! ## Isolation
//!
//! The shutdown infra is process-global and one-way. `.config/nextest.toml` forks every test into
//! its own process with `retries = 0`, so this cannot leak sideways and a red cannot be retried away.

use std::sync::atomic::Ordering;
use wat::comms::{process, RecvError};

/// Fire the cascade the way production fires it, and return only once the broadcast arm is
/// genuinely readable.
///
/// Returns the broadcast fd so the caller can assert it was real.
fn fire_real_cascade_and_wait_until_broadcast_is_ready() -> i32 {
    wat::runtime::init_shutdown_signal();

    let broadcast_fd = wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(Ordering::SeqCst);
    let wake_fd = wat::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);

    // ⛔ NON-VACUITY GUARD. Without an armed broadcast fd, `recv` takes a BOOTSTRAP FALLBACK that
    // polls the data fd ALONE — one arm, no tie, nothing under test. An unguarded run would be
    // measuring the harness and I would read the result as a verdict on the rule.
    assert!(broadcast_fd >= 0, "broadcast fd absent ({broadcast_fd}) — recv would single-arm poll");
    assert!(wake_fd >= 0, "wake-pipe write fd absent ({wake_fd}) — cannot fire the real cascade");

    // The signal handler's move: one byte on the wake pipe. The worker owns the broadcast
    // write-end and writes it on wake — we must NOT write the broadcast ourselves, or we would be
    // simulating the mechanism instead of driving it.
    let byte = b"!";
    let n = unsafe { libc::write(wake_fd, byte.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "failed to write the wake byte");

    // Wait ON THE WIRE for the worker to have written the broadcast. This is what makes the tie
    // deterministic instead of a race we might lose.
    let mut pfd = libc::pollfd { fd: broadcast_fd, events: libc::POLLIN | libc::POLLHUP, revents: 0 };
    loop {
        let r = unsafe { libc::poll(&mut pfd, 1, 5_000) };
        if r > 0 {
            break;
        }
        assert!(r != 0, "the shutdown worker never wrote the broadcast within 5s — the cascade did not fire");
        let err = std::io::Error::last_os_error();
        assert!(
            err.raw_os_error() == Some(libc::EINTR),
            "poll on the broadcast fd failed: {err}"
        );
    }

    eprintln!("[gate] cascade fired for real: broadcast fd {broadcast_fd} is READY (worker wrote it)");
    broadcast_fd
}

/// ★★ THE TIE, at the tier the failing test actually uses.
///
/// One frame already in the pipe, the substrate cascade genuinely fired, both arms ready.
/// The delivered frame must win — the sender completed its release and will never re-send.
#[test]
#[ignore = "RED BY DESIGN — arc 278 #79. Proves `broadcast wins ties` discards an already-delivered \
frame. IGNORED ONLY so main stays green (standing rule: green commits only); it is NOT a deferral. \
UN-IGNORING THIS IS THE FIX'S ACCEPTANCE CRITERION. Run: cargo nextest run --release -E \
'test(a_wake_is_not_a_preemption)' --run-ignored all"]
fn a_delivered_frame_outranks_a_fired_cascade() {
    let (tx, rx) = process::pair::<String>().expect("process::pair");

    // The sender's release. After this returns the bytes ARE in the pipe.
    tx.send("from-string".to_string()).expect("send must succeed before the cascade fires");

    fire_real_cascade_and_wait_until_broadcast_is_ready();

    // Both arms ready. The poll must choose, and `broadcast wins ties` says it chooses wrong.
    eprintln!("[gate] -> recv with BOTH arms ready");
    let got = rx.recv();
    eprintln!("[gate] <- recv returned {got:?}");

    match got {
        Ok(v) => assert_eq!(v, "from-string", "the frame arrived but its payload is wrong"),
        Err(RecvError::Shutdown) => panic!(
            "LOST TRANSFER REPORTED AS A STOP. The frame was already in the pipe — the sender had \
             completed its release and will never re-send — but `broadcast wins ties` discarded it \
             and returned Shutdown. In wat this surfaces as RecvOutcome::Stopped, which is exactly \
             what wat-tests/test.wat:290's arm calls \"stopped before the child sent its value — \
             the child was ALIVE\". The child DID send."
        ),
        Err(other) => panic!("unexpected recv error: {other:?}"),
    }
}

/// ★★ THE GRACEFUL RULING — builder, 2026-08-05: *"winner or loser, the result must be graceful."*
///
/// Three frames are in the pipe when the cascade lands. All three were delivered; all three must be
/// handed over. Only when nothing remains is the stop the honest answer.
///
/// This is always affordable because **there is no timeout** — arc 170 pinned that deliberately
/// (*"a wedged stop must hang VISIBLY"*; the deadline belongs to the supervisor). A deadline is the
/// only thing that could justify abandoning delivered work, and we chose not to have one.
#[test]
#[ignore = "RED BY DESIGN — arc 278 #79, the graceful half (builder: \"winner or loser, the result \
must be graceful\"). Three frames delivered, cascade fired, ZERO handed back. Ignored only to keep \
main green; un-ignoring is the fix's acceptance criterion."]
fn a_fired_cascade_drains_what_was_already_delivered() {
    let (tx, rx) = process::pair::<String>().expect("process::pair");

    for frame in ["one", "two", "three"] {
        tx.send(frame.to_string()).expect("send must succeed before the cascade fires");
    }

    fire_real_cascade_and_wait_until_broadcast_is_ready();

    for expected in ["one", "two", "three"] {
        eprintln!("[gate] -> recv for {expected:?}");
        let got = rx.recv();
        eprintln!("[gate] <- got {got:?}");
        match got {
            Ok(v) => assert_eq!(v, expected, "frames must arrive in order"),
            Err(RecvError::Shutdown) => panic!(
                "STOP DISCARDED IN-FLIGHT WORK: expected already-delivered frame {expected:?}, got \
                 Shutdown. The send side already stops at its own safe point (Admin::Stop -> finish \
                 the op -> reply -> Status::Stopped); the receive side's safe point is an EMPTY PIPE."
            ),
            Err(other) => panic!("unexpected recv error while draining: {other:?}"),
        }
    }

    // Nothing left to hand back — NOW the stop is the honest answer.
    eprintln!("[gate] -> final recv (drained; expecting Shutdown)");
    let final_recv = rx.recv();
    eprintln!("[gate] <- final recv {final_recv:?}");
    match final_recv {
        Err(RecvError::Shutdown) => {}
        Ok(v) => panic!("drained everything, then received an unexpected extra frame: {v:?}"),
        Err(other) => panic!("after the drain the stop must be Shutdown, not {other:?}"),
    }
}
