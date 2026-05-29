//! FM-2-bis probe for Stone 241.1 — canonical `parse_argspec_triples` at `src/argspec/`.
//!
//! Per `docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.md` D6:
//! 10 contracts covering the canonical `[name <- :T name <- :T ... [-> :Ret]]` triple form.
//!
//! Pre-stone: ALL 10 fail to compile because `wat::argspec` doesn't exist.
//! The failure is module-resolution at the `use wat::argspec::...` line — isolated to
//! the missing module. Every other piece (WatAST destructure, parse_one!, TypeExpr
//! inspection, Result matching) compiles and works at HEAD.
//!
//! Post-stone: 10/10 PASS.
//!
//! This probe IS the contract sonnet satisfies. It is design substrate the Shadowdancer
//! mirrors, not assertion (per DUNGEON-CRAWL Phase 2 + FM 2-bis discipline).
//!
//! Run: `cargo test --release --test probe_arc241_stone1_argspec_canonical`

use wat::argspec::{parse_argspec_triples, ArgSpec, ArgSpecError, ParseOptions};
use wat::ast::WatAST;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Parse a wat source-form Vector `[...]` and return its inner items + the form span.
/// Used by every contract to construct argspec inputs without manual WatAST building.
fn argspec_inputs(src: &str) -> (Vec<WatAST>, impl std::ops::Deref<Target = wat::span::Span>) {
    let ast = wat::parse_one!(src).expect("parse_one! should succeed for argspec source");
    match ast {
        WatAST::Vector(items, span) => {
            // Heap-pin the span so the &Span reference passed to parse_argspec_triples
            // can outlive the match arm without naming wat::span::Span as a type annotation.
            (items, Box::new(span))
        }
        other => panic!("expected Vector form, got {:?}", other),
    }
}

/// Invoke `parse_argspec_triples` with the standard test head + the parsed-form span.
fn invoke(
    src: &str,
    include_ret_type: bool,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError> {
    let (items, span) = argspec_inputs(src);
    parse_argspec_triples(
        &items,
        ":wat::test::fn",
        &span,
        ParseOptions {
            include_ret_type,
            allow_rest_binder,
        },
    )
}

// ─── Contracts 1–4: success cases ─────────────────────────────────────────────

#[test]
fn contract_01_empty_argspec_no_ret_type_expected() {
    // [] with include_ret_type=false → fixed_params empty, rest_param None, ret_type None.
    let result = invoke("[]", false, false);
    let spec = result.expect("empty argspec parses cleanly");
    assert!(spec.fixed_params.is_empty(), "fixed_params should be empty");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
    assert!(spec.ret_type.is_none(), "ret_type should be None");
}

#[test]
fn contract_02_single_fixed_param_no_ret() {
    // [x <- :wat::core::i64] with include_ret_type=false → one fixed param, no ret.
    let result = invoke("[x <- :wat::core::i64]", false, false);
    let spec = result.expect("single fixed param parses cleanly");
    assert_eq!(spec.fixed_params.len(), 1, "exactly one fixed param");
    assert_eq!(spec.fixed_params[0].0, "x", "name slot is 'x'");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
    assert!(spec.ret_type.is_none(), "ret_type should be None");
}

#[test]
fn contract_03_multiple_fixed_params_with_ret() {
    // [x <- :wat::core::i64 y <- :wat::core::i64 -> :wat::core::i64] include_ret_type=true.
    let result = invoke(
        "[x <- :wat::core::i64 y <- :wat::core::i64 -> :wat::core::i64]",
        true,
        false,
    );
    let spec = result.expect("multi-param signature parses cleanly");
    assert_eq!(spec.fixed_params.len(), 2, "two fixed params");
    assert_eq!(spec.fixed_params[0].0, "x", "first name is 'x'");
    assert_eq!(spec.fixed_params[1].0, "y", "second name is 'y'");
    assert!(spec.ret_type.is_some(), "ret_type populated");
}

#[test]
fn contract_04_ret_only_signature() {
    // [-> :wat::core::i64] with include_ret_type=true → no fixed params, ret populated.
    let result = invoke("[-> :wat::core::i64]", true, false);
    let spec = result.expect("ret-only signature parses cleanly");
    assert!(spec.fixed_params.is_empty(), "no fixed params");
    assert!(spec.rest_param.is_none(), "no rest param");
    assert!(spec.ret_type.is_some(), "ret_type populated");
}

// ─── Contracts 5–10: error cases ──────────────────────────────────────────────

#[test]
fn contract_05_non_symbol_at_name_slot() {
    // [:keyword-not-symbol <- :wat::core::i64] — slot 0 is Keyword, must be Symbol.
    let result = invoke("[:keyword-not-symbol <- :wat::core::i64]", false, false);
    let err = result.expect_err("non-Symbol at name slot must error");
    assert!(
        matches!(err, ArgSpecError::NameNotSymbol { .. }),
        "expected NameNotSymbol, got {:?}",
        err
    );
}

#[test]
fn contract_06_missing_arrow_token() {
    // [x = :wat::core::i64] — slot 1 is Symbol("=") not Symbol("<-").
    let result = invoke("[x = :wat::core::i64]", false, false);
    let err = result.expect_err("missing <- arrow must error");
    assert!(
        matches!(err, ArgSpecError::MissingArrow { .. }),
        "expected MissingArrow, got {:?}",
        err
    );
}

#[test]
fn contract_07_non_keyword_at_type_slot() {
    // [x <- "string-not-keyword"] — slot 2 is StringLit, must be Keyword.
    let result = invoke(r#"[x <- "string-not-keyword"]"#, false, false);
    let err = result.expect_err("non-Keyword at type slot must error");
    assert!(
        matches!(err, ArgSpecError::TypeNotKeyword { .. }),
        "expected TypeNotKeyword, got {:?}",
        err
    );
}

#[test]
fn contract_08_missing_ret_arrow_when_expected() {
    // [x <- :wat::core::i64] with include_ret_type=true → expects "->" after triple but
    // finds end of items → MissingRetArrow.
    let result = invoke("[x <- :wat::core::i64]", true, false);
    let err = result.expect_err("missing -> when ret expected must error");
    assert!(
        matches!(err, ArgSpecError::MissingRetArrow { .. }),
        "expected MissingRetArrow, got {:?}",
        err
    );
}

#[test]
fn contract_09_trailing_items_after_ret() {
    // [x <- :wat::core::i64 -> :wat::core::i64 garbage] — extra item after ret type.
    let result = invoke(
        "[x <- :wat::core::i64 -> :wat::core::i64 garbage]",
        true,
        false,
    );
    let err = result.expect_err("trailing items must error");
    assert!(
        matches!(err, ArgSpecError::TrailingItems { .. }),
        "expected TrailingItems, got {:?}",
        err
    );
}

#[test]
fn contract_10_rest_binder_rejected_when_disallowed() {
    // [x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]
    // with allow_rest_binder=false → must error explicitly (rest support is Stone 241.4).
    let result = invoke(
        "[x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]",
        false,
        false,
    );
    let err = result.expect_err("& rest-binder must error when disallowed");
    assert!(
        matches!(err, ArgSpecError::RestBinderNotSupported { .. }),
        "expected RestBinderNotSupported, got {:?}",
        err
    );
}
