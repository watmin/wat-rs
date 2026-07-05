//! Arc 278 — user `extend-type` impl bodies MUST be type-checked against the surface they satisfy.
//!
//! extend-type is THE satisfier construct (arc 232/293) — every `Store`/`ReadStore` impl, every
//! surface a user implements. Core is truthful by construction; this is the ONE place a user can
//! still ship a wrong type green. `check_function_body` (check.rs:826) sweeps `sym.functions`, but
//! USER extend-type impls only enter `sym.functions` at freeze step 9 (`register_runtime_defs_form`,
//! runtime.rs:1936) — AFTER `check_program` (freeze.rs:816). BAKED impls enter at step 7.6
//! (`register_stdlib_runtime_defs`), BEFORE the sweep, and ARE checked. The fix registers user impls
//! (with surface-inherited sigs) before the sweep, closing the lie.
//!
//! RED at HEAD: the wrong-typed impl freezes clean → the `Ok` arm panics.
//! GREEN after the fix: freeze returns a `ReturnTypeMismatch` check error.
//!
//! Run: `cargo test --release -p wat --test types -- user_extend_type`

use wat::freeze::startup_from_file;

/// A user extend-type impl whose body returns `String` against a surface method declaring
/// `-> :i64` MUST be rejected at type-check. If freeze succeeds, the impl body was never checked.
// ⛔ IGNORE-LEDGER(278): RED at HEAD (the flaw). UNLOCK — un-ignore when the "user extend-type
//    impl bodies are checked" strike lands; must be GREEN before the strike closes.
#[ignore = "RED gate for arc-278 user-extend-type-checked strike; un-ignore + green when the fix lands"]
#[test]
fn user_extend_type_wrong_return_rejected() {
    match startup_from_file("tests/types/probe_arc278_extend_user_body_checked_bad.wat") {
        Err(_) => {}
        Ok(_) => panic!(
            "expected a ReturnTypeMismatch: user extend-type impl `emit` returns a String against \
             surface `-> :i64`, but freeze SUCCEEDED — user extend-type impl bodies are NOT checked"
        ),
    }
}
