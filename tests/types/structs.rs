//! End-to-end tests for struct declarations, auto-generated
//! `<struct>/new` constructors, and auto-generated `<struct>/<field>`
//! accessors.
//!
//! Design reference: the struct-runtime slice's commit message; the
//! user-facing contract is "know the positions, use let to bind them"
//! at both construction and reading. Each struct declaration produces:
//!
//! - `<struct>/new` — positional constructor, one arg per declared
//!   field, types checked against the field declarations.
//! - `<struct>/<field>` — one accessor per field, type
//!   `:fn(<struct>) -> <field-type>`.
//!
//! The auto-methods live in the symbol table like ordinary `define`
//! entries; authors invoke them by full keyword path. Destructuring
//! is not part of this slice — accessors + let bindings do the work.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

fn check_errors(path: &str) -> Vec<CheckError> {
    match startup_from_file(path) {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {:?}", other),
        Ok(_) => panic!("expected Check errors; startup succeeded"),
    }
}

// ─── User-declared struct: construction + accessors ──────────────────

#[test]
fn user_struct_constructor_and_accessor_round_trip() {
    match run("tests/types/structs_ctor_accessor_roundtrip.wat") {
        Value::f64(x) if (x - 1.0).abs() < 1e-12 => {}
        other => panic!("expected f64 1.0; got {:?}", other),
    }
}

#[test]
fn user_method_can_use_auto_accessors_in_body() {
    match run("tests/types/structs_user_method_auto_accessors.wat") {
        Value::f64(x) if (x - 7.0).abs() < 1e-12 => {}
        other => panic!("expected f64 7.0; got {:?}", other),
    }
}

#[test]
fn struct_can_hold_heterogeneous_fields() {
    match run("tests/types/structs_heterogeneous_fields.wat") {
        Value::i64(1000) => {}
        other => panic!("expected i64 1000; got {:?}", other),
    }
}

#[test]
fn structs_are_values_that_survive_rebinding() {
    match run("tests/types/structs_survive_rebinding.wat") {
        Value::i64(7) => {}
        other => panic!("expected i64 7; got {:?}", other),
    }
}

// ─── Check-time refusals ─────────────────────────────────────────────

#[test]
fn constructor_arity_mismatch_rejected_at_check() {
    let errs = check_errors("tests/types/structs_ctor_arity_mismatch.wat.bad");
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::ArityMismatch { callee, expected: 2, got: 1, .. }, .. }
            if callee == ":my::market::Bar"
    ));
    assert!(saw_arity, "expected ArityMismatch on Bar/new; got {:?}", errs);
}

#[test]
fn constructor_field_type_mismatch_rejected_at_check() {
    let errs = check_errors("tests/types/structs_ctor_type_mismatch.wat.bad");
    let saw_type = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::TypeMismatch { callee, .. }, .. }
            if callee == ":my::market::Bar"
    ));
    assert!(saw_type, "expected TypeMismatch on Bar/new's open param; got {:?}", errs);
}

#[test]
fn accessor_returns_correct_field_type() {
    let errs = check_errors("tests/types/structs_accessor_return_type.wat.bad");
    let saw_ret = errs.iter().any(|e| matches!(
        e,
        CheckError { kind: CheckErrorKind::ReturnTypeMismatch { .. }, .. }
    ));
    assert!(saw_ret, "expected ReturnTypeMismatch (body :wat::core::i64 vs declared :wat::core::f64); got {:?}", errs);
}

// ─── Built-in struct: :wat::holon::CapacityExceeded ────────────────

#[test]
fn builtin_capacity_exceeded_struct_is_usable() {
    match run("tests/types/structs_builtin_capacity_exceeded.wat") {
        Value::i64(100) => {}
        other => panic!("expected i64 100; got {:?}", other),
    }
}

#[test]
fn builtin_capacity_exceeded_cannot_be_redeclared() {
    match startup_from_file("tests/types/structs_builtin_redeclare.wat.bad") {
        Err(_) => {}
        Ok(_) => panic!("expected startup to reject redeclaration of builtin"),
    }
}
