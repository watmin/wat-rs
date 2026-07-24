//! End-to-end tests for the `:wat::core::if` and `:wat::core::match`
//! forms. Arc 258.4/258.5 — the `-> :T` ascription is RETIRED on both:
//! the result type is inferred by unifying the branches/arm bodies
//! (a stray `->` is a located migration-hint error). A divergent
//! body produces a per-body TypeMismatch naming the branch
//! (`then-branch`, `else-branch`, `arm #2`, ...).
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
use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("compute should run")
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
fn bare_match_is_valid_and_infers() {
    // Arc 258.5 — bare `(match scrut (pat body) ...)` is now VALID; the
    // result type is inferred by unifying the arm bodies (both :i64 → :i64).
    // This was previously a migration-hint rejection; it now runs and returns
    // the matching arm's value (Some 1 → v = 1).
    assert!(matches!(run("tests/types/typed_if_match_untyped_match_bare.wat"), Value::i64(1)));
}

// ─── Structural refusals ──────────────────────────────────────────────

#[test]
fn if_with_stray_arrow_rejected() {
    // Arc 258.4 — a stray `->` in ascription position (the retired 5-arg `-> :T`
    // form) is refused with the migration hint.
    let errs = check_errors("tests/types/typed_if_match_if_no_type_kw.wat.bad");
    assert_malformed_mentioning(&errs, ":wat::core::if", "no longer takes `-> :T`");
}

#[test]
fn match_with_stray_arrow_rejected() {
    // Arc 258.5 — a stray `->` in ascription position (the retired `-> :T`
    // shape) is refused with the migration hint, whatever follows the arrow.
    let errs = check_errors("tests/types/typed_if_match_match_no_type_kw.wat.bad");
    assert_malformed_mentioning(&errs, ":wat::core::match", "no longer takes `-> :T`");
}

#[test]
fn if_wrong_arity_rejected_with_shape_guidance() {
    // Arc 258.4 — bare 4-arg `if` (one too many); the error names the only valid
    // shape, `(:wat::core::if cond then else) — 3 args`.
    let errs = check_errors("tests/types/typed_if_match_if_wrong_arity.wat.bad");
    assert_malformed_mentioning(
        &errs,
        ":wat::core::if",
        include_str!("typed_if_match__if_wrong_arity_needle.wat"),
    );
}

#[test]
fn match_too_few_args_rejected_with_shape_guidance() {
    let errs = check_errors("tests/types/typed_if_match_match_too_few.wat.bad");
    assert_malformed_mentioning(&errs, ":wat::core::match", "at least a scrutinee and one arm");
}

// ─── Branch-type-mismatch locality ────────────────────────────────────

#[test]
fn if_branch_type_mismatch_named_by_branch() {
    // Arc 258.4 — bare `if` unifies the branches; a divergence is named on the
    // `else-branch` (the branch that fails to unify with the first), like bare
    // `match` names the divergent arm. (Was per-declared-type "then-branch".)
    let errs = check_errors("tests/types/typed_if_match_then_branch_mismatch.wat.bad");
    assert_type_mismatch_on(&errs, ":wat::core::if", "else-branch");
}

#[test]
fn if_else_branch_type_mismatch_named_by_branch() {
    let errs = check_errors("tests/types/typed_if_match_else_branch_mismatch.wat.bad");
    assert_type_mismatch_on(&errs, ":wat::core::if", "else-branch");
}

#[test]
fn match_arm_type_mismatch_named_by_arm_index() {
    // Arm #2 (the :None arm) produces a String instead of i64.
    let errs = check_errors("tests/types/typed_if_match_arm_type_mismatch.wat.bad");
    assert_type_mismatch_on(&errs, ":wat::core::match", "arm #2");
}

// ─── Condition-type refusal on if ─────────────────────────────────────

#[test]
fn if_non_bool_cond_rejected_at_check() {
    let errs = check_errors("tests/types/typed_if_match_non_bool_cond.wat.bad");
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
    let errs = check_errors("tests/types/typed_if_match_bare_symbol_variant.wat.bad");
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        ":wat::kernel::LociDiedError::Panic",
    );
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        "bare-symbol",
    );
}
