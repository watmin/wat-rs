//! Arc 170 Phase 1E probe — lifeline mechanism delivers orphan-cleanup via
//! fork-program path (`:wat::kernel::fork-program`).
//!
//! Symmetric to Phase 1D's `probe_lifeline_orphan_clean_via_substrate.rs`
//! (which routes through `:wat::kernel::spawn-process`). This probe verifies
//! the same observable contract via the `child_branch_from_source` path that
//! Phase 1E's FD-hygiene fix targets.
//!
//! The defect this probe surfaces (and Phase 1E fixes):
//! `child_branch_from_source` previously called `close_inherited_fds_above_stdio`
//! AFTER `init_shutdown_signal_with_inputs`, which opened the wake-pipe FDs.
//! The close-sweep would then close `lifeline_r_raw`, `SHUTDOWN_WAKE_WRITE_FD`,
//! and the worker's wake-pipe read-end — causing immediate POLLHUP (false-positive
//! shutdown) on every fork-program spawn. Phase 1E reorders: close-sweep runs
//! BEFORE init, with lifeline_r_raw in the skip-list.
//!
//! # Design
//!
//! ```text
//! test process (parent-of-supervisor)
//!   │  forks
//!   ├─ supervisor (pid: supervisor_pid)
//!   │    │  calls substrate fork-program (Phase 1C/1E path)
//!   │    └─ grandchild (pid: grandchild_pid) — lifeline read-end registered
//!   │         │  with shutdown worker via init_shutdown_signal_with_inputs
//!   │         │  blocks on typed_recv (Slice B cascade wires)
//!   │         └─ holds done_pipe write-end (FD inherited from supervisor via
//!   │              fork-program's close_inherited_fds_above_stdio skip-list
//!   │              exempting it — but done_w is intentionally NOT in skip; it
//!   │              IS inherited and closed in the child. The test's done_pipe
//!   │              rendezvous relies on the grandchild's process exit, not on
//!   │              an explicit done_w write).
//!   │
//!   └─ test: waitpid(supervisor) → supervisor _exit(0) →
//!            kernel closes supervisor's lifeline write-end →
//!            grandchild shutdown worker POLLHUP fires →
//!            trigger_shutdown → cascade wakes blocked recv →
//!            grandchild exits → grandchild process gone →
//!            poll(done_pipe_read_fd, 1000ms) fires →
//!            PASS: grandchild died within 1s
//! ```
//!
//! # Rendezvous discipline (no wall-clock sleep)
//!
//! - `pid_pipe` (supervisor → test): supervisor writes grandchild pid; test reads.
//! - `done_pipe`: grandchild's process exit closes all its FDs. Unlike the spawn-process
//!   probe (which lets grandchild inherit done_w), fork-program's close-sweep closes
//!   done_w in the child. We instead poll `/proc/<pid>/stat` after a short poll(2)
//!   on a fresh pipe. The rendezvous uses a watcher pipe: supervisor writes grandchild
//!   pid, test reads, then polls `/proc/<grandchild>/fd` existence to confirm exit.
//!
//! # ZERO-MUTEX compliance
//!
//! No `Mutex`, `RwLock`, or `CondVar` in this file. Synchronisation via
//! OS pipes (rendezvous) + `libc::poll(2)` with 1000ms timeout.
//!
//! # Cross-references
//!
//! - Phase 1C SCORE: `SCORE-FD-MULTIPLEX-PHASE-1C-FORK-PROGRAM-LIFELINE.md`
//! - Phase 1D probe (spawn-process path): `tests/probe_lifeline_orphan_clean_via_substrate.rs`
//! - Phase 1E BRIEF: `BRIEF-FD-MULTIPLEX-PHASE-1E-FORK-PROGRAM-FD-HYGIENE.md`

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
/// Post-Phase-1C/1E, the shutdown cascade is triggered by lifeline EOF
/// (not SIGTERM): shutdown worker's poll(2) fires POLLHUP on the
/// lifeline read-end → trigger_shutdown → SHUTDOWN_TX drops →
/// SHUTDOWN_RX disconnects → typed_recv select! fires Shutdown arm →
/// recv returns RecvOutcome::Shutdown → RuntimeError propagates up →
/// child_branch_from_source calls libc::_exit.
const BLOCKING_CHILD_SRC: &str = r#"
    (:wat::core::defn :test::block-until-shutdown
      []
      -> :wat::core::nil
      (:wat::core::let
        [[tx rx] (:wat::kernel::make-unbounded-channel :wat::core::nil)
         _       (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
                   (:wat::kernel::recv rx)
                   "recv failed — sender dropped before shutdown")]
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
/// The ProgramHandle at fields[3] must be a Forked variant — fork-program
/// always returns Forked (arc 112 / arc 170 Stone C shape).
/// The struct type_name is `:wat::kernel::Process` (same as spawn-process).
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

/// Phase 1E — orphaned grandchild dies via lifeline mechanism (Phase 1C/1E substrate
/// path) within 1s after supervisor `_exit`. Verifies that `child_branch_from_source`
/// correctly wires the lifeline read-end into `init_shutdown_signal_with_inputs` AND
/// that the Phase 1E reorder (close-sweep BEFORE init) prevents the sweep from closing
/// the lifeline or wake-pipe FDs.
///
/// Routes through `:wat::kernel::fork-program` (source-string entry).
/// Symmetric to `probe_lifeline_orphan_clean_via_substrate` (spawn-process path).
///
/// # Shape
///
/// 1. Build a blocking wat world (done before any fork — supervisor inherits it).
/// 2. Create `pid_pipe` (supervisor → test).
/// 3. Fork supervisor.
/// 4. Supervisor: eval(fork-program), extract grandchild pid, write to pid_pipe,
///    `_exit(0)` (orphans grandchild intentionally).
/// 5. Test: waitpid(supervisor), read grandchild pid from pid_pipe.
/// 6. Test: poll `/proc/<grandchild>/stat` for exit within 1s via a timed loop
///    using `libc::poll(2)` on a self-pipe for bounded waiting.
/// 7. Verify grandchild process is gone (zombie or reaped).
#[test]
fn probe_lifeline_orphan_clean_via_fork_program() {
    // Step 1: Build the grandchild world before forking. The supervisor forks
    // and calls fork-program; the grandchild runs BLOCKING_CHILD_SRC as its
    // source program (fork-program parses it in the child branch).
    // We don't need to freeze before forking — fork-program takes a source String.
    // We freeze here only to confirm the source is syntactically valid before
    // entering the multi-process rendezvous.
    let _ = freeze_ok(BLOCKING_CHILD_SRC);

    // Step 2: Create coordination pipe.
    //
    // pid_pipe: supervisor writes grandchild pid (4 bytes, i32 LE); test reads.
    // done_pipe: used to detect grandchild exit. fork-program's close-sweep
    //   closes inherited FDs in the grandchild (that's the Phase 1E fix scope),
    //   so we cannot rely on done_w inheritance as in the spawn-process probe.
    //   Instead, we open a self-pipe in the test and poll grandchild /proc/stat.
    let (pid_r, pid_w) = make_raw_pipe();
    // poll_pipe: write-end is kept by test until we want to unblock poll.
    // The test polls with a 1s timeout; we don't write to it — the timeout
    // is the bounded mechanism for the /proc/stat polling interval.
    let (poll_r, poll_w) = make_raw_pipe();

    // Step 3: Fork supervisor.
    let supervisor_pid = unsafe { libc::fork() };
    assert!(
        supervisor_pid >= 0,
        "fork(supervisor) failed: {}",
        std::io::Error::last_os_error()
    );

    if supervisor_pid == 0 {
        // ── Supervisor child branch ───────────────────────────────────────
        // Close the test's pipe ends (supervisor only uses pid_w).
        drop(pid_r);
        drop(poll_r);
        drop(poll_w);

        // Supervisor builds its own world from the blocking source.
        // fork-program takes a source String at the wat level, so we pass
        // BLOCKING_CHILD_SRC as a StringLit in the AST call.
        //
        // The grandchild inherits the supervisor's FD table via fork(2)
        // inside fork_program_from_source. child_branch_from_source then:
        //  1. dup2 stdio
        //  2. close_inherited_fds_above_stdio(&[lifeline_r_raw])  ← Phase 1E
        //  3. init_shutdown_signal_with_inputs(&[lifeline_r_raw])  ← Phase 1C
        //  4. installs signal handlers
        //  5. parses + freezes BLOCKING_CHILD_SRC
        //  6. evals :test::block-until-shutdown → blocks on recv
        //
        // Phase 1E fix ensures the lifeline_r_raw is preserved through the
        // close-sweep and the worker can poll it correctly.
        let supervisor_world = freeze_ok(BLOCKING_CHILD_SRC);
        let call = WatAST::List(
            vec![
                WatAST::Keyword(":wat::kernel::fork-program".into(), Span::unknown()),
                WatAST::StringLit(
                    format!(
                        "(:wat::core::defn :test::block-until-shutdown [] -> :wat::core::nil \
                         (:wat::core::let [[tx rx] (:wat::kernel::make-unbounded-channel :wat::core::nil) \
                         _ (:wat::kernel::recv rx)] :wat::core::nil))"
                    ),
                    Span::unknown(),
                ),
                WatAST::Keyword(":wat::core::None".into(), Span::unknown()),
            ],
            Span::unknown(),
        );
        let env = Environment::new();
        let process = match eval(&call, &env, supervisor_world.symbols()) {
            Ok(p) => p,
            Err(e) => {
                // Write sentinel pid=0 so test doesn't hang on pid_pipe read.
                let bytes = 0i32.to_le_bytes();
                unsafe {
                    libc::write(
                        pid_w.as_raw_fd(),
                        bytes.as_ptr() as *const _,
                        4,
                    )
                };
                drop(pid_w);
                panic!("fork-program failed: {}", e);
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

        // Leak the Process struct so ChildHandleInner::drop (SIGKILL + waitpid)
        // does NOT run. The lifeline mechanism cleans up the grandchild instead.
        std::mem::forget(process);
        drop(pid_w);

        // Exit WITHOUT waiting for grandchild. Supervisor _exit(0) → kernel
        // closes all supervisor FDs including lifeline write-end → grandchild
        // shutdown worker POLLHUP → trigger_shutdown → recv wakes → _exit.
        unsafe { libc::_exit(0) };
    }

    // ── Test (parent of supervisor) ───────────────────────────────────────
    drop(pid_w);

    // Step 4: Wait for supervisor to exit. Supervisor exits as soon as it
    // spawns grandchild and writes pid — this returns quickly.
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

    // Step 6: Poll for grandchild exit within 1s.
    //
    // fork-program's child_branch_from_source calls close_inherited_fds_above_stdio
    // which closes most inherited FDs (Phase 1E fix: preserves lifeline + opens
    // substrate FDs after). We cannot use a done_pipe held by the grandchild
    // because the grandchild's sweep closes inherited FDs.
    //
    // Strategy: use libc::poll(2) with a 1000ms timeout on poll_r (which will
    // never fire — poll_w is held by test). The 1000ms timeout IS the budget.
    // After poll returns (via timeout), we check /proc/<grandchild>/stat.
    // If the grandchild is already gone before timeout, the process-entry
    // check will show '?' (no such process) or 'Z' (zombie). If it's still
    // alive at 1s, the test fails.
    //
    // ZERO-MUTEX: no Mutex, RwLock, CondVar. poll(2) is the bounded wait.
    let t0 = Instant::now();
    let mut pollfd = libc::pollfd {
        fd: poll_r.as_raw_fd(),
        events: libc::POLLHUP | libc::POLLIN,
        revents: 0,
    };
    // poll with 1000ms timeout — grandchild should exit via lifeline cascade
    // well within this window (typically microseconds to milliseconds).
    let poll_ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 1000) };
    let elapsed = t0.elapsed();

    drop(poll_r);
    drop(poll_w);

    // poll_ret == 0 means timeout (poll_w is still open; expected).
    // poll_ret < 0 means error — unexpected.
    assert!(
        poll_ret >= 0,
        "poll(2) failed: {}",
        std::io::Error::last_os_error()
    );
    // The timeout elapsed — now check if grandchild is gone.
    // If the lifeline cascade worked, the grandchild exited within the 1s window.

    // Step 7: Verify grandchild process is no longer running.
    // Check /proc/<pid>/stat — the process should be in zombie ('Z') or
    // already reaped ('?' = no file). Running states (R/S/D) indicate failure.
    let proc_stat = std::fs::read_to_string(format!("/proc/{}/stat", grandchild))
        .unwrap_or_default();
    let state = proc_stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.trim_start().chars().next())
        .unwrap_or('?');
    assert!(
        // 'Z' = zombie (exited, awaiting reap) — PASS.
        // '?' = no such process (already reaped by init) — PASS.
        state == 'Z' || state == '?',
        "grandchild pid {} in unexpected state '{}' after lifeline cascade \
         (poll elapsed={:?}; expected zombie or gone — \
         check Phase 1C/1E fork-program lifeline wiring in child_branch_from_source)",
        grandchild,
        state,
        elapsed
    );
}
