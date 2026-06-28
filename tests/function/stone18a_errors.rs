//! Error-path contracts for `src/function/` — fn-form malformed inputs.
//!
//! Stone 241.18a R0-remediation (C10). Companion to `stone18a.rs` which
//! covers happy-path behavioral preservation.
//!
//! These 6 contracts exercise the check-tier and eval-tier error gates that
//! live in `src/function/{eval,parse,infer}.rs` after the namespaced-home
//! migration. Each contract: load a malformed fn-form fixture; assert
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
//! Wat sources live in tests/function/stone18a_eNN.wat (one per error contract).
//! Run: `cargo test --release --test function stone18a_errors`

use super::stone18a::try_startup;

// ─── E01: fn-form argspec with disallowed rest binder ─────────────────────────

#[test]
fn error_01_fn_argspec_rest_binder_disallowed() {
    let result = try_startup("tests/function/stone18a_e01.wat");
    assert!(
        result.is_err(),
        "fn-form with rest binder must fail; got Ok"
    );
}

// ─── E02: fn-form body type mismatch ──────────────────────────────────────────

#[test]
fn error_02_fn_body_return_type_mismatch() {
    let result = try_startup("tests/function/stone18a_e02.wat");
    assert!(
        result.is_err(),
        "fn-form with body/ret-type mismatch must fail; got Ok"
    );
}

// ─── E03: fn-form missing arrow symbol ───────────────────────────────────────

#[test]
fn error_03_fn_missing_arrow() {
    let result = try_startup("tests/function/stone18a_e03.wat");
    assert!(
        result.is_err(),
        "fn-form missing `->` must fail; got Ok"
    );
}

// ─── E04: fn-form non-keyword return type ────────────────────────────────────

#[test]
fn error_04_fn_non_keyword_ret_type() {
    let result = try_startup("tests/function/stone18a_e04.wat");
    assert!(
        result.is_err(),
        "fn-form with non-keyword return type must fail; got Ok"
    );
}

// ─── E05: fn-form wrong arrow symbol ─────────────────────────────────────────

#[test]
fn error_05_fn_wrong_arrow_symbol() {
    let result = try_startup("tests/function/stone18a_e05.wat");
    assert!(
        result.is_err(),
        "fn-form with `=>` instead of `->` must fail; got Ok"
    );
}

// ─── E06: infer_fn check path — keyword in argspec name slot ─────────────────

#[test]
fn error_06_infer_fn_malformed_argspec_name_slot() {
    let result = try_startup("tests/function/stone18a_e06.wat");
    assert!(
        result.is_err(),
        "fn-form with keyword in name slot must fail; got Ok"
    );
}
