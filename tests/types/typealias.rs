//! End-to-end tests for `:wat::core::typealias` expansion at
//! unification. Per the 2026-04-20 inscription, the type checker
//! walks an alias to its definition (substituting declared type
//! parameters) before the structural unify match — so `:MyAlias<K,V>`
//! and its expansion are interchangeable in every signature.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
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

// ─── Simple non-parametric alias ──────────────────────────────────────

#[test]
fn simple_alias_unifies_with_its_expansion() {
    match run("tests/types/typealias_simple_alias.wat") {
        Value::f64(n) => assert!((n - 42.0).abs() < 1e-9),
        other => panic!("expected f64 42.0; got {:?}", other),
    }
}

// ─── Alias-of-alias chain ─────────────────────────────────────────────

#[test]
fn alias_of_alias_chain_expands_to_root() {
    match run("tests/types/typealias_alias_chain.wat") {
        Value::f64(n) => assert!((n - 42.0).abs() < 1e-9),
        other => panic!("expected f64 42.0; got {:?}", other),
    }
}

// ─── Cycle refusal at registration ────────────────────────────────────

#[test]
fn cyclic_alias_halts_at_startup() {
    match startup_from_file("tests/types/typealias_cyclic_bad.wat") {
        Err(StartupError::Type(_)) => {}
        Err(other) => panic!("expected Type error (cyclic alias); got {:?}", other),
        Ok(_) => panic!("expected startup to fail due to cyclic alias"),
    }
}

#[test]
fn self_referential_alias_halts_at_startup() {
    match startup_from_file("tests/types/typealias_self_ref_bad.wat") {
        Err(StartupError::Type(_)) => {}
        Err(other) => panic!("expected Type error; got {:?}", other),
        Ok(_) => panic!("expected self-referential alias to halt startup"),
    }
}

// ─── Alias does not hide type errors ──────────────────────────────────

#[test]
fn alias_preserves_type_mismatches() {
    let errs = check_errors("tests/types/typealias_preserves_type_mismatches_bad.wat");
    let hit = errs.iter().any(|e| matches!(e, CheckError { kind: CheckErrorKind::TypeMismatch { .. }, .. }));
    assert!(hit, "expected TypeMismatch; got {:?}", errs);
}

// ─── Alias at shape-inspection sites (post-reduce) ────────────────────

#[test]
fn type_alias_works_at_hashmap_k_and_v_args() {
    assert!(matches!(run("tests/types/typealias_hashmap_args.wat"), Value::i64(2)));
}

#[test]
fn alias_over_hashmap_passes_through_std_get() {
    assert!(matches!(run("tests/types/typealias_hashmap_std_get.wat"), Value::i64(10)));
}

#[test]
fn alias_over_fn_type_works_at_spawn() {
    assert!(matches!(run("tests/types/typealias_fn_type_spawn.wat"), Value::i64(7)));
}

// ─── Alias in return position unifies with its expansion ──────────────

#[test]
fn alias_return_type_accepts_expanded_literal() {
    match run("tests/types/typealias_return_type.wat") {
        Value::f64(n) => assert_eq!(n, 0.0),
        other => panic!("expected f64 0.0; got {:?}", other),
    }
}
