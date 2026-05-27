//! FM 2-bis probe — arc 237 Stone S-B.1: `:wat::core::recordtype` + `TypeDef::Record`.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-B1-recordtype.md`.
//! Mints the substrate type-declaration form that makes a record class a real
//! `TypeDef::Record` — so it inherits, autonomously, the type system's uniform
//! services: ∀T `is-<Name>?` synthesis (via `register_type_predicates`) + `typesub`
//! hierarchy membership (the parent edge wired at registration). The `defrecord`
//! macro will EMIT this form (S-B.2); it does NOT mint the predicate itself.
//!
//! Drives `startup_from_source` with the `recordtype` form DIRECTLY (it is a real
//! surface primitive — no defrecord macro, no record VALUE needed for these
//! contracts). The is-X? TRUE-path (a real `:my::Circle` value answering true) is
//! S-B.2 (needs the macro's constructor); noted, not asserted here.
//!
//! Three-tier doctrine (intueri-locked): `is-X?` is tier-1 EXACT; `conforms?` is
//! tier-3 (untouched, no parent-walk); lineage is the separate `subtype-of?` stone.
//! B.1 adds only a NOMINAL `conforms?` Record arm (not exercised here — needs a value).
//!
//! Probe contracts (6):
//!   1. recordtype form registers — `(:wat::core::recordtype :my::Circle :wat::Record)`
//!      at top level → startup succeeds.
//!   2. is-X? synthesized ∀T (THE asymmetry-killer) — `(:my::is-Circle? 42)` → `false`,
//!      NOT a type error. (Pre-stone, records' hand-emitted predicate type-errored here.)
//!   3. edge wired by recordtype — `(subtype? :my::Circle :wat::Record)` → `true`.
//!   4. directional — `(subtype? :wat::Record :my::Circle)` → `false`.
//!   5. holon-flavor parent + transitive — `(recordtype :my::Sphere :wat::holon::Record)`;
//!      `(subtype? :my::Sphere :wat::Record)` → `true` (Sphere→holon::Record→Record).
//!   6. unknown parent rejected — `(recordtype :my::Bad :my::DoesNotExist)` → startup error.
//!
//! Initial state: FAILS — `:wat::core::recordtype` is an unknown form / `TypeDef::Record`
//! doesn't exist. Post-stone S-B.1: 6/6 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites this file.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Top-level record-type declarations shared across contracts.
const PRELUDE: &str = r#"
(:wat::core::recordtype :my::Circle :wat::Record)
(:wat::core::recordtype :my::Sphere :wat::holon::Record)
"#;

/// Build `PRELUDE + (:user::compute -> :bool <expr>) + main`, evaluate
/// `(:user::compute)`, return its Value (`Value::bool`) or an Err string.
fn run_bool(compute_expr: &str) -> Result<Value, String> {
    let full = format!(
        "{prelude}\n\
         (:wat::core::define (:user::compute -> :wat::core::bool) {expr})\n\
         (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        prelude = PRELUDE,
        expr = compute_expr
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn assert_true(expr: &str) {
    match run_bool(expr) {
        Ok(Value::bool(true)) => {}
        other => panic!("expected true for `{}`; got {:?}", expr, other),
    }
}

fn assert_false(expr: &str) {
    match run_bool(expr) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false for `{}`; got {:?}", expr, other),
    }
}

// ─── Probe 1: recordtype form registers (startup succeeds with the prelude) ───
#[test]
fn probe_01_recordtype_registers() {
    // If the form parses + registers, this trivial true compiles + runs.
    assert_true("(:wat::core::= 1 1)");
}

// ─── Probe 2: is-X? synthesized ∀T — THE asymmetry-killer ─────────────────────
#[test]
fn probe_02_is_predicate_synthesized_forall_t() {
    // A non-record value → false, NOT a type error. Pre-stone this path errored
    // (the macro's hand-emitted predicate narrowed `[v <- :wat::Record]`).
    assert_false("(:my::is-Circle? 42)");
}

// ─── Probe 3: edge wired by recordtype (Circle is-a Record) ───────────────────
#[test]
fn probe_03_edge_wired() {
    assert_true("(:wat::core::subtype? :my::Circle :wat::Record)");
}

// ─── Probe 4: directional (Record is NOT-a Circle) ────────────────────────────
#[test]
fn probe_04_directional() {
    assert_false("(:wat::core::subtype? :wat::Record :my::Circle)");
}

// ─── Probe 5: holon-flavor parent + transitive (Sphere→holon::Record→Record) ──
#[test]
fn probe_05_holon_flavor_transitive() {
    assert_true("(:wat::core::subtype? :my::Sphere :wat::Record)");
    assert_true("(:wat::core::subtype? :my::Sphere :wat::holon::Record)");
}

// ─── Probe 6: unknown parent rejected at registration ─────────────────────────
#[test]
fn probe_06_unknown_parent_rejected() {
    let src = r#"
        (:wat::core::recordtype :my::Bad :my::DoesNotExist)
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        r.is_err(),
        "recordtype with an unknown parent must be rejected at registration; got Ok"
    );
}
