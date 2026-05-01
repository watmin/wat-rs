//! Arc 098 slice 1 — `:wat::form::matches?` type-check side.
//!
//! Slice 1 lands the pattern grammar + classifier + type-check
//! pipeline. The runtime arm is a stub that errors on call; these
//! tests exercise the type checker only by either (a) wrapping a
//! valid pattern in an `(if false ...)` so the checker sees it but
//! the runtime never dispatches, or (b) asserting that an invalid
//! pattern is REJECTED at startup with the expected diagnostic.
//!
//! Slice 2 swaps the runtime stub for the real walker and adds
//! end-to-end `wat-tests/std/form/matches.wat` coverage.
//!
//! ──────────────────────────────────────────────────────────────────
//!
//! Per the DESIGN, valid pattern shape is:
//!
//! ```text
//! (:wat::form::matches? SUBJECT
//!   (:TYPE-NAME (= ?var :field) ... <constraint> ...))
//! ```
//!
//! Recognized constraint heads inside clauses: `=`, `<`, `>`, `<=`,
//! `>=`, `not=`, `and`, `or`, `not`, `where`. Each invalid pattern
//! produces a `MalformedForm` diagnostic naming the offense; this
//! file exercises every error class enumerated in the DESIGN's
//! "Errors at expansion" list.

use std::sync::Arc;
use wat::freeze::{startup_from_source, StartupError};
use wat::load::InMemoryLoader;

fn check_only(src: &str) -> Result<(), String> {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}

fn expect_check_error(src: &str, expected_substring: &str) {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Err(StartupError::Check(errs)) => {
            let rendered = format!("{}", errs);
            assert!(
                rendered.contains(expected_substring),
                "expected check error containing {:?} but got:\n{}",
                expected_substring,
                rendered
            );
        }
        Ok(_) => panic!("expected type-check failure containing {:?}; got success", expected_substring),
        Err(other) => panic!(
            "expected type-check failure containing {:?}; got {}",
            expected_substring, other
        ),
    }
}

/// Wrap a `matches?` invocation so the type checker walks the
/// pattern but the slice-1 runtime stub never fires. The `if false`
/// path is dead at runtime; type-check still walks both branches.
const PROLOGUE_VALID: &str = r#"
(:wat::core::struct :test::PaperResolved
  (outcome :wat::core::String)
  (grace-residue :wat::core::f64))
(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((p :test::PaperResolved)
      (:test::PaperResolved/new "Grace" 7.5))
     ((b :wat::core::bool)
      (:wat::core::if true -> :wat::core::bool true SUBSTITUTE_HERE)))
    (:wat::io::IOWriter/println stdout (:wat::core::bool::to-string b))))
"#;

fn valid_src(matches_call: &str) -> String {
    PROLOGUE_VALID.replace("SUBSTITUTE_HERE", matches_call)
}

const PROLOGUE_INVALID: &str = r#"
(:wat::core::struct :test::PaperResolved
  (outcome :wat::core::String)
  (grace-residue :wat::core::f64))
(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((p :test::PaperResolved)
      (:test::PaperResolved/new "Grace" 7.5))
     ((b :wat::core::bool)
      (:wat::core::if true -> :wat::core::bool true SUBSTITUTE_HERE)))
    (:wat::io::IOWriter/println stdout (:wat::core::bool::to-string b))))
"#;

fn invalid_src(matches_call: &str) -> String {
    PROLOGUE_INVALID.replace("SUBSTITUTE_HERE", matches_call)
}

// ─── Valid patterns: type-check passes ──────────────────────────────

#[test]
fn valid_simple_binding_and_comparison() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?outcome :outcome)
            (= ?grace-residue :grace-residue)
            (= ?outcome "Grace")
            (> ?grace-residue 5.0)))
    "#;
    check_only(&valid_src(call)).expect("valid pattern should type-check");
}

#[test]
fn valid_logical_combinators() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?outcome :outcome)
            (= ?grace-residue :grace-residue)
            (:and
              (= ?outcome "Grace")
              (:or
                (> ?grace-residue 5.0)
                (< ?grace-residue 0.0))
              (:not (= ?outcome "Loss")))))
    "#;
    check_only(&valid_src(call)).expect("logical combinators should type-check");
}

#[test]
fn valid_where_escape_returns_bool() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?outcome :outcome)
            (:where (:wat::core::string::contains? ?outcome "Grace"))))
    "#;
    check_only(&valid_src(call)).expect("where-body returning :bool should type-check");
}

// ─── Invalid patterns: each error class ─────────────────────────────

#[test]
fn rejects_unknown_struct_type() {
    let call = r#"
        (:wat::form::matches? p
          (:test::DoesNotExist
            (= ?o :outcome)))
    "#;
    expect_check_error(&invalid_src(call), "unknown struct type :test::DoesNotExist");
}

#[test]
fn rejects_unknown_field() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?o :unknown-field)))
    "#;
    expect_check_error(
        &invalid_src(call),
        "struct :test::PaperResolved has no field :unknown-field",
    );
}

#[test]
fn rejects_unknown_constraint_head() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?o :outcome)
            (:foo ?o "x")))
    "#;
    expect_check_error(&invalid_src(call), "unknown matcher head: :foo");
}

#[test]
fn rejects_where_body_non_bool() {
    let call = r#"
        (:wat::form::matches? p
          (:test::PaperResolved
            (= ?o :outcome)
            (:where ?o)))
    "#;
    // `?o` is `:String`, not `:bool` — should reject.
    expect_check_error(&invalid_src(call), "where-body");
}

#[test]
fn rejects_arity_zero() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((b :bool) (:wat::core::if true -> :bool true (:wat::form::matches?))))
            (:wat::io::IOWriter/println stdout "ok")))
    "#;
    expect_check_error(src, ":wat::form::matches?");
}

#[test]
fn rejects_pattern_head_non_keyword() {
    let call = r#"
        (:wat::form::matches? p
          (42
            (= ?o :outcome)))
    "#;
    expect_check_error(&invalid_src(call), "pattern head must be a struct type keyword");
}
