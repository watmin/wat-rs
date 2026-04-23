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
//! - Declared `:T` is the form's inferred result (so a `let*`
//!   surrounding it sees `:T`, not "some branch type").
//! - Nested typed forms compose normally.

use std::sync::Arc;
use wat::check::CheckError;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

fn run(src: &str) -> Value {
    let world = startup(src).expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

fn check_errors(src: &str) -> Vec<CheckError> {
    match startup(src) {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {:?}", other),
        Ok(_) => panic!("expected Check errors; startup succeeded"),
    }
}

fn assert_malformed_mentioning(errs: &[CheckError], head: &str, needle: &str) {
    let hit = errs.iter().any(|e| match e {
        CheckError::MalformedForm { head: h, reason } => h == head && reason.contains(needle),
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
        CheckError::TypeMismatch {
            callee: c,
            param: p,
            ..
        } => c == callee && p == param,
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
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true -> :i64 11 22))
    "#;
    assert!(matches!(run(src), Value::i64(11)));
}

#[test]
fn typed_if_returns_else_branch_on_false() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if false -> :i64 11 22))
    "#;
    assert!(matches!(run(src), Value::i64(22)));
}

#[test]
fn typed_match_on_some_returns_some_arm() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 7) -> :i64
            ((Some v) v)
            (:None 0)))
    "#;
    assert!(matches!(run(src), Value::i64(7)));
}

#[test]
fn typed_match_on_none_returns_none_arm() {
    // Type-annotate the :None literal through a let-bound var so the
    // checker knows the scrutinee is Option<i64>.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((o :Option<i64>) :None))
            (:wat::core::match o -> :i64
              ((Some v) v)
              (:None -1))))
    "#;
    assert!(matches!(run(src), Value::i64(-1)));
}

// ─── Migration-hint refusals (old untyped shape) ──────────────────────

#[test]
fn untyped_if_gives_migration_hint() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true 1 2))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::if", "now requires `-> :T`");
}

#[test]
fn untyped_match_gives_migration_hint() {
    // Three args, where the second is NOT `->` — detected as the
    // old untyped shape.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 1)
            ((Some v) v)
            (:None 0)))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::match", "now requires `-> :T`");
}

// ─── Structural refusals ──────────────────────────────────────────────

#[test]
fn if_without_type_keyword_after_arrow_rejected() {
    // `-> :i64 then` is correct; this uses `-> then else without ty`.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true -> 1 2 3))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::if", "type keyword");
}

#[test]
fn match_without_type_keyword_after_arrow_rejected() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 1) -> oops ((Some v) v) (:None 0)))
    "#;
    let errs = check_errors(src);
    // `oops` is a bare symbol at args[2], not a keyword — triggers
    // the migration-hint path because args[1] isn't `->`... actually
    // args[1] IS `->`, args[2] is a Symbol "oops", not a Keyword.
    // The type-keyword guard catches that.
    assert_malformed_mentioning(&errs, ":wat::core::match", "type keyword");
}

#[test]
fn if_wrong_arity_rejected_with_shape_guidance() {
    // Six args — one too many.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true -> :i64 1 2 99))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::if", "expected (:wat::core::if cond -> :T then else)");
}

#[test]
fn match_too_few_args_rejected_with_shape_guidance() {
    // Scrutinee + `->` + type but no arm at all.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 1) -> :i64))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::match", "at least 4 args");
}

// ─── Branch-type-mismatch locality ────────────────────────────────────

#[test]
fn if_then_branch_type_mismatch_named_by_branch() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true -> :i64 "oops" 0))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::if", "then-branch");
}

#[test]
fn if_else_branch_type_mismatch_named_by_branch() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if true -> :i64 1 "oops"))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::if", "else-branch");
}

#[test]
fn match_arm_type_mismatch_named_by_arm_index() {
    // Arm #2 (the :None arm) produces a String instead of i64.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 7) -> :i64
            ((Some v) v)
            (:None "oops")))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::match", "arm #2");
}

// ─── Condition-type refusal on if ─────────────────────────────────────

#[test]
fn if_non_bool_cond_rejected_at_check() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::if 42 -> :i64 1 2))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::if", "cond");
}

// ─── Declared type is the form's result type ──────────────────────────

#[test]
fn typed_if_result_flows_into_enclosing_let_bind() {
    // The `let*` binding `x :i64` only unifies if infer_if reports
    // `:i64` as the if-form's result type — proving the declared `:T`
    // flows out.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((x :i64) (:wat::core::if true -> :i64 10 20)))
            x))
    "#;
    assert!(matches!(run(src), Value::i64(10)));
}

#[test]
fn typed_match_result_flows_into_enclosing_let_bind() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :String)
          (:wat::core::let*
            (((s :String)
              (:wat::core::match (Some 1) -> :String
                ((Some _) "yes")
                (:None "no"))))
            s))
    "#;
    match run(src) {
        Value::String(s) => assert_eq!(&*s, "yes"),
        other => panic!("expected \"yes\"; got {:?}", other),
    }
}

// ─── Nesting ──────────────────────────────────────────────────────────

#[test]
fn typed_if_inside_typed_match_arm_composes() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (Some 3) -> :i64
            ((Some v)
              (:wat::core::if (:wat::core::> v 0) -> :i64 v 0))
            (:None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(3)));
}
