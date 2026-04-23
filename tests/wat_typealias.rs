//! End-to-end tests for `:wat::core::typealias` expansion at
//! unification. Per the 2026-04-20 inscription, the type checker
//! walks an alias to its definition (substituting declared type
//! parameters) before the structural unify match — so `:MyAlias<K,V>`
//! and its expansion are interchangeable in every signature.

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

// ─── Simple non-parametric alias ──────────────────────────────────────

#[test]
fn simple_alias_unifies_with_its_expansion() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::Amount :f64)

        (:wat::core::define (:app::double (x :my::Amount) -> :my::Amount)
          (:wat::core::f64::* x 2.0))

        (:wat::core::define (:user::main -> :f64)
          (:app::double 21.0))
    "#;
    match run(src) {
        Value::f64(n) => assert!((n - 42.0).abs() < 1e-9),
        other => panic!("expected f64 42.0; got {:?}", other),
    }
}

// ─── Alias-of-alias chain ─────────────────────────────────────────────

#[test]
fn alias_of_alias_chain_expands_to_root() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::B :f64)
        (:wat::core::typealias :my::A :my::B)

        (:wat::core::define (:app::inc (x :my::A) -> :my::A)
          (:wat::core::f64::+ x 1.0))

        (:wat::core::define (:user::main -> :f64)
          (:app::inc 41.0))
    "#;
    match run(src) {
        Value::f64(n) => assert!((n - 42.0).abs() < 1e-9),
        other => panic!("expected f64 42.0; got {:?}", other),
    }
}

// ─── Cycle refusal at registration ────────────────────────────────────

#[test]
fn cyclic_alias_halts_at_startup() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::A :my::B)
        (:wat::core::typealias :my::B :my::A)
    "#;
    match startup(src) {
        Err(StartupError::Type(_)) => {}
        Err(other) => panic!("expected Type error (cyclic alias); got {:?}", other),
        Ok(_) => panic!("expected startup to fail due to cyclic alias"),
    }
}

#[test]
fn self_referential_alias_halts_at_startup() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::A :my::A)
    "#;
    match startup(src) {
        Err(StartupError::Type(_)) => {}
        Err(other) => panic!("expected Type error; got {:?}", other),
        Ok(_) => panic!("expected self-referential alias to halt startup"),
    }
}

// ─── Alias does not hide type errors ──────────────────────────────────

#[test]
fn alias_preserves_type_mismatches() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::Amount :f64)

        (:wat::core::define (:app::double (x :my::Amount) -> :my::Amount)
          (:wat::core::f64::* x 2.0))

        (:wat::core::define (:user::main -> :my::Amount)
          (:app::double "not a number"))
    "#;
    let errs = check_errors(src);
    let hit = errs.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. }));
    assert!(hit, "expected TypeMismatch; got {:?}", errs);
}

// ─── Alias at shape-inspection sites (post-reduce) ────────────────────

#[test]
fn alias_over_hashmap_passes_through_std_get() {
    // `:my::Row` aliases HashMap<String,i64>. `:wat::core::get` inspects
    // its container argument's shape (HashMap / HashSet). With alias
    // reduction at the shape-inspection site, the alias resolves to
    // its HashMap root and the call type-checks.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::Row :HashMap<String,i64>)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((row :my::Row) (:wat::core::HashMap :(String,i64) "a" 10 "b" 20))
             ((got :Option<i64>) (:wat::core::get row "a")))
            (:wat::core::match got -> :i64
              ((Some v) v)
              (:None -1))))
    "#;
    assert!(matches!(run(src), Value::i64(10)));
}

#[test]
fn alias_over_fn_type_works_at_spawn() {
    // `:my::Job` aliases :fn(Sender<i64>)->:(). The spawn-extract-Fn
    // site at infer_spawn must see through the alias to find the Fn
    // shape.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias
          :my::Job
          :fn(rust::crossbeam_channel::Sender<i64>)->())

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((job :my::Job)
              (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                (:wat::core::let*
                  (((_ :Option<()>) (:wat::kernel::send tx 7)))
                  ())))
             ((pair :(rust::crossbeam_channel::Sender<i64>,rust::crossbeam_channel::Receiver<i64>))
              (:wat::kernel::make-bounded-queue :i64 1))
             ((tx :rust::crossbeam_channel::Sender<i64>) (:wat::core::first pair))
             ((rx :rust::crossbeam_channel::Receiver<i64>) (:wat::core::second pair))
             ((h :wat::kernel::ProgramHandle<()>) (:wat::kernel::spawn job tx))
             ((_ :()) (:wat::kernel::join h)))
            (:wat::core::match (:wat::kernel::recv rx) -> :i64
              ((Some v) v)
              (:None 0))))
    "#;
    assert!(matches!(run(src), Value::i64(7)));
}

// ─── Alias in return position unifies with its expansion ──────────────

#[test]
fn alias_return_type_accepts_expanded_literal() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::typealias :my::Amount :f64)

        (:wat::core::define (:app::zero -> :my::Amount)
          0.0)

        (:wat::core::define (:user::main -> :f64)
          (:app::zero))
    "#;
    match run(src) {
        Value::f64(n) => assert_eq!(n, 0.0),
        other => panic!("expected f64 0.0; got {:?}", other),
    }
}
