//! FM 2-bis probe — arc 237 Stone S-A: the is-a hierarchy mechanism.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A-records-hierarchy.md`.
//! Mints the `typesub` child→parent edge-registry on `TypeEnv` + `is_subtype`
//! (directional / transitive / reflexive walk) + the `:wat::core::subtype?` wat
//! primitive + two built-in roots (`:wat::holon::Record typesub :wat::Record`).
//!
//! This is Clojure's `derive`/`isa?` hierarchy axis — NOT typeunion (closed
//! symmetric sum) and NOT defprotocol (behavior). `is_subtype` walks the NEW
//! typesub registry, NOT `collect_union_members` (the one place precedent would
//! mislead — see the sub-DESIGN's proven-moves section).
//!
//! Scope: mechanism only. NO `unify`-site edits (that is S-A1: the `assignable`
//! choke point). NO `conforms?` change (that is S-B: it needs a subtype-typed
//! VALUE to exercise, which doesn't exist until records derive edges). So every
//! contract here is provable in S-A's own scope.
//!
//! Probe contracts (10):
//!   Rust-API (the `is_subtype` engine + `register_subtype`):
//!   1.  edge + directional — A typesub B ⇒ is_subtype(A,B) true; is_subtype(B,A) false
//!   2.  transitive — A→B, B→C ⇒ is_subtype(A,C) true
//!   3.  reflexive — is_subtype(X,X) true (no edges needed)
//!   4.  leaf-safe — is_subtype(:bool,:i64) false (types with no edges)
//!   5.  cycle rejected — register_subtype closing a cycle → Err
//!   6.  built-in roots — is_subtype(:wat::holon::Record, :wat::Record) true; reverse false
//!   wat-surface (`:wat::core::subtype?`, validation at the surface):
//!   7.  (subtype? :wat::holon::Record :wat::Record) → true
//!   8.  (subtype? :wat::Record :wat::holon::Record) → false   (directional)
//!   9.  (subtype? :wat::core::i64 :wat::core::f64) → false     (unrelated leaves)
//!   10. (subtype? :my::Nonexistent :wat::Record) → Err         (unknown name; mirror conforms?)
//!
//! Initial state: file FAILS to compile — `register_subtype` / `is_subtype` /
//! `:wat::core::subtype?` do not exist. Post-stone S-A: 10/10 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites
//! this file verbatim as "the working contract sonnet must satisfy."

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use wat::types::{is_subtype, TypeEnv};

// ─── Rust-API helpers (mirror probe_arc237_stone1_typeunion_substrate) ────────

fn fresh_env() -> TypeEnv {
    TypeEnv::new()
}

// ─── Probe 1 — edge + directional ─────────────────────────────────────────────
#[test]
fn probe_01_edge_directional() {
    let mut env = fresh_env();
    env.register_subtype(":my::Child", ":my::Parent")
        .expect("register_subtype Child→Parent");
    assert!(is_subtype(":my::Child", ":my::Parent", &env), "Child is-a Parent");
    assert!(
        !is_subtype(":my::Parent", ":my::Child", &env),
        "Parent is NOT-a Child (directional)"
    );
}

// ─── Probe 2 — transitive ─────────────────────────────────────────────────────
#[test]
fn probe_02_transitive() {
    let mut env = fresh_env();
    env.register_subtype(":my::A", ":my::B").expect("A→B");
    env.register_subtype(":my::B", ":my::C").expect("B→C");
    assert!(is_subtype(":my::A", ":my::C", &env), "A is-a C transitively");
}

// ─── Probe 3 — reflexive ──────────────────────────────────────────────────────
#[test]
fn probe_03_reflexive() {
    let env = fresh_env();
    assert!(
        is_subtype(":my::X", ":my::X", &env),
        "X is-a X (reflexive; no edge needed)"
    );
}

// ─── Probe 4 — leaf-safe (no edges → false, not error) ────────────────────────
#[test]
fn probe_04_leaf_safe() {
    let env = fresh_env();
    assert!(
        !is_subtype(":wat::core::bool", ":wat::core::i64", &env),
        "leaf types with no edges → false"
    );
}

// ─── Probe 5 — cycle rejected at registration ─────────────────────────────────
#[test]
fn probe_05_cycle_rejected() {
    let mut env = fresh_env();
    env.register_subtype(":my::A", ":my::B").expect("A→B ok");
    let closes_cycle = env.register_subtype(":my::B", ":my::A");
    assert!(
        closes_cycle.is_err(),
        "B→A closes a cycle through A→B; must be rejected at registration"
    );
}

// ─── Probe 6 — built-in roots: holon::Record is-a Record ──────────────────────
#[test]
fn probe_06_builtin_roots() {
    let env = TypeEnv::with_builtins();
    assert!(
        is_subtype(":wat::holon::Record", ":wat::Record", &env),
        "seeded root edge: :wat::holon::Record typesub :wat::Record"
    );
    assert!(
        !is_subtype(":wat::Record", ":wat::holon::Record", &env),
        "directional: base is NOT-a holonic"
    );
}

// ─── wat-surface helper (mirror probe_arc237_stone5_conforms run_bool) ────────

/// Evaluate `(:wat::core::subtype? a b)` and return its Value (`Value::bool`) or Err.
fn subtype_q(a: &str, b: &str) -> Result<Value, String> {
    let full = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::bool
          \
                     (:wat::core::subtype? {a} {b}))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        a = a,
        b = b
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 7 — subtype? roots → true ──────────────────────────────────────────
#[test]
fn probe_07_subtype_roots_true() {
    match subtype_q(":wat::holon::Record", ":wat::Record") {
        Ok(Value::bool(true)) => {}
        other => panic!("expected true; got {:?}", other),
    }
}

// ─── Probe 8 — subtype? directional → false ───────────────────────────────────
#[test]
fn probe_08_subtype_directional_false() {
    match subtype_q(":wat::Record", ":wat::holon::Record") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false (directional); got {:?}", other),
    }
}

// ─── Probe 9 — subtype? unrelated leaves → false ──────────────────────────────
#[test]
fn probe_09_subtype_unrelated_false() {
    match subtype_q(":wat::core::i64", ":wat::core::f64") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false (unrelated); got {:?}", other),
    }
}

// ─── Probe 10 — subtype? unknown name → Err (mirror conforms? contract) ───────
#[test]
fn probe_10_subtype_unknown_name_errors() {
    let r = subtype_q(":my::Nonexistent", ":wat::Record");
    assert!(
        r.is_err(),
        "unknown type name is bad input → Err, not false (keeps `false` honest); got {:?}",
        r
    );
}
