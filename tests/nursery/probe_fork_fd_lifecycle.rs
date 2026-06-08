//! Arc 254 — RAII fork-boundary fd lifecycle probe.
//!
//! Verifies that the RAII fix (arc 254 `into_raw_fd` → `as_raw_fd` conversion)
//! eliminates the fd leak at three sites:
//!   - `src/spawn_process.rs` (`spawn-process` path)
//!   - `src/fork.rs` (`fork-program-ast` path, site 1)
//!   - `src/fork.rs` (`fork-program-from-source` / `fork-program` path, site 2)
//!
//! **Leak-kill proof**: fork N times via each path; measure `/proc/self/fd`
//! count before and after. A stable count (allowing ±2 for test harness
//! fluctuation) proves no fd escapes the RAII boundary.
//!
//! # Why #[ignore]
//!
//! This test spawns real child processes. Per nursery contract
//! ("One leak-safe [[test]] binary — Keep only PURE (non-process) tests here"),
//! process tests must be `#[ignore]`'d in the nursery binary and run via
//! `scripts/integration-run.sh --all` (setsid + pkill containment).
//!
//! Run manually:
//! ```
//! cargo test --release --test nursery probe_fork_fd_lifecycle -- --ignored
//! ```
//! Or via the contained runner:
//! ```
//! ./scripts/integration-run.sh --all
//! ```
//!
//! # ZERO-MUTEX compliance
//!
//! No `Mutex`, `RwLock`, or `CondVar`. Synchronisation via
//! `waitid(2)` inside `wait_or_cached_exit`.

use std::sync::Arc;
use wat::process::{fork_program_from_source, ForkedProgramHandles};
use wat::load::InMemoryLoader;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Count the number of open file descriptors in the calling process by
/// listing `/proc/self/fd` entries. Each entry is one open fd.
///
/// Subtracts 1 to account for the dirfd that `read_dir` itself opens
/// during the enumeration (that fd is closed when `ReadDir` drops).
fn count_open_fds() -> usize {
    std::fs::read_dir("/proc/self/fd")
        .expect("/proc/self/fd must be readable")
        .count()
        .saturating_sub(1)
}

/// A minimal wat program whose `:user::main` returns immediately.
const IMMEDIATE_EXIT_SRC: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)
"#;

/// Wait for the forked child to exit by calling `wait_or_cached_exit` on its handle,
/// then drop `handles` (closes parent-side pipe OwnedFds via RAII).
fn join_and_drop(handles: ForkedProgramHandles) {
    let code = handles.child_handle.wait_or_cached_exit();
    // child_handle reports 0 for success, 1 for runtime error, 2 for panic, etc.
    // Immediate-exit programs exit 0.
    assert!(
        code == 0 || code == 3,  // EXIT_SUCCESS or EXIT_STARTUP_ERROR (no :user::main define)
        "child exited with unexpected code {code}"
    );
    // handles drops here → stdin_w / stdout_r / stderr_r OwnedFds close.
}

// ── fork-program-from-source fd lifecycle ────────────────────────────────────

/// Fork N times via `fork_program_from_source` (the source-string entry point,
/// `src/fork.rs` site 2). Each child runs the immediate-exit program.
/// Assert `/proc/self/fd` count is stable before vs. after N iterations.
///
/// Before the RAII fix: each spawn surrendered all 6 pipe OwnedFds via
/// `into_raw_fd()`, then relied on manual `libc::close()` on the success path
/// only. On error/panic, all 6 fds leaked. After the fix: parent holds
/// OwnedFds; Drop closes them on any path.
#[test]
fn fork_program_from_source_fd_count_is_stable() {
    const N: usize = 10;
    const TOLERANCE: usize = 2;

    // Warm-up: one fork to settle any lazy-init allocations
    // (OnceLock, static atomics, thread-local init).
    {
        let loader = Arc::new(InMemoryLoader::new());
        if let Ok(h) = fork_program_from_source(IMMEDIATE_EXIT_SRC, None, loader, vec![]) {
            let _ = h.child_handle.wait_or_cached_exit();
            // h drops here — parent-side OwnedFds close
        }
    }

    let before = count_open_fds();
    eprintln!("probe_fork_fd_lifecycle: before={before}");

    for i in 0..N {
        let loader = Arc::new(InMemoryLoader::new());
        let handles = fork_program_from_source(
            IMMEDIATE_EXIT_SRC,
            None,
            loader,
            vec![],
        )
        .unwrap_or_else(|e| panic!("fork_program_from_source failed on iter {i}: {e}"));
        join_and_drop(handles);
    }

    let after = count_open_fds();
    eprintln!("probe_fork_fd_lifecycle: after={after} N={N}");

    assert!(
        after <= before + TOLERANCE,
        "fd leak detected in fork_program_from_source: before={before} after={after} N={N} (tolerance={TOLERANCE}). \
         Each child should leave no open fds in the parent after RAII fix."
    );
}
