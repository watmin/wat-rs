//! Integration coverage for arc 143 slice 3 — two HolonAST manipulation
//! substrate primitives:
//!   `:wat::runtime::rename-callable-name`
//!   `:wat::runtime::extract-arg-names`
//!
//! Both operate on signature heads (Bundle ASTs) produced by slice 1's
//! `signature-of-defn`. Tests cover:
//!
//! rename-callable-name:
//!   1. Happy path — rename :wat::core::foldl head to :wat::list::reduce;
//!      verify first symbol becomes ":wat::list::reduce<T,Acc>".
//!   2. No type-params — rename a bare user-defined function; verify
//!      new symbol has no "<...>" suffix.
//!   3. Error — input is a non-Bundle HolonAST leaf (a keyword Symbol).
//!   4. Error — `from` name doesn't match the head's base name.
//!
//! extract-arg-names:
//!   5. Happy path — extract from `signature-of-defn :wat::core::foldl`;
//!      returns [:_a0, :_a1, :_a2].
//!   6. Zero-args — extract from a thunk (zero-param function);
//!      returns empty Vec.
//!   7. Stops at "->" arrow — only arg names before the arrow are collected.
//!   8. Error — input is not a Bundle.
//!
//! Composing with slice 1:
//!   9. rename composed with signature-of-defn — full pipeline:
//!      (rename (signature-of-defn :fn) :fn :alias) returns Some with the
//!      renamed name in the head.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn startup() -> wat::freeze::FrozenWorld {
    startup_beside(file!()).expect("startup")
}

fn run_expr(expr: &str) -> Value {
    let world = startup();
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn run_expr_expect_err(expr: &str) -> bool {
    let world = startup();
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new()).is_err()
}

fn unwrap_string(v: Value, ctx: &str) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("{}: expected String; got {:?}", ctx, other),
    }
}

// ─── :wat::runtime::rename-callable-name ────────────────────────────────────

#[test]
fn rename_callable_name_happy_path_foldl_to_reduce() {
    let line = unwrap_string(run_expr("(:t::test1-rename-foldl-to-reduce)"), "test1");
    assert!(
        line.contains("reduce"),
        "expected 'reduce' in renamed head, got: {}",
        line
    );
    assert!(
        line.contains("T") && line.contains("Acc"),
        "expected type params T and Acc preserved, got: {}",
        line
    );
}

#[test]
fn rename_callable_name_no_type_params() {
    let line = unwrap_string(run_expr("(:t::test2-rename-no-type-params)"), "test2");
    assert!(
        line.contains("my-triple"),
        "expected 'my-triple' in renamed head, got: {}",
        line
    );
    assert!(
        !line.starts_with(":t::my-double"),
        "expected 'my-double' to be gone from first symbol, got: {}",
        line
    );
    assert!(
        line.contains(":t::my-triple"),
        "expected ':t::my-triple' literal in rendered output, got: {}",
        line
    );
}

#[test]
fn rename_callable_name_error_from_mismatch() {
    assert!(
        run_expr_expect_err("(:t::test3-rename-mismatch)"),
        "expected runtime error for from-name mismatch, got Ok"
    );
}

// ─── :wat::runtime::extract-arg-names ───────────────────────────────────────

#[test]
fn extract_arg_names_foldl_returns_three_names() {
    let line = unwrap_string(run_expr("(:t::test4-extract-foldl-names)"), "test4");
    assert!(line.contains("_a0"), "expected ':_a0' in extracted names, got: {}", line);
    assert!(line.contains("_a1"), "expected ':_a1' in extracted names, got: {}", line);
    assert!(line.contains("_a2"), "expected ':_a2' in extracted names, got: {}", line);
    // edn::write renders HolonAST::Symbol as `Symbol "_aN"`.
    // eval_in_frozen gives us the string directly (no kernel::println encoding),
    // so inner quotes are not escaped — match Symbol "_a (unescaped).
    let count = line.matches("Symbol \"_a").count();
    assert_eq!(
        count, 3,
        "expected exactly 3 arg names (_a0/_a1/_a2), counted {} in: {}",
        count, line
    );
}

#[test]
fn extract_arg_names_zero_args_returns_empty() {
    let s = unwrap_string(run_expr("(:t::test5-extract-zero-args)"), "test5");
    assert_eq!(s.trim(), "0", "expected edn::write of length 0 to be '0', got: {}", s);
}

#[test]
fn extract_arg_names_stops_before_return_type() {
    let line = unwrap_string(run_expr("(:t::test6-extract-stops-before-return)"), "test6");
    assert!(line.contains("2 "), "expected length 2 in output, got: {}", line);
    // eval_in_frozen gives the string directly; inner quotes from edn::write are not
    // kernel::println-escaped, so Symbol "x" appears literally (unescaped).
    assert!(
        line.contains("Symbol \"x\"") && line.contains("Symbol \"y\""),
        "expected arg-name Symbols x and y in output, got: {}",
        line
    );
}

#[test]
fn extract_arg_names_error_non_bundle() {
    assert!(
        run_expr_expect_err("(:t::test7-extract-non-bundle-err)"),
        "expected runtime error for non-Bundle input to extract-arg-names, got Ok"
    );
}

// ─── Composition test: rename-callable-name ∘ signature-of-defn ─────────────

#[test]
fn rename_then_extract_preserves_arg_names() {
    let line = unwrap_string(run_expr("(:t::test8-rename-then-extract)"), "test8");
    assert!(line.contains("2 "), "expected length 2 preserved after rename, got: {}", line);
    // eval_in_frozen gives the string directly; inner quotes from edn::write are
    // not kernel::println-escaped, so Symbol "x" appears literally (unescaped).
    assert!(
        line.contains("Symbol \"x\"") && line.contains("Symbol \"y\""),
        "expected arg-name Symbols x and y preserved after rename, got: {}",
        line
    );
}
