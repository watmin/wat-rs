//! Integration tests for arc 169 / arc 257.2 — struct-destructure via
//! `{:keys [field1 field2 ...]}` (keys-destructure) in let bindings.
//!
//! Arc 257.2 replaced the old non-EDN `WatAST::StructPattern` form
//! (`{field1 field2 ...}` bare symbols) with a proper EDN map in
//! binder position: `{:keys [field1 field2 ...]}`. The semantics are
//! identical: each name in the `:keys` vector is BOTH the field-name
//! (resolved against the struct type the rhs evaluates to) AND the
//! local binding-name in the let scope.
//!
//! The 12-word user-authored rule: *bind the field's value to the
//! field's name in this scope*.
//!
//! ## Test cases
//!
//!  1. single_field — `[{:keys [outcome]} p]`
//!  2. multi_field — `[{:keys [outcome grace-residue]} p]`
//!  3. mixed_with_regular_bindings — `[whole p {:keys [outcome residue]} p]`
//!  4. nested_let — outer destructure + inner uses bindings
//!  5. field_order_can_differ_from_declaration — `[{:keys [grace-residue outcome]} p]`
//!  6. unknown_field_name_is_clean_malformed_form
//!  7. non_struct_subject_is_clean_type_mismatch
//!  8. empty_brace_form_is_clean_malformed_form
//!  9. non_symbol_inside_brace_form_is_clean_malformed_form
//! 10. multi_form_body_with_destructure
//! 11. hyphenated_field_names_work

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROLOGUE: &str = r#"
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
"#;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Asserts startup succeeds and `:user::compute` returns the given value.
fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

/// Asserts startup fails and returns `format!("{}\n---\n{:?}", e, e)`.
fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Test 1 — single_field ──────────────────────────────────────────────

/// `[{:keys [outcome]} p]` binds `outcome :String`; body returns it.
#[test]
fn single_field() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 7.5)
                       {{:keys [outcome]}} p]
                      outcome))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::String(s) => assert_eq!(s.as_str(), "Grace"),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Test 2 — multi_field ───────────────────────────────────────────────

/// `[{:keys [outcome grace-residue]} p]` binds both fields.
#[test]
fn multi_field() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::f64
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 7.5)
                       {{:keys [outcome grace-residue]}} p]
                      grace-residue))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::f64(x) => assert_eq!(x, 7.5),
        other => panic!("expected f64; got {:?}", other),
    }
}

// ─── Test 3 — mixed_with_regular_bindings ───────────────────────────────

/// `[whole p {:keys [outcome grace-residue]} whole]` — regular Symbol binder
/// alongside a keys-destructure binder, both in one let.
#[test]
fn mixed_with_regular_bindings() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::f64
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 3.5)
                       whole p
                       {{:keys [outcome grace-residue]}} whole]
                      grace-residue))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::f64(x) => assert_eq!(x, 3.5),
        other => panic!("expected f64; got {:?}", other),
    }
}

// ─── Test 4 — nested_let ────────────────────────────────────────────────

/// Outer destructure feeds an inner let that uses one of the bound
/// names. Confirms the bindings reach inner scopes intact.
#[test]
fn nested_let() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::f64
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 4.0)
                       {{:keys [outcome grace-residue]}} p]
                      (:wat::core::let
                        [doubled (:wat::core::f64::* grace-residue 2.0)]
                        doubled)))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::f64(x) => assert_eq!(x, 8.0),
        other => panic!("expected f64; got {:?}", other),
    }
}

// ─── Test 5 — field_order_can_differ_from_declaration ───────────────────

/// `[{:keys [grace-residue outcome]} p]` — declaration order is `(outcome,
/// grace-residue)`; the `:keys` vector lists fields in reverse and still
/// binds correctly. Field-name lookup, not positional.
#[test]
fn field_order_can_differ_from_declaration() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 5.5)
                       {{:keys [grace-residue outcome]}} p]
                      outcome))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::String(s) => assert_eq!(s.as_str(), "Grace"),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Test 6 — unknown_field_name_is_clean_malformed_form ────────────────

/// `[{:keys [nonexistent]} p]` — the `:keys` vector names a field that the
/// struct doesn't declare. Substrate-as-teacher: error should name
/// the offending field AND list the struct's actual fields so the
/// user can fix without going back to the declaration.
#[test]
fn unknown_field_name_is_clean_malformed_form() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 5.5)
                       {{:keys [nonexistent]}} p]
                      nonexistent))
        "#,
        prologue = PROLOGUE
    );
    let err = startup_err(&src);
    assert!(
        err.contains("nonexistent"),
        "diagnostic must name the offending field; got: {}",
        err
    );
    assert!(
        err.contains("outcome") && err.contains("grace-residue"),
        "diagnostic must list struct's declared fields; got: {}",
        err
    );
}

// ─── Test 7 — non_struct_subject_is_clean_type_mismatch ─────────────────

/// `[{:keys [outcome]} 42]` — rhs is an i64, not a struct. Type-check time
/// surfaces a clean TypeMismatch.
#[test]
fn non_struct_subject_is_clean_type_mismatch() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [{{:keys [outcome]}} 42]
                      outcome))
        "#,
        prologue = PROLOGUE
    );
    let err = startup_err(&src);
    assert!(
        err.contains("TypeMismatch") || err.contains("type mismatch") || err.contains("malformed"),
        "diagnostic must surface a type or malformed error; got: {}",
        err
    );
    assert!(
        err.contains("struct") || err.contains("Struct") || err.contains("i64"),
        "diagnostic must mention struct expectation or got type; got: {}",
        err
    );
}

// ─── Test 8 — empty_brace_form_is_clean_malformed_form ──────────────────

/// `[{} p]` — empty map in binder position; classified as non-destructure,
/// surfaces as a malformed binder error.
#[test]
fn empty_brace_form_is_clean_malformed_form() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 5.5)
                       {{}} p]
                      "ok"))
        "#,
        prologue = PROLOGUE
    );
    let err = startup_err(&src);
    assert!(
        err.contains("empty")
            || err.contains("at least one")
            || err.contains("MalformedStructPattern")
            || err.contains("malformed")
            || err.contains("binder"),
        "diagnostic must explain the empty brace-form rejection; got: {}",
        err
    );
}

// ─── Test 9 — non_symbol_inside_brace_form_is_clean_malformed_form ──────

/// `[{42} p]` — odd-arity map body (1 item); parse-time MalformedBraceLiteral.
#[test]
fn non_symbol_inside_brace_form_is_clean_malformed_form() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 5.5)
                       {{42}} p]
                      "ok"))
        "#,
        prologue = PROLOGUE
    );
    let err = startup_err(&src);
    assert!(
        err.contains("bare symbol")
            || err.contains("bare symbols")
            || err.contains("integer literal")
            || err.contains("MalformedStructPattern")
            || err.contains("malformed")
            || err.contains("key")
            || err.contains("pair"),
        "diagnostic must reject the odd-arity brace-form; got: {}",
        err
    );
}

// ─── Test 10 — multi_form_body_with_destructure ─────────────────────────

/// Destructure binding + multi-form body — the implicit-do body
/// shape (arc 168) keeps working with arc 257.2 keys-destructure.
#[test]
fn multi_form_body_with_destructure() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::f64
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 1.0)
                       {{:keys [outcome grace-residue]}} p]
                      (:wat::core::f64::+ grace-residue 99.0)
                      (:wat::core::f64::+ grace-residue 50.0)
                      (:wat::core::f64::+ grace-residue 41.0)))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::f64(x) => assert_eq!(x, 42.0, "expected last form 1.0+41.0=42.0"),
        other => panic!("expected f64; got {:?}", other),
    }
}

// ─── Test 11 — hyphenated_field_names_work ──────────────────────────────

/// `grace-residue` is a hyphenated identifier — confirms it binds as
/// a legal local just like the declared field name.
#[test]
fn hyphenated_field_names_work() {
    let src = format!(
        r#"
        {prologue}
        (:wat::core::defn :user::compute [] -> :wat::core::f64
          (:wat::core::let
                      [p (:test::PaperResolved/new "Grace" 9.25)
                       {{:keys [grace-residue]}} p]
                      grace-residue))
        "#,
        prologue = PROLOGUE
    );
    let v = run(&src);
    match v {
        Value::f64(x) => assert_eq!(x, 9.25),
        other => panic!("expected f64; got {:?}", other),
    }
}
