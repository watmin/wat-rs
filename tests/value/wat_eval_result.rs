//! End-to-end tests for the eval-family Result-wrapping
//! (INSCRIPTION 2026-04-20).
//!
//! Every eval-* form now returns
//! `:wat::core::Result<wat::holon::HolonAST, :wat::core::EvalError>`. Dynamic
//! evaluation failures — verification mismatch, parse error,
//! mutation-form refusal, unknown function at the call site, type
//! mismatch inside the eval'd code — surface as Err values with
//! `kind` and `message` fields. The `:wat::core::Result/try` form
//! propagates the Err through a Result-returning helper; `match`
//! at the caller handles both arms.
//!
//! Wat source lives in the co-located fixture: wat_eval_result.wat
//! (slurped via startup_beside(file!())).
//! Negative startup test uses: tests/value/wat_eval_result_wrong_arity.wat

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `:t::…` fixture fn is a zero-arg entry; fetch it from the frozen
// world and `apply_function` it — no inline wat driver.
fn run(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Value {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should run")
}

/// Pull the `kind` string from a `Value::Result(Err(Struct(EvalError)))`.
/// Panics with diagnostic if the value isn't a Result-Err-Struct of the
/// expected shape.
fn err_kind(v: &Value) -> String {
    match v {
        Value::Result(r) => match &**r {
            Err(Value::Aggregate(sv)) => {
                assert_eq!(sv.class.as_ref(), "wat::core::EvalError");
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
    let world = startup_beside(file!()).expect("startup");
    match run(&world, ":t::test1") {
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
    let world = startup_beside(file!()).expect("startup");
    // Stone 241.16 — :wat::core::define HARD CUT total; is_mutation_head no longer
    // recognizes it. Fixture migrated to :wat::core::defstruct (still a mutation head).
    let result = run(&world, ":t::test2");
    assert_eq!(err_kind(&result), "mutation-form-refused");
}

#[test]
fn eval_edn_bang_parse_failure_surfaces_as_err() {
    let world = startup_beside(file!()).expect("startup");
    let result = run(&world, ":t::test3");
    assert_eq!(err_kind(&result), "malformed-form");
}

#[test]
fn eval_digest_string_bang_hash_mismatch_surfaces_as_err() {
    let world = startup_beside(file!()).expect("startup");
    let result = run(&world, ":t::test4");
    assert_eq!(err_kind(&result), "verification-failed");
}

#[test]
fn eval_edn_bang_wrong_arity_surfaces_as_err() {
    // Structural arity mismatch fires before the EvalError wrap; this
    // shows up at startup (the type checker catches it as wrong-arity).
    // Negative fixture fails to freeze; startup_from_file returns Err.
    let result = startup_from_file("tests/value/wat_eval_result_wrong_arity.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::ArityMismatch { callee, expected, got }
            if callee == ":wat::eval-edn!"
            && *expected == 1
            && *got == 2
    );
}

// ─── try-based propagation through a Result-returning helper ─────────

#[test]
fn try_propagates_eval_err_through_helper() {
    let world = startup_beside(file!()).expect("startup");
    // Stone 241.16 — :wat::core::define HARD CUT total; migrated to :wat::core::defstruct.
    match run(&world, ":t::test6") {
        Value::String(s) => {
            assert_eq!(&*s, "mutation-form-refused");
        }
        other => panic!("expected String; got {:?}", other),
    }
}

#[test]
fn eval_err_exposes_both_kind_and_message() {
    let world = startup_beside(file!()).expect("startup");
    // Stone 241.16 — :wat::core::define HARD CUT total; migrated to :wat::core::defstruct.
    // is_mutation_head no longer recognizes define; defstruct still is a mutation head.
    match run(&world, ":t::test7") {
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
            assert_eq!(
                message,
                "eval refused mutation form: :wat::core::defstruct",
                "message must name the refused head"
            );
        }
        other => panic!("expected tuple; got {:?}", other),
    }
}
