//! RED probe — Stone C1: `bigint` as a first-class arithmetic integer type + numeric-scalar naming.
//!
//! clj numeric parity (ratified): rational arithmetic collapses to BigInt (`(+ 1/2 1/2)` => `1N`), and
//! BigInt is a FULL arithmetic integer type — contagious, never-demotes, arbitrary-precision. The runtime
//! has none. C1 adds `bigint` (lowercase per Doctrine 2) and cleans up the numeric-scalar naming.
//!
//! Grounded vs clj 1.12.4 this session (AD ORACVLVM):
//!   1N => bigint, renders "1N"          (+ 1N 1N) => 2N     (+ 1 1N) => 2N   (contagion)
//!   (/ 6N 3N) => 2N (bigint)            (/ 1N 2N) => 1/2 (rational)
//!   (= 1N 1) => true  (= 1N 1.0) => false   (category-aware =)
//!   scalar types are lowercase: 1/2 -> "wat::core::rational",  \a -> "wat::core::char"
//!
//! RED at HEAD: `1N` does not lex (InvalidNumber); `1/2` type_name is capital "wat::core::Rational";
//! `\a` type_name is capital "wat::core::Char".

// rune:lint(no-inlined-wat) — reader/eval unit tests: the inline forms (`1N`,
// `(:wat::core::+ 1N 1N)`, `\a`) ARE the subject under test — proving that bigint literals
// lex/render and that arithmetic evals. A co-located `.wat` fixture cannot test the reader/eval
// of a literal from outside; these are not a constructed world or driver.
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, ValueSnapshot};

/// (type_name, rendered) for a source form evaluated through the real loader.
fn eval_render(src: &str) -> (String, String) {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!(src).unwrap_or_else(|e| panic!("{src:?} should parse: {e:?}"));
    let tv = eval_in_frozen(&ast, &world, &env)
        .unwrap_or_else(|e| panic!("{src:?} should eval: {e:?}"));
    (
        tv.value().type_name().to_string(),
        ValueSnapshot::of_tracked(&tv).rendered,
    )
}

// ─── bigint: literal + arbitrary-precision arithmetic + contagion ───────────────

#[test]
fn bigint_literal_reads_and_renders() {
    let (ty, r) = eval_render("1N");
    assert_eq!(ty, "wat::core::bigint", "1N must eval to a bigint");
    assert_eq!(r, "1N", "bigint renders with the N suffix (pr/edn form)");
}

#[test]
fn bigint_arithmetic_stays_bigint_and_is_contagious() {
    assert_eq!(eval_render("(:wat::core::+ 1N 1N)"), ("wat::core::bigint".into(), "2N".into()));
    // contagion: i64 ⊕ bigint → bigint
    assert_eq!(eval_render("(:wat::core::+ 1 1N)"), ("wat::core::bigint".into(), "2N".into()));
    assert_eq!(eval_render("(:wat::core::* 2 3N)"), ("wat::core::bigint".into(), "6N".into()));
}

#[test]
fn bigint_arithmetic_never_overflows() {
    // i64::MAX as bigint, times 2 — arbitrary precision, NO wrap/overflow/error.
    let (ty, r) = eval_render("(:wat::core::* 9223372036854775807N 2)");
    assert_eq!(ty, "wat::core::bigint");
    assert_eq!(r, "18446744073709551614N");
}

#[test]
fn bigint_division_collapses_like_clj() {
    assert_eq!(eval_render("(:wat::core::/ 6N 3N)"), ("wat::core::bigint".into(), "2N".into()));
    assert_eq!(eval_render("(:wat::core::/ 1N 2N)"), ("wat::core::rational".into(), "1/2".into()));
}

#[test]
fn bigint_equality_is_category_aware() {
    // (= 1N 1) => true (both INTEGER category); (= 1N 1.0) => false (bigint vs f64)
    assert_eq!(eval_render("(:wat::core::= 1N 1)").1, "true");
    assert_eq!(eval_render("(:wat::core::= 1N 1.0)").1, "false");
}

// ─── naming cleanup: scalar types are lowercase (Doctrine 2) ─────────────────────

#[test]
fn rational_type_name_is_lowercase() {
    // Stone B shipped capital "wat::core::Rational" — a Doctrine-2 mumble. C1 fixes it.
    assert_eq!(eval_render("1/2").0, "wat::core::rational");
}

#[test]
fn char_type_name_is_lowercase() {
    // Doctrine 2 renamed the surface to `char`, but type_name still emits capital "wat::core::Char".
    assert_eq!(eval_render("\\a").0, "wat::core::char");
}
