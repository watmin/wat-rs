//! Gate — arc 278 numeric-tower increment: `mod` / `rem` / `quot` for i64 (clj-faithful).
//!
//! Scope: i64 only (bigint/rational mod/rem/quot is a named out-of-scope follow-on).
//! The three ops differ ONLY by sign — see BRIEF-STONE-int-modrem.md's table:
//!   quot truncates toward zero (`checked_div`); rem takes the DIVIDEND's sign
//!   (`checked_rem`); mod takes the DIVISOR's sign, floored (`r=a%b; if r!=0 &&
//!   (r<0)!=(b<0) {r+b} else {r}`).
//!
//! The co-located fixture (`probe_int_modrem.wat`) asserts every Ok-valued cell of the
//! sign table (incl. the i64::MIN/-1 rem/mod=0 edge) via a `deftest'`. This file asserts
//! the deftest' passes, plus the three div-by-zero cases and the i64::MIN quot -1
//! overflow — all Err-valued, so they can't live inside an assert-eq deftest' body; this
//! mirrors how `probe_rational_C3_i64_overflow.rs` asserts a runtime error structurally
//! (RuntimeErrorKind, not a message substring — per the 296 loose-assert doctrine).

// rune:lint(no-inlined-wat) — the inline arithmetic forms below ARE the subject under
// test (div-by-zero / MIN overflow edges); the sign-table world is a co-located `.wat`
// fixture (driven via call_beside), not inlined.
use wat::freeze::{call_beside, eval_in_frozen, startup_bare};
use wat::runtime::Environment;
use wat::value::{EvalBreak, RuntimeErrorKind};

/// Ok(rendered) on success, Err(EvalBreak) on eval failure — mirrors
/// `probe_rational_C3_i64_overflow.rs`'s `eval_res` helper.
fn eval_res(src: &str) -> Result<String, EvalBreak> {
    let world = startup_bare().expect("startup");
    let env = Environment::new();
    let ast = wat::parse_one!(src).expect("parse");
    let tv = eval_in_frozen(&ast, &world, &env)?;
    Ok(wat::runtime::ValueSnapshot::of_tracked(&tv).rendered)
}

// ─── the sign table (every Ok-valued cell) ────────────────────────────────────────

#[test]
fn int_modrem_sign_table() {
    // Arc 278 the vacuous-gate wall — was `call_beside(..).is_ok()`, which certified only
    // that the fixture froze and ran; every assert-eq in the sign table was decoration.
    call_beside(file!(), ":user::modrem_sign_table")
        .expect_passed("modrem_sign_table deftest must pass (arc 278 mod/rem/quot sign table)");
}

// ─── divide-by-zero → DivisionByZero, never a panic (STOP-PANIC) ──────────────────

#[test]
fn int_modrem_division_by_zero() {
    for s in [
        "(:wat::core::quot 1 0)",
        "(:wat::core::rem 1 0)",
        "(:wat::core::mod 1 0)",
    ] {
        assert!(
            matches!(eval_res(s), Err(EvalBreak::Diagnostic(ref e)) if matches!(e.kind, RuntimeErrorKind::DivisionByZero)),
            "{s} must be a RuntimeErrorKind::DivisionByZero"
        );
    }
}

// ─── i64::MIN edge: quot(MIN,-1) overflows (rem/mod(MIN,-1)=0 is asserted in the fixture) ──

#[test]
fn int_modrem_quot_min_overflows() {
    let s = "(:wat::core::quot -9223372036854775808 -1)";
    assert!(
        matches!(eval_res(s), Err(EvalBreak::Diagnostic(ref e)) if matches!(e.kind, RuntimeErrorKind::IntegerOverflow { .. })),
        "{s} must be a RuntimeErrorKind::IntegerOverflow (clj: (quot Long/MIN_VALUE -1) throws)"
    );
}
