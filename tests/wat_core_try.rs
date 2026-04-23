//! End-to-end tests for `:wat::core::try` — the error-propagation form.
//!
//! Covered:
//! - Happy path: `try` on `Ok(v)` evaluates to `v`.
//! - Propagation: `try` on `Err(e)` short-circuits the enclosing
//!   function/lambda, packaging `e` as that function's own `Err(e)`.
//! - Multi-hop propagation across function boundaries.
//! - Check-time refusals: bad arity, non-Result argument, `try` in a
//!   non-Result-returning enclosing scope, mismatched `Err` types.
//! - Integration with `let*`, `match` arms, and lambdas.
//!
//! Runtime design matches `crate::runtime::eval_try` +
//! `apply_function`'s `TryPropagate` catch; type-check design matches
//! `crate::check::infer_try`. See `src/runtime.rs` and
//! `src/check.rs` for the implementations.

use std::sync::Arc;
use wat::check::CheckError;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

fn run(src: &str) -> Value {
    let world = startup(src).expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

fn check_errors(src: &str) -> Vec<CheckError> {
    match startup(src) {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {:?}", other),
        Ok(_) => panic!("expected Check errors; startup succeeded"),
    }
}

// ─── Happy path / propagation ─────────────────────────────────────────

#[test]
fn try_on_ok_extracts_inner_value() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try (Ok 42))))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => {}
            other => panic!("expected Ok(42); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_on_err_propagates_through_function() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try (Err "boom"))))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "boom" => {}
            other => panic!("expected Err(\"boom\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_propagates_across_helper_function() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:app::unwrap-or-propagate
                             (r :Result<i64,String>)
                             -> :Result<i64,String>)
          (Ok (:wat::core::try r)))

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:app::unwrap-or-propagate (Err "from-helper")))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "from-helper" => {}
            other => panic!("expected Err(\"from-helper\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_chains_two_bindings_in_let_star() {
    // try inside let* binding positions — the classic use. Each try
    // unwraps its Result into the bound name; the final Ok wraps the
    // sum to satisfy the function's declared return type.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:wat::core::let*
            (((a :i64) (:wat::core::try (Ok 10)))
             ((b :i64) (:wat::core::try (Ok 32))))
            (Ok (:wat::core::i64::+ a b))))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => {}
            other => panic!("expected Ok(42); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_short_circuits_let_star_on_first_err() {
    // Err on the first binding propagates; subsequent bindings never
    // evaluate. The body never runs either.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:wat::core::let*
            (((a :i64) (:wat::core::try (Err "early")))
             ((b :i64) (:wat::core::try (Ok 99))))
            (Ok (:wat::core::i64::+ a b))))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "early" => {}
            other => panic!("expected Err(\"early\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_match_arm_propagates() {
    // try inside the body of a match arm still propagates to the
    // enclosing function — not just to the match.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:app::describe
                             (o :Option<Result<i64,String>>)
                             -> :Result<i64,String>)
          (:wat::core::match o -> :Result<i64,String>
            ((Some r) (Ok (:wat::core::try r)))
            (:None (Err "missing"))))

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:app::describe (Some (Err "inner-boom"))))
    "#;
    match run(src) {
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
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try)))
    "#;
    let errs = check_errors(src);
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError::ArityMismatch { callee, expected: 1, got: 0 }
            if callee == ":wat::core::try"
    ));
    assert!(saw_arity, "expected ArityMismatch on :wat::core::try; got {:?}", errs);
}

#[test]
fn try_with_two_args_rejected_at_check() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try (Ok 1) (Ok 2))))
    "#;
    let errs = check_errors(src);
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError::ArityMismatch { callee, expected: 1, got: 2 }
            if callee == ":wat::core::try"
    ));
    assert!(saw_arity, "expected ArityMismatch on :wat::core::try; got {:?}", errs);
}

#[test]
fn try_on_non_result_arg_rejected_at_check() {
    // Passing a bare i64 — not a Result.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try 42)))
    "#;
    let errs = check_errors(src);
    let saw_type_mismatch = errs.iter().any(|e| matches!(
        e,
        CheckError::TypeMismatch { callee, .. } if callee == ":wat::core::try"
    ));
    assert!(saw_type_mismatch, "expected TypeMismatch on :wat::core::try; got {:?}", errs);
}

#[test]
fn try_inside_non_result_function_rejected_at_check() {
    // Enclosing fn returns :i64, not :Result. `try` has no place to
    // propagate to; MalformedForm fires.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::try (Ok 42)))
    "#;
    let errs = check_errors(src);
    let saw_malformed = errs.iter().any(|e| matches!(
        e,
        CheckError::MalformedForm { head, .. } if head == ":wat::core::try"
    ));
    assert!(saw_malformed, "expected MalformedForm on :wat::core::try; got {:?}", errs);
}

#[test]
fn try_mismatched_err_types_rejected_at_check() {
    // Enclosing fn's Err is :String; try's arg has Err :i64 — strict
    // equality refuses (no auto-conversion, per 2026-04-19 stance).
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:app::produce-i64-err -> :Result<i64,i64>)
          (Err 99))

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (Ok (:wat::core::try (:app::produce-i64-err))))
    "#;
    let errs = check_errors(src);
    let saw_type_mismatch = errs.iter().any(|e| matches!(
        e,
        CheckError::TypeMismatch { callee, .. } if callee == ":wat::core::try"
    ));
    assert!(saw_type_mismatch, "expected TypeMismatch on :wat::core::try; got {:?}", errs);
}

// ─── Lambda scope ─────────────────────────────────────────────────────

#[test]
fn try_inside_result_returning_lambda_propagates_to_lambda() {
    // The lambda itself is Result-returning, so try short-circuits the
    // lambda. The outer function (also Result-returning) receives the
    // lambda's Err as a Value::Result and wraps it back as-is.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:wat::core::let
            (((f :fn(Result<i64,String>)->Result<i64,String>)
              (:wat::core::lambda
                ((r :Result<i64,String>) -> :Result<i64,String>)
                (Ok (:wat::core::try r)))))
            (f (Err "lambda-err"))))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) if s.as_ref() == "lambda-err" => {}
            other => panic!("expected Err(\"lambda-err\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_non_result_lambda_rejected_at_check() {
    // Lambda's return type is :i64, not Result — the innermost
    // enclosing scope for `try` is the lambda, not the outer fn.
    // MalformedForm fires.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<i64,String>)
          (:wat::core::let
            (((f :fn(Result<i64,String>)->i64)
              (:wat::core::lambda
                ((r :Result<i64,String>) -> :i64)
                (:wat::core::try r))))
            (Ok (f (Ok 1)))))
    "#;
    let errs = check_errors(src);
    let saw_malformed = errs.iter().any(|e| matches!(
        e,
        CheckError::MalformedForm { head, .. } if head == ":wat::core::try"
    ));
    assert!(saw_malformed, "expected MalformedForm on :wat::core::try; got {:?}", errs);
}
