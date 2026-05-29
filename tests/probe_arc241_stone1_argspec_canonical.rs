//! FM-2-bis probe for Stone 241.1 — canonical `parse_argspec_triples` at `src/argspec/`.
//!
//! Per `docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.fix.md` D5:
//! 9 contracts covering the canonical `[name <- :T name <- :T ... [& rest <- :T]]` triple form.
//! Ret-clause concerns removed per Stone 241.1.fix Layer 2 scope correction.
//!
//! Pre-stone: ALL 10 fail to compile because `wat::argspec` doesn't exist.
//! The failure is module-resolution at the `use wat::argspec::...` line — isolated to
//! the missing module. Every other piece (WatAST destructure, parse_one!, TypeExpr
//! inspection, Result matching) compiles and works at HEAD.
//!
//! Post-stone 241.1: 10/10 PASS.
//! Post-stone 241.1.fix Layer 1: 13/13 PASS (contracts 11/12/13 added).
//! Post-stone 241.1.fix Layer 2: 9/9 PASS (ret-related contracts removed; scope corrected).
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
fn parse_vector_items(src: &str) -> (Vec<WatAST>, wat::span::Span) {
    let ast = wat::parse_one!(src).expect("parse_one! should succeed for argspec source");
    match ast {
        WatAST::Vector(items, span) => (items, span),
        other => panic!("expected Vector form, got {:?}", other),
    }
}

/// Invoke `parse_argspec_triples` with the standard test head + the parsed-form span.
fn parse_triples(
    src: &str,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError> {
    let (items, span) = parse_vector_items(src);
    parse_argspec_triples(
        &items,
        ":wat::test::fn",
        &span,
        ParseOptions { allow_rest_binder },
    )
}

// ─── Contracts 1–3: success cases ─────────────────────────────────────────────

#[test]
fn contract_01_empty_argspec() {
    // [] → fixed_params empty, rest_param None.
    let result = parse_triples("[]", false);
    let spec = result.expect("empty argspec parses cleanly");
    assert!(spec.fixed_params.is_empty(), "fixed_params should be empty");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
}

#[test]
fn contract_02_single_fixed_param() {
    // [x <- :wat::core::i64] → one fixed param, no rest.
    let result = parse_triples("[x <- :wat::core::i64]", false);
    let spec = result.expect("single fixed param parses cleanly");
    assert_eq!(spec.fixed_params.len(), 1, "exactly one fixed param");
    assert_eq!(spec.fixed_params[0].0, "x", "name slot is 'x'");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
}

#[test]
fn contract_03_multiple_fixed_params() {
    // [x <- :wat::core::i64 y <- :wat::core::i64] — two fixed params, no ret.
    let result = parse_triples("[x <- :wat::core::i64 y <- :wat::core::i64]", false);
    let spec = result.expect("multi-param argspec parses cleanly");
    assert_eq!(spec.fixed_params.len(), 2, "two fixed params");
    assert_eq!(spec.fixed_params[0].0, "x", "first name is 'x'");
    assert_eq!(spec.fixed_params[1].0, "y", "second name is 'y'");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
}

// ─── Contracts 4–9: error cases ───────────────────────────────────────────────

#[test]
fn contract_04_non_symbol_at_name_slot() {
    // [:keyword-not-symbol <- :wat::core::i64] — slot 0 is Keyword, must be Symbol.
    let result = parse_triples("[:keyword-not-symbol <- :wat::core::i64]", false);
    let err = result.expect_err("non-Symbol at name slot must error");
    assert!(
        matches!(err, ArgSpecError::NameNotSymbol { .. }),
        "expected NameNotSymbol, got {:?}",
        err
    );
}

#[test]
fn contract_05_missing_arrow_token() {
    // [x = :wat::core::i64] — slot 1 is Symbol("=") not Symbol("<-").
    let result = parse_triples("[x = :wat::core::i64]", false);
    let err = result.expect_err("missing <- arrow must error");
    assert!(
        matches!(err, ArgSpecError::MissingArrow { .. }),
        "expected MissingArrow, got {:?}",
        err
    );
}

#[test]
fn contract_06_non_keyword_at_type_slot() {
    // [x <- "string-not-keyword"] — slot 2 is StringLit, must be Keyword.
    let result = parse_triples(r#"[x <- "string-not-keyword"]"#, false);
    let err = result.expect_err("non-Keyword at type slot must error");
    assert!(
        matches!(err, ArgSpecError::TypeNotKeyword { .. }),
        "expected TypeNotKeyword, got {:?}",
        err
    );
}

#[test]
fn contract_07_rest_binder_rejected() {
    // [x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]
    // with allow_rest_binder=false → must error explicitly (rest support is Stone 241.4).
    let result = parse_triples(
        "[x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]",
        false,
    );
    let err = result.expect_err("& rest-binder must error when disallowed");
    assert!(
        matches!(err, ArgSpecError::RestBinderNotSupported { .. }),
        "expected RestBinderNotSupported, got {:?}",
        err
    );
}

#[test]
fn contract_08_malformed_type_keyword() {
    // [x <- :Any] — parse_type_expr_with_span rejects :Any via reject_any() → AnyBanned →
    // MalformedTypeKeyword. :Any is a structurally valid keyword but the type system bans it.
    let result = parse_triples("[x <- :Any]", false);
    let err = result.expect_err("banned :Any type keyword must error");
    assert!(
        matches!(err, ArgSpecError::MalformedTypeKeyword { .. }),
        "expected MalformedTypeKeyword, got {:?}",
        err
    );
}

#[test]
fn contract_09_incomplete_triple() {
    // [x <-] — fewer than 3 items, runs out before triple completes.
    let result = parse_triples("[x <-]", false);
    let err = result.expect_err("incomplete triple must error");
    assert!(
        matches!(err, ArgSpecError::IncompleteTriple { .. }),
        "expected IncompleteTriple, got {:?}",
        err
    );
}
