//! RED probe — Stone C2: rational arithmetic (the piece that makes `1/2` compute).
//!
//! Stone B made `rational` representable; C1 made `bigint` a full arithmetic type. C2 adds rational
//! `+ - * /`, comparison, and `to-f64` — clj-faithful, grounded vs the running oracle this session:
//!   (+ 1/2 1/2) => 1N (bigint)   (+ 1/2 1/4) => 3/4   (+ 1/2 1) => 3/2   (+ 1/2 1N) => 3/2
//!   (+ 1/2 1.0) => 1.5 (f64)     (* 1/2 2) => 1N       (= 1/2 1/2) => true   (= 1/2 0.5) => false
//!   (< 1/2 2/3) => true
//! Collapse: ratio arithmetic reducing to a whole number becomes a `bigint` (C1's type). Contagion:
//! ratio⊕i64/bigint → rational; ratio⊕f64 → f64.
//!
//! RED at HEAD (post-C1): `(:wat::core::+ 1/2 1/2)` → NoMatchingClause (no rational arm on the `+`
//! defclause); `(:wat::core::< 1/2 2/3)` → TypeMismatch (no rational compare arm).

// rune:lint(no-inlined-wat) — reader/eval unit tests: the inline arithmetic forms
// (`(:wat::core::+ 1/2 1/2)`, …) ARE the subject under test — proving rational arithmetic
// evaluates. A co-located `.wat` fixture cannot test the eval of a form from outside;
// these are not a constructed world or driver.
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, ValueSnapshot};

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

// ─── collapse: ratio arithmetic reducing to a whole number → bigint ─────────────

#[test]
fn rational_arithmetic_collapses_to_bigint() {
    assert_eq!(eval_render("(:wat::core::+ 1/2 1/2)"), ("wat::core::bigint".into(), "1N".into()));
    assert_eq!(eval_render("(:wat::core::- 5/2 3/2)"), ("wat::core::bigint".into(), "1N".into()));
    assert_eq!(eval_render("(:wat::core::* 1/2 2)"),   ("wat::core::bigint".into(), "1N".into()));
    assert_eq!(eval_render("(:wat::core::/ 1/2 1/2)"), ("wat::core::bigint".into(), "1N".into()));
}

// ─── stays rational when the denominator survives ───────────────────────────────

#[test]
fn rational_arithmetic_stays_rational() {
    assert_eq!(eval_render("(:wat::core::+ 1/2 1/4)"), ("wat::core::rational".into(), "3/4".into()));
    assert_eq!(eval_render("(:wat::core::* 2/3 3/2)"), ("wat::core::bigint".into(), "1N".into()));
}

// ─── contagion: ratio ⊕ i64/bigint → rational; ratio ⊕ f64 → f64 ────────────────

#[test]
fn rational_arithmetic_contagion() {
    assert_eq!(eval_render("(:wat::core::+ 1/2 1)"),   ("wat::core::rational".into(), "3/2".into()));
    assert_eq!(eval_render("(:wat::core::+ 1/2 1N)"),  ("wat::core::rational".into(), "3/2".into()));
    assert_eq!(eval_render("(:wat::core::+ 1/2 1.0)").0, "wat::core::f64"); // float contagion
}

// ─── comparison + category-aware equality ───────────────────────────────────────

#[test]
fn rational_comparison_and_equality() {
    assert_eq!(eval_render("(:wat::core::< 1/2 2/3)").1, "true");
    assert_eq!(eval_render("(:wat::core::= 1/2 1/2)").1, "true");
    assert_eq!(eval_render("(:wat::core::= 1/2 0.5)").1, "false"); // rational vs f64 → false (category-aware)
}
