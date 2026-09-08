//! Arc 170 — a delivered signal is observable.
//!
//! After `kill` returns, a reader must see the flag without waiting
//! for the shutdown worker. These tests do **not** call
//! `init_shutdown_signal`: a running worker would drain SIGTERM via
//! signalfd before the pending-set assertion.
//!
//! The SIGTERM-pending case runs in a child so a leftover stop signal
//! cannot poison a sibling test in the same cargo-test process.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

const CHILD_ENV: &str = "ARC170_DELIVERED_SIGNAL_CHILD";

fn signal_is_pending(sig: libc::c_int) -> bool {
    unsafe {
        let mut pending: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut pending);
        libc::sigpending(&mut pending);
        libc::sigismember(&pending, sig) == 1
    }
}

fn reset_latches() {
    wat::runtime::KERNEL_SIGUSR1.store(false, Ordering::SeqCst);
    wat::runtime::KERNEL_SIGUSR2.store(false, Ordering::SeqCst);
    wat::runtime::KERNEL_SIGHUP.store(false, Ordering::SeqCst);
    wat::runtime::KERNEL_STOPPED.store(false, Ordering::SeqCst);
}

/// Row 7: a never-signalled process reads false on all four, without blocking.
#[test]
fn never_signalled_process_reads_false() {
    wat::runtime::block_substrate_signals();
    reset_latches();

    let t0 = Instant::now();
    assert!(
        !wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGHUP, libc::SIGHUP),
        "sighup? must be false when nothing was delivered"
    );
    assert!(
        !wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGUSR1, libc::SIGUSR1),
        "sigusr1? must be false when nothing was delivered"
    );
    assert!(
        !wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGUSR2, libc::SIGUSR2),
        "sigusr2? must be false when nothing was delivered"
    );
    assert!(
        !wat::runtime::stop_signal_is_pending(),
        "stopped? must be false when nothing was delivered"
    );
    assert!(
        t0.elapsed() < Duration::from_millis(50),
        "a never-signalled read must not block; took {:?}",
        t0.elapsed()
    );
}

/// Row 4: SIGHUP → observed true → reset-sighup! → observed false, no drain between.
#[test]
fn reset_sighup_clears_with_no_drain_between() {
    wat::runtime::block_substrate_signals();
    reset_latches();

    let rc = unsafe { libc::kill(libc::getpid(), libc::SIGHUP) };
    assert_eq!(rc, 0, "kill(SIGHUP)");

    assert!(
        wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGHUP, libc::SIGHUP),
        "sighup? must be true after Delivered, before any worker drain"
    );
    wat::runtime::KERNEL_SIGHUP.store(false, Ordering::SeqCst);
    assert!(
        !wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGHUP, libc::SIGHUP),
        "reset-sighup! must clear durably; a still-pending SIGHUP would re-report"
    );
}

fn run_selective_consume_child() -> ! {
    wat::runtime::block_substrate_signals();
    reset_latches();

    let pid = unsafe { libc::getpid() };
    assert_eq!(unsafe { libc::kill(pid, libc::SIGHUP) }, 0, "kill(SIGHUP)");
    assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0, "kill(SIGTERM)");

    assert!(
        wat::runtime::observed_user_signal(&wat::runtime::KERNEL_SIGHUP, libc::SIGHUP),
        "SIGHUP must be observable at the reader"
    );
    assert!(
        !signal_is_pending(libc::SIGHUP),
        "the resettable reader must have consumed SIGHUP"
    );
    assert!(
        signal_is_pending(libc::SIGTERM),
        "consuming SIGHUP must leave SIGTERM pending"
    );
    assert!(
        wat::runtime::stop_signal_is_pending(),
        "stopped? must see the still-pending SIGTERM without consuming it"
    );
    assert!(
        signal_is_pending(libc::SIGTERM),
        "stopped? must not dequeue SIGTERM"
    );
    assert!(
        !wat::runtime::KERNEL_STOPPED.load(Ordering::SeqCst),
        "the stop reader must not latch; the worker still has to see the fd"
    );
    eprintln!("SELECTIVE-OK");
    std::process::exit(0);
}

/// Row 5: consuming SIGHUP at the reader leaves a pending SIGTERM untouched.
#[test]
fn consuming_sighup_leaves_sigterm_pending() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_selective_consume_child();
    }

    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc170_delivered_signal_is_observable::consuming_sighup_leaves_sigterm_pending")
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn selective-consume child");
    let mut child_stderr = BufReader::new(child.stderr.take().expect("stderr"));
    let mut line = String::new();
    child_stderr
        .read_line(&mut line)
        .expect("child report");
    let status = child.wait().expect("wait");
    assert!(
        status.success(),
        "selective-consume child exited {status:?}; stderr={line:?}"
    );
    assert_eq!(
        line.trim(),
        "SELECTIVE-OK",
        "child must confirm SIGTERM survived; got {line:?}"
    );
}
