//! Arc 296 S3 — `:wat::kernel::raise!` re-gate probe.
//!
//! Proves BOTH sides of the wall:
//!
//! **Wall holds (bad path):** `(:wat::kernel::raise! 42)` is a COMPILE error.
//! At HEAD (after re-gate): the type checker rejects it because `:wat::core::i64`
//! does not satisfy the `:wat::core::Error` surface. `startup_from_file` returns
//! `Err(...)`.
//!
//! **Right path runs (good path):** `(:wat::core::Fault/of "boom")` type-checks,
//! satisfies `:wat::core::Error`, and when raised in a spawned child the crash is
//! caught over the wire correctly. Startup succeeds and main passes all assertions.
//!
//! IPC de-prime (arc 278): the sandboxed-raise leg was migrated off the non-prime
//! `:wat::test::run-thread` (spawn-thread + Thread/join-result → RunResult) onto the
//! PRIMED peer wire — `spawn-program' :process` + `recv'`. "The raise is caught" now
//! means the child crash surfaces as `recv'` → `Lost[LociDiedError::Panic]` whose
//! message is the raised Fault's message ("boom"). The `:user::main` assertions
//! (below, run via `call_beside_value`) fire a panic if that mapping doesn't hold.

use wat::freeze::{call_beside_value, startup_from_file};

/// Wall proof: (raise! 42) is rejected at compile time.
/// The type checker requires :wat::core::Error; i64 does not satisfy it.
#[test]
fn raise_bare_integer_is_compile_error() {
    let result = startup_from_file(
        "tests/diagnostics/probe_arc296_raise_gate.wat.bad",
    );
    assert!(
        result.is_err(),
        "raise! 42 should be rejected at compile time (wall holds); \
         startup unexpectedly succeeded"
    );
    let msg = format!("{}", result.unwrap_err());
    // The error must be the exact type-mismatch diagnostic from the checker (EDN face,
    // Stone B). 296 recapture: staleness — same message/span/callee/param/expected/got as
    // the pre-stone-B prose face, additive :message/:causes/:location.
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc296_raise_gate__raise_bare_integer_is_compile_error.edn",
        "compile error must be exact type-mismatch diagnostic"
    );
}

/// Right path: Fault/of type-checks, satisfies :wat::core::Error, raise is caught.
#[test]
fn fault_of_type_checks_and_raise_is_caught() {
    // call_beside_value loads probe_arc296_raise_gate.wat (same stem as this .rs) and runs
    // :user::main — asserts inside the .wat fire panic if wrong.
    let _result = call_beside_value(file!(), ":user::main")
        .unwrap_or_else(|e| panic!("(:user::main) raised a runtime error: {e:?}"));
}
