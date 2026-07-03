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

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

/// Asserts startup succeeds and `:user::compute` returns the given value.
fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

/// Asserts startup fails and returns `format!("{}\n---\n{:?}", e, e)`.
fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Test 1 — single_field ──────────────────────────────────────────────

/// `[{:keys [outcome]} p]` binds `outcome :String`; body returns it.
#[test]
fn single_field() {
    let v = run("tests/types/struct_destructure_single_field.wat");
    match v {
        Value::String(s) => assert_eq!(s.as_str(), "Grace"),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Test 2 — multi_field ───────────────────────────────────────────────

/// `[{:keys [outcome grace-residue]} p]` binds both fields.
#[test]
fn multi_field() {
    let v = run("tests/types/struct_destructure_multi_field.wat");
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
    let v = run("tests/types/struct_destructure_mixed_bindings.wat");
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
    let v = run("tests/types/struct_destructure_nested_let.wat");
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
    let v = run("tests/types/struct_destructure_field_order.wat");
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
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn unknown_field_name_is_clean_malformed_form() {
    let err = startup_err("tests/types/struct_destructure_unknown_field_bad.wat");
    assert_eq!(err, "check:\n1 type-check error(s):\n  - tests/types/struct_destructure_unknown_field_bad.wat:8:6: malformed :wat::core::let form: struct-destructure: field \"nonexistent\" is not declared on struct :test::PaperResolved (declared fields: outcome, grace-residue)\n\n---\nCheck(CheckErrors([CheckError { span: Span { file: \"tests/types/struct_destructure_unknown_field_bad.wat\", line: 8, col: 6, end_line: 8, end_col: 27 }, kind: MalformedForm { head: \":wat::core::let\", reason: \"struct-destructure: field \\\"nonexistent\\\" is not declared on struct :test::PaperResolved (declared fields: outcome, grace-residue)\", remedies: [] } }]))");
}

// ─── Test 7 — non_struct_subject_is_clean_type_mismatch ─────────────────

/// `[{:keys [outcome]} 42]` — rhs is an i64, not a struct. Type-check time
/// surfaces a clean TypeMismatch.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn non_struct_subject_is_clean_type_mismatch() {
    let err = startup_err("tests/types/struct_destructure_non_struct_subject_bad.wat");
    assert_eq!(err, "check:\n1 type-check error(s):\n  - tests/types/struct_destructure_non_struct_subject_bad.wat:7:24: :wat::core::let: parameter struct-destructure (outcome) expects a struct type; got :wat::core::i64\n\n---\nCheck(CheckErrors([CheckError { span: Span { file: \"tests/types/struct_destructure_non_struct_subject_bad.wat\", line: 7, col: 24, end_line: 7, end_col: 26 }, kind: TypeMismatch { callee: \":wat::core::let\", param: \"struct-destructure (outcome)\", expected: \"a struct type\", got: \":wat::core::i64\" } }]))");
}

// ─── Test 8 — empty_brace_form_is_clean_malformed_form ──────────────────

/// `[{} p]` — empty map in binder position; classified as non-destructure,
/// surfaces as a malformed binder error.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn empty_brace_form_is_clean_malformed_form() {
    let err = startup_err("tests/types/struct_destructure_empty_brace_bad.wat");
    assert_eq!(err, "check:\n1 type-check error(s):\n  - tests/types/struct_destructure_empty_brace_bad.wat:8:6: malformed :wat::core::let form: let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position\n\n---\nCheck(CheckErrors([CheckError { span: Span { file: \"tests/types/struct_destructure_empty_brace_bad.wat\", line: 8, col: 6, end_line: 8, end_col: 8 }, kind: MalformedForm { head: \":wat::core::let\", reason: \"let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position\", remedies: [] } }]))");
}

// ─── Test 9 — non_symbol_inside_brace_form_is_clean_malformed_form ──────

/// `[{42} p]` — odd-arity map body (1 item); parse-time MalformedBraceLiteral.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn non_symbol_inside_brace_form_is_clean_malformed_form() {
    let err = startup_err("tests/types/struct_destructure_non_symbol_bad.wat");
    assert_eq!(err, "parse: tests/types/struct_destructure_non_symbol_bad.wat:8:6: malformed brace-literal: map-literal body must alternate key + value pairs; got 1 forms\n---\nParse(ParseError { span: Span { file: \"tests/types/struct_destructure_non_symbol_bad.wat\", line: 8, col: 6, end_line: 8, end_col: 10 }, kind: MalformedBraceLiteral { reason: \"map-literal body must alternate key + value pairs; got 1 forms\" } })");
}

// ─── Test 10 — multi_form_body_with_destructure ─────────────────────────

/// Destructure binding + multi-form body — the implicit-do body
/// shape (arc 168) keeps working with arc 257.2 keys-destructure.
#[test]
fn multi_form_body_with_destructure() {
    let v = run("tests/types/struct_destructure_multi_form_body.wat");
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
    let v = run("tests/types/struct_destructure_hyphenated_field.wat");
    match v {
        Value::f64(x) => assert_eq!(x, 9.25),
        other => panic!("expected f64; got {:?}", other),
    }
}
