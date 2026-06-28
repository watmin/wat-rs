//! Disconfirming repro — arc-242 false-positive on a primitive-typed fn with a
//! BARE-VALUE body.
//!
//! `(fn [] -> :wat::core::nil nil)` is a VALID 0-ary function:
//!   - `:wat::core::nil` after `->` is the RETURN TYPE (type position).
//!   - bare `nil` is the BODY (value position — the nil value per arc 242).
//!
//! The checker currently mislocates the return-type keyword as a value expression
//! and false-fires the arc-242 "type-keyword-in-value-position" doctrine.
//!
//! These tests are RED until the substrate stops treating the return-type
//! annotation as a body expression. DO NOT "fix" them by changing the (correct)
//! source — the source is right; the checker is wrong.
//!
//! All four tests share the combined co-located fixture which covers every
//! triggering combination: nil-bare-nil, i64-bare-int, defclause+nil-main.

use wat::freeze::startup_beside;

/// The reported case: return type `:wat::core::nil`, body bare `nil`.
#[test]
fn nil_typed_defn_with_bare_nil_body_checks_clean() {
    let r = startup_beside(file!());
    assert!(
        r.is_ok(),
        "return type `:wat::core::nil` is in TYPE position, not value position; \
         the checker must not reject this valid 0-ary nil-returning fn. Got: {:?}",
        r.err()
    );
}

/// Class-characterizer: i64 return type with bare int body.
#[test]
fn i64_typed_defn_with_bare_int_body_checks_clean() {
    let r = startup_beside(file!());
    assert!(
        r.is_ok(),
        "return type `:wat::core::i64` is in TYPE position, not value position; \
         a bare-int-bodied fn must check clean. Got: {:?}",
        r.err()
    );
}

/// BISECTION — defclause + nil main (triggering combination per arc 242 bug report).
#[test]
fn defclause_then_nil_main_checks_clean() {
    let r = startup_beside(file!());
    assert!(r.is_ok(), "defclause + nil-main must check clean. Got: {:?}", r.err());
}

/// BISECTION — the full gate-2 shape: defclause + a caller fn + nil-main.
#[test]
fn defclause_with_caller_and_nil_main_checks_clean() {
    let r = startup_beside(file!());
    assert!(r.is_ok(), "defclause + caller + nil-main must check clean. Got: {:?}", r.err());
}
