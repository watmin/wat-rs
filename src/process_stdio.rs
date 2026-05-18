//! Process-scope stdio surface for substrate-internal use.
//!
//! # The two-phase contract
//!
//! The substrate needs fd 0/1/2 at TWO non-overlapping times:
//!
//! 1. **During `:user::main`** — user code's `println` / `eprintln` /
//!    `readln` flow through `AmbientStdio` (a thread-local installed
//!    by `freeze::bootstrap_wat_vm_process`). AmbientStdio's
//!    `PipeReader` / `PipeWriter` own **dup'd copies** of fd 0/1/2
//!    minted by [`lend_ambient`]. AmbientStdio's `Drop` closes the
//!    dup'd copies — NOT the raw fds.
//!
//! 2. **AFTER `:user::main` returns or panics** — substrate's
//!    `emit_structured_exit` paths (in `spawn_process.rs` + `fork.rs`)
//!    emit `#wat.kernel/ProcessPanics{...}` EDN envelopes via
//!    [`emit_panic_envelope`] so the parent's pipe reader can extract
//!    the panic chain (per arc 170 slice 1i structured-exit protocol).
//!    This MUST work AFTER AmbientStdio has dropped — i.e., fd 2 must
//!    still be open at substrate-teardown time.
//!
//! # The dup is load-bearing
//!
//! The dup pattern in [`lend_ambient`] separates lifetimes:
//! - AmbientStdio's drop releases its dup'd copies (fd 3/4/5 or similar)
//! - Raw fd 0/1/2 stay open for the process lifetime (no Rust OwnedFd
//!   governs them; they are OS-managed standard streams)
//! - [`emit_panic_envelope`] writes raw fd 2 directly via `libc::write`;
//!   succeeds because fd 2 was never closed
//!
//! # The dup-removal incident (arc 211)
//!
//! Commit `3c1cb51` removed the dup believing it redundant. This broke
//! step 2: AmbientStdio drop closed fd 0/1/2 directly; subsequent
//! `emit_panic_envelope` writes to a closed fd; parent saw empty
//! stderr; "structured-stderr-only contract violation" fired across 7+
//! test targets. The arc 211c audit pinpointed the cause; arc 211d
//! reverted via `7071b27`.
//!
//! **DO NOT remove the dup** in `lend_ambient` without first replacing
//! it with explicit Rust-level ownership coordination. The dup IS the
//! coordination between phase 1 (AmbientStdio's lifetime) and phase 2
//! (substrate-teardown's raw-fd write). Removing the OS-level
//! coordination without providing a Rust-level replacement breaks step
//! 2 silently (the write fails; the substrate panics fail to surface).
//!
//! # Why a module, not a type
//!
//! Per four-questions discipline + `feedback_no_new_types`: these are
//! two related free functions namespaced by module. A wrapper type
//! (ZST with methods) was rejected as over-engineering — the borrow
//! checker can't enforce the lifetime ordering across `&'static`
//! anyway, so a type's shape would imply discipline it can't deliver.
//! Free functions in a module name the contract via the module name +
//! function names without false type-level promises.

use std::sync::Arc;

/// Construct an [`AmbientStdio`](crate::thread_io::AmbientStdio) with
/// dup'd copies of fd 0/1/2. AmbientStdio's drop closes the dup'd
/// copies; raw fd 0/1/2 stay open for the process lifetime.
///
/// Called from `freeze::bootstrap_wat_vm_process` to install the
/// thread-local AmbientStdio that user code (`:user::main`) uses for
/// `println` / `eprintln` / `readln`. The dup pattern is load-bearing
/// per the module-level docs.
///
/// # SAFETY
///
/// `libc::dup` returns a freshly-opened fd on success or -1 on error.
/// On failure we hand back -1; the resulting `PipeReader` / `PipeWriter`
/// carries an unusable fd that surfaces clean diagnostics on
/// read/write attempts. Orchestrator-level work still proceeds.
pub fn lend_ambient() -> crate::thread_io::AmbientStdio {
    use std::os::fd::FromRawFd;
    fn dup_fd(fd: i32) -> i32 {
        let r = unsafe { libc::dup(fd) };
        if r < 0 {
            -1
        } else {
            r
        }
    }
    let stdin_fd = dup_fd(0);
    let stdout_fd = dup_fd(1);
    let stderr_fd = dup_fd(2);
    let stdin: Arc<dyn crate::io::WatReader> = Arc::new(
        crate::io::PipeReader::from_owned_fd(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(stdin_fd)
        }),
    );
    let stdout: Arc<dyn crate::io::WatWriter> = Arc::new(
        crate::io::PipeWriter::from_owned_fd(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(stdout_fd)
        }),
    );
    let stderr: Arc<dyn crate::io::WatWriter> = Arc::new(
        crate::io::PipeWriter::from_owned_fd(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(stderr_fd)
        }),
    );
    crate::thread_io::AmbientStdio {
        stdin,
        stdout,
        stderr,
    }
}

/// Emit a panic-envelope line directly to fd 2 via `libc::write`.
///
/// Used by substrate's `emit_structured_exit` paths to write the
/// `#wat.kernel/ProcessPanics{...}` EDN envelope to stderr after
/// AmbientStdio has dropped (per the two-phase contract; see
/// module-level docs).
///
/// Bypasses `std::io::Stderr`'s Mutex (which the substrate doesn't
/// hold in panic paths) and uses raw fd 2 directly. Handles partial
/// writes via loop — `libc::write` is permitted to return fewer bytes
/// than requested per POSIX.
///
/// Write errors are ignored: stderr failure has no recovery path at
/// substrate-teardown time. If the write fails (e.g., parent pipe
/// closed early), the substrate's exit code still signals the failure
/// via the IPC contract (recovery doc § 13).
pub fn emit_panic_envelope(s: &str) {
    let bytes = s.as_bytes();
    let mut written = 0;
    while written < bytes.len() {
        let n = unsafe {
            libc::write(
                2,
                bytes.as_ptr().add(written) as *const libc::c_void,
                bytes.len() - written,
            )
        };
        if n <= 0 {
            break;
        }
        written += n as usize;
    }
}
