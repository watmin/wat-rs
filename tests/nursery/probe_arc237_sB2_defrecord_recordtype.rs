//! FM 2-bis probe — arc 237 Stone S-B.2: defrecord emits `recordtype` + drops its predicate.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-B2-defrecord-emits-recordtype.md`.
//! Rewires the `:wat::core::defrecord` macro (wat/Record.wat) to (1) emit
//! `(:wat::core::recordtype ~fqdn :wat::Record)` so the class becomes a real
//! `TypeDef::Record` (B.1), and (2) DROP its hand-rolled `is-<Name>?` predicate so
//! the type system's `register_type_predicates` synthesizes it ∀T autonomously.
//! Constructor return stays `-> :wat::Record` (per-class return is the S-A1 pairing).
//!
//! This brings the asymmetry-kill to the EVERYDAY surface, and proves the is-X?
//! TRUE-path that B.1 couldn't (B.1 had no constructor; defrecord does).
//!
//! Probe contracts (5):
//!   1. everyday is-X? ∀T — `(:my::is-Circle? 42)` → false, NOT a type error.
//!   2. is-X? TRUE-path (B.1-deferred) — `(:my::is-Circle? (:my::Circle 1.0))` → true.
//!   3. is-X? cross-class false — `(:my::is-Circle? (:my::Square 2.0))` → false.
//!   4. edge wired by emitted recordtype — `(subtype? :my::Circle :wat::Record)` → true.
//!   5. accessors + constructor still work — `(:my::Circle/radius (:my::Circle 1.0))` → 1.0.
//!
//! (No-DuplicateDefine is implied: every contract requires startup to succeed with
//! both the defrecord expansion AND the synthesized predicate present.)
//!
//! Initial state: FAILS — the everyday macro doesn't emit recordtype, so the class
//! is not a type (subtype? errors on unknown type name; is-X? is the narrowing form).
//! Post-stone S-B.2: 5/5 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Two record classes via the EVERYDAY defrecord surface.
const PRELUDE: &str = r#"
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
"#;

fn run(compute_expr: &str, ret_ty: &str) -> Result<Value, String> {
    let full = format!(
        "{prelude}\n\
         (:wat::core::defn :user::compute [] -> {ret} {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        prelude = PRELUDE,
        ret = ret_ty,
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

fn assert_bool(expr: &str, want: bool) {
    match run(expr, ":wat::core::bool") {
        Ok(Value::bool(b)) if b == want => {}
        other => panic!("expected {} for `{}`; got {:?}", want, expr, other),
    }
}

// ─── Probe 1: everyday is-X? ∀T — asymmetry dead on the real surface ──────────
#[test]
fn probe_01_everyday_is_predicate_forall_t() {
    assert_bool("(:my::is-Circle? 42)", false);
}

// ─── Probe 2: is-X? TRUE-path (B.1-deferred, now provable via the constructor) ─
#[test]
fn probe_02_is_predicate_true_path() {
    assert_bool("(:my::is-Circle? (:my::Circle 1.0))", true);
}

// ─── Probe 3: is-X? cross-class false ─────────────────────────────────────────
#[test]
fn probe_03_is_predicate_cross_class_false() {
    assert_bool("(:my::is-Circle? (:my::Square 2.0))", false);
}

// ─── Probe 4: edge wired by the emitted recordtype ────────────────────────────
#[test]
fn probe_04_edge_wired() {
    assert_bool("(:wat::core::subtype? :my::Circle :wat::Record)", true);
}

// ─── Probe 5: accessors + constructor still work (regression) ─────────────────
#[test]
fn probe_05_accessors_still_work() {
    match run("(:my::Circle/radius (:my::Circle 1.0))", ":wat::core::f64") {
        Ok(Value::f64(x)) if (x - 1.0).abs() < 1e-9 => {}
        other => panic!("expected 1.0 from accessor; got {:?}", other),
    }
}
