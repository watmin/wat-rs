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
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if true -> :wat::core::i64 11 22))
    "#;
    assert!(matches!(run(src), Value::i64(11)));
}

#[test]
fn typed_if_returns_else_branch_on_false() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if false -> :wat::core::i64 11 22))
    "#;
    assert!(matches!(run(src), Value::i64(22)));
}

#[test]
fn typed_match_on_some_returns_some_arm() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 7) -> :wat::core::i64
            ((:wat::core::Some v) v)
            (:wat::core::None 0)))
    "#;
    assert!(matches!(run(src), Value::i64(7)));
}

#[test]
fn typed_match_on_none_returns_none_arm() {
    // Type-annotate the :None literal through a let-bound var so the
    // checker knows the scrutinee is Option<i64>.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((o :wat::core::Option<wat::core::i64>) :wat::core::None))
            (:wat::core::match o -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert!(matches!(run(src), Value::i64(-1)));
}

// ─── Migration-hint refusals (old untyped shape) ──────────────────────

#[test]
fn untyped_if_gives_migration_hint() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
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
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 1)
            ((:wat::core::Some v) v)
            (:wat::core::None 0)))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::match", "now requires `-> :T`");
}

// ─── Structural refusals ──────────────────────────────────────────────

#[test]
fn if_without_type_keyword_after_arrow_rejected() {
    // `-> :wat::core::i64 then` is correct; this uses `-> then else without ty`.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if true -> 1 2 3))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::if", "type keyword");
}

#[test]
fn match_without_type_keyword_after_arrow_rejected() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 1) -> oops ((:wat::core::Some v) v) (:wat::core::None 0)))
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
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if true -> :wat::core::i64 1 2 99))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::if", "expected (:wat::core::if cond -> :T then else)");
}

#[test]
fn match_too_few_args_rejected_with_shape_guidance() {
    // Scrutinee + `->` + type but no arm at all.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 1) -> :wat::core::i64))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(&errs, ":wat::core::match", "at least 4 args");
}

// ─── Branch-type-mismatch locality ────────────────────────────────────

#[test]
fn if_then_branch_type_mismatch_named_by_branch() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if true -> :wat::core::i64 "oops" 0))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::if", "then-branch");
}

#[test]
fn if_else_branch_type_mismatch_named_by_branch() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if true -> :wat::core::i64 1 "oops"))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::if", "else-branch");
}

#[test]
fn match_arm_type_mismatch_named_by_arm_index() {
    // Arm #2 (the :None arm) produces a String instead of i64.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 7) -> :wat::core::i64
            ((:wat::core::Some v) v)
            (:wat::core::None "oops")))
    "#;
    let errs = check_errors(src);
    assert_type_mismatch_on(&errs, ":wat::core::match", "arm #2");
}

// ─── Condition-type refusal on if ─────────────────────────────────────

#[test]
fn if_non_bool_cond_rejected_at_check() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::if 42 -> :wat::core::i64 1 2))
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
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((x :wat::core::i64) (:wat::core::if true -> :wat::core::i64 10 20)))
            x))
    "#;
    assert!(matches!(run(src), Value::i64(10)));
}

#[test]
fn typed_match_result_flows_into_enclosing_let_bind() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::String)
          (:wat::core::let*
            (((s :wat::core::String)
              (:wat::core::match (:wat::core::Some 1) -> :wat::core::String
                ((:wat::core::Some _) "yes")
                (:wat::core::None "no"))))
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
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:wat::core::Some 3) -> :wat::core::i64
            ((:wat::core::Some v)
              (:wat::core::if (:wat::core::> v 0) -> :wat::core::i64 v 0))
            (:wat::core::None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(3)));
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
    //
    // Wat-rs convention: built-in `Some` / `Ok` / `Err` use bare
    // symbols; user-enum variants must be qualified with the enum's
    // keyword path (`:wat::kernel::ThreadDiedError::Panic`, not
    // `Panic`). Disambiguation discipline — two enums could both
    // declare `Panic`; the keyword path resolves the namespace.
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::String)
          (:wat::core::let*
            (((handle :wat::kernel::Thread<(),()>)
              (:wat::kernel::spawn-thread
                (:wat::core::lambda
                  ((_in :rust::crossbeam_channel::Receiver<()>)
                   (_out :rust::crossbeam_channel::Sender<()>)
                   -> :())
                  ())))
             ((result :wat::core::Result<wat::core::unit,Vec<wat::kernel::ThreadDiedError>>)
              (:wat::kernel::Thread/join-result handle))
             ((chain :wat::core::Vector<wat::kernel::ThreadDiedError>)
              (:wat::core::match result -> :wat::core::Vector<wat::kernel::ThreadDiedError>
                ((:wat::core::Ok _)   (:wat::core::panic! "test wants Err"))
                ((:wat::core::Err e)  e)))
             ((err :wat::kernel::ThreadDiedError)
              (:wat::core::match (:wat::core::first chain) -> :wat::kernel::ThreadDiedError
                ((:wat::core::Some e) e)
                (:wat::core::None    (:wat::core::panic! "expected non-empty chain")))))
            ;; The bug-trigger pattern: bare-symbol `Panic` head
            ;; against ThreadDiedError. Pre-fix produced
            ;; "expected Option<?>"; post-fix produces a hint
            ;; pointing at :wat::kernel::ThreadDiedError::Panic.
            (:wat::core::match err -> :wat::core::String
              ((Panic m)        m)
              ((RuntimeError m) m)
              (:wat::kernel::ThreadDiedError::ChannelDisconnected "disc"))))
    "#;
    let errs = check_errors(src);
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        ":wat::kernel::ThreadDiedError::Panic",
    );
    // The hint should also explain WHY (bare-symbol heads are reserved).
    assert_malformed_mentioning(
        &errs,
        ":wat::core::match",
        "bare-symbol",
    );
}
