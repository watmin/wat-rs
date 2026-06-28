//! Integration coverage for arc 144 slice 1 — uniform `lookup_form`
//! reflection across the five form-kinds.
//!
//! The arc-143 reflection primitives (`:wat::runtime::lookup-define`,
//! `:wat::runtime::signature-of-defn`, `:wat::runtime::body-of`) now
//! dispatch on a uniform `Binding` enum (5 variants). Slice 1 ships
//! UserFunction, Macro, Primitive, and Type coverage; SpecialForm
//! arrives in slice 2 (registry not yet populated; lookup_form's
//! SpecialForm path returns None today).
//!
//! These tests verify:
//!   1. Macro lookup — defmacro is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::defmacro`; signature-of-defn
//!      returns Some; body-of returns the template.
//!   2. Type lookup — struct decl is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::struct`; signature-of-defn
//!      returns Some + emission carries the type's name; body-of
//!      returns :None (types are body-less in the wat sense).
//!   3. User-function lookup — no regression vs arc 143's existing
//!      coverage; the refactor preserves UserFunction behavior exactly.
//!   4. Substrate-primitive lookup — same regression-guard for
//!      `:wat::core::foldl` post-Binding-refactor.
//!   5. Unknown name — all three primitives return :None for an
//!      unregistered name.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn unwrap_bool(v: Value, ctx: &str) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("{}: expected bool; got {:?}", ctx, other),
    }
}

fn unwrap_string(v: Value, ctx: &str) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("{}: expected String; got {:?}", ctx, other),
    }
}

// ─── Macro lookup (NEW kind for arc 144) ────────────────────────────────────

#[test]
fn lookup_define_macro_returns_some_and_emits_defmacro_head() {
    let line = unwrap_string(run_expr("(:t::test1-lookup-macro-render)"), "test1");
    assert!(
        line.contains("defmacro"),
        "expected 'defmacro' head in rendered macro define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::ident"),
        "expected macro name 'my::ident' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_defn_macro_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test2-sig-macro)"), "test2"),
        "signature-of-defn :my::ident should return Some"
    );
}

#[test]
fn body_of_macro_returns_some_with_template() {
    assert!(
        unwrap_bool(run_expr("(:t::test3-body-macro)"), "test3"),
        "body-of :my::ident should return Some"
    );
}

// ─── Type lookup (NEW kind for arc 144) ─────────────────────────────────────

#[test]
fn lookup_define_struct_returns_some_and_emits_struct_head() {
    let line = unwrap_string(run_expr("(:t::test4-lookup-struct-render)"), "test4");
    assert!(
        line.contains("defstruct"),
        "expected 'defstruct' head in rendered type define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::Bar"),
        "expected type name 'my::Bar' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_defn_struct_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test5-sig-struct)"), "test5"),
        "signature-of-defn :my::Point should return Some"
    );
}

#[test]
fn body_of_struct_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::test6-body-struct-none)"), "test6"),
        "body-of :my::Tick should return None (types have no body)"
    );
}

// ─── Regression guards: UserFunction + Primitive behavior preserved ─────────

#[test]
fn lookup_define_user_function_still_returns_some_post_refactor() {
    assert!(
        unwrap_bool(run_expr("(:t::test7-lookup-user-fn)"), "test7"),
        "lookup-define :t::my-add should return Some"
    );
}

#[test]
fn signature_of_defn_substrate_primitive_still_returns_some_post_refactor() {
    assert!(
        unwrap_bool(run_expr("(:t::test8-sig-foldl)"), "test8"),
        "signature-of-defn :wat::core::foldl should return Some"
    );
}

// ─── Unknown name returns None across all three primitives ──────────────────

#[test]
fn all_three_primitives_return_none_on_unknown_name() {
    assert!(
        unwrap_bool(run_expr("(:t::test9-all-none)"), "test9"),
        "all three primitives should return None for unknown name"
    );
}
