//! A dead child must fail `spawn_process_peer` with a NAME, not hang the parent.
//!
//! `spawn_process_peer` used to keep the child's stdout write end (`output_tx`)
//! in the parent across `deliver_to_child`. The boot handshake waits for an
//! ack on that pipe. A dead child closes only ITS copy; the parent's copy
//! kept the pipe open; the wait never ended. A `wat --mcp` eval of a
//! process-locus spawn blocked the one-line JSON-RPC loop forever (attested
//! over 30s). Brackets already multiplex process IPC; the MCP turn is serial, but
//! spawn must *return* so the turn can finish.
//!
//! This gate execs a missing binary so the child dies at `execve`. GREEN is a
//! `RuntimeError` in well under a second. A hang is the old leak.
//!
//! Isolated by nextest (own process), so `WAT_RUNTIME_BIN` cannot leak into a
//! sibling spawn. A `Drop` guard restores it anyway.

use std::time::{Duration, Instant};
use wat::runtime::RuntimeErrorKind;

struct RestoreRuntimeBin(Option<std::ffi::OsString>);

impl Drop for RestoreRuntimeBin {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => std::env::set_var("WAT_RUNTIME_BIN", v),
            None => std::env::remove_var("WAT_RUNTIME_BIN"),
        }
    }
}

#[test]
fn a_doomed_child_fails_spawn_by_name_instead_of_hanging() {
    let _restore = RestoreRuntimeBin(std::env::var_os("WAT_RUNTIME_BIN"));
    std::env::set_var("WAT_RUNTIME_BIN", "/this/wat-runtime/does-not-exist");

    let world = wat::freeze::startup_beside(file!())
        .expect("startup for doomed-spawn noop post-spawn fn must succeed");
    let noop = world
        .symbols
        .get(":my::noop-post-spawn")
        .expect(":my::noop-post-spawn must be in the symbol table")
        .clone();
    let dummy_span = wat::rust_caller_span!();

    let t0 = Instant::now();
    let result = wat::kernel::spawn::spawn_process_peer(
        Vec::new(),
        noop,
        include_str!("doomed_child_boot_ack_does_not_hang__empty_env.wat")
            .trim()
            .to_string(),
        wat::edn::render::DEFAULT_MAX_FRAME_BYTES,
        None,
        &world.symbols,
        &dummy_span,
    );
    let elapsed = t0.elapsed();

    // Not a `StartupError` — `spawn_process_peer` returns `Result<Value, RuntimeError>`
    // ⚠ MEASURED, NOT READ — arc 296 Stone L, corrected by the central floor run.
    // The Phase 2 rider (which could not run cargo) READ `src/process/boot/mod.rs::boot_err`,
    // saw it is the one constructor on the boot path, and pinned
    // `head == ":wat::process::boot"` with the dead-child-ack reason. It flagged the guess as
    // "not independently confirmed". It was wrong, and the floor said so:
    //
    //     got #wat.runtime/MalformedForm {:head ":wat::kernel::spawn-process"
    //          :reason "could not build the exec plan: No such file or directory (os error 2)"}
    //
    // ★ THE BOOT-ACK PATH IS UNREACHABLE FROM THIS FIXTURE. The test sets
    // `WAT_RUNTIME_BIN=/this/wat-runtime/does-not-exist`, so exec-plan construction fails
    // BEFORE any child is spawned and therefore before any ack can be awaited. The dead-child
    // ack arm this test was named around is a DIFFERENT scenario (binary exists, child dies
    // mid-boot) and nothing here exercises it. What this test does prove — and its name says —
    // is that a doomed spawn fails BY NAME rather than hanging, which the exec-plan error
    // satisfies exactly. `[[feedback_measure_the_decomposition_never_read_it]]`
    let err = result.expect_err("doomed exec must be a named failure, not a live peer");
    assert!(
        matches!(
            err.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::kernel::spawn-process"
                && reason.starts_with("could not build the exec plan: ")
        ),
        "expected RuntimeErrorKind::MalformedForm(:wat::kernel::spawn-process, exec-plan); got {:?}",
        err
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "doomed spawn took {elapsed:?} — parent is still holding the ack write end"
    );
}
