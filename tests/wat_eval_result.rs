//! End-to-end tests for the eval-family Result-wrapping
//! (INSCRIPTION 2026-04-20).
//!
//! Every eval-* form now returns
//! `:wat::core::Result<wat::holon::HolonAST, :wat::core::EvalError>`. Dynamic
//! evaluation failures — verification mismatch, parse error,
//! mutation-form refusal, unknown function at the call site, type
//! mismatch inside the eval'd code — surface as Err values with
//! `kind` and `message` fields. The `:wat::core::try` form
//! propagates the Err through a Result-returning helper; `match`
//! at the caller handles both arms.
//!
//! See `docs/arc/2026/04/003-...` — wait, this inscription landed
//! before arc 003 (TCO); see `FOUNDATION-CHANGELOG.md` for the
//! 2026-04-20 entry.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

/// Pull the `kind` string from a `Value::Result(Err(Struct(EvalError)))`.
/// Panics with diagnostic if the value isn't a Result-Err-Struct of the
/// expected shape.
fn err_kind(v: &Value) -> String {
    match v {
        Value::Result(r) => match &**r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::core::EvalError");
                match &sv.fields[0] {
                    Value::String(s) => (**s).clone(),
                    other => panic!("EvalError.kind not String; got {:?}", other),
                }
            }
            Err(other) => panic!("expected Err(Struct(EvalError)); got Err({:?})", other),
            Ok(inner) => panic!("expected Err; got Ok({:?})", inner),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Happy path: eval-ast! returns Ok(holon) ─────────────────────────

#[test]
fn eval_ast_bang_happy_path_returns_ok_holon() {
    // A well-formed AST that evaluates to a holon; the outer result
    // is Ok(Value::holon__HolonAST(_)).
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::core::let*
            (((program :wat::WatAST) (:wat::core::quote (:wat::holon::Atom "hello"))))
            (:wat::eval-ast! program)))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Err variants ─────────────────────────────────────────────────────

#[test]
fn eval_ast_bang_mutation_form_surfaces_as_err() {
    // An AST that contains `(:wat::core::define ...)` — a mutation
    // form constrained eval refuses. Becomes
    // Err(EvalError{kind="mutation-form-refused"}).
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::core::let*
            (((program :wat::WatAST)
              (:wat::core::quote
                (:wat::core::define (:evil (x :wat::core::i64) -> :wat::core::i64) x))))
            (:wat::eval-ast! program)))
    "#;
    let result = run(src);
    assert_eq!(err_kind(&result), "mutation-form-refused");
}

#[test]
fn eval_edn_bang_parse_failure_surfaces_as_err() {
    // Malformed EDN source — the parser rejects it; the failure
    // surfaces as EvalError (kind="malformed-form" today; a future
    // slice may introduce a dedicated "parse-failed" kind if the
    // distinction earns it).
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::eval-edn! "(:wat::core::i64::+ 1"))
    "#;
    let result = run(src);
    assert_eq!(err_kind(&result), "malformed-form");
}

#[test]
fn eval_digest_string_bang_hash_mismatch_surfaces_as_err() {
    // Provide a wrong SHA-256 digest; verification fails with
    // kind="verification-failed". Arc 028 slice 3: inline source
    // variant is `eval-digest-string!` (mirrors `load-string!`).
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::eval-digest-string!
 "(:wat::holon::Atom \"x\")"
            :wat::verify::digest-sha256
            :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"))
    "#;
    let result = run(src);
    assert_eq!(err_kind(&result), "verification-failed");
}

#[test]
fn eval_edn_bang_wrong_arity_surfaces_as_err() {
    // Arc 028 slice 3 retired the :wat::eval::* interface-keyword
    // test — those keywords don't exist anymore. Arity mismatch is
    // the new structural-error surface to guard.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::eval-edn! "foo" "bar-extra"))
    "#;
    // Structural arity mismatch fires before the EvalError wrap; this
    // shows up at startup (the type checker catches it as wrong-arity).
    // The wrapper `run(...)` surfaces it as a startup failure — we
    // expect the test to PANIC from the startup phase, not return a
    // clean EvalError value. Use std::panic::catch_unwind.
    let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(src)));
    assert!(got.is_err(), "expected startup-time arity failure");
}

// ─── try-based propagation through a Result-returning helper ─────────

#[test]
fn try_propagates_eval_err_through_helper() {
    // Helper returns Result; its body uses `try` to propagate eval's
    // Err cleanly. The caller matches at main and accesses the
    // EvalError struct's `kind` field via the auto-generated accessor.
    let src = r#"

        (:wat::core::define (:app::run-dynamic (program :wat::WatAST)
                             -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (Ok (:wat::core::try (:wat::eval-ast! program))))

        (:wat::core::define (:user::main -> :wat::core::String)
          (:wat::core::let*
            (((bad :wat::WatAST)
              (:wat::core::quote
                (:wat::core::define (:injected (x :wat::core::i64) -> :wat::core::i64) x))))
            (:wat::core::match (:app::run-dynamic bad) -> :wat::core::String
              ((Ok _) "should-not-reach")
              ((Err e) (:wat::core::EvalError/kind e)))))
    "#;
    match run(src) {
        Value::String(s) => {
            assert_eq!(&*s, "mutation-form-refused");
        }
        other => panic!("expected String; got {:?}", other),
    }
}

#[test]
fn eval_err_exposes_both_kind_and_message() {
    // Access both accessors; the message should contain the
    // mutation-head name for diagnostic clarity.
    let src = r#"

        (:wat::core::define (:user::main -> :(wat::core::String,wat::core::String))
          (:wat::core::let*
            (((bad :wat::WatAST)
              (:wat::core::quote
                (:wat::core::define (:injected (x :wat::core::i64) -> :wat::core::i64) x)))
             ((r :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
              (:wat::eval-ast! bad)))
            (:wat::core::match r -> :(wat::core::String,wat::core::String)
              ((Ok _)
                (:wat::core::Tuple "unreachable" "unreachable"))
              ((Err e)
                (:wat::core::Tuple
                  (:wat::core::EvalError/kind e)
                  (:wat::core::EvalError/message e))))))
    "#;
    match run(src) {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 2);
            let kind = match &t[0] {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            };
            let message = match &t[1] {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            };
            assert_eq!(kind, "mutation-form-refused");
            assert!(
                message.contains(":wat::core::define"),
                "message should name the refused head; got {:?}",
                message
            );
        }
        other => panic!("expected tuple; got {:?}", other),
    }
}
