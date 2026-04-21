//! End-to-end tests for `:wat::kernel::spawn` accepting lambda values.
//!
//! Per the 2026-04-20 relaxation, spawn's first argument may be either
//! a keyword-path literal (the classic named-define path) or any
//! expression that evaluates to a `:wat::core::lambda` value. Both
//! produce the same `Arc<Function>` under the hood; the trampoline
//! inside `apply_function` handles both (closed_env for lambdas,
//! fresh root for defines). This closes the asymmetry between
//! `:wat::kernel::spawn` and the existing `apply_value` / lambda-call
//! paths — same concept, same surface.
//!
//! Coverage:
//!
//! - Named define still spawns (no regression).
//! - Let-bound lambda spawns; `join` returns the lambda's result.
//! - Lambda-valued param spawned from inside a function body.
//! - Inline `(:wat::core::lambda ...)` spawned directly.
//! - Lambda's closed env survives the spawn (closure capture works).
//! - Non-callable first arg still errors cleanly.

use std::sync::Arc;
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

// ─── Named define baseline (no regression) ────────────────────────────

#[test]
fn spawn_named_define_still_works() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:app::produce (n :i64) -> :i64)
          (:wat::core::i64::+ n 1))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn :app::produce 41)))
            (:wat::kernel::join h)))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Let-bound lambda spawned ─────────────────────────────────────────

#[test]
fn spawn_let_bound_lambda() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((f :fn(i64)->i64)
              (:wat::core::lambda ((n :i64) -> :i64)
                (:wat::core::i64::+ n 1)))
             ((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn f 41)))
            (:wat::kernel::join h)))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Inline lambda literal spawned ────────────────────────────────────

#[test]
fn spawn_inline_lambda_literal() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::* n 2))
                21)))
            (:wat::kernel::join h)))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Lambda-valued param spawned ──────────────────────────────────────

#[test]
fn spawn_lambda_valued_param_from_enclosing_function() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:app::run-on-thread
                              (f :fn(i64)->i64)
                              (n :i64)
                              -> :i64)
          (:wat::core::let*
            (((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn f n)))
            (:wat::kernel::join h)))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((sq :fn(i64)->i64)
              (:wat::core::lambda ((n :i64) -> :i64)
                (:wat::core::i64::* n n))))
            (:app::run-on-thread sq 6)))
    "#;
    assert!(matches!(run(src), Value::i64(36)));
}

// ─── Closure capture survives spawn ───────────────────────────────────

#[test]
fn spawn_preserves_lambda_closed_env() {
    // The lambda captures `delta` from the enclosing let*. Spawning the
    // lambda on a new thread must carry its closed_env across the
    // boundary — otherwise `delta` is unbound on the worker thread.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((delta :i64) 100)
             ((add-delta :fn(i64)->i64)
              (:wat::core::lambda ((n :i64) -> :i64)
                (:wat::core::i64::+ n delta)))
             ((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn add-delta 23)))
            (:wat::kernel::join h)))
    "#;
    assert!(matches!(run(src), Value::i64(123)));
}

// ─── Non-callable first arg errors cleanly ────────────────────────────

#[test]
fn spawn_rejects_non_callable_value() {
    // 42 is neither a keyword path nor a lambda value. The spawn's
    // runtime dispatch catches it via TypeMismatch. The checker path
    // would also catch it (int's inferred type doesn't unify with
    // :fn(...)->_) — this test hits the runtime path through dynamic
    // evaluation.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((not-fn :i64) 42)
             ((h :wat::kernel::ProgramHandle<i64>)
              (:wat::kernel::spawn not-fn)))
            (:wat::kernel::join h)))
    "#;
    // This is a check-time error: let* declares `(h :ProgramHandle<i64>)`
    // but spawn can't derive a handle from an i64. The checker's
    // TypeMismatch arm fires.
    match startup(src) {
        Err(StartupError::Check(errs)) => {
            let hit = errs.0.iter().any(|e| {
                matches!(
                    e,
                    wat::check::CheckError::TypeMismatch { callee, .. } if callee.starts_with(":wat::kernel::spawn")
                )
            });
            assert!(hit, "expected spawn TypeMismatch; got {:?}", errs.0);
        }
        Err(other) => panic!("expected Check error; got {:?}", other),
        Ok(_) => panic!("expected check-time failure"),
    }
}
