//! Arc 170 Phase 1D probe — lifeline mechanism delivers orphan-cleanup via
//! substrate path (`:wat::kernel::spawn-process`).
//!
//! Verifies the substrate claim (Phase 1B + 1C): when the supervisor process
//! exits for any reason, the kernel closes the supervisor's copy of the
//! lifeline write-end → grandchild's shutdown worker detects POLLHUP on
//! lifeline read-end → trigger_shutdown cascade fires → grandchild exits.
//!
//! This is the substrate-mechanism equivalent of `probe_pdeathsig_kills_orphan_child`
//! (Slice C historical artifact). The observable contract is identical —
//! grandchild dies within 1s — but the mechanism is the lifeline pipe wired
//! through `spawn_process_child_branch` (Phase 1B: `init_shutdown_signal_with_inputs`)
//! and `child_branch_from_source` (Phase 1C: symmetric lifeline for fork-program).
//!
//! Cross-references:
//! - Phase 1B SCORE: `SCORE-FD-MULTIPLEX-PHASE-1B-SPAWN-PROCESS-LIFELINE.md`
//! - Phase 1C SCORE: `SCORE-FD-MULTIPLEX-PHASE-1C-FORK-PROGRAM-LIFELINE.md`
//! - Slice D baseline (10% race at delay=0 with PDEATHSIG): `SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md`
//! - Pure-libc lifeline proof (100/100 trials, 0 orphans): `tests/probe_lifeline_pipe_proof.rs`
//!
//! # Rendezvous mechanism update (2026-05-13)
//!
//! Original rendezvous: `done_pipe` FD-inheritance — grandchild inherited done_w;
//! when grandchild exited, done_w closed → POLLHUP on done_r. Phase 3 of arc 170
//! FD-multiplex adds canonical `close_inherited_fds_above_stdio` to spawn-process,
//! which closes inherited test-pipes in the grandchild at startup. done_pipe
//! rendezvous no longer works. Replaced with `pidfd_open(grandchild_pid, 0)` +
//! `poll(pidfd, POLLIN, 1000ms)` — POLLIN fires when the process exits.
//! Kernel-guaranteed, Linux 5.3+. Observable contract unchanged.
//!
//! # Design
//!
//! ```text
//! test process (parent-of-supervisor)
//!   │  forks
//!   ├─ supervisor (pid: supervisor_pid)
//!   │    │  calls substrate spawn-process (Phase 1B path)
//!   │    └─ grandchild (pid: grandchild_pid) — lifeline read-end registered
//!   │         │  with shutdown worker via init_shutdown_signal_with_inputs
//!   │         │  blocks on typed_recv (Slice B cascade wires)
//!   │
//!   └─ test: waitpid(supervisor) → supervisor _exit(0) →
//!            kernel closes supervisor's lifeline write-end →
//!            grandchild shutdown worker POLLHUP fires →
//!            trigger_shutdown → cascade wakes blocked recv →
//!            grandchild exits →
//!            pidfd_open(grandchild_pid) → poll(pidfd, POLLIN, 1000ms) fires →
//!            PASS: grandchild died within 1s
//! ```
//!
//! # Why this is structurally race-free
//!
//! The lifeline write-end is created BEFORE fork. The child inherits the
//! read-end atomically with fork(). No subsequent registration required —
//! contrast with PDEATHSIG's `prctl` call which must race against the parent's
//! `_exit`. The kernel FD-close-on-process-death invariant fires unconditionally.
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
//! Identical to `probe_pdeathsig_kills_orphan_child` — blocking recv on an
//! unbounded channel. The mechanism that wakes it differs:
//! - Pre-Phase-1B: SIGTERM → signal handler → wake-pipe → shutdown worker → trigger_shutdown
//! - Post-Phase-1B: lifeline EOF → POLLHUP on shutdown worker's poll(2) → trigger_shutdown
//! Same `RecvOutcome::Shutdown` outcome; same `(:wat::kernel::recv rx)` unblocks;
//! same clean exit. The test cannot tell the difference at the wat surface.
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
/// Post-Phase-1B, the shutdown cascade is triggered by lifeline EOF
/// (not SIGTERM): shutdown worker's poll(2) fires POLLHUP on the
/// lifeline read-end → trigger_shutdown → SHUTDOWN_TX drops →
/// SHUTDOWN_RX disconnects → typed_recv select! fires Shutdown arm →
/// recv returns RecvOutcome::Shutdown → RuntimeError propagates up →
/// spawn_process_child_branch calls libc::_exit.
const BLOCKING_CHILD_SRC: &str = r#"
    (:wat::core::defn :test::block-until-shutdown
      []
      -> :wat::core::nil
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

/// Phase 1D — orphaned grandchild dies via lifeline mechanism (Phase 1B substrate
/// path) within 1s after supervisor `_exit`. Verifies that `spawn_process_child_branch`
/// correctly wires the lifeline read-end into `init_shutdown_signal_with_inputs` so
/// the shutdown worker's poll(2) fires on POLLHUP when the parent process dies.
///
/// Structural counterpart of `probe_pdeathsig_kills_orphan_child` (Slice C
/// historical artifact) — same observable contract; different substrate mechanism.
///
/// # Shape
///
/// 1. Build a blocking wat world (done before any fork — supervisor inherits it).
/// 2. Create `pid_pipe` (supervisor → test).
/// 3. Fork supervisor.
/// 4. Supervisor: eval(spawn-process), extract grandchild pid, write to pid_pipe,
///    `_exit(0)`.
/// 5. Test: waitpid(supervisor), read grandchild pid from pid_pipe.
/// 6. Test: pidfd_open(grandchild_pid, 0) → poll(pidfd, POLLIN, 1000ms) — fires
///    when grandchild process exits.
/// 7. Verify grandchild process is gone (zombie or reaped).
#[test]
fn probe_lifeline_orphan_clean_via_substrate() {
    // Step 1: Build the world before forking. Supervisor inherits it via
    // fork's copy-on-write semantics. InMemoryLoader has no disk state.
    let world = freeze_ok(BLOCKING_CHILD_SRC);

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

        // Spawn grandchild via substrate spawn-process (Phase 1B path).
        // spawn_process_child_branch registers the lifeline read-end via
        // init_shutdown_signal_with_inputs before the bootstrap call. When
        // supervisor _exit(0)s below, the kernel closes the supervisor's
        // copy of lifeline_w → grandchild's worker POLLHUP fires → cascade.
        //
        // Phase 3: spawn_process_child_branch now calls
        // close_inherited_fds_above_stdio — no FD-inheritance rendezvous
        // possible; test uses pidfd_open instead.
        let call = WatAST::List(
            vec![
                WatAST::Keyword(":wat::kernel::spawn-process".into(), Span::unknown()),
                WatAST::Keyword(":test::block-until-shutdown".into(), Span::unknown()),
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
        // (which sends SIGKILL + waitpid) does NOT run. The lifeline mechanism
        // will clean up the grandchild instead.
        std::mem::forget(process);
        drop(pid_w);

        // Exit WITHOUT waiting for grandchild. This orphans the grandchild
        // whose PPID becomes 1 (init/subreaper). The kernel closes all FDs
        // held by this supervisor process — including the lifeline write-end
        // that spawn-process created inside eval() above. The grandchild's
        // shutdown worker detects POLLHUP on the lifeline read-end and
        // triggers the shutdown cascade. No signal. No timer. No race.
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
        "pidfd_open POLLIN did not fire within 1s — lifeline cascade broken \
         (grandchild pid={}, poll_ret={}, elapsed={:?}; check Phase 1B/1C substrate wiring)",
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
        "grandchild pid {} in unexpected state '{}' after lifeline cascade \
         (pidfd POLLIN fired at elapsed={:?}; expected zombie or gone)",
        grandchild,
        state,
        elapsed
    );
}
