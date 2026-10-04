//! End-to-end tests for the eval-family Result-wrapping
//! (INSCRIPTION 2026-04-20; excursus 003 strike E).
//!
//! Every eval-* form now returns
//! `:wat::core::Result<wat::holon::HolonAST, :wat::kernel::Failure>`. Dynamic
//! evaluation failures — verification mismatch, parse error,
//! mutation-form refusal, unknown function at the call site, type
//! mismatch inside the eval'd code — surface as Err values carrying the
//! REAL error the eval raised: its own declared `:wat::runtime::<Kind>`
//! record (not a hand-maintained `kind` string) plus the frames it
//! produced. The `:wat::core::Result/try` form propagates the Err through
//! a Result-returning helper; `match` at the caller handles both arms.
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

/// Pull the error's declared record-class FQDN and its `message` out of a
/// `Value::Result(Err(Record(Failure)))` — excursus 003 strike E: the eval
/// family's Err is the real `:wat::kernel::Failure` (`{error frames
/// frames-elided}`, wat/kernel/diagnostics.wat), not the retired
/// `:wat::core::EvalError{kind message}`. `Failure.error` is whichever
/// `:wat::runtime::<Kind>` record `RuntimeError::to_record()` produced; every
/// such record's own floor is `(message, location, ...)` in that order
/// (`src/value/runtime_records.rs`), so `error`'s own `fields[0]` is always
/// its message string. The class is read the same way a wat program would —
/// `(:wat::core::type (:wat::kernel::Failure/error e))` — just off the Rust
/// value directly rather than through the accessor.
fn err_class_and_message(v: &Value) -> (String, String) {
    match v {
        Value::Result(r) => match &**r {
            Err(Value::Aggregate(failure)) => {
                assert_eq!(failure.class.as_ref(), "wat::kernel::Failure");
                match &failure.fields[0] {
                    Value::Aggregate(error) => {
                        let class = error.class.to_string();
                        let message = match &error.fields[0] {
                            Value::String(s) => (**s).clone(),
                            other => panic!("error.message not String; got {:?}", other),
                        };
                        (class, message)
                    }
                    other => panic!("Failure.error not Aggregate; got {:?}", other),
                }
            }
            Err(other) => panic!("expected Err(Record(Failure)); got Err({:?})", other),
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
            Ok(Value::wat__holon__HolonAST(_)) => {}
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
    let (class, _) = err_class_and_message(&result);
    assert_eq!(class, "wat::runtime::EvalForbidsMutationForm");
}

#[test]
fn eval_edn_bang_parse_failure_surfaces_as_err() {
    let world = startup_beside(file!()).expect("startup");
    let result = run(&world, ":t::test3");
    let (class, _) = err_class_and_message(&result);
    assert_eq!(class, "wat::runtime::MalformedForm");
}

#[test]
fn eval_digest_string_bang_hash_mismatch_surfaces_as_err() {
    let world = startup_beside(file!()).expect("startup");
    let result = run(&world, ":t::test4");
    let (class, _) = err_class_and_message(&result);
    assert_eq!(class, "wat::runtime::EvalVerificationFailed");
}

#[test]
fn eval_edn_bang_wrong_arity_surfaces_as_err() {
    // Structural arity mismatch fires before the Failure wrap; this
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
            assert_eq!(&*s, "wat::runtime::EvalForbidsMutationForm");
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
            let class = match &t[0] {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            };
            let message = match &t[1] {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            };
            assert_eq!(class, "wat::runtime::EvalForbidsMutationForm");
            assert_eq!(
                message,
                "constrained eval refuses mutation form :wat::core::defstruct; eval evaluates \
                 against the frozen symbol table and cannot register / replace / load definitions",
                "message must name the refused head"
            );
        }
        other => panic!("expected tuple; got {:?}", other),
    }
}
