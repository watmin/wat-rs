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

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn run_expr_expect_err(name: &str) -> bool {
    call_beside_value(file!(), name).is_err()
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
    let line = unwrap_string(run_expr(":t::test1-rename-foldl-to-reduce"), "test1");
    // rune:clojure-flip — string-eq bridge (not assert_edn_eq), historically because reflection
    // emitted a `<T,Acc>` multi-param generic head plain EDN could not round-trip. STONE-
    // defservice-emits-the-binder (arc 109) retired that head — the multi-param binder is now
    // `:- [T Acc]` siblings, itself plain EDN-representable — but the bridge is left in place
    // here (only the golden's TEXT changed) since lifting it to `assert_edn_eq!` is a separate,
    // unscoped cleanup, not part of this stone.
    assert_eq!(
        line,
        include_str!("wat_arc143_manipulation__reduce_head.edn").trim_end(),
        "renamed head must be :wat::list::reduce with type params T and Acc preserved"
    );
}

#[test]
fn rename_callable_name_no_type_params() {
    let line = unwrap_string(run_expr(":t::test2-rename-no-type-params"), "test2");
    wat::assert_edn_matches_file!(line, "wat_arc143_manipulation__no_type_params.edn", "renamed head must be :t::my-triple with no type params");
}

#[test]
fn rename_callable_name_error_from_mismatch() {
    assert!(
        run_expr_expect_err(":t::test3-rename-mismatch"),
        "expected runtime error for from-name mismatch, got Ok"
    );
}

// ─── :wat::runtime::extract-arg-names ───────────────────────────────────────

#[test]
fn extract_arg_names_foldl_returns_three_names() {
    // TYPE-reflection HolonAST eviction: extract-arg-names now returns
    // plain keywords, not HolonAST Symbol nodes.
    let line = unwrap_string(run_expr(":t::test4-extract-foldl-names"), "test4");
    wat::assert_edn_matches_file!(line, "wat_arc143_manipulation__extract_arg_names.edn", "extracted foldl arg names must be exactly _a0/_a1/_a2");
}

#[test]
fn extract_arg_names_zero_args_returns_empty() {
    let s = unwrap_string(run_expr(":t::test5-extract-zero-args"), "test5");
    assert_eq!(s.trim(), "0", "expected edn::write of length 0 to be '0', got: {}", s);
}

#[test]
fn extract_arg_names_stops_before_return_type() {
    // TYPE-reflection HolonAST eviction: names render as plain keywords.
    let line = unwrap_string(run_expr(":t::test6-extract-stops-before-return"), "test6");
    assert_eq!(
        line,
        r#"2 [:x :y]"#,
        "extract must stop before return type arrow and yield exactly x and y"
    );
}

#[test]
fn extract_arg_names_error_non_bundle() {
    assert!(
        run_expr_expect_err(":t::test7-extract-non-bundle-err"),
        "expected runtime error for non-Bundle input to extract-arg-names, got Ok"
    );
}

// ─── Composition test: rename-callable-name ∘ signature-of-defn ─────────────

#[test]
fn rename_then_extract_preserves_arg_names() {
    // TYPE-reflection HolonAST eviction: names render as plain keywords.
    let line = unwrap_string(run_expr(":t::test8-rename-then-extract"), "test8");
    assert_eq!(
        line,
        r#"2 [:x :y]"#,
        "arg names x and y must be preserved after rename"
    );
}
