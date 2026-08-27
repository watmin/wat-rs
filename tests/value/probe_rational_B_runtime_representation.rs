//! RED probe — Stone B: rationals representable in the wat RUNTIME (the language layer).
//!
//! Stone A gave `wat-edn` (the EDN data layer) a `Value::Rational`. Stone B gives the RUNTIME its
//! representation: `1/2` in wat SOURCE lexes → evals → renders as a rational; `4/2` reduces to an
//! Integer. REPRESENTATION only — arithmetic is Stone C.
//!
//! Grounded vs clj 1.12.4 this session (AD ORACVLVM NON AD LIBRVM — the running oracle, not the doc):
//!   4/2 6/3 1/1 0/5 -> 2 2 1 0        (java.lang.Long — a LITERAL reducing to a whole number is a Long)
//!   1/2 -6/4 10/4   -> 1/2 -3/2 5/2   (clojure.lang.Ratio — reduced, sign on numerator, den > 0)
//!   1/0             -> Divide by zero (reader error)
//! (Arithmetic collapse — `(+ 1/2 1/2) -> 1N`, a BigInt — is Stone C, deliberately NOT tested here.)
//!
//! RED at HEAD: `wat-reader`'s `lex_numeric_or_symbol` scans `1/2` as one token, fails i64/f64 parse,
//! and returns `InvalidNumber("1/2")` — so `parse_one!("1/2")` is `Err` and every case below panics.

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value, ValueSnapshot};
use wat::{LexError, LexErrorKind, ParseErrorKind};

/// (type_name, rendered) for a source form evaluated through the real loader.
fn eval_render(src: &str) -> (String, String) {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!(src).unwrap_or_else(|e| panic!("{src:?} should parse: {e:?}"));
    let tv = eval_in_frozen(&ast, &world, &env)
        .unwrap_or_else(|e| panic!("{src:?} should eval: {e:?}"));
    (
        tv.value().type_name().to_string(),
        // NOTE (Stone B implementation, not spec intent): the doc comment
        // above says this tuple is "(type_name, rendered)" — the BARE
        // rendered form, not a full diagnostic ValueSnapshot::Display
        // (which unconditionally prefixes "{type_name} `{rendered}` (from
        // ...)" for every value — confirmed against a plain `42` literal
        // too, so this isn't specific to Rational). `format!("{}", ...)`
        // on the whole snapshot could never equal a bare "1/2"/"-3/2" for
        // ANY literal; `.rendered` is the field that matches the doc
        // comment's stated intent and every other stone's render checks.
        ValueSnapshot::of_tracked(&tv).rendered,
    )
}

// ─── a genuine ratio (den >= 2) reads as a runtime Rational ─────────────────────

#[test]
fn rational_literal_reads_as_runtime_rational() {
    let (ty, rendered) = eval_render("1/2");
    assert_eq!(ty, "wat::core::rational", "1/2 must eval to a rational (clj: clojure.lang.Ratio)");
    assert_eq!(rendered, "1/2", "1/2 renders canonically");
}

#[test]
fn rational_literal_reduces_and_signs_like_clj() {
    // reduced to lowest terms, sign on numerator, denominator > 0
    assert_eq!(eval_render("-6/4"), ("wat::core::rational".into(), "-3/2".into()));
    assert_eq!(eval_render("10/4"), ("wat::core::rational".into(), "5/2".into()));
}

// ─── a literal reducing to a whole number is an Integer, NOT a Ratio (clj Long) ──

#[test]
fn rational_literal_denominator_one_is_integer_not_ratio() {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    // clj (grounded): 4/2 -> 2 (java.lang.Long). wat: a runtime Integer, i64.
    for (src, want) in [("4/2", 2i64), ("6/3", 2), ("1/1", 1), ("0/5", 0)] {
        let ast = wat::parse_one!(src)
            .unwrap_or_else(|e| panic!("{src:?} should parse: {e:?}"));
        let tv = eval_in_frozen(&ast, &world, &env)
            .unwrap_or_else(|e| panic!("{src:?} should eval: {e:?}"));
        assert!(
            matches!(tv.value(), Value::i64(n) if *n == want),
            "{src} must reduce to Integer {want} (clj Long), got {:?}",
            tv.value().type_name()
        );
    }
}

// ─── zero denominator is a clean reader error, never a panic ─────────────────────

#[test]
fn rational_literal_zero_denominator_is_clean_error() {
    // clj: "Divide by zero" at read time. wat: a clean parse Err, no panic.
    // `parse_one!` returns `Result<WatAST, ParseError>` directly — not a `StartupError` (no
    // startup pipeline runs here at all), so `assert_startup_error!` doesn't apply; grounded
    // directly against `ParseErrorKind::Lex(LexErrorKind::InvalidNumber(_))` (verified via
    // `--check` on a scratch fixture evaluating each literal).
    let err_pos = wat::parse_one!("1/0").expect_err("1/0 must refuse (divide by zero)");
    assert!(
        matches!(
            &err_pos.kind,
            ParseErrorKind::Lex(LexError { kind: LexErrorKind::InvalidNumber(msg), .. })
                if msg == "divide by zero"
        ),
        "expected ParseErrorKind::Lex(InvalidNumber(\"divide by zero\")); got {:?}",
        err_pos.kind
    );
    let err_neg = wat::parse_one!("-5/0").expect_err("-5/0 must refuse");
    assert!(
        matches!(
            &err_neg.kind,
            ParseErrorKind::Lex(LexError { kind: LexErrorKind::InvalidNumber(msg), .. })
                if msg == "divide by zero"
        ),
        "expected ParseErrorKind::Lex(InvalidNumber(\"divide by zero\")); got {:?}",
        err_neg.kind
    );
}
