//! Arc 170 — DIAGNOSTIC / leak-zero gate for orphan-cleanup mechanism.
//!
//! Same shape as `probe_pdeathsig_kills_orphan_child` BUT the supervisor's
//! pre-`_exit` sleep is configurable via env var `WAT_PROBE_SUPERVISOR_DELAY_MS`
//! (default 0).
//!
//! # Rendezvous mechanism update (2026-05-13)
//!
//! Original rendezvous: `done_pipe` FD-inheritance — grandchild inherited done_w
//! from the supervisor; when grandchild exited, done_w closed → POLLHUP on done_r.
//! Phase 3 of arc 170 FD-multiplex adds canonical `close_inherited_fds_above_stdio`
//! to spawn-process, which closes inherited test-pipes in the grandchild at startup.
//! done_pipe rendezvous no longer works. Replaced with `pidfd_open(grandchild_pid, 0)`
//! + `poll(pidfd, POLLIN, 1000ms)` — POLLIN fires when the process exits.
//! Kernel-guaranteed, Linux 5.3+. Observable contract unchanged.
//!
//! # Post-Phase-1C mechanism
//!
//! After Phase 1B (spawn-process lifeline) and Phase 1C (fork-program lifeline),
//! the PDEATHSIG mechanism is retired. The grandchild's death is now driven by
//! the lifeline pipe: the parent holds the write-end; when the parent exits for
//! ANY reason, the kernel closes its FDs including the lifeline write-end →
//! grandchild's shutdown worker detects POLLHUP on the lifeline read-end →
//! trigger_shutdown cascade fires → grandchild exits.
//!
//! The `WAT_PROBE_SUPERVISOR_DELAY_MS` env var was originally designed to ablate
//! the PDEATHSIG race (supervisor exits before grandchild reaches prctl). With
//! the lifeline mechanism the env var is vestigial: the lifeline write-end is
//! inherited atomically at fork() — no subsequent registration, no race window.
//! Both delay=0 and delay=10 should now produce 50/50 PASS.
//!
//! # Leak-zero gate
//!
//! This probe is now the empirical regression gate for the lifeline mechanism.
//! Run with delay=0 (the previously-racy case):
//!
//!   WAT_PROBE_SUPERVISOR_DELAY_MS=0  cargo test --release --test probe_pdeathsig_diagnostic
//!   WAT_PROBE_SUPERVISOR_DELAY_MS=10 cargo test --release --test probe_pdeathsig_diagnostic
//!
//! Pass criterion (Phase 1D): 50/50 PASS at delay=0. The Slice D baseline was
//! 45/50 (10% race rate) with PDEATHSIG. Any regression back to orphan leaks at
//! delay=0 indicates a Phase 1B/1C substrate defect.
//!
//! # Design
//!
//! ```text
//! test process (parent-of-supervisor)
//!   │  forks
//!   ├─ supervisor (pid: supervisor_pid)
//!   │    │  calls substrate spawn-process (Phase 1B path)
//!   │    └─ grandchild (pid: grandchild_pid) — lifeline read-end registered
//!   │         │  with shutdown worker (init_shutdown_signal_with_inputs)
//!   │         │  blocks on typed_recv (Slice B cascade wires)
//!   │
//!   └─ test: waitpid(supervisor) → supervisor exits →
//!            kernel closes supervisor's lifeline write-end →
//!            grandchild shutdown worker POLLHUP fires →
//!            trigger_shutdown → cascade wakes blocked recv →
//!            grandchild exits →
//!            pidfd_open(grandchild_pid) → poll(pidfd, POLLIN, 1000ms) fires →
//!            PASS: grandchild died within 1s
//! ```
//!
//! # Rendezvous discipline (no wall-clock sleep)
//!
//! - `pid_pipe` (supervisor → test): supervisor writes grandchild pid; test reads it.
//! - After reading grandchild_pid, test calls `pidfd_open(grandchild_pid, 0)` to
//!   obtain a file descriptor that becomes POLLIN-readable when the process exits.
//!   `poll(pidfd, POLLIN, 1000ms)` is the bounded rendezvous mechanism.
//! - No `thread::sleep`, `recv_timeout`, or `std::time::Instant` as a timer.
//!   `std::time::Instant` is used ONLY to measure elapsed time after poll
//!   returns (measurement, not timeout mechanism).
//!
//! # Grandchild program
//!
//! The grandchild runs a wat function that:
//! 1. Creates an unbounded channel (sender + receiver).
//! 2. Keeps the sender alive in a `let` binding (channel stays open).
//! 3. Calls `recv` on the receiver (blocks).
//! 4. Post-Phase-1B: lifeline EOF → shutdown worker POLLHUP → trigger_shutdown
//!    → recv returns RecvOutcome::Shutdown → RuntimeError propagates up →
//!    spawn_process_child_branch calls libc::_exit.
//!
//! # ZERO-MUTEX compliance
//!
//! No `Mutex`, `RwLock`, or `CondVar` in this file. Synchronisation via
//! OS pipes (rendezvous) + `libc::poll(2)` with 1000ms timeout.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::time::Instant;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, ProgramHandleInner, Value};
use wat::span::Span;

/// Source for the grandchild's blocking wat program.
///
/// Creates an unbounded channel, keeps sender alive, blocks on recv.
/// On Slice B cascade (SIGTERM → shutdown worker drops SHUTDOWN_TX →
/// SHUTDOWN_RX disconnects → typed_recv select! fires Shutdown arm),
/// recv returns RecvOutcome::Shutdown → RuntimeError propagates up →
/// spawn_process_child_branch calls libc::_exit.
/// Arc 170 slice 6 — the child program for spawn-process is a top-level
/// (:user::main -> :nil) define; the parent world only needs a trivial
/// :user::main to freeze.
const PARENT_SRC: &str = r#"
    (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
"#;

const CHILD_PROGRAM_SRC: &str = r#"
    (:wat::core::define (:user::main -> :wat::core::nil)
      (:wat::core::let
        [[tx rx] (:wat::kernel::make-unbounded-channel :wat::core::nil)
         _       (:wat::kernel::recv rx)]
        :wat::core::nil))
"#;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Extract the forked child's PID from a Process Value.
///
/// The ProgramHandle at fields[3] must be a Forked variant — spawn-process
/// always returns Forked (arc 170 Stone C).
fn grandchild_pid(process: &Value) -> libc::pid_t {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => {
            match &s.fields[3] {
                Value::wat__kernel__ProgramHandle(h) => match h.as_ref() {
                    ProgramHandleInner::Forked(child) => child.pid,
                    other => panic!("expected Forked ProgramHandle; got {:?}", other),
                },
                other => panic!("expected ProgramHandle at fields[3]; got {:?}", other),
            }
        }
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

/// Make a raw OS pipe. Returns (read_fd, write_fd) as OwnedFds.
fn make_raw_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "pipe(2) failed: {}", std::io::Error::last_os_error());
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (r, w)
}

/// Row H — orphaned grandchild receives SIGTERM via PR_SET_PDEATHSIG and exits
/// within 1s. Verifies that `spawn_process_child_branch` correctly calls
/// `prctl(PR_SET_PDEATHSIG, SIGTERM)` after `setpgid`.
///
/// # Shape
///
/// 1. Build a blocking wat world (done before any fork — supervisor inherits it).
/// 2. Create `pid_pipe` (supervisor → test).
/// 3. Fork supervisor.
/// 4. Supervisor: eval(spawn-process), extract grandchild pid, write to pid_pipe,
///    optional delay, `_exit(0)`.
/// 5. Test: waitpid(supervisor), read grandchild pid from pid_pipe.
/// 6. Test: pidfd_open(grandchild_pid, 0) → poll(pidfd, POLLIN, 1000ms) — fires
///    when grandchild process exits.
/// 7. Verify grandchild process is gone (zombie or reaped).
#[test]
fn probe_pdeathsig_diagnostic() {
    // Read supervisor-pre-exit delay from env (default 0 = original behaviour).
    let delay_ms: u32 = std::env::var("WAT_PROBE_SUPERVISOR_DELAY_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    eprintln!("[diagnostic] WAT_PROBE_SUPERVISOR_DELAY_MS = {}", delay_ms);

    // Step 1: Build the world before forking. Supervisor inherits it via
    // fork's copy-on-write semantics. InMemoryLoader has no disk state.
    let world = freeze_ok(PARENT_SRC);

    // Step 2: Create coordination pipe.
    //
    // pid_pipe: supervisor writes grandchild pid (4 bytes, i32 LE); test reads.
    let (pid_r, pid_w) = make_raw_pipe();

    // Step 3: Fork supervisor.
    let supervisor_pid = unsafe { libc::fork() };
    assert!(
        supervisor_pid >= 0,
        "fork(supervisor) failed: {}",
        std::io::Error::last_os_error()
    );

    if supervisor_pid == 0 {
        // ── Supervisor child branch ───────────────────────────────────────
        // Close the test's pipe end (supervisor only uses pid_w).
        drop(pid_r);

        // Spawn grandchild via substrate spawn-process.
        // spawn_process_child_branch (after Slice C edit) calls:
        //   setpgid(0, 0) → prctl(PR_SET_PDEATHSIG, SIGTERM) → bootstrap → eval.
        // Phase 3: spawn_process_child_branch now calls
        // close_inherited_fds_above_stdio — no FD-inheritance rendezvous
        // possible; test uses pidfd_open instead.
        // Arc 170 slice 6 — spawn-process now takes a wat PROGRAM
        // (`Vec<WatAST>`); construct the child program inline.
        let child_forms = wat::parser::parse_all_with_file(CHILD_PROGRAM_SRC, "<probe>")
            .expect("child program parse");
        let mut forms_items =
            vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
        forms_items.extend(child_forms);
        let forms_call = WatAST::List(forms_items, Span::unknown());
        let call = WatAST::List(
            vec![
                WatAST::Keyword(":wat::kernel::spawn-process".into(), Span::unknown()),
                forms_call,
            ],
            Span::unknown(),
        );
        let env = Environment::new();
        let process = match eval(&call, &env, world.symbols()) {
            Ok(p) => p,
            Err(e) => {
                // Write sentinel pid=0 so test doesn't hang on read.
                let bytes = 0i32.to_le_bytes();
                unsafe {
                    libc::write(
                        pid_w.as_raw_fd(),
                        bytes.as_ptr() as *const _,
                        4,
                    )
                };
                drop(pid_w);
                panic!("spawn-process failed: {}", e);
            }
        };

        // Extract grandchild pid and write to pid_pipe.
        let gchild_pid = grandchild_pid(&process);
        let pid_bytes = gchild_pid.to_le_bytes();
        let written = unsafe {
            libc::write(
                pid_w.as_raw_fd(),
                pid_bytes.as_ptr() as *const _,
                4,
            )
        };
        assert_eq!(written, 4, "pid_pipe write failed");

        // Drop pid_w (supervisor's copy). Drop the Process struct WITHOUT
        // calling wait — we're about to _exit, which orphans the grandchild
        // intentionally. Arc's Drop on ChildHandleInner would SIGKILL it —
        // prevent that by leaking.
        //
        // Leak strategy: forget the process value so ChildHandleInner::drop
        // (which sends SIGKILL + waitpid) does NOT run.
        std::mem::forget(process);
        drop(pid_w);

        // DIAGNOSTIC: optionally sleep before _exit to widen the gap
        // between fork and supervisor death. If grandchild's prctl wins
        // the race against supervisor's _exit, this sleep should give the
        // grandchild plenty of time to call prctl before supervisor dies.
        if delay_ms > 0 {
            unsafe {
                let ts = libc::timespec {
                    tv_sec: (delay_ms / 1000) as libc::time_t,
                    tv_nsec: ((delay_ms % 1000) * 1_000_000) as libc::c_long,
                };
                libc::nanosleep(&ts, std::ptr::null_mut());
            }
        }

        // Exit WITHOUT waiting for grandchild. This orphans the grandchild
        // whose PPID becomes 1 (init/subreaper). The kernel detects parent
        // death and delivers SIGTERM to the grandchild via PR_SET_PDEATHSIG.
        unsafe { libc::_exit(0) };
    }

    // ── Test (parent of supervisor) ───────────────────────────────────────
    // Close supervisor's end of pid_pipe.
    drop(pid_w);

    // Step 4: Wait for supervisor to exit. Supervisor exits as soon as it
    // spawns grandchild and writes pid — this returns immediately.
    let mut status: libc::c_int = 0;
    let waited = unsafe { libc::waitpid(supervisor_pid, &mut status, 0) };
    assert!(
        waited >= 0,
        "waitpid(supervisor) failed: {}",
        std::io::Error::last_os_error()
    );

    // Step 5: Read grandchild pid from pid_pipe.
    let mut pid_bytes = [0u8; 4];
    let nread = unsafe {
        libc::read(
            pid_r.as_raw_fd(),
            pid_bytes.as_mut_ptr() as *mut _,
            4,
        )
    };
    assert_eq!(nread, 4, "pid_pipe read returned {} (expected 4)", nread);
    let grandchild = i32::from_le_bytes(pid_bytes);
    assert!(grandchild > 0, "grandchild pid must be positive; got {}", grandchild);
    drop(pid_r);

    // Step 6: Open a pidfd for the grandchild and poll for process exit.
    // pidfd_open(pid, 0) returns a file descriptor that becomes POLLIN-readable
    // when the process exits. Linux 5.3+. No FD-inheritance required.
    let pidfd = unsafe {
        libc::syscall(libc::SYS_pidfd_open, grandchild as libc::c_long, 0i32 as libc::c_long)
    } as libc::c_int;
    assert!(
        pidfd >= 0,
        "pidfd_open(grandchild_pid={}) failed: {}",
        grandchild,
        std::io::Error::last_os_error()
    );

    let mut pollfd = libc::pollfd { fd: pidfd, events: libc::POLLIN, revents: 0 };
    let t0 = Instant::now();
    let poll_ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 1000) };
    let elapsed = t0.elapsed();
    unsafe { libc::close(pidfd) };

    assert!(
        poll_ret > 0,
        "pidfd_open POLLIN did not fire within 1s — cascade broken \
         (grandchild pid={}, poll_ret={}, elapsed={:?})",
        grandchild,
        poll_ret,
        elapsed
    );

    // Step 7: Verify grandchild process is no longer running.
    // pidfd POLLIN (step 6) is the definitive signal: the process has exited.
    // A zombie in the process table is acceptable — it means the process
    // exited and is awaiting reaping by init (since supervisor already
    // exited). We check /proc/<pid>/stat for the zombie state rather than
    // using kill(pid, 0) which returns 0 for both live AND zombie processes.
    //
    // Z = zombie (process exited, waiting to be reaped) — PASS.
    // R/S/D = still running (should not happen after pidfd POLLIN) — FAIL.
    let proc_stat = std::fs::read_to_string(format!("/proc/{}/stat", grandchild))
        .unwrap_or_default();
    // /proc/pid/stat field 3 is the state character (between the last ')' and next space).
    let state = proc_stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.trim_start().chars().next())
        .unwrap_or('?');
    assert!(
        // 'Z' = zombie (exited, awaiting reap) — PASS.
        // '?' = no such process (already reaped by init) — PASS.
        state == 'Z' || state == '?',
        "grandchild pid {} in unexpected state '{}' after cascade \
         (pidfd POLLIN fired at elapsed={:?}; expected zombie or gone)",
        grandchild,
        state,
        elapsed
    );
}
