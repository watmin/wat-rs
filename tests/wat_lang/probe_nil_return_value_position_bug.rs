//! Disconfirming repro — arc-242 false-positive on a primitive-typed fn with a
//! BARE-VALUE body.
//!
//! `(fn [] -> :wat::core::nil nil)` is a VALID 0-ary function:
//!   - `:wat::core::nil` after `->` is the RETURN TYPE (type position — a type
//!     keyword belongs here per arc 242 Doctrine 1).
//!   - bare `nil` is the BODY (value position — the nil value per arc 242).
//!
//! The checker currently mislocates the return-type keyword as a value expression
//! and false-fires the arc-242 "type-keyword-in-value-position" doctrine at
//! `src/check.rs:3373` (the `WatAST::Keyword if is_primitive_type_keyword_in_value_position`
//! arm of `infer_expr`), rejecting a correct function with:
//!     MalformedForm { head: ":wat::core::nil",
//!                     reason: "...use bare `nil` in value position" }
//!
//! It hid because the stdlib always writes `-> :wat::core::nil (some-form)` — a
//! FORM body — so the return-type/body split is unambiguous. The BARE-VALUE body
//! is the untested path: the return-type annotation leaks into the body and reaches
//! `infer_expr`.
//!
//! These tests are RED until the substrate stops treating the return-type
//! annotation as a body expression. DO NOT "fix" them by changing the (correct)
//! source — the source is right; the checker is wrong (`feedback_nonintuitive_error_is_pivot`).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// The reported case: return type `:wat::core::nil`, body bare `nil`.
#[test]
fn nil_typed_defn_with_bare_nil_body_checks_clean() {
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        r.is_ok(),
        "return type `:wat::core::nil` is in TYPE position, not value position; \
         the checker must not reject this valid 0-ary nil-returning fn. Got: {:?}",
        r.err()
    );
}

/// Class-characterizer: is the bug nil-specific, or does ANY primitive-keyword
/// return type with a bare-value body trip it? Return type `:wat::core::i64`,
/// body bare `42`.
#[test]
fn i64_typed_defn_with_bare_int_body_checks_clean() {
    let src = "(:wat::core::defn :user::main [] -> :wat::core::i64 42)";
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        r.is_ok(),
        "return type `:wat::core::i64` is in TYPE position, not value position; \
         a bare-int-bodied fn must check clean. Got: {:?}",
        r.err()
    );
}

/// BISECTION — does adding a `defclause` to the program trip the `:wat::core::nil`
/// error? The defclause's generated no-match/`:NoMatchingClause` fallback (arc
/// 237.4) is the suspect: it may emit `:wat::core::nil` in value position, which
/// arc 242 later outlawed.
#[test]
fn defclause_then_nil_main_checks_clean() {
    let src = r#"
(:wat::core::defclause :my::label
  ([x <- :wat::core::i64] -> :wat::core::String "i64")
  ([x <- :wat::core::f64] -> :wat::core::String "f64"))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(r.is_ok(), "defclause + nil-main must check clean. Got: {:?}", r.err());
}

/// BISECTION — the full probe_arc237_8b gate-2 shape: defclause + a caller fn +
/// nil-main. This is the confirmed failing structure.
#[test]
fn defclause_with_caller_and_nil_main_checks_clean() {
    let src = r#"
(:wat::core::defclause :my::label
  ([x <- :wat::core::i64] -> :wat::core::String "i64")
  ([x <- :wat::core::f64] -> :wat::core::String "f64"))
(:wat::core::defn :user::compute [] -> :wat::core::String (:my::label 42))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(r.is_ok(), "defclause + caller + nil-main must check clean. Got: {:?}", r.err());
}
