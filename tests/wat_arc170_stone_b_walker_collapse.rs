//! Arc 170 Stone B — walker collapse: hide `*_join-result` from user
//! namespace.
//!
//! A new walker check in `src/check.rs` rejects user-namespace calls to
//! `:wat::kernel::Thread/join-result` and `:wat::kernel::Process/join-result`.
//! Substrate-namespace callers (`:wat::*` prefix on the enclosing fn FQDN)
//! remain unaffected.
//!
//! Binary rule:
//! - Caller's enclosing wat `define` FQDN starts with `:wat::` → ALLOWED
//! - Otherwise → compile error pointing users to the canonical
//!   replacement (`:wat::kernel::Thread/drain-and-join` /
//!   `:wat::kernel::Process/drain-and-join`).
//!
//! ## Tests
//!
//! - **Negative (Thread)**: user-namespace fn calls `Thread/join-result`
//!   → startup fails; error message names `Thread/drain-and-join`.
//! - **Negative (Process)**: same shape for Process.
//! - **Positive (Thread)**: `:wat::*` namespace fn calls
//!   `Thread/join-result` → startup succeeds.
//! - **Positive (Process)**: `:wat::*` namespace fn calls
//!   `Process/join-result` → startup succeeds.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Returns the Debug-formatted error bundle from a startup that MUST
/// fail. Tests grep this for the new walker variant + message text.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts the given source starts up cleanly.
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

// ─── Negative cases — user-namespace callers MUST be rejected ──────────

#[test]
fn stone_b_user_namespace_thread_join_result_is_rejected() {
    // A user-namespace fn (`:my::test::call-thread-join`) reaches for
    // `:wat::kernel::Thread/join-result` directly. Post-Stone-B, the
    // walker refuses; the diagnostic teaches the canonical replacement
    // (`Thread/drain-and-join`).
    let src = r#"
        (:wat::core::define
          (:my::test::call-thread-join
            (thr :wat::kernel::Thread<wat::core::nil,wat::core::nil>)
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>)
          (:wat::kernel::Thread/join-result thr))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("Thread/join-result"),
        "error should name the rejected verb; got: {}",
        err
    );
    assert!(
        err.contains("drain-and-join"),
        "error should teach the canonical replacement (`Thread/drain-and-join`); got: {}",
        err
    );
}

#[test]
fn stone_b_user_namespace_process_join_result_is_rejected() {
    // Mirror of the Thread negative case for Process.
    let src = r#"
        (:wat::core::define
          (:my::test::call-process-join
            (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>)
          (:wat::kernel::Process/join-result proc))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("Process/join-result"),
        "error should name the rejected verb; got: {}",
        err
    );
    assert!(
        err.contains("drain-and-join"),
        "error should teach the canonical replacement (`Process/drain-and-join`); got: {}",
        err
    );
}

// ─── Positive cases — substrate-namespace callers stay allowed ─────────

// The substrate stdlib loaded on every `startup_from_source` already
// contains substrate-namespace fns that call `Thread/join-result` and
// `Process/join-result` directly — for Thread, `:wat::test::run-thread-
// driver` at `wat/test.wat`; for Process, `:wat::test::run-hermetic-
// driver-with-io` at `wat/test.wat` (plus
// `:wat::kernel::run-sandboxed-fork-direct` at `wat/kernel/sandbox.wat`
// and `:wat::kernel::fork-program-with-inputs` at
// `wat/kernel/hermetic.wat`). A trivial user-code startup exercising the
// full freeze pipeline runs the new walker over those substrate bodies;
// IF the substrate-namespace exemption is broken, freeze fails with the
// new walker variant on those substrate bodies.
//
// These positive tests therefore prove the exemption holds by asserting
// that startup with trivial user-namespace source succeeds — the freeze
// implicitly runs the walker over the stdlib's substrate-namespace
// `*_join-result` calls and they must pass.

#[test]
fn stone_b_substrate_namespace_thread_join_result_is_allowed() {
    // Freeze exercises the walker over `:wat::test::run-thread-driver`
    // (wat/test.wat) and other substrate fns that call
    // `Thread/join-result`. If the substrate exemption fails, freeze
    // fails. Trivial user source + clean startup = exemption proven.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

#[test]
fn stone_b_substrate_namespace_process_join_result_is_allowed() {
    // Mirror for Process — the substrate's `wat/kernel/sandbox.wat` and
    // `wat/kernel/hermetic.wat` call `Process/join-result` directly. The
    // freeze pipeline walks them; the new walker must not fire on those
    // substrate-namespace bodies.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}
