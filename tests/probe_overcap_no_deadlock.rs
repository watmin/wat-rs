//! Stone 259-flood — the TRUE over-cap deadlock probe.
//!
//! A child floods its stdout with 1 MiB (1,048,576 bytes) of un-terminated
//! data (no newline). The pipe's default buffer is ~64 KiB; after reading
//! DEFAULT_MAX_FRAME_BYTES (512 KiB), the parent's framer fires TooLarge and
//! must return IMMEDIATELY without blocking on the error channel.
//!
//! At HEAD (before the fix), `ProcessPeerBundle::recv()` maps TooLarge →
//! Disconnected (in `take_frame`), then calls `self.err.recv()` — but the
//! child is alive and blocked in `write_all` (pipe full, because parent stopped
//! draining), so:
//!   parent blocks on `err.recv()` ↔ child blocks on `write_all` → DEADLOCK.
//!
//! ## Why Rust-level (not WAT eval for recv')?
//!
//! `ProcessPeerBundle` is wrapped in `ThreadOwnedCell` by `spawn-program'`.
//! When `recv'` is called from a DIFFERENT thread (a timeout thread), the
//! `ThreadOwnedCell` check fires with a cross-thread error — not the deadlock
//! signal. Testing at the Rust level bypasses `ThreadOwnedCell` and exercises
//! `ProcessPeerBundle::recv()` directly from the test thread with a hard
//! timeout via a background watchdog thread.
//!
//! The child is a WAT-spawned process that writes 1 MiB to stdout (the comms
//! pipe) then exits. At HEAD: parent reads ~512 KiB, TooLarge fires, maps to
//! Disconnected, parent calls `err.recv()`, which blocks because the child is
//! alive and stuck on `write_all`. DEADLOCK. Watchdog calls `_exit(124)`.
//!
//! After fix: parent reads ~512 KiB, TooLarge returns `FrameTooLarge`
//! distinctly, `ProcessPeerBundle::recv()` short-circuits (no `err.recv()`
//! call), drops the bundle (EPIPE kills the child), returns a rejection.
//! Test passes within milliseconds. Watchdog sleeps its full 8s and exits.

use std::sync::Arc;
use std::time::Duration;

use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment};
use wat::span::Span;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Build `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program'".into(), Span::unknown()),
            WatAST::List(
                vec![WatAST::Keyword(":wat::spawn::process".into(), Span::unknown())],
                Span::unknown(),
            ),
            forms_call,
        ],
        Span::unknown(),
    )
}

// ─── TRUE FLOOD test (Rust-level, bypassing ThreadOwnedCell) ─────────────────

/// Child floods stdout with 1 MiB (2^20 bytes) of un-terminated data.
///
/// Uses WAT child program to build 2^20 = 1,048,576 bytes via `double-string`.
/// The parent calls `ProcessPeerBundle::recv()` DIRECTLY (not via WAT eval),
/// bypassing `ThreadOwnedCell` so the recv runs on the test thread.
///
/// A background watchdog thread calls `libc::_exit(124)` after 8 seconds.
/// At HEAD: parent deadlocks → watchdog fires → test process exits non-zero
/// → test harness reports FAIL.
/// After fix: `ProcessPeerBundle::recv()` returns a `FrameTooLarge` rejection
/// fast (< 1 second) → watchdog never fires → test PASSES.
#[test]
fn true_flood_overcap_no_deadlock() {
    // Arm the watchdog: if this test blocks more than 8 seconds, kill the process.
    arm_watchdog(Duration::from_secs(8));

    let world = freeze_ok("");

    // WAT child program: build a 1 MiB string via doubling and print-raw' it.
    // double-string "x" 20 → 2^20 = 1,048,576 'x' bytes. No newline.
    const FLOOD_CHILD_SRC: &str = r#"
        (:wat::core::defn :user::double-string
            [s <- :wat::core::String n <- :wat::core::i64]
            -> :wat::core::String
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::String
            s
            (:user::double-string (:wat::core::String/concat s s) (:wat::core::- n 1))))

        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::kernel::print-raw'
            (:user::double-string "x" 20)))
    "#;

    let spawn_call = build_spawn_process_call(FLOOD_CHILD_SRC);
    let child_value = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    // Extract the ProcessPeerBundle directly (bypassing ThreadOwnedCell).
    // This lets us call recv() from the test thread without the thread-boundary check.
    let bundle = extract_process_bundle(child_value);

    // Call recv() directly — this is the path that deadlocks at HEAD.
    // If this blocks, the watchdog fires after 8 seconds with _exit(124).
    let result = bundle.recv();

    // recv() must return an error — never Ok on a TooLarge frame.
    match result {
        Ok(v) => panic!(
            "true-flood: recv must raise on FrameTooLarge; got Ok({:?})",
            v
        ),
        Err(e) => {
            eprintln!("[true_flood_overcap_no_deadlock] recv raised (expected): {:?}", e);
            // Any error is acceptable — the key invariant is no deadlock.
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract the `ProcessPeerBundle` from a `Value::RustOpaque(Process')`.
///
/// Takes the bundle OUT of the `Option` so we get a concrete `ProcessPeerBundle`
/// we can call `recv()` on without going through `ThreadOwnedCell::with_ref`.
///
/// SAFETY rationale: we're on the same thread that created the bundle
/// (the test thread spawned the child via WAT eval). The `ThreadOwnedCell`
/// owner IS the current thread. We bypass `with_mut` only to avoid the
/// borrowing complications of holding a mutable borrow across `recv()`.
fn extract_process_bundle(child: wat::runtime::Value) -> wat::kernel::spawn::ProcessPeerBundle {
    use wat::kernel::spawn::{ProcessPeerCell, PROCESS_PEER_TYPE_PATH};
    use wat::rust_deps::marshal::{downcast_ref_opaque, rust_opaque_arc};

    // Validate the outer Value shape.
    let inner_arc = rust_opaque_arc(&child, PROCESS_PEER_TYPE_PATH, "test", Span::unknown())
        .expect("child must be RustOpaque(Process')");

    // Downcast the payload to `ProcessPeerCell` =
    // `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`.
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &inner_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:extract_bundle",
        Span::unknown(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Take the ProcessPeerBundle out of the Option via with_mut.
    // We're on the owner thread, so this succeeds.
    cell.with_mut("test:take", Span::unknown(), |opt| {
        opt.take().expect("bundle must not already be taken")
    })
    .expect("with_mut must not cross thread boundary")
}

/// Arm a watchdog thread that calls `_exit(124)` after `timeout`.
///
/// If the test deadlocks (parent blocked in `err.recv()`), the watchdog
/// terminates the test process with exit code 124. The test harness
/// (Cargo test runner) sees a non-zero exit and reports FAIL.
///
/// If the test completes before the timeout, the watchdog sleeps
/// its full `timeout` and calls `_exit(124)` — but the test is already
/// done, so this is a benign late cleanup (the test process exits normally
/// before the watchdog fires in the no-deadlock case, since Cargo exits
/// the process after all tests complete).
///
/// Note: the watchdog thread is intentionally NOT joined. It is a
/// fire-and-forget sentinel. The test either finishes fast (before the
/// watchdog) or the watchdog kills the stuck process.
fn arm_watchdog(timeout: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        // If we reach here, the test deadlocked.
        eprintln!(
            "\n[WATCHDOG] true_flood_overcap_no_deadlock: recv() did not return within {:?} \
             — the parent is blocked on err.recv() while the child is blocked on write_all \
             (pipe full). This is the TooLarge deadlock bug at HEAD. \
             Killing test process with exit code 124.",
            timeout
        );
        // SAFETY: _exit is always safe to call; the process exits immediately.
        unsafe { libc::_exit(124) };
    });
}
