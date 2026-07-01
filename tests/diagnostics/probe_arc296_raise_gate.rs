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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::Environment;

/// Wall proof: (raise! 42) is rejected at compile time.
/// The type checker requires :wat::core::Error; i64 does not satisfy it.
#[test]
fn raise_bare_integer_is_compile_error() {
    let result = startup_from_file(
        "tests/diagnostics/probe_arc296_raise_gate_bad.wat",
    );
    assert!(
        result.is_err(),
        "raise! 42 should be rejected at compile time (wall holds); \
         startup unexpectedly succeeded"
    );
    let msg = format!("{}", result.unwrap_err());
    // The error must mention the type mismatch — either the Error surface name
    // or "assignable" / "expected" language from the checker.
    assert!(
        msg.contains(":wat::core::Error") || msg.contains("Error") || msg.contains("assignable") || msg.contains("expected"),
        "compile error should mention the type mismatch; got:\n{}",
        msg
    );
}

/// Right path: Fault/of type-checks, satisfies :wat::core::Error, raise is caught.
#[test]
fn fault_of_type_checks_and_raise_is_caught() {
    // startup_beside loads probe_arc296_raise_gate.wat (same stem as this .rs).
    let world = startup_beside(file!()).unwrap_or_else(|e| {
        panic!(
            ":wat::core::Fault/of must type-check and startup must succeed; got: {}",
            e
        )
    });
    // Run main — asserts inside the .wat fire panic if wrong.
    let ast = wat::parse_one!("(:user::main)").expect("parse");
    let _result = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("(:user::main) raised a runtime error: {e:?}"));
}
