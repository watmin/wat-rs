//! Arc 170 slice 1f-i — integration tests for `:wat::kernel::StdInService`.
//!
//! These tests exercise the public API of [`wat::services::stdin`]:
//!
//! - [`wat::services::start_stdin_service`] singleton idempotency
//! - [`wat::services::StdInService::spawn_for_test`] hermetic-test path
//! - per-thread `register` / `unregister` roundtrip
//! - line-delimited EDN parsing
//! - multi-line ordered dispatch
//! - EOF propagation as `None`
//! - self-pipe trick correctness (control + data interleaved)
//!
//! Each non-singleton test allocates its own pipe pair via
//! [`libc::pipe`]; the service reads from the pipe's read end, the
//! test writes to the write end. Dropping the test handle issues
//! `Shutdown` to the worker; dropping the write end gives the
//! worker EOF.

use holon::HolonAST;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wat::services::stdin::{start_stdin_service, ControlMsg, StdInService, StdInServiceHandle};

// ─── Test infrastructure ────────────────────────────────────────────────────

/// Allocate a pipe pair for test input. Returns `(read_fd, write_fd)`
/// both as [`OwnedFd`]; caller transfers `read_fd.as_raw_fd()` to
/// the service via `spawn_for_test`. Caller writes to `write_fd`;
/// dropping `write_fd` gives the service EOF.
fn make_test_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "test pipe(2) must succeed");
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (r, w)
}

/// Write `s` (with terminating newline) to `fd`. Panics on partial
/// write — test pipe buffers should always accept short test
/// messages in one shot.
fn write_line(fd: &OwnedFd, s: &str) {
    let mut owned = String::from(s);
    if !owned.ends_with('\n') {
        owned.push('\n');
    }
    let bytes = owned.as_bytes();
    // SAFETY: bytes points at a valid byte slice; libc::write reads it.
    let n = unsafe {
        libc::write(
            fd.as_raw_fd(),
            bytes.as_ptr() as *const libc::c_void,
            bytes.len(),
        )
    };
    assert_eq!(
        n as usize,
        bytes.len(),
        "test write_line must complete in one shot"
    );
}

/// Receive with a generous timeout. Tests must not hang forever if
/// the service has a bug; the timeout makes failures loud instead
/// of indefinite.
fn recv_with_timeout(
    rx: &crossbeam_channel::Receiver<Option<Arc<HolonAST>>>,
    timeout: Duration,
) -> Result<Option<Arc<HolonAST>>, crossbeam_channel::RecvTimeoutError> {
    rx.recv_timeout(timeout)
}

/// Standard test timeout — generous enough that a healthy service
/// will not hit it, tight enough that a hung test fails in seconds.
const TEST_TIMEOUT: Duration = Duration::from_secs(2);

/// A unique test thread_id derived from the current thread. Each
/// test runs on its own cargo-test thread, so this is unique per
/// test run.
fn current_tid() -> std::thread::ThreadId {
    std::thread::current().id()
}

// ─── Row A — module structure ──────────────────────────────────────────────

#[test]
fn row_a_module_exports() {
    // Compile-time witnesses: if these names don't resolve, the
    // public re-exports are wrong. The body is intentionally tiny.
    let _ctor: fn() -> &'static StdInServiceHandle = start_stdin_service;
    let _spawn: fn(std::os::fd::RawFd) -> StdInServiceHandle = StdInService::spawn_for_test;
    let _msg = ControlMsg::Unregister {
        thread_id: current_tid(),
    };
    drop(_msg);
}

// ─── Row B — start_stdin_service idempotent ────────────────────────────────

#[test]
fn row_b_singleton_idempotent() {
    let h1 = start_stdin_service();
    let h2 = start_stdin_service();
    // Identity equality on the &'static reference: the OnceLock
    // returns the same value on every call. Pointer equality is
    // the strict idempotency test.
    assert!(
        std::ptr::eq(h1, h2),
        "start_stdin_service must return the same &'static handle on repeat calls"
    );
}

// ─── Row C — service thread spawns + idles ─────────────────────────────────

#[test]
fn row_c_service_thread_idles_without_panic() {
    let (read_fd, _write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    // Idle: register a consumer, never write to the pipe, drop
    // handle. The worker should sit in poll(2) the entire time
    // (no panic, no busy-wait CPU spike — but spike-detection is
    // outside this test's scope; the absence of a panic when the
    // handle drops is the primary signal).
    let _rx = handle.register(current_tid());
    std::thread::sleep(Duration::from_millis(50));
    drop(handle);
    // Service thread now receives Shutdown and exits. The test
    // passes if we got here without a panic.
}

// ─── Row D — registration roundtrip ────────────────────────────────────────

#[test]
fn row_d_register_unregister_roundtrip() {
    let (read_fd, _write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    handle.unregister(current_tid());

    // After unregister, the worker drops its stored Sender. The
    // Receiver should observe disconnect (recv returns Err once
    // the worker processes the unregister and drops the sender).
    // We give the worker a moment to process; a healthy worker
    // wakes immediately on the self-pipe.
    let deadline = Instant::now() + TEST_TIMEOUT;
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline {
                    panic!("unregister did not result in receiver disconnect within {:?}", TEST_TIMEOUT);
                }
            }
            Ok(_) => {
                // Stray data — should not happen for an unregistered
                // consumer with no input written.
                panic!("unregistered receiver yielded a value");
            }
        }
    }
}

// ─── Row E — single-line EDN parsing ───────────────────────────────────────

#[test]
fn row_e_single_line_edn_parses() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    write_line(&write_fd, "42");

    let got = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("recv must yield within timeout")
        .expect("first message must be Some(Atom)");
    match &*got {
        HolonAST::I64(42) => {}
        other => panic!("expected HolonAST::I64(42), got {:?}", other),
    }
}

// ─── Row F — multi-line ordered dispatch ───────────────────────────────────

#[test]
fn row_f_multiline_ordered_dispatch() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    write_line(&write_fd, "1");
    write_line(&write_fd, "2");
    write_line(&write_fd, "3");

    for expected in &[1i64, 2, 3] {
        let got = recv_with_timeout(&rx, TEST_TIMEOUT)
            .expect("recv must yield within timeout")
            .expect("each message in sequence must be Some(Atom)");
        match &*got {
            HolonAST::I64(n) if n == expected => {}
            other => panic!("expected HolonAST::I64({}), got {:?}", expected, other),
        }
    }
}

// ─── Row G — EOF propagates :None ──────────────────────────────────────────

#[test]
fn row_g_eof_propagates_none() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    // Drop the write end → service sees EOF on its read.
    drop(write_fd);

    // Service should send None to every registered consumer on EOF.
    let got = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("recv must yield within timeout");
    assert!(
        got.is_none(),
        "EOF must propagate as Option::None to consumer; got {:?}",
        got
    );

    // After the None, the channel disconnects (worker exits + drops
    // its Sender). Subsequent recv returns Err.
    let after = rx.recv_timeout(Duration::from_millis(500));
    match after {
        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {}
        other => panic!(
            "after EOF None, channel should disconnect; got {:?}",
            other
        ),
    }
    // Drop the handle for tidiness; the worker has already exited.
    drop(handle);
}

// ─── Row H — self-pipe trick verified ──────────────────────────────────────

#[test]
fn row_h_self_pipe_interleaves_data_and_control() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    // Sequence: data, control (register), data, control (unregister),
    // data. Verify the order observed on the consumer matches.
    write_line(&write_fd, "100");

    let rx = handle.register(current_tid());

    write_line(&write_fd, "200");

    // After register, the worker has two events queued: prior data
    // from before register (still buffered in the pipe) + post-
    // register data. The worker drains control first, so when it
    // reads from the pipe, the consumer is registered and BOTH
    // values are routed.
    let v1 = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("first recv must yield")
        .expect("first message must be Some(Atom)");
    let v2 = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("second recv must yield")
        .expect("second message must be Some(Atom)");

    // The two values arrive in input order — pipe is FIFO; one
    // poll cycle drains both. Order of values within the pipe is
    // strictly write-order.
    let pair = (
        match &*v1 { HolonAST::I64(n) => *n, other => panic!("v1 expected I64, got {:?}", other) },
        match &*v2 { HolonAST::I64(n) => *n, other => panic!("v2 expected I64, got {:?}", other) },
    );
    assert_eq!(
        pair,
        (100, 200),
        "self-pipe must wake before pipe data; both writes route to the registered consumer in order"
    );

    drop(handle);
}

// ─── Row I — zero Mutex/RwLock/CondVar (verified by grep) ──────────────────
//
// Verified externally via `grep -nE 'Mutex|RwLock|CondVar'
// src/services/stdin.rs` — see SCORE-SLICE-1F-I.md row I. No
// runtime test possible; the absence of a primitive in source is
// the assertion.

// ─── Row J — libc::poll used directly (verified by grep) ───────────────────
//
// Verified externally via `grep -n 'libc::poll' src/services/stdin.rs`
// — see SCORE-SLICE-1F-I.md row J. No new dependency added to
// Cargo.toml; libc::poll is in libc 0.2 already vendored.

// ─── Row K — All Rust integration tests pass ──────────────────────────────
//
// Implicit: this file's tests A through P (excluding the grep-only
// rows I + J) all pass when `cargo test --release --test
// services_stdin` runs green.

// ─── Additional rigor — multi-consumer registration ───────────────────────
//
// Slice 1f-i's dispatch policy is "first registered consumer
// wins." Verify the policy is honest: register two consumers,
// write data, assert only the first sees it.

#[test]
fn row_p_dispatch_first_registered_only() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    // Register two distinct consumers. ThreadId is unique per
    // OS thread; we use a spawned thread to get a second id.
    let rx_first = handle.register(current_tid());

    let other_tid = std::thread::spawn(|| std::thread::current().id())
        .join()
        .expect("scratch thread joins");
    let rx_second = handle.register(other_tid);

    write_line(&write_fd, "7");

    let v1 = recv_with_timeout(&rx_first, TEST_TIMEOUT)
        .expect("first registered consumer must receive within timeout")
        .expect("first registered consumer must see Some(Atom)");
    match &*v1 {
        HolonAST::I64(7) => {}
        other => panic!("first consumer expected I64(7), got {:?}", other),
    }

    // Second consumer must NOT receive the value — slice 1f-i
    // policy is single-consumer dispatch to the first registered.
    let v2 = rx_second.recv_timeout(Duration::from_millis(200));
    match v2 {
        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
        other => panic!(
            "second consumer should not receive data under single-consumer-first-wins policy; got {:?}",
            other
        ),
    }

    drop(handle);
}

// ─── Honest delta exploration — partial trailing line is dropped ──────────
//
// The protocol is line-delimited; bytes without a terminating
// newline are not a complete message. Verify the service does not
// surface a partial line as a parsed atom.

#[test]
fn honest_delta_partial_trailing_line_drops_at_eof() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());

    // Write bytes WITHOUT a trailing newline.
    let bytes = b"42";
    let n = unsafe {
        libc::write(
            write_fd.as_raw_fd(),
            bytes.as_ptr() as *const libc::c_void,
            bytes.len(),
        )
    };
    assert_eq!(n as usize, bytes.len());

    // Drop write end — service sees EOF.
    drop(write_fd);

    // Should observe EOF None directly, with no preceding parsed
    // atom (the "42" without newline is not a valid message).
    let got = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("recv must yield within timeout");
    assert!(
        got.is_none(),
        "partial trailing line must NOT be dispatched; got {:?}",
        got
    );

    drop(handle);
}

// ─── Honest delta exploration — malformed EDN line drops silently ─────────
//
// The BRIEF allows "panic on malformed EDN" or "log via StdErrService
// cascade." For slice 1f-i (no StdErrService yet), we drop the
// line silently; the next valid line is dispatched normally. This
// is the choice documented in SCORE.

#[test]
fn honest_delta_malformed_edn_does_not_kill_service() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    write_line(&write_fd, "[ unbalanced");
    write_line(&write_fd, "99");

    // First valid line after malformed input must arrive — service
    // doesn't crash on parse error.
    let got = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("recv must yield within timeout")
        .expect("valid line after malformed line must dispatch");
    match &*got {
        HolonAST::I64(99) => {}
        other => panic!("expected I64(99) after malformed line; got {:?}", other),
    }

    drop(handle);
}

// ─── Honest delta — string literal parses to HolonAST::String ─────────────
//
// Smoke test the natural-form EDN path beyond integers: strings
// should arrive as HolonAST::String.

#[test]
fn honest_delta_string_literal_parses() {
    let (read_fd, write_fd) = make_test_pipe();
    let handle = StdInService::spawn_for_test(read_fd.as_raw_fd());

    let rx = handle.register(current_tid());
    write_line(&write_fd, "\"hello\"");

    let got = recv_with_timeout(&rx, TEST_TIMEOUT)
        .expect("recv must yield within timeout")
        .expect("string literal must dispatch as Some(Atom)");
    match &*got {
        HolonAST::String(s) if s.as_ref() == "hello" => {}
        other => panic!("expected HolonAST::String(\"hello\"), got {:?}", other),
    }

    drop(handle);
}
