//! End-to-end tests for `:wat::core::typealias` expansion at
//! unification. Per the 2026-04-20 inscription, the type checker
//! walks an alias to its definition (substituting declared type
//! parameters) before the structural unify match — so `:MyAlias<K,V>`
//! and its expansion are interchangeable in every signature.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::check::CheckError;
use wat::freeze::{eval_in_frozen, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup(&src).expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
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

        (:wat::core::typealias :my::Amount :wat::core::f64)

        (:wat::core::define (:app::double (x :my::Amount) -> :my::Amount)
          (:wat::core::f64::*'2 x 2.0))

        (:wat::core::define (:my::compute -> :wat::core::f64)
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

        (:wat::core::typealias :my::B :wat::core::f64)
        (:wat::core::typealias :my::A :my::B)

        (:wat::core::define (:app::inc (x :my::A) -> :my::A)
          (:wat::core::f64::+'2 x 1.0))

        (:wat::core::define (:my::compute -> :wat::core::f64)
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
    // Bad code in :my::probe; canonical nil main appended.
    let src = r#"

        (:wat::core::typealias :my::Amount :wat::core::f64)

        (:wat::core::define (:app::double (x :my::Amount) -> :my::Amount)
          (:wat::core::f64::*'2 x 2.0))

        (:wat::core::define (:my::probe -> :my::Amount)
          (:app::double "not a number"))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let errs = check_errors(src);
    let hit = errs.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. }));
    assert!(hit, "expected TypeMismatch; got {:?}", errs);
}

// ─── Alias at shape-inspection sites (post-reduce) ────────────────────

#[test]
fn tuple_alias_works_at_hashmap_constructor_arg() {
    // `:my::KV` aliases the K,V tuple `:(wat::core::String,wat::core::i64)`. The HashMap
    // constructor's first-arg check expands aliases before its
    // Tuple-shape match, so `(:wat::core::HashMap :my::KV ...)` is
    // accepted exactly as if the literal `:(wat::core::String,wat::core::i64)` were
    // written. Mirrors `:wat::core::Bytes ≡ :wat::core::Vector<wat::core::u8>` resolving
    // structurally at call sites.
    let src = r#"
        (:wat::core::typealias :my::KV :(wat::core::String,wat::core::i64))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [row
              (:wat::core::HashMap :my::KV "a" 1 "b" 2)
             got (:wat::core::get row "b")]
            (:wat::core::match got -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert!(matches!(run(src), Value::i64(2)));
}

#[test]
fn alias_over_hashmap_passes_through_std_get() {
    // `:my::Row` aliases HashMap<String,i64>. `:wat::core::get` inspects
    // its container argument's shape (HashMap / HashSet). With alias
    // reduction at the shape-inspection site, the alias resolves to
    // its HashMap root and the call type-checks.
    let src = r#"

        (:wat::core::typealias :my::Row :wat::core::HashMap<wat::core::String,wat::core::i64>)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [row (:wat::core::HashMap :(wat::core::String,wat::core::i64) "a" 10 "b" 20)
             got (:wat::core::get row "a")]
            (:wat::core::match got -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert!(matches!(run(src), Value::i64(10)));
}

#[test]
fn alias_over_fn_type_works_at_spawn() {
    // `:my::Job` aliases :wat::core::Fn(Sender<wat::core::i64>)->:(). The spawn-extract-Fn
    // site at infer_spawn must see through the alias to find the Fn
    // shape.
    let src = r#"

        (:wat::core::typealias
          :my::Job
          :wat::core::Fn(rust::crossbeam_channel::Sender<wat::core::i64>)->wat::core::nil)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [job
              (:wat::core::fn [tx <- :rust::crossbeam_channel::Sender<wat::core::i64>] -> :wat::core::nil
                (:wat::core::do
                  (:wat::core::Result/expect -> :wat::core::nil (:wat::kernel::send tx 7) "test producer: tx disconnected")
                  ()))
             pair
              (:wat::kernel::make-bounded-channel :wat::core::i64 1)
             tx (:wat::core::first pair)
             rx (:wat::core::second pair)
             h
              (:wat::kernel::spawn-thread
                (:wat::core::fn
                  [_in <- :rust::crossbeam_channel::Receiver<wat::core::nil>
                   _out <- :rust::crossbeam_channel::Sender<wat::core::nil>]
                   -> :wat::core::nil
                  (job tx)))
             _
              (:wat::kernel::Thread/drain-and-join h)]
            (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::i64
              ((:wat::core::Ok (:wat::core::Some v)) v)
              ((:wat::core::Ok :wat::core::None) 0)
              ((:wat::core::Err _died) -1))))
    "#;
    assert!(matches!(run(src), Value::i64(7)));
}

// ─── Alias in return position unifies with its expansion ──────────────

#[test]
fn alias_return_type_accepts_expanded_literal() {
    let src = r#"

        (:wat::core::typealias :my::Amount :wat::core::f64)

        (:wat::core::define (:app::zero -> :my::Amount)
          0.0)

        (:wat::core::define (:my::compute -> :wat::core::f64)
          (:app::zero))
    "#;
    match run(src) {
        Value::f64(n) => assert_eq!(n, 0.0),
        other => panic!("expected f64 0.0; got {:?}", other),
    }
}
