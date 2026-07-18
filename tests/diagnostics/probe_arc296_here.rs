//! Arc 296 — `(:wat::kernel::here)` nullary intrinsic probe.
//!
//! Verifies that `(:wat::kernel::here)` returns a `:wat::kernel::Location`
//! record whose `line` field is greater than zero — i.e. the form's own
//! source coordinate, not a synthetic zero.
//!
//! RED at HEAD: `:wat::kernel::here` is unknown to the type checker
//! → startup returns `Err(CheckError { … unresolved verb … })`.
//!
//! GREEN after: type-checker accepts `[] -> :wat::kernel::Location`,
//! startup succeeds, and the assert in `main` (line > 0) passes at runtime.

use wat::freeze::call_beside;

#[test]
fn kernel_here_returns_location_with_positive_line() {
    // RED AT HEAD: startup fails — :wat::kernel::here is unknown.
    // GREEN after: startup succeeds and main's assert-line-gt-0 passes.
    // Run main — the body asserts Location/line > 0 via (:wat::test::assert-true ...).
    // A failing assert fires assertion-failed! (panic_any) → propagates as a test
    // failure. A passing assert returns :wat::core::nil.
    let _result = call_beside(file!(), ":user::main")
        .unwrap_or_else(|e| panic!("(:user::main) raised a runtime error: {e:?}"));
}
