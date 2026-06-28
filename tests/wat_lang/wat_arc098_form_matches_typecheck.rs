//! Arc 098 slice 1 — `:wat::form::matches?` type-check side.
//!
//! Slice 1 lands the pattern grammar + classifier + type-check
//! pipeline. The runtime arm is a stub that errors on call; these
//! tests exercise the type checker only by either (a) putting a
//! valid pattern in a named defn (checker walks the body), or (b)
//! asserting that an invalid pattern is REJECTED at startup with
//! the expected diagnostic.
//!
//! Slice 2 swaps the runtime stub for the real walker and adds
//! end-to-end `wat-tests/form/matches.wat` coverage.
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

use wat::freeze::{startup_beside, startup_from_file, StartupError};

fn expect_startup_ok(rel_path: Option<&str>) {
    let result = match rel_path {
        Some(p) => startup_from_file(p),
        None => startup_beside(file!()),
    };
    result.expect("startup should succeed for valid patterns");
}

fn expect_check_error(rel_path: &str, expected_substring: &str) {
    match startup_from_file(rel_path) {
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

// ─── Valid patterns: type-check passes ──────────────────────────────

#[test]
fn valid_simple_binding_and_comparison() {
    expect_startup_ok(None);
}

#[test]
fn valid_logical_combinators() {
    expect_startup_ok(None);
}

#[test]
fn valid_where_escape_returns_bool() {
    expect_startup_ok(None);
}

// ─── Invalid patterns: each error class ─────────────────────────────

#[test]
fn rejects_unknown_struct_type() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_unknown_struct_bad.wat",
        "unknown struct type :test::DoesNotExist",
    );
}

#[test]
fn rejects_unknown_field() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_unknown_field_bad.wat",
        "struct :test::PaperResolved has no field :unknown-field",
    );
}

#[test]
fn rejects_unknown_constraint_head() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_unknown_constraint_bad.wat",
        "unknown matcher head: :foo",
    );
}

#[test]
fn rejects_where_body_non_bool() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_where_nonbool_bad.wat",
        "where-body",
    );
}

#[test]
fn rejects_arity_zero() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_arity_zero_bad.wat",
        ":wat::form::matches?",
    );
}

#[test]
fn rejects_pattern_head_non_keyword() {
    expect_check_error(
        "tests/wat_lang/wat_arc098_form_matches_typecheck_pattern_head_nonkw_bad.wat",
        "pattern head must be a struct type keyword",
    );
}
