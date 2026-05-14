//! Proof: deterministic parent-death detection via lifeline pipe.
//!
//! Demonstrates the lock-step alternative to PR_SET_PDEATHSIG.
//!
//! # Mechanism
//!
//! Test → fork supervisor → supervisor fork grandchild. Three processes.
//!
//! - `lifeline_pipe`: created BEFORE supervisor fork. Read-end goes to
//!   the grandchild (via supervisor's fork-inheritance). Write-end is
//!   held ONLY by the supervisor. Nobody ever writes to it.
//!
//! - `done_pipe`: created BEFORE supervisor fork. Write-end inherited
//!   by grandchild (via supervisor's fork). When grandchild exits, OS
//!   closes its FDs → done_pipe EOFs → test learns grandchild died.
//!
//! When supervisor `_exit`s for ANY reason (clean exit, panic, segfault,
//! SIGKILL, OOM), the kernel closes all FDs the supervisor held —
//! including the lifeline write-end. Grandchild's `read(lifeline_r)`
//! returns 0 (EOF) deterministically. No signal. No timer. No race.
//!
//! # Contrast with PR_SET_PDEATHSIG
//!
//! PDEATHSIG: child must call `prctl(PR_SET_PDEATHSIG, SIGTERM)` BEFORE
//! parent dies. If parent dies in the fork→prctl window, kernel does
//! not deliver SIGTERM, child orphans forever. (Demonstrated 10% rate
//! in probe_pdeathsig_diagnostic.)
//!
//! Lifeline: child inherits the FD across fork() — atomic with the fork
//! itself. No subsequent registration. Cannot race.
//!
//! # Strong claim
//!
//! 100/100 trials produce zero orphans regardless of supervisor exit
//! timing. The test runs this loop inline (no shell driver).

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::Instant;

fn make_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "pipe(2) failed: {}", std::io::Error::last_os_error());
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (r, w)
}

/// One trial. Returns Ok(()) on clean grandchild death; Err on leak.
fn one_trial(trial_idx: usize) -> Result<(), String> {
    // Create the two pipes BEFORE any fork.
    let (lifeline_r, lifeline_w) = make_pipe();
    let (done_r, done_w) = make_pipe();
    let (pid_r, pid_w) = make_pipe();

    let supervisor_pid = unsafe { libc::fork() };
    assert!(supervisor_pid >= 0, "supervisor fork failed");

    if supervisor_pid == 0 {
        // ── SUPERVISOR ────────────────────────────────────────────────
        // Close test-only ends.
        drop(pid_r);
        drop(done_r);
        // NOTE: supervisor holds lifeline_w. Will NOT close it
        // explicitly — _exit will. The whole point.
        // Supervisor also holds lifeline_r briefly until after forking
        // the grandchild (grandchild inherits it). Close after fork.

        let grandchild_pid = unsafe { libc::fork() };
        assert!(grandchild_pid >= 0, "grandchild fork failed");

        if grandchild_pid == 0 {
            // ── GRANDCHILD ────────────────────────────────────────────
            // Close pipes the grandchild does not own:
            //   - pid_w: supervisor's path, not grandchild's
            //   - lifeline_w: ONLY supervisor must hold this
            drop(pid_w);
            drop(lifeline_w);

            // done_w stays open: when grandchild exits, kernel closes it
            // → test sees done_pipe EOF.

            // Block on lifeline_r read(). Returns 0 (EOF) when supervisor
            // dies and kernel closes lifeline_w. Returns >0 if someone
            // writes (shouldn't happen). Returns -1 on error.
            let mut buf = [0u8; 1];
            let n = unsafe {
                libc::read(
                    lifeline_r.as_raw_fd(),
                    buf.as_mut_ptr() as *mut _,
                    1,
                )
            };

            if n == 0 {
                // EOF — supervisor died. Clean exit.
                unsafe { libc::_exit(0) };
            } else {
                // Unexpected: someone wrote to the lifeline (n>0) or
                // read failed (n<0). Either way, abort.
                unsafe { libc::_exit(99) };
            }
        }

        // Supervisor: report grandchild pid to test, then exit.
        // Drop lifeline_r — only the grandchild should hold the read end.
        drop(lifeline_r);
        // Drop done_w — only the grandchild should hold this so its
        // closure on grandchild exit is detectable by the test.
        drop(done_w);

        let pid_bytes = grandchild_pid.to_le_bytes();
        let n = unsafe {
            libc::write(
                pid_w.as_raw_fd(),
                pid_bytes.as_ptr() as *const _,
                4,
            )
        };
        assert_eq!(n, 4, "supervisor pid_pipe write failed");
        drop(pid_w);

        // Exit IMMEDIATELY — no sleep, no special ordering.
        // The kernel will close lifeline_w as part of process teardown.
        unsafe { libc::_exit(0) };
    }

    // ── TEST ──────────────────────────────────────────────────────────
    // Close all child-side ends so only the children hold them.
    drop(pid_w);
    drop(lifeline_r);
    drop(lifeline_w); // Test never held a meaningful copy; close.
    drop(done_w);

    // Wait for supervisor.
    let mut status: libc::c_int = 0;
    let waited = unsafe { libc::waitpid(supervisor_pid, &mut status, 0) };
    assert!(waited >= 0, "waitpid failed");

    // Read grandchild pid.
    let mut pid_bytes = [0u8; 4];
    let n = unsafe {
        libc::read(
            pid_r.as_raw_fd(),
            pid_bytes.as_mut_ptr() as *mut _,
            4,
        )
    };
    assert_eq!(n, 4, "pid_pipe read failed");
    let grandchild = i32::from_le_bytes(pid_bytes);
    drop(pid_r);

    // Poll done_pipe with 500ms budget. If lifeline mechanism is
    // deterministic, grandchild exits in low ms regardless of which
    // process won any race. 500ms is generous; lock-step should fire
    // in microseconds-to-milliseconds.
    let t0 = Instant::now();
    let mut pollfd = libc::pollfd {
        fd: done_r.as_raw_fd(),
        events: libc::POLLHUP | libc::POLLIN,
        revents: 0,
    };
    let poll_ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 500) };
    let elapsed = t0.elapsed();
    drop(done_r);

    if poll_ret <= 0 {
        // Reap manually to avoid leak even on failure.
        unsafe { libc::kill(grandchild, libc::SIGKILL) };
        unsafe { libc::waitpid(grandchild, std::ptr::null_mut(), 0) };
        return Err(format!(
            "trial {}: grandchild {} did not exit within 500ms via lifeline (poll_ret={})",
            trial_idx, grandchild, poll_ret
        ));
    }

    // Verify grandchild reaped or zombie.
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", grandchild)).unwrap_or_default();
    let state = stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.trim_start().chars().next())
        .unwrap_or('?');
    if state != 'Z' && state != '?' {
        return Err(format!(
            "trial {}: grandchild {} state '{}' after poll (expected Z or ?) elapsed={:?}",
            trial_idx, grandchild, state, elapsed
        ));
    }

    Ok(())
}

#[test]
fn lifeline_pipe_zero_orphans_across_100_trials() {
    let trials = 100;
    let mut failures = Vec::new();
    let t0 = Instant::now();
    for i in 0..trials {
        match one_trial(i) {
            Ok(()) => {}
            Err(e) => failures.push(e),
        }
    }
    let elapsed = t0.elapsed();
    eprintln!(
        "[lifeline proof] {} trials in {:?} — failures: {}",
        trials,
        elapsed,
        failures.len()
    );
    for f in &failures {
        eprintln!("  {}", f);
    }
    assert_eq!(
        failures.len(),
        0,
        "lifeline pipe failed in {}/{} trials — see stderr",
        failures.len(),
        trials
    );
}
