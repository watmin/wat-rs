//! Integration coverage for `:wat::core::forms` (the variadic-quote
//! substrate primitive) and the stdlib-level `:wat::test::program`
//! defmacro that expands to it.
//!
//! `forms` is the variadic sibling of `quote`. `(:wat::core::forms
//! f1 f2 ... fn)` evaluates to a `:wat::core::Vector<wat::WatAST>` where each
//! element is the corresponding unevaluated form captured as data.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn unwrap_bool(v: Value) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── :wat::core::forms — basic behavior ─────────────────────────────────

#[test]
fn forms_captures_each_arg_as_wat_ast() {
    assert!(unwrap_bool(run_expr(":t::test1-forms-3")), "expected forms to capture 3 args");
}

#[test]
fn forms_empty_produces_empty_vec() {
    assert!(
        unwrap_bool(run_expr(":t::test2-forms-empty")),
        "expected forms() to produce empty vec"
    );
}

#[test]
fn forms_args_are_not_evaluated() {
    assert!(
        unwrap_bool(run_expr(":t::test3-forms-unevaluated")),
        "expected forms to capture 1 unevaluated form"
    );
}

// ─── End-to-end: program body → run-hermetic → evaluation ──────────────

#[test]
fn forms_composes_with_run_sandboxed_ast() {
    // Arc 278 IPC de-prime — the value now crosses the peer wire as a recv' Message
    // carrying the DECODED String (not the EDN-quoted stdout line the old RunResult/stdout
    // captured), so it is "hello-from-inside" without the outer quotes.
    assert_eq!(
        unwrap_string(run_expr(":t::test4-run-sandboxed")),
        "hello-from-inside"
    );
}

// ─── :wat::test::program defmacro expands to :wat::core::forms ─────────

#[test]
fn test_program_macro_expands_correctly() {
    assert!(
        unwrap_bool(run_expr(":t::test5-program-macro")),
        "expected :wat::test::program to capture 3 forms"
    );
}

// ─── :wat::test::run-ast end-to-end via :wat::test::program ────────────

#[test]
fn test_run_ast_via_test_program_roundtrips_hello() {
    // Arc 278 IPC de-prime — the value crosses the peer wire as a recv' Message carrying
    // the DECODED String ("hi", no EDN quotes), not the EDN-quoted stdout line.
    assert_eq!(unwrap_string(run_expr(":t::test6-run-ast-hello")), "hi");
}
