//! FM 2-bis probe — arc 251 Stone 251.5a-i: the homoiconic `read`.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_read_string`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed bool.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_bool(fn_name: &str) -> Result<bool, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "bool",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

fn eval_string(fn_name: &str) -> Result<String, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_read_string_returns_walkable_forms() {
    assert!(
        eval_bool(":user::c01").expect("eval_bool"),
        "read-string must return a forms-List the macro engine can walk (List? recognizes it)"
    );
}

#[test]
fn contract_02_read_string_reads_the_dirty_surface() {
    // Arc 109 wave 2 "annihilate the angle bracket" — THE PERMISSION IS GONE. This
    // contract's whole subject WAS the angle form: that `read-string` could read a
    // "dirty" pre-251.5 `Vector<...>` spelling the strict EDN reader refused. The
    // lexer wall (this stone) refuses `<` in a name universally — `read-string` shares
    // the same lexer as everything else — so there is no longer any surface `read-
    // string` reads that a stricter reader wouldn't also refuse; the whole
    // "dirty surface" `read-string` existed partly to read is gone. Class 3 (b):
    // re-pointed as a refusal control on the mechanism that actually fires now.
    //
    // Arc 109 — this reports CLEANLY now. `ReadOutcome::Malformed`'s cause is declared
    // `:wat::core::Error`, and the decode ladder's FOREIGN arm used to return a
    // `Value::ForeignRecord` that satisfied that surface nowhere, so every consumer
    // calling `(:wat::core::Error/message __cause)` — 75 sites across 57 files — died
    // with `UnknownFunction` instead of reporting the failure. The decoded diagnostic
    // now rides as a CAUSE under a real `:wat::core::Fault`, through the one
    // `fault_with_cause` door its sibling `check_failed_cause` already used. So this
    // asserts the REFUSAL — the thing a refusal control is for — not a crash.
    let msg = eval_string(":user::c02").expect("the refusal must be REPORTABLE, not a crash");
    assert!( // rune:lint(loose-assert) — targeted substring: the read-string crash's mechanism, not the whole located error's structure
        msg.contains("annihilate the angle bracket"),
        "expected the arc 109 angle wall's refusal, reported cleanly through the \
         ReadOutcome::Malformed cause; got: {msg}"
    );
}
