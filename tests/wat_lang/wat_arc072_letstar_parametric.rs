//! Arc 072 regression — parametric type keywords with `<>` lex
//! cleanly, and whitespace inside `:<...>` produces a clean
//! diagnostic at the lexer layer instead of silently truncating
//! into a mysterious downstream type-check error.
//!
//! Pre-arc-072, the lexer tracked `()` depth but ignored `<>` depth.
//! `:Result<(i64,i64), i64>` (space after the comma) tokenized as
//! `:Result<(i64,i64),` (whitespace truncated the keyword) plus a
//! separate `i64>` symbol — the type parser saw a malformed Result
//! with one arg, the rest dropped. Downstream the type checker
//! surfaced as "fresh-var :?N unsolved" at pattern-arm sites.
//! Probe 018 chased that opaque error through several layers
//! before tracing it back to the lexer.
//!
//! The fix: lexer now tracks `<>` depth alongside `()` for type-
//! head brackets (operator `<` / `>` in keyword paths like
//! `:wat::core::<` are disambiguated by the preceding char — only
//! `<` after an alphanumeric counts toward depth). Whitespace
//! inside an unclosed `<` raises `LexError::UnclosedBracketInKeyword`
//! at the lex layer — the user gets a clean error pointing at the
//! exact byte, not a downstream "fresh var unsolved."
//!
//! The substrate's whitespace rule for type keywords stays strict
//! (per the existing convention — `:Result<i64,String>` not
//! `:Result<i64, String>`). The arc fixes the diagnostic, not the
//! rule.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

/// `:Result<i64,String>` (canonical, no whitespace) lexes, parses,
/// type-checks, and runs end-to-end. The chain proof 018's walker
/// rewrite intends to use.
#[test]
fn letstar_result_no_whitespace_simple_payload() {
    match run_expr(":t::test1-result-simple") {
        Value::i64(n) => assert_eq!(n, 43, "expected extracted+1 = 43; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `:Result<(i64,i64),i64>` (canonical) — tuple inside parametric.
/// The exact shape that surfaced this arc from proof 018's walker
/// rewrite. Pre-fix: lexer truncated, downstream "fresh var :?71"
/// at the (second pair) call. Post-fix: lexes cleanly, runs.
#[test]
fn letstar_result_no_whitespace_tuple_payload() {
    match run_expr(":t::test2-result-tuple") {
        Value::i64(n) => assert_eq!(n, 11, "expected second of Tuple(7,11) = 11; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// **RE-POINTED, arc 109 "annihilate the angle bracket".** This test's
/// original subject — whitespace inside an OPEN `<...>` type head raising
/// `UnclosedBracketInKeyword` — no longer exists: `<` opening a type head at
/// all is now refused at the reader, before the lexer ever reaches the
/// interior of the fixture's `HashMap<String >`. The clean-lex-error claim
/// still holds (this remains a lex-layer error, not a downstream "fresh var
/// unsolved"); only the mechanism moved from the whitespace rule to the arc
/// 109 wall. Assert the MECHANISM, not the whole diagnostic (fixture-fragile
/// byte offset) — arc 109's own precedent.
#[test]
fn whitespace_inside_angle_brackets_raises_clean_lex_error() {
    let result = startup_from_file(
        "tests/wat_lang/wat_arc072_letstar_parametric_whitespace.wat.bad",
    );
    let err = result
        .map(|_| ())
        .expect_err("expected the angle-bracket type head on `:HashMap<String, i64>` to be refused");
    let err_msg = format!("{}", err);
    // rune:lint(loose-assert) — targeted PRESENCE over a large structured diagnostic; the
    // assertion names the MECHANISM (the arc 109 wall), the whole claim this re-pointed test
    // makes. An exact-match golden would pin the lex error's byte offset and re-break on every
    // unrelated edit to the fixture — same reasoning as probe_arc232_generic_method_type_
    // application.rs's precedent.
    assert!(
        err_msg.contains("annihilate the angle bracket"),
        "must be refused by the arc 109 angle wall, at the lex layer; got: {err_msg}"
    );
}

/// Operator `<` and `>` in keyword paths must still lex as part of
/// the keyword (they're not bracket openers; they follow `::`). This
/// test confirms the lexer's disambiguation didn't break operators.
#[test]
fn operator_lt_gt_keywords_still_lex() {
    match run_expr(":t::test4-operator-lt-ge") {
        Value::bool(true) => {}
        other => panic!("expected bool true; got {:?}", other),
    }
}
