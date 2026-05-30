//! Error-path contracts for `src/function/` — fn-form malformed inputs.
//!
//! Stone 241.18a R0-remediation (C10). Companion to `stone18a.rs` which
//! covers happy-path behavioral preservation.
//!
//! These 6 contracts exercise the check-tier and eval-tier error gates that
//! live in `src/function/{eval,parse,infer}.rs` after the namespaced-home
//! migration. Each contract: construct a malformed fn-form; assert
//! `try_startup` returns Err.
//!
//! Contracts:
//!   E01 — fn-form argspec with rest binder (disallowed; fn has no variadic)
//!   E02 — fn-form body type mismatch (declared return vs inferred body type)
//!   E03 — fn-form missing arrow symbol between args-vector and return type
//!   E04 — fn-form non-keyword return type (symbol where keyword expected)
//!   E05 — fn-form wrong arrow symbol (=> instead of ->)
//!   E06 — fn-form argspec name slot is a keyword (not a symbol)
//!
//! Run: `cargo test --release --test function stone18a_errors`

use super::stone18a::try_startup;

// ─── E01: fn-form argspec with disallowed rest binder ─────────────────────────

#[test]
fn error_01_fn_argspec_rest_binder_disallowed() {
    // `(:wat::core::fn [& rest <- :wat::core::i64] -> :wat::core::i64 rest)` —
    // the `[& rest <- :T]` rest-binder is disallowed for fn-forms
    // (`allow_rest_binder: false`); error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::i64
          ((:wat::core::fn [& rest <- :wat::core::i64] -> :wat::core::i64 rest) 42))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form with rest binder must fail; got Ok"
    );
}

// ─── E02: fn-form body type mismatch ──────────────────────────────────────────

#[test]
fn error_02_fn_body_return_type_mismatch() {
    // `(:wat::core::fn [] -> :wat::core::i64 "a string")` — declared return
    // type is :i64 but body infers :String; error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::nil
          ((:wat::core::fn [] -> :wat::core::i64 "a string")))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form with body/ret-type mismatch must fail; got Ok"
    );
}

// ─── E03: fn-form missing arrow symbol ───────────────────────────────────────

#[test]
fn error_03_fn_missing_arrow() {
    // `(:wat::core::fn [] :wat::core::nil nil)` — no `->` symbol between
    // args-vector and return type; error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::nil
          ((:wat::core::fn [] :wat::core::nil nil)))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form missing `->` must fail; got Ok"
    );
}

// ─── E04: fn-form non-keyword return type ────────────────────────────────────

#[test]
fn error_04_fn_non_keyword_ret_type() {
    // `(:wat::core::fn [] -> nil nil)` — symbol `nil` where keyword expected; error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::nil
          ((:wat::core::fn [] -> nil nil)))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form with non-keyword return type must fail; got Ok"
    );
}

// ─── E05: fn-form wrong arrow symbol ─────────────────────────────────────────

#[test]
fn error_05_fn_wrong_arrow_symbol() {
    // `(:wat::core::fn [] => :wat::core::nil nil)` — symbol `=>` where `->` is
    // expected; error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::nil
          ((:wat::core::fn [] => :wat::core::nil nil)))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form with `=>` instead of `->` must fail; got Ok"
    );
}

// ─── E06: infer_fn check path — keyword in argspec name slot ─────────────────

#[test]
fn error_06_infer_fn_malformed_argspec_name_slot() {
    // `(:wat::core::fn [:kw <- :wat::core::i64] -> :wat::core::i64 42)` —
    // the name slot is a keyword instead of a symbol; error surfaced.
    let src = r#"
        (:wat::core::defn :test::bad [] -> :wat::core::nil
          ((:wat::core::fn [:kw <- :wat::core::i64] -> :wat::core::i64 42)))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "fn-form with keyword in name slot must fail; got Ok"
    );
}
