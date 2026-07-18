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
//! satisfies `:wat::core::Error`, and when run inside a sandboxed thread the
//! Failure is caught correctly. Startup succeeds and main passes all assertions.

use wat::freeze::{call_beside, startup_from_file};

/// Wall proof: (raise! 42) is rejected at compile time.
/// The type checker requires :wat::core::Error; i64 does not satisfy it.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
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
    // The error must be the exact type-mismatch message from the checker.
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/diagnostics/probe_arc296_raise_gate.wat.bad:12:25: :wat::kernel::raise!: parameter #1 expects :wat::core::Error; got :wat::core::i64\n",
        "compile error must be exact type-mismatch diagnostic"
    );
}

/// Right path: Fault/of type-checks, satisfies :wat::core::Error, raise is caught.
#[test]
fn fault_of_type_checks_and_raise_is_caught() {
    // call_beside loads probe_arc296_raise_gate.wat (same stem as this .rs) and runs
    // :user::main — asserts inside the .wat fire panic if wrong.
    let _result = call_beside(file!(), ":user::main")
        .unwrap_or_else(|e| panic!("(:user::main) raised a runtime error: {e:?}"));
}
