//! FM 2-bis probe — arc 237 Stone S-A: the is-a hierarchy mechanism.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A-records-hierarchy.md`.
//! Mints the `typesub` child→parent edge-registry on `TypeEnv` + `is_subtype`
//! (directional / transitive / reflexive walk) + the `:wat::core::subtype?` wat
//! primitive + two built-in roots (`:wat::holon::Record typesub :wat::core::Record`).
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
//!   6.  built-in roots — is_subtype(:wat::holon::Record, :wat::core::Record) true; reverse false
//!
//!   wat-surface (`:wat::core::subtype?`, validation at the surface):
//!   7.  (subtype? :wat::holon::Record :wat::core::Record) → true
//!   8.  (subtype? :wat::core::Record :wat::holon::Record) → false   (directional)
//!   9.  (subtype? :wat::core::i64 :wat::core::f64) → false     (unrelated leaves)
//!   10. (subtype? :my::Nonexistent :wat::core::Record) → Err         (unknown name; mirror conforms?)
//!
//! Initial state: file FAILS to compile — `register_subtype` / `is_subtype` /
//! `:wat::core::subtype?` do not exist. Post-stone S-A: 10/10 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites
//! this file verbatim as "the working contract sonnet must satisfy."

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeErrorKind, Value};
use wat::types::{is_subtype, TypeEnv, TypeErrorKind};

// ─── Rust-API helpers (mirror probe_arc237_stone1_typeunion_substrate) ────────

fn fresh_env() -> TypeEnv {
    TypeEnv::new()
}

// ─── Probe 1 — edge + directional ─────────────────────────────────────────────
#[test]
fn probe_01_edge_directional() {
    let mut env = fresh_env();
    env.register_subtype(":my::Child", ":my::Parent", wat::rust_caller_span!())
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
    env.register_subtype(":my::A", ":my::B", wat::rust_caller_span!()).expect("A→B");
    env.register_subtype(":my::B", ":my::C", wat::rust_caller_span!()).expect("B→C");
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
    env.register_subtype(":my::A", ":my::B", wat::rust_caller_span!()).expect("A→B ok");
    let closes_cycle = env.register_subtype(":my::B", ":my::A", wat::rust_caller_span!());
    assert!(
        matches!(&closes_cycle, Err(e) if matches!(e.kind(), TypeErrorKind::CyclicSubtype { child, parent }
            if child == ":my::B" && parent == ":my::A")),
        "B→A closes a cycle through A→B; must be rejected at registration; got {:?}",
        closes_cycle
    );
}

// ─── Probe 6 — built-in roots: holon::Record is-a Record ──────────────────────
#[test]
fn probe_06_builtin_roots() {
    let env = TypeEnv::with_builtins();
    assert!(
        is_subtype(":wat::holon::Record", ":wat::core::Record", &env),
        "seeded root edge: :wat::holon::Record typesub :wat::core::Record"
    );
    assert!(
        !is_subtype(":wat::core::Record", ":wat::holon::Record", &env),
        "directional: base is NOT-a holonic"
    );
}

// ─── wat-surface helper ────────────────────────────────────────────────────────

fn eval_probe(file: &str, fn_name: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(file)?;
    // Arc 296 Stone M: "no entry fn" is a fixture/test-authorship bug, not a startup-pipeline
    // failure with a StartupError variant to wrap into — mirrors `call_beside_value`'s own
    // `.unwrap_or_else(|| panic!(...))` for the identical condition (src/freeze.rs).
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no entry fn {fn_name:?} in {file:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(StartupError::from)
}

// ─── Probe 7 — subtype? roots → true ──────────────────────────────────────────
#[test]
fn probe_07_subtype_roots_true() {
    match eval_probe("tests/types/probe_arc237_sA_hierarchy_probe07.wat", ":user::probe07") {
        Ok(Value::bool(true)) => {}
        other => panic!("expected true; got {:?}", other),
    }
}

// ─── Probe 8 — subtype? directional → false ───────────────────────────────────
#[test]
fn probe_08_subtype_directional_false() {
    match eval_probe("tests/types/probe_arc237_sA_hierarchy_probe08.wat", ":user::probe08") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false (directional); got {:?}", other),
    }
}

// ─── Probe 9 — subtype? unrelated leaves → false ──────────────────────────────
#[test]
fn probe_09_subtype_unrelated_false() {
    match eval_probe("tests/types/probe_arc237_sA_hierarchy_probe09.wat", ":user::probe09") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false (unrelated); got {:?}", other),
    }
}

// ─── Probe 10 — subtype? unknown name → Err (mirror conforms? contract) ───────
#[test]
fn probe_10_subtype_unknown_name_errors() {
    // Error may fire at startup (type check) or at eval (runtime); either satisfies the contract.
    match startup_from_file("tests/types/probe_arc237_sA_hierarchy_probe10.wat") {
        Err(_) => {} // startup error satisfies the contract
        Ok(world) => {
            let func = world.symbols().get(":user::probe10").expect(":user::probe10").clone();
            let r = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!());
            assert!(
                matches!(&r, Err(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { head, reason }
                    if head == ":wat::core::subtype?"
                    && reason == "unknown type name ':my::Nonexistent' is not registered in the \
                                   TypeEnv and is not a built-in primitive; cannot determine \
                                   subtype relationship (this is bad input, not a negative result \
                                   — check the spelling and ensure the type is declared before use)")),
                "unknown type name must error at startup or eval (keeps `false` honest); got {:?}",
                r
            );
        }
    }
}
