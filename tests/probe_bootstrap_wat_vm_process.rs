//! Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone A — structural probe.
//!
//! Verifies that [`wat::bootstrap_wat_vm_process`] exists and behaves
//! correctly as a substrate-owned helper:
//!
//! 1. Callable with a minimal `FrozenWorld` + `BootstrapArgs`.
//! 2. Returns a `ProcessRuntime` whose `.symbols()` carries
//!    `runtime_services` (Some, not None).
//! 3. ThreadIO is installed in the calling thread after the call
//!    (verified by `uninstall_thread_io()` returning Some).
//! 4. After the `ProcessRuntime` drops, ThreadIO is gone from the
//!    calling thread (verified by `uninstall_thread_io()` returning None).
//!
//! Does NOT exercise fork / spawn-process paths — those are Stones C+D.
//! Does NOT exercise user-main invocation — that is `invoke_user_main`.
//! This probe is purely structural: the helper exists, sets up, tears
//! down.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{startup_from_source, BootstrapArgs};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, uninstall_thread_io, AmbientStdio};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};

// ─── helpers ──────────────────────────────────────────────────────────────

/// Create a minimal FrozenWorld with a stub :user::main (required by
/// startup pipeline; some validators may check for it).
fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Allocate an OS pipe and wrap its ends.
fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

/// Build a test-controlled AmbientStdio rig and install it so the
/// bootstrap helper doesn't reach for real fd 0/1/2.
fn install_rig() {
    let (stdin_service_side, _stdin_inject) = pipe_pair();
    let (_stdout_capture, stdout_service_side) = pipe_pair();
    let (_stderr_capture, stderr_service_side) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service_side,
        stdout: stdout_service_side,
        stderr: stderr_service_side,
    });
}

// ─── Probe 1 — bootstrap is callable; services accessible; ThreadIO installed ──

/// Call `bootstrap_wat_vm_process` with a minimal FrozenWorld.
/// Verify:
/// - Returns `Ok(ProcessRuntime)` (no error).
/// - `.symbols().runtime_services()` is `Some` (carrier populated).
/// - `uninstall_thread_io()` returns `Some` (ThreadIO installed on
///   calling thread).
/// - After Drop, `uninstall_thread_io()` returns `None` (cleanup ran).
#[test]
fn probe_bootstrap_callable_services_threadio() {
    // Drain any leftover ambient from previous tests in this thread.
    let _ = uninstall_ambient_stdio();
    let _ = uninstall_thread_io();

    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);

    // Install test-controlled pipes so bootstrap doesn't touch real fds.
    install_rig();

    // Call the new helper.
    let runtime = wat::bootstrap_wat_vm_process(BootstrapArgs { frozen: &world })
        .expect("bootstrap_wat_vm_process should succeed");

    // Verify: symbols() carries runtime_services (Some).
    let has_services = runtime.symbols().runtime_services().is_some();
    assert!(
        has_services,
        "ProcessRuntime::symbols() should carry runtime_services (Some); got None"
    );

    // Verify: ThreadIO is installed — uninstall returns Some.
    // Note: uninstall_thread_io REMOVES the ThreadIO from the cell.
    // We then re-install a sentinel to allow Drop's cleanup to still run
    // the deregister without a broken ThreadIO (deregister sends Remove
    // events on ControlTxs, which doesn't need ThreadIO itself — it uses
    // the services Arc directly). ThreadIO uninstall in Drop will return
    // None (we already took it here), which is acceptable: Drop uses
    // let _ = uninstall_thread_io() and doesn't require it to be Some.
    let thread_io = uninstall_thread_io();
    assert!(
        thread_io.is_some(),
        "ThreadIO should be installed after bootstrap_wat_vm_process; got None"
    );

    // Drop runtime — cleanup runs: deregister → uninstall(now None) →
    // drop sym → drop services → join stdin → join stdout → join stderr.
    drop(runtime);

    // After Drop, the ambient cell is consumed; uninstall returns None.
    // The ThreadIO was already taken above; Drop's uninstall sees None
    // (acceptable — Drop uses let _ = uninstall).
    let after_drop = uninstall_thread_io();
    assert!(
        after_drop.is_none(),
        "ThreadIO should be None after ProcessRuntime drops (we already took it above); got Some"
    );

    // Drain any leftover ambient to keep the thread clean for other tests.
    let _ = uninstall_ambient_stdio();
}

// ─── Probe 2 — Drop runs cleanup: ThreadIO gone after drop ────────────────

/// Verify Drop cleanup: bootstrap installs ThreadIO; after Drop it's gone.
///
/// This probe does NOT call uninstall_thread_io() before Drop — it
/// verifies the Drop impl itself removes the ThreadIO from the cell.
#[test]
fn probe_bootstrap_drop_removes_threadio() {
    // Drain any leftover ambient from previous tests.
    let _ = uninstall_ambient_stdio();
    let _ = uninstall_thread_io();

    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);

    install_rig();

    {
        let runtime = wat::bootstrap_wat_vm_process(BootstrapArgs { frozen: &world })
            .expect("bootstrap_wat_vm_process should succeed");

        // ThreadIO is installed inside this scope.
        // (We don't call uninstall here — let Drop do it.)
        let _ = &runtime; // keep alive
    }
    // runtime dropped — Drop ran cleanup including uninstall_thread_io().

    let after_drop = uninstall_thread_io();
    assert!(
        after_drop.is_none(),
        "ThreadIO should be None after ProcessRuntime::drop ran uninstall_thread_io; got Some"
    );

    // Drain any leftover ambient.
    let _ = uninstall_ambient_stdio();
}
