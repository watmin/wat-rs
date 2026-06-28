//! End-to-end tests for the typed `:wat::core::if` and
//! `:wat::core::match` forms. Per the 2026-04-20 INSCRIPTION, both
//! forms now require an explicit `-> :T` between the scrutinee/cond
//! and the arms/branches. Each arm/branch is checked against `:T`
//! independently, so a divergent body produces a per-body
//! TypeMismatch that names the branch (`then-branch`, `else-branch`,
//! `arm #1`, ...) instead of a unifier-flavored "branches didn't
//! unify" message.
//!
//! Coverage:
//!
//! - Happy path: typed `if` on true/false returns its branch; typed
//!   `match` returns the matching arm's body.
//! - Migration-hint MalformedForm when the old untyped shape is used
//!   (`(if cond then else)` / `(match scrut arm1 arm2)`).
//! - Missing `->` marker rejected with a specific MalformedForm.
//! - Missing type keyword after `->` rejected.
//! - Wrong-arity forms rejected with guidance.
//! - Then/else branch type mismatch surfaces a per-branch error.
//! - Match arm body type mismatch surfaces a per-arm error.
//! - Declared `:T` is the form's inferred result (so a `let`
//!   surrounding it sees `:T`, not "some branch type").
//! - Nested typed forms compose normally.

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

fn check_errors(path: &str) -> Vec<CheckError> {
    match startup_from_file(path) {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {:?}", other),
        Ok(_) => panic!("expected Check errors; startup succeeded"),
    }
}

fn assert_malformed_mentioning(errs: &[CheckError], head: &str, needle: &str) {
    let hit = errs.iter().any(|e| match e {
        CheckError { kind: CheckErrorKind::MalformedForm { head: h, reason, .. }, .. } => h == head && reason.contains(needle),
        _ => false,
    });
    assert!(
        hit,
        "expected MalformedForm on {} mentioning {:?}; got {:?}",
        head, needle, errs
    );
}

fn assert_type_mismatch_on(errs: &[CheckError], callee: &str, param: &str) {
    let hit = errs.iter().any(|e| match e {
        CheckError { kind: CheckErrorKind::TypeMismatch { callee: c, param: p, .. }, .. } => c == callee && p == param,
        _ => false,
    });
    assert!(
        hit,
        "expected TypeMismatch on {} param {:?}; got {:?}",
        callee, param, errs
    );
}

// ─── Happy path ───────────────────────────────────────────────────────

#[test]
fn typed_if_returns_then_branch_on_true() {
    assert!(matches!(run("tests/types/typed_if_match_if_true.wat"), Value::i64(11)));
}

#[test]
fn typed_if_returns_else_branch_on_false() {
    assert!(matches!(run("tests/types/typed_if_match_if_false.wat"), Value::i64(22)));
}

#[test]
fn typed_match_on_some_returns_some_arm() {
    assert!(matches!(run("tests/types/typed_if_match_match_some.wat"), Value::i64(7)));
}

#[test]
fn typed_match_on_none_returns_none_arm() {
    assert!(matches!(run("tests/types/typed_if_match_match_none.wat"), Value::i64(-1)));
}

// ─── Migration-hint refusals (old untyped shape) ──────────────────────

#[test]
fn untyped_if_gives_migration_hint() {
    // Arc 258.1: bare `(if cond then else)` is now VALID — the mandatory `-> :T`
    // annotation is gone; the form's type is inferred from branch unification.
    // This test was previously checking a migration-hint rejection; it now asserts
    // the new behavior: bare if type-checks and evals correctly.
    assert!(matches!(run("tests/types/typed_if_match_untyped_if_bare.wat"), Value::i64(1)));
}

#[test]
fn untyped_match_gives_migration_hint() {
    // Three args, where the second is NOT `->` — detected as the
    // old untyped shape.
    let errs = check_errors("tests/types/typed_if_match_untyped_match_bad.wat");
    assert_malformed_mentioning(&errs, ":wat::core::match", "now requires `-> :T`");
}

// ─── Structural refusals ──────────────────────────────────────────────

#[test]
fn if_without_type_keyword_after_arrow_rejected() {
    let errs = check_errors("tests/types/typed_if_match_if_no_type_kw_bad.wat");
    assert_malformed_mentioning(&errs, ":wat::core::if", "type keyword");
}

#[test]
fn match_without_type_keyword_after_arrow_rejected() {
    let errs = check_errors("tests/types/typed_if_match_match_no_type_kw_bad.wat");
    assert_malformed_mentioning(&errs, ":wat::core::match", "type keyword");
}

#[test]
fn if_wrong_arity_rejected_with_shape_guidance() {
    // Six args — one too many for both the bare 3-arg and annotated 5-arg forms.
    // Arc 258.1 updated the error to name both valid shapes; the needle matches
    // the annotated-shape portion of the message.
    let errs = check_errors("tests/types/typed_if_match_if_wrong_arity_bad.wat");
    assert_malformed_mentioning(&errs, ":wat::core::if", "(:wat::core::if cond -> :T then else)");
}

#[test]
fn match_too_few_args_rejected_with_shape_guidance() {
    let errs = check_errors("tests/types/typed_if_match_match_too_few_bad.wat");
    assert_malformed_mentioning(&errs, ":wat::core::match", "at least 4 args");
}

// ─── Branch-type-mismatch locality ────────────────────────────────────

#[test]
fn if_then_branch_type_mismatch_named_by_branch() {
    let errs = check_errors("tests/types/typed_if_match_then_branch_mismatch_bad.wat");
    assert_type_mismatch_on(&errs, ":wat::core::if", "then-branch");
}

#[test]
fn if_else_branch_type_mismatch_named_by_branch() {
    let errs = check_errors("tests/types/typed_if_match_else_branch_mismatch_bad.wat");
    assert_type_mismatch_on(&errs, ":wat::core::if", "else-branch");
}

#[test]
fn match_arm_type_mismatch_named_by_arm_index() {
    // Arm #2 (the :None arm) produces a String instead of i64.
    let errs = check_errors("tests/types/typed_if_match_arm_type_mismatch_bad.wat");
    assert_type_mismatch_on(&errs, ":wat::core::match", "arm #2");
}

// ─── Condition-type refusal on if ─────────────────────────────────────

#[test]
fn if_non_bool_cond_rejected_at_check() {
    let errs = check_errors("tests/types/typed_if_match_non_bool_cond_bad.wat");
    assert_type_mismatch_on(&errs, ":wat::core::if", "cond");
}

// ─── Declared type is the form's result type ──────────────────────────

#[test]
fn typed_if_result_flows_into_enclosing_let_bind() {
    assert!(matches!(run("tests/types/typed_if_match_if_result_in_let.wat"), Value::i64(10)));
}

#[test]
fn typed_match_result_flows_into_enclosing_let_bind() {
    match run("tests/types/typed_if_match_match_result_in_let.wat") {
        Value::String(s) => assert_eq!(&*s, "yes"),
        other => panic!("expected \"yes\"; got {:?}", other),
    }
}

// ─── Nesting ──────────────────────────────────────────────────────────

#[test]
fn typed_if_inside_typed_match_arm_composes() {
    assert!(matches!(run("tests/types/typed_if_match_if_inside_match.wat"), Value::i64(3)));
}

// ─── Bare-symbol variant pattern hint (arc 105 follow-up) ──────────────

#[test]
fn match_bare_symbol_user_variant_pattern_emits_keyword_hint() {
    // Pre-fix: detect_match_shape silently defaulted to Option<fresh>
    // when patterns didn't classify; the resulting "expected
    // Option<?>, got <enum>" misled users into thinking the
    // SCRUTINEE was wrong. Fix: when scrutinee/shape unify fails AND
    // the scrutinee is a user enum AND any arm pattern uses a bare-
    // symbol head matching one of that enum's variants, emit a
    // MalformedForm pointing the user at the keyword form.
    let errs = check_errors("tests/types/typed_if_match_bare_symbol_variant_bad.wat");
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        ":wat::kernel::ThreadDiedError::Panic",
    );
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        "bare-symbol",
    );
}
