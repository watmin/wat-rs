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

// rune:lint(no-inlined-wat) — arc 296 Stone L. The expected/got strings in this file are
// golden COMPARISON text for a rendered diagnostic field (a TypeMismatch's `expected`/`got`),
// never a wat world or driver. They parse as forms only because the checker's error renderer
// emits real `(Head :- [args])` syntax; nothing here builds or runs a wat program from them,
// and there is no file a single field of a compound match-guard could move to. Same class and
// same reason as tests/services/probe_arc170_w2a_kwargs_check_mint.rs:35.
use wat::check::error::CheckErrorKind;

use super::stone18a::try_startup;

// arc 296 Stone M: `try_startup` (stone18a.rs) now returns the raw `StartupError` itself
// rather than a flattened `String` — the parallel `try_startup_typed` helper this file used
// to carry (added at Stone L because widening `try_startup` was out of that stone's scope)
// is retired; there is one typed path now, shared with the positive `is_ok()` contracts in
// stone18a.rs.

// ─── E01: fn-form argspec with disallowed rest binder ─────────────────────────

#[test]
fn error_01_fn_argspec_rest_binder_disallowed() {
    let result = try_startup("tests/function/stone18a_e01.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::ArityMismatch { callee, expected, got }
            if callee == "(value head)"
            && *expected == 0
            && *got == 1
    );
}

// ─── E02: fn-form body type mismatch ──────────────────────────────────────────

#[test]
fn error_02_fn_body_return_type_mismatch() {
    let result = try_startup("tests/function/stone18a_e02.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":anonymous"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}

// ─── E03: fn-form missing arrow symbol ───────────────────────────────────────

#[test]
fn error_03_fn_missing_arrow() {
    let result = try_startup("tests/function/stone18a_e03.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "fn signature: expected `->` between args-vector and return type; got keyword"
    );
}

// ─── E04: fn-form non-keyword return type ────────────────────────────────────

#[test]
fn error_04_fn_non_keyword_ret_type() {
    let result = try_startup("tests/function/stone18a_e04.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got nil"
    );
}

// ─── E05: fn-form wrong arrow symbol ─────────────────────────────────────────

#[test]
fn error_05_fn_wrong_arrow_symbol() {
    let result = try_startup("tests/function/stone18a_e05.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "fn signature: expected `->` between args-vector and return type; got symbol"
    );
}

// ─── E06: infer_fn check path — keyword in argspec name slot ─────────────────

#[test]
fn error_06_infer_fn_malformed_argspec_name_slot() {
    let result = try_startup("tests/function/stone18a_e06.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "name must be a plain symbol (not a keyword, literal, or nested form)"
    );
}
