//! RED probe — Stone C3: i64 arithmetic overflow → clean error, not silent wrap.
//!
//! wat's `i64 + - *` currently `wrapping_*` — `(+ i64::MAX 1)` silently returns `i64::MIN` and reports OK,
//! a wrong value with no signal (the substrate's own honesty doctrine violated). clj's default `+` THROWS
//! on overflow ("long overflow"). The builder's ruling: don't wrap, error. C3 → `checked_*` + a distinct
//! `IntegerOverflow` error ("do what rust does" — a Rust Result error, not promotion to bigint).
//!
//! Grounded vs clj 1.12.4: (+ Long/MAX 1) => THROW "long overflow"; (* Long/MAX 2) => THROW.
//! RED at HEAD: the overflow cases return OK with a wrapped value.
//!
//! Asserts on the error KIND structurally (RuntimeErrorKind::IntegerOverflow / DivisionByZero), not on a
//! message substring — per the 296 loose-assert doctrine.

// rune:lint(no-inlined-wat) — reader/eval unit tests: the inline arithmetic forms ARE the
// subject under test (proving i64 overflow errors instead of wrapping). Not a world/driver.
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, ValueSnapshot};
use wat::value::{EvalBreak, RuntimeErrorKind};

/// Ok((type_name, rendered)) on success, Err(EvalBreak) on eval failure.
fn eval_res(src: &str) -> Result<(String, String), EvalBreak> {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!(src).expect("parse");
    let tv = eval_in_frozen(&ast, &world, &env)?;
    Ok((
        tv.value().type_name().to_string(),
        ValueSnapshot::of_tracked(&tv).rendered,
    ))
}

// ─── i64 overflow is a structural IntegerOverflow, not a wrapped value ───────────

#[test]
fn i64_overflow_is_integer_overflow_not_wrap() {
    for s in [
        "(:wat::core::+ 9223372036854775807 1)", // i64::MAX + 1
        "(:wat::core::* 9223372036854775807 2)", // i64::MAX * 2
        "(:wat::core::- -9223372036854775808 1)", // i64::MIN - 1
    ] {
        assert!(
            matches!(eval_res(s), Err(EvalBreak::Diagnostic(ref e)) if matches!(e.kind, RuntimeErrorKind::IntegerOverflow { .. })),
            "{s} must be a RuntimeErrorKind::IntegerOverflow, not a wrapped value"
        );
    }
}

// ─── overflow is DISTINCT from division-by-zero (not conflated) ──────────────────

#[test]
fn overflow_is_distinct_from_division_by_zero() {
    assert!(
        matches!(eval_res("(:wat::core::+ 9223372036854775807 1)"), Err(EvalBreak::Diagnostic(ref e)) if matches!(e.kind, RuntimeErrorKind::IntegerOverflow { .. })),
        "overflow must be IntegerOverflow"
    );
    assert!(
        matches!(eval_res("(:wat::core::/ 1 0)"), Err(EvalBreak::Diagnostic(ref e)) if matches!(e.kind, RuntimeErrorKind::DivisionByZero)),
        "division-by-zero must stay its own distinct DivisionByZero kind"
    );
}

// ─── normal i64 arithmetic is unaffected ────────────────────────────────────────

#[test]
fn normal_i64_arithmetic_unaffected() {
    assert_eq!(eval_res("(:wat::core::+ 1 1)").unwrap().1, "2");
    assert_eq!(eval_res("(:wat::core::- 5 3)").unwrap().1, "2");
    assert_eq!(eval_res("(:wat::core::* 6 7)").unwrap().1, "42");
}
