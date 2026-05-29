//! FM 2-bis probe — arc 237 Stone 237.6: auto-mint `is-<Name>?` (named convenience over conforms?).
//!
//! Every type-introducing declaration hands you a membership predicate
//! `:ns::is-<Name>?` ≡ `(conforms? x :ns::Name)`. NOT a second mechanism — a named
//! convenience over the one foundation (cf. accessors over field-at, arc 226
//! is-Map? over is?). Records already mint it (Record.wat); this stone adds the
//! four TypeEnv-registered forms (struct/enum/newtype/union) + unifies Record.wat's
//! body onto conforms?.
//!
//! Contracts (10):
//!   1.  struct  : is-Point? on a Point      → true
//!   2.  struct  : is-Point? on a non-Point  → false
//!   3.  enum    : is-Color? on a variant    → true
//!   4.  enum    : is-Color? on a non-enum   → false
//!   5.  newtype : is-Price? on a Price      → true
//!   6.  newtype : is-Price? on a plain f64  → false   (nominally distinct from inner)
//!   7.  UNION   : is-Shape? on a member (Circle) → true   ← THE PAYLOAD (conforms? unwraps
//!                 union membership; `(= (type v) "Shape")` never could)
//!   8.  UNION   : is-Shape? on a member (Square) → true
//!   9.  UNION   : is-Shape? on a non-member (i64) → false
//!   10. record  : is-Circle? on a Circle    → true   (regression — exists via Record.wat;
//!                 must stay green after its body switches to conforms?)
//!
//! Pre-stone: the four TypeEnv-form predicates (is-Point?/is-Color?/is-Price?/is-Shape?)
//! do not exist → fail (UnknownFunction). is-Circle? (record) already exists → green.
//! Post-stone 237.6: 10/10 PASS.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PRELUDE: &str = r#"
(:wat::Record::def :my::Circle [radius <- :wat::core::f64])
(:wat::Record::def :my::Square [side <- :wat::core::f64])
(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])
(:wat::core::defenum :my::Color :Red :Blue :Green)
(:wat::core::newtype :my::Price :wat::core::f64)
(:wat::core::defstruct :my::Point [x <- :wat::core::i64 y <- :wat::core::i64])
"#;

fn run_bool(compute_expr: &str) -> Result<Value, String> {
    let full = format!(
        "{prelude}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::bool {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
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

// ─── struct ───────────────────────────────────────────────────────────────────

#[test]
fn probe_01_struct_is_self() {
    assert_true("(:my::is-Point? (:wat::core::struct-new :my::Point 3 4))");
}
#[test]
fn probe_02_struct_is_other_false() {
    assert_false("(:my::is-Point? 1)");
}

// ─── enum ─────────────────────────────────────────────────────────────────────

#[test]
fn probe_03_enum_is_self() {
    assert_true("(:my::is-Color? :my::Color::Red)");
}
#[test]
fn probe_04_enum_is_other_false() {
    assert_false("(:my::is-Color? 1)");
}

// ─── newtype ──────────────────────────────────────────────────────────────────

#[test]
fn probe_05_newtype_is_self() {
    assert_true("(:my::is-Price? (:my::Price/new 1.5))");
}
#[test]
fn probe_06_newtype_is_inner_false() {
    assert_false("(:my::is-Price? 1.5)");
}

// ─── union (THE PAYLOAD — membership, which conforms? unwraps) ─────────────────

#[test]
fn probe_07_union_member_circle_true() {
    assert_true("(:my::is-Shape? (:my::Circle 1.0))");
}
#[test]
fn probe_08_union_member_square_true() {
    assert_true("(:my::is-Shape? (:my::Square 2.0))");
}
#[test]
fn probe_09_union_non_member_false() {
    assert_false("(:my::is-Shape? 1)");
}

// ─── record (regression: exists via Record.wat; stays green after body→conforms?) ─

#[test]
fn probe_10_record_is_self_regression() {
    assert_true("(:my::is-Circle? (:my::Circle 1.0))");
}
