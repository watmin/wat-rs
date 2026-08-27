//! Strike 3 (examinare disconfirming probe) — fix-source's position-aware LOCAL rules.
//!
//! RED at HEAD: fix.wat only does {strip-if, head-rule} — no arrows, no post-arrow/structural types.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_local_rules`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed String.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
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
fn contract_01_arrow_in_binder() {
    assert_eq!(
        eval_string(":user::c01").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-01-arrow-in-binder.wat")
    );
}

#[test]
fn contract_02_post_arrow_scalar_type() {
    assert_eq!(
        eval_string(":user::c02").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-02-post-arrow-scalar.wat")
    );
}

#[test]
fn contract_03_structural_parametric_type() {
    // Arc 109 ③ (a PRIOR stone) walled the TYPE PARSER, so C03 used to prove `Vector<i64>`
    // got refused by `keyword/to-type-form` with "angle-bracket parametric types are
    // illegal". Arc 109 wave 2 (THIS stone) walls the LEXER, one door earlier:
    // `:user::topform`'s `read-string` on `"[x <- :wat::core::Vector<wat::core::i64>]"`
    // now fails at the READER, before `fix-source`/`keyword/to-type-form` ever run.
    // Arc 109 — this reports CLEANLY now. `ReadOutcome::Malformed`'s cause is declared
    // `:wat::core::Error`, and the decode ladder's FOREIGN arm used to return a
    // `Value::ForeignRecord` that satisfied that surface nowhere, so every consumer
    // calling `(:wat::core::Error/message __cause)` — 75 sites across 57 files — died
    // with `UnknownFunction` instead of reporting the failure. The decoded diagnostic
    // now rides as a CAUSE under a real `:wat::core::Fault`, through the one
    // `fault_with_cause` door its sibling `check_failed_cause` already used. So this
    // asserts the REFUSAL — the thing a refusal control is for — not a crash.
    let msg = eval_string(":user::c03").expect("the refusal must be REPORTABLE, not a crash");
    assert!( // rune:lint(loose-assert) — targeted substring: the read-string crash's mechanism, not the whole located error's structure
        msg.contains("annihilate the angle bracket"),
        "expected the arc 109 angle wall's refusal, reported cleanly through the \
         ReadOutcome::Malformed cause; got: {msg}"
    );
}

#[test]
fn contract_04_head_still_inverts() {
    assert_eq!(
        eval_string(":user::c04").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-04-head-inverts.wat")
    );
}

#[test]
fn contract_05_full_fn_literal() {
    assert_eq!(
        eval_string(":user::c05").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-05-full-fn-literal.wat"),
        "head inverts, binder + return arrows -> :-, both types -> wat.type/, in one pass"
    );
}

#[test]
fn contract_06_less_than_operator_is_not_a_type() {
    assert_eq!(
        eval_string(":user::c06a").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-06a-less-than.wat")
    );
    assert_eq!(
        eval_string(":user::c06b").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat")
    );
}

#[test]
fn contract_07_greater_than_operator() {
    assert_eq!(
        eval_string(":user::c07").expect("eval_string"),
        include_str!("probe_arc251_fix_source_local_rules__contract-07-greater-than.wat")
    );
}
