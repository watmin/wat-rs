//! End-to-end tests for `:wat::core::Result/try` — the error-propagation form.
//!
//! Covered:
//! - Happy path: `try` on `Ok(v)` evaluates to `v`.
//! - Propagation: `try` on `Err(e)` short-circuits the enclosing
//!   function, packaging `e` as that function's own `Err(e)`.
//! - Multi-hop propagation across function boundaries.
//! - Check-time refusals: bad arity, non-Result argument, `try` in a
//!   non-Result-returning enclosing scope, mismatched `Err` types.
//! - Integration with `let`, `match` arms, and fns.
//!
//! Runtime design matches `crate::result::eval_try` +
//! `apply_function`'s `TryPropagate` catch; type-check design matches
//! `crate::check::infer_try`. See `src/runtime.rs` and
//! `src/check.rs` for the implementations.

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn check_errors_from_file(rel_path: &str) -> Vec<CheckError> {
    match startup_from_file(rel_path) {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors for {}; got {:?}", rel_path, other),
        Ok(_) => panic!("expected Check errors for {}; startup succeeded", rel_path),
    }
}

// ─── Happy path / propagation ─────────────────────────────────────────

#[test]
fn try_on_ok_extracts_inner_value() {
    match run_expr(":t::test1-try-ok") {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => {}
            other => panic!("expected Ok(42); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_on_err_propagates_through_function() {
    match run_expr(":t::test2-try-err-prop") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "boom" => {}
            other => panic!("expected Err(\"boom\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_propagates_across_helper_function() {
    match run_expr(":t::test3-try-helper") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "from-helper" => {}
            other => panic!("expected Err(\"from-helper\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_chains_two_bindings_in_let() {
    match run_expr(":t::test4-try-let-chain") {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => {}
            other => panic!("expected Ok(42); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_short_circuits_let_on_first_err() {
    match run_expr(":t::test5-try-let-short-circuit") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "early" => {}
            other => panic!("expected Err(\"early\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_match_arm_propagates() {
    match run_expr(":t::test6-try-match-arm") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "inner-boom" => {}
            other => panic!("expected Err(\"inner-boom\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

// ─── Check-time refusals ──────────────────────────────────────────────

#[test]
fn try_with_zero_args_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_arity_zero.wat.bad");
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::ArityMismatch { callee, expected: 1, got: 0, .. }, .. }
            if callee == ":wat::core::Result/try"
    ));
    assert!(saw_arity, "expected ArityMismatch on :wat::core::Result/try; got {:?}", errs);
}

#[test]
fn try_with_two_args_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_arity_two.wat.bad");
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::ArityMismatch { callee, expected: 1, got: 2, .. }, .. }
            if callee == ":wat::core::Result/try"
    ));
    assert!(saw_arity, "expected ArityMismatch on :wat::core::Result/try; got {:?}", errs);
}

#[test]
fn try_on_non_result_arg_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_non_result_arg.wat.bad");
    let saw_type_mismatch = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::TypeMismatch { callee, .. }, .. } if callee == ":wat::core::Result/try"
    ));
    assert!(saw_type_mismatch, "expected TypeMismatch on :wat::core::Result/try; got {:?}", errs);
}

#[test]
fn try_inside_non_result_function_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_non_result_enclosing.wat.bad");
    let saw_malformed = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::MalformedForm { head, .. }, .. } if head == ":wat::core::Result/try"
    ));
    assert!(saw_malformed, "expected MalformedForm on :wat::core::Result/try; got {:?}", errs);
}

#[test]
fn try_mismatched_err_types_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_err_type_mismatch.wat.bad");
    let saw_type_mismatch = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::TypeMismatch { callee, .. }, .. } if callee == ":wat::core::Result/try"
    ));
    assert!(saw_type_mismatch, "expected TypeMismatch on :wat::core::Result/try; got {:?}", errs);
}

// ─── Fn scope ─────────────────────────────────────────────────────────

#[test]
fn try_inside_result_returning_fn_propagates_to_fn() {
    match run_expr(":t::test7-try-in-fn-scope") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "fn-err" => {}
            other => panic!("expected Err(\"fn-err\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_non_result_fn_rejected_at_check() {
    let errs = check_errors_from_file("tests/wat_lang/wat_core_try_fn_non_result.wat.bad");
    let saw_malformed = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::MalformedForm { head, .. }, .. } if head == ":wat::core::Result/try"
    ));
    assert!(saw_malformed, "expected MalformedForm on :wat::core::Result/try; got {:?}", errs);
}
