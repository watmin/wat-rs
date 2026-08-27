//! RED probe — Stone C4: mixed-float contagion (the `(+ 1 2.0)` unlock; the last numeric stone).
//!
//! The numeric tower is one contagion pattern (C1/C2). Two mixed pairs remain — both → f64 (float wins):
//! `i64 ⊕ f64` and `f64 ⊕ bigint`. C4 installs their arms, lighting up `(+ 1 2.0) => 3.0`. It also folds in
//! the `i64 ↔ f64` equality arm: clj `(= 1 1.0)` => false (category-aware), but wat currently ERRORS.
//!
//! Grounded vs clj 1.12.4: (+ 1 2.0) => 3.0 Double; (+ 2.0 1N) => 3.0; (= 1 1.0) => false; (< 1 2.0) => true.
//! RED at HEAD: the mixed-float arithmetic → NoMatchingClause; (= 1 1.0) => TypeMismatch.

// rune:lint(no-inlined-wat) — reader/eval unit tests: the inline arithmetic forms ARE the
// subject under test (proving mixed-float contagion evaluates). Not a world/driver.
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, RuntimeErrorKind, ValueSnapshot};

/// Ok((type_name, rendered)) on success, Err(debug string) on eval failure.
fn eval_try(src: &str) -> Result<(String, String), String> {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!(src).map_err(|e| format!("parse: {e:?}"))?;
    let tv = eval_in_frozen(&ast, &world, &env).map_err(|e| format!("{e:?}"))?;
    Ok((
        tv.value().type_name().to_string(),
        ValueSnapshot::of_tracked(&tv).rendered,
    ))
}

// ─── float wins: i64 ⊕ f64 → f64, f64 ⊕ bigint → f64 ────────────────────────────

#[test]
fn mixed_float_arithmetic_promotes_to_f64() {
    for s in [
        "(:wat::core::+ 1 2.0)",  // i64 ⊕ f64
        "(:wat::core::+ 2.0 1)",  // f64 ⊕ i64
        "(:wat::core::+ 2.0 1N)", // f64 ⊕ bigint
        "(:wat::core::+ 1N 2.0)", // bigint ⊕ f64
        "(:wat::core::* 3 2.0)",  // i64 ⊕ f64, *
        "(:wat::core::- 5.0 2)",  // f64 ⊕ i64, -
    ] {
        let (ty, _) = eval_try(s).unwrap_or_else(|e| panic!("{s} should promote to f64: {e}"));
        assert_eq!(ty, "wat::core::f64", "{s} — float wins the mixed op");
    }
}

// ─── equality is category-aware: (= i64 f64) → false, never an error ─────────────

#[test]
fn mixed_numeric_equality_is_category_aware_false() {
    // clj: (= 1 1.0) => false (int vs float — different category; NOT an error, NOT true).
    let (ty, rendered) = eval_try("(:wat::core::= 1 1.0)").expect("= must not error on mixed numerics");
    assert_eq!(ty, "wat::core::bool");
    assert_eq!(rendered, "false");
}

// ─── the honest N-ary gap: heterogeneous N-ary tosses (permanent guard) ──────────

#[test]
fn mixed_n_ary_is_an_honest_gap() {
    // C4 adds only 2-ary mixed arms; a heterogeneous N-ary call tosses a clean NoMatchingClause —
    // the caller homogenizes ((apply + (map to-f64 …))) then folds. This must STAY true after C4.
    //
    // NOT via `eval_try` here — that helper collapses the error to a Debug-formatted `String`
    // (`.map_err(|e| format!("{e:?}"))`), which erases the discriminant. This eval path is a
    // direct `eval_in_frozen` on a hand-parsed AST (no `check_program` pass runs), so the
    // failure surfaces as `RuntimeError` — not a `StartupError` — at dispatch time; grounded
    // against the doc comment's own named mechanism (`RuntimeErrorKind::NoMatchingClause`).
    // The full `attempted_clauses` list (25 clause shapes) is deliberately NOT asserted
    // field-by-field here — it is `+`'s whole numeric-tower clause set, and reproducing it
    // verbatim would be as brittle as it is long; `name` + `called_arity` + the called arg
    // TYPES (the actual discriminating shape of a "heterogeneous N-ary" call) are the guard.
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!("(:wat::core::+ 1 2.0 3)").expect("must parse");
    let err = eval_in_frozen(&ast, &world, &env)
        .expect_err("mixed N-ary must toss (the honest gap), not silently coerce");
    assert!(
        matches!(
            err.kind(),
            RuntimeErrorKind::NoMatchingClause { name, called_arity, called_args, .. }
                if name == ":wat::core::+"
                && *called_arity == 3
                && called_args.iter().map(|v| v.type_name).collect::<Vec<_>>()
                    == ["wat::core::i64", "wat::core::f64", "wat::core::i64"]
        ),
        "expected RuntimeErrorKind::NoMatchingClause(+, arity 3, [i64,f64,i64]); got {:?}",
        err
    );
}
