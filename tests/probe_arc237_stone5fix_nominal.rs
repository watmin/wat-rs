//! FM 2-bis probe — arc 237 Stone 237.5.fix-nominal-identity.
//!
//! Stone 237.5's conforms? was probed on record/primitive/union/vector/alias —
//! NEVER on enum, newtype, or the `:wat::core::struct` form. The 237.6 crawl
//! traced a defect: `Value::Enum.type_name()` returns the GENERIC "wat::core::Enum",
//! not the declared FQDN, so `(conforms? color-val :my::Color)` is always false.
//! This probe locks the contract for all three under-tested nominal forms; the
//! enum rows are the confirmed break (red pre-fix), newtype/struct rows reveal
//! their actual state.
//!
//! Contracts:
//!   1.  enum value conforms its own enum type        → true   (CONFIRMED BROKEN pre-fix)
//!   2.  enum value does NOT conform a different enum  → false
//!   3.  non-enum value does NOT conform an enum type  → false
//!   4.  newtype value conforms its own newtype        → true
//!   5.  newtype value does NOT conform its inner type → false  (nominally distinct)
//!   6.  struct value conforms its own struct type     → true
//!   7.  struct value does NOT conform a different struct → false
//!   8.  (regression) record conforms self             → true
//!   9.  (regression) i64 conforms :i64                → true
//!
//! Post-stone 237.5.fix: 9/9 PASS. The 237.5 probe (12/12) must also stay green.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PRELUDE: &str = r#"
(:wat::core::enum :my::Color :Red :Blue :Green)
(:wat::core::enum :my::Suit :Hearts :Spades)
(:wat::core::newtype :my::Price :wat::core::f64)
(:wat::core::struct :my::Point (x :wat::core::i64) (y :wat::core::i64))
(:wat::core::struct :my::Pair (a :wat::core::i64) (b :wat::core::i64))
(:wat::Record::def :my::Circle [radius <- :wat::core::f64])
"#;

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
        other => panic!("expected conforms? true for `{}`; got {:?}", expr, other),
    }
}
fn assert_false(expr: &str) {
    match run_bool(expr) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected conforms? false for `{}`; got {:?}", expr, other),
    }
}

/// Evaluate `(:wat::core::type <inner>)` and return its String result.
/// The SECOND consumer of the value→type authority — proving the fix
/// lands in one place that BOTH `type` and conforms? ride.
fn run_type(inner: &str) -> Result<String, String> {
    let full = format!(
        "{prelude}\n\
         (:wat::core::define (:user::compute -> :wat::core::String) (:wat::core::type {inner}))\n\
         (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        prelude = PRELUDE,
        inner = inner
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => Ok((*s).clone()),
        other => Err(format!("expected String; got {:?}", other)),
    }
}

fn assert_type_is(inner: &str, expected: &str) {
    match run_type(inner) {
        Ok(s) if s == expected => {}
        other => panic!("expected (type {}) == {:?}; got {:?}", inner, expected, other),
    }
}

// ─── enum (the confirmed break) ───────────────────────────────────────────────

#[test]
fn probe_01_enum_conforms_self() {
    assert_true("(:wat::core::conforms? :my::Color::Red :my::Color)");
}

#[test]
fn probe_02_enum_not_conforms_other_enum() {
    assert_false("(:wat::core::conforms? :my::Color::Red :my::Suit)");
}

#[test]
fn probe_03_non_enum_not_conforms_enum() {
    assert_false("(:wat::core::conforms? 1 :my::Color)");
}

// ─── newtype ──────────────────────────────────────────────────────────────────

#[test]
fn probe_04_newtype_conforms_self() {
    assert_true("(:wat::core::conforms? (:my::Price/new 1.5) :my::Price)");
}

#[test]
fn probe_05_newtype_not_conforms_inner() {
    assert_false("(:wat::core::conforms? (:my::Price/new 1.5) :wat::core::f64)");
}

// ─── struct (:wat::core::struct form) ──────────────────────────────────────────

#[test]
fn probe_06_struct_conforms_self() {
    assert_true("(:wat::core::conforms? (:wat::core::struct-new :my::Point 3 4) :my::Point)");
}

#[test]
fn probe_07_struct_not_conforms_other_struct() {
    assert_false("(:wat::core::conforms? (:wat::core::struct-new :my::Point 3 4) :my::Pair)");
}

// ─── regression sentinels (must stay green) ────────────────────────────────────

#[test]
fn probe_08_record_conforms_self_regression() {
    assert_true("(:wat::core::conforms? (:my::Circle 1.0) :my::Circle)");
}

#[test]
fn probe_09_primitive_regression() {
    assert_true("(:wat::core::conforms? 1 :wat::core::i64)");
}

// ─── the OTHER consumer of the one authority: :wat::core::type ─────────────────
// Proves the value→type extraction is fixed in ONE place that both `type` and
// conforms? ride. Pre-fix, `type` ALSO returns the generic kind for enum/newtype.

#[test]
fn probe_10_type_on_enum_is_declared_fqdn() {
    assert_type_is(":my::Color::Red", "my::Color");
}

#[test]
fn probe_11_type_on_newtype_is_declared_fqdn() {
    assert_type_is("(:my::Price/new 1.5)", "my::Price");
}

#[test]
fn probe_12_type_on_struct_is_declared_fqdn() {
    assert_type_is("(:wat::core::struct-new :my::Point 3 4)", "my::Point");
}
