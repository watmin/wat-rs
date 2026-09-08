//! Smoke probe: `Pidfd` type + `spawn_lifelined` helper (arc 213 stone α).
//!
//! Two tests:
//!   1. `pidfd_observes_normal_exit`  — child exits with code 42; parent
//!      observes `ExitStatus::Exited(42)` via waitid(P_PIDFD).
//!   2. `pidfd_observes_signal_exit`  — parent sends SIGTERM via Pidfd;
//!      observes `ExitStatus::Signaled(SIGTERM)`.
//!
//! Both tests prove the primitive works end-to-end: clone3 + CLONE_PIDFD
//! creates the pidfd atomically; `wait_status` observes exit via
//! waitid(P_PIDFD); `send_signal` signals via pidfd_send_signal.

#[test]
fn pidfd_observes_normal_exit() {
    let (pidfd, _lifeline) = wat::process::spawn_lifelined(|_lifeline_r| {
        // Child exits with code 42 immediately.
        unsafe { libc::_exit(42) };
    })
    .expect("spawn_lifelined succeeds");

    let status = pidfd.wait_status().expect("wait_status returns exit status");
    assert_eq!(status, wat::process::ExitStatus::Exited(42));
}

#[test]
fn pidfd_observes_signal_exit() {
    let (pidfd, lifeline) = wat::process::spawn_lifelined(|_lifeline_r| {
        // clone3 copies the parent's sigmask (CLONE_CLEAR_SIGHAND resets
        // handlers, not the mask). The parent blocked SIGTERM for signalfd;
        // this child never execs and never creates a signalfd, so SIGTERM
        // would stay pending and pause() would never return. Unblock so
        // the default action (terminate) fires — the pidfd_send_signal
        // path this test exists to observe.
        unsafe {
            let mut mask: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut mask);
            libc::sigaddset(&mut mask, libc::SIGTERM);
            libc::pthread_sigmask(libc::SIG_UNBLOCK, &mask, std::ptr::null_mut());
            libc::pause();
        }
    })
    .expect("spawn_lifelined succeeds");

    // Send SIGTERM via the canonical Pidfd interface.
    pidfd
        .send_signal(libc::SIGTERM)
        .expect("send_signal succeeds");

    let status = pidfd
        .wait_status()
        .expect("wait_status returns signal status");
    assert_eq!(status, wat::process::ExitStatus::Signaled(libc::SIGTERM));

    drop(lifeline); // explicit drop for clarity
}
