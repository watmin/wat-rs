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

// ─── User-declared struct: construction + accessors ──────────────────

#[test]
fn user_struct_constructor_and_accessor_round_trip() {
    // Declare a Candle-like struct with two fields; construct via
    // /new; read back via /open and /close accessors.
    let src = r#"

        (:wat::core::struct :my::market::Bar
          (open  :wat::core::f64)
          (close :wat::core::f64))

        (:wat::core::define (:user::main -> :wat::core::f64)
          (:wat::core::let*
            (((b :my::market::Bar) (:my::market::Bar/new 1.0 2.0))
             ((o :wat::core::f64)             (:my::market::Bar/open b))
             ((c :wat::core::f64)             (:my::market::Bar/close b)))
            (:wat::core::f64::-,2 c o)))
    "#;
    match run(src) {
        Value::f64(x) if (x - 1.0).abs() < 1e-12 => {}
        other => panic!("expected f64 1.0; got {:?}", other),
    }
}

#[test]
fn user_method_can_use_auto_accessors_in_body() {
    // The FOUNDATION framing: user-defined methods on a struct type
    // use the auto-generated accessors. Here the method
    // :my::market::spread/of computes high - low from a Bar.
    let src = r#"

        (:wat::core::struct :my::market::Bar
          (high :wat::core::f64)
          (low  :wat::core::f64))

        (:wat::core::define (:my::market::spread-of (b :my::market::Bar) -> :wat::core::f64)
          (:wat::core::f64::-,2 (:my::market::Bar/high b) (:my::market::Bar/low b)))

        (:wat::core::define (:user::main -> :wat::core::f64)
          (:wat::core::let*
            (((b :my::market::Bar) (:my::market::Bar/new 10.0 3.0)))
            (:my::market::spread-of b)))
    "#;
    match run(src) {
        Value::f64(x) if (x - 7.0).abs() < 1e-12 => {}
        other => panic!("expected f64 7.0; got {:?}", other),
    }
}

#[test]
fn struct_can_hold_heterogeneous_fields() {
    let src = r#"

        (:wat::core::struct :my::market::Tick
          (symbol :wat::core::String)
          (price  :wat::core::f64)
          (volume :wat::core::i64))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((t :my::market::Tick)
              (:my::market::Tick/new "BTC" 50000.0 1000))
             ((v :wat::core::i64) (:my::market::Tick/volume t)))
            v))
    "#;
    match run(src) {
        Value::i64(1000) => {}
        other => panic!("expected i64 1000; got {:?}", other),
    }
}

#[test]
fn structs_are_values_that_survive_rebinding() {
    // A struct value binds to a name and remains readable after
    // passing through let bindings and function calls.
    let src = r#"

        (:wat::core::struct :my::Point
          (x :wat::core::i64)
          (y :wat::core::i64))

        (:wat::core::define (:my::y-of (p :my::Point) -> :wat::core::i64)
          (:my::Point/y p))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((p :my::Point) (:my::Point/new 3 7))
             ((q :my::Point) p))
            (:my::y-of q)))
    "#;
    match run(src) {
        Value::i64(7) => {}
        other => panic!("expected i64 7; got {:?}", other),
    }
}

// ─── Check-time refusals ─────────────────────────────────────────────

#[test]
fn constructor_arity_mismatch_rejected_at_check() {
    // Bar/new expects 2 args (open, close); we pass 1.
    let src = r#"

        (:wat::core::struct :my::market::Bar
          (open  :wat::core::f64)
          (close :wat::core::f64))

        (:wat::core::define (:user::main -> :my::market::Bar)
          (:my::market::Bar/new 1.0))
    "#;
    let errs = check_errors(src);
    let saw_arity = errs.iter().any(|e| matches!(
        e,
        CheckError::ArityMismatch { callee, expected: 2, got: 1, .. }
            if callee == ":my::market::Bar/new"
    ));
    assert!(saw_arity, "expected ArityMismatch on Bar/new; got {:?}", errs);
}

#[test]
fn constructor_field_type_mismatch_rejected_at_check() {
    // Bar/new expects f64 for `open`; we pass a :String.
    let src = r#"

        (:wat::core::struct :my::market::Bar
          (open  :wat::core::f64)
          (close :wat::core::f64))

        (:wat::core::define (:user::main -> :my::market::Bar)
          (:my::market::Bar/new "not-a-float" 2.0))
    "#;
    let errs = check_errors(src);
    let saw_type = errs.iter().any(|e| matches!(
        e,
        CheckError::TypeMismatch { callee, .. }
            if callee == ":my::market::Bar/new"
    ));
    assert!(saw_type, "expected TypeMismatch on Bar/new's open param; got {:?}", errs);
}

#[test]
fn accessor_returns_correct_field_type() {
    // :Bar/volume is declared :i64 in the struct; using it where
    // :f64 is expected is a type error. Proves the accessor's
    // return type flows from the field declaration.
    let src = r#"

        (:wat::core::struct :my::market::Bar
          (open  :wat::core::f64)
          (volume :i64))

        (:wat::core::define (:user::main -> :wat::core::f64)
          (:wat::core::let*
            (((b :my::market::Bar) (:my::market::Bar/new 1.0 100)))
            (:my::market::Bar/volume b)))
    "#;
    let errs = check_errors(src);
    let saw_ret = errs.iter().any(|e| matches!(
        e,
        CheckError::ReturnTypeMismatch { .. }
    ));
    assert!(saw_ret, "expected ReturnTypeMismatch (body :i64 vs declared :f64); got {:?}", errs);
}

// ─── Built-in struct: :wat::holon::CapacityExceeded ────────────────

#[test]
fn builtin_capacity_exceeded_struct_is_usable() {
    // wat-rs seeds :wat::holon::CapacityExceeded as a built-in
    // struct; its /new and /cost / /budget accessors must be
    // available at startup without any user declaration.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((e :wat::holon::CapacityExceeded)
              (:wat::holon::CapacityExceeded/new 200 100))
             ((cost   :wat::core::i64) (:wat::holon::CapacityExceeded/cost   e))
             ((budget :wat::core::i64) (:wat::holon::CapacityExceeded/budget e)))
            (:wat::core::i64::-,2 cost budget)))
    "#;
    match run(src) {
        Value::i64(100) => {}
        other => panic!("expected i64 100; got {:?}", other),
    }
}

#[test]
fn builtin_capacity_exceeded_cannot_be_redeclared() {
    // User source cannot claim `:wat::holon::CapacityExceeded`
    // because the reserved-prefix gate on `TypeEnv::register` blocks
    // user struct registrations under `:wat::*`. This test shows the
    // duplicate surfaces as a startup error (not a silent override).
    let src = r#"

        (:wat::core::struct :wat::holon::CapacityExceeded
          (boom :bool))

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match startup(src) {
        Err(_) => {}
        Ok(_) => panic!("expected startup to reject redeclaration of builtin"),
    }
}
