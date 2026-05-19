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
    let (pidfd, _lifeline) = wat::fork::spawn_lifelined(|_lifeline_r| {
        // Child exits with code 42 immediately.
        unsafe { libc::_exit(42) };
    })
    .expect("spawn_lifelined succeeds");

    let status = pidfd.wait_status().expect("wait_status returns exit status");
    assert_eq!(status, wat::fork::ExitStatus::Exited(42));
}

#[test]
fn pidfd_observes_signal_exit() {
    let (pidfd, lifeline) = wat::fork::spawn_lifelined(|_lifeline_r| {
        // Child blocks until any signal arrives — libc::pause(2) is the
        // honest event-wait. The old shape was `loop { sleep(60s) }`
        // which was a mora L1 violation (chosen-duration mechanism; on a
        // fast kernel SIGTERM could wait up to 60s for the next iter).
        // pause() returns when the kernel delivers the signal — no
        // duration; no race. SAFETY: pause(2) has no preconditions; it
        // always returns -1 with errno=EINTR after a signal is handled.
        unsafe { libc::pause(); }
    })
    .expect("spawn_lifelined succeeds");

    // Send SIGTERM via the canonical Pidfd interface.
    pidfd
        .send_signal(libc::SIGTERM)
        .expect("send_signal succeeds");

    let status = pidfd
        .wait_status()
        .expect("wait_status returns signal status");
    assert_eq!(status, wat::fork::ExitStatus::Signaled(libc::SIGTERM));

    drop(lifeline); // explicit drop for clarity
}
