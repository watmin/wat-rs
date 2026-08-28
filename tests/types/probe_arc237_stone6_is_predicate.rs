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
//!       union membership; `(= (type v) "Shape")` never could)
//!   8.  UNION   : is-Shape? on a member (Square) → true
//!   9.  UNION   : is-Shape? on a non-member (i64) → false
//!   10. record  : is-Circle? on a Circle    → true   (regression — exists via Record.wat;
//!       must stay green after its body switches to conforms?)
//!
//! Pre-stone: the four TypeEnv-form predicates (is-Point?/is-Color?/is-Price?/is-Shape?)
//! do not exist → fail (UnknownFunction). is-Circle? (record) already exists → green.
//! Post-stone 237.6: 10/10 PASS.

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

fn run_bool(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
}

fn assert_true(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(true)) => {}
        other => panic!("expected true for `{}`; got {:?}", fn_name, other),
    }
}
fn assert_false(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected false for `{}`; got {:?}", fn_name, other),
    }
}

// ─── struct ───────────────────────────────────────────────────────────────────

#[test]
fn probe_01_struct_is_self() {
    assert_true(":user::probe01");
}
#[test]
fn probe_02_struct_is_other_false() {
    assert_false(":user::probe02");
}

// ─── enum ─────────────────────────────────────────────────────────────────────

#[test]
fn probe_03_enum_is_self() {
    assert_true(":user::probe03");
}
#[test]
fn probe_04_enum_is_other_false() {
    assert_false(":user::probe04");
}

// ─── newtype ──────────────────────────────────────────────────────────────────

#[test]
fn probe_05_newtype_is_self() {
    assert_true(":user::probe05");
}
#[test]
fn probe_06_newtype_is_inner_false() {
    assert_false(":user::probe06");
}

// ─── union (THE PAYLOAD — membership, which conforms? unwraps) ─────────────────

#[test]
fn probe_07_union_member_circle_true() {
    assert_true(":user::probe07");
}
#[test]
fn probe_08_union_member_square_true() {
    assert_true(":user::probe08");
}
#[test]
fn probe_09_union_non_member_false() {
    assert_false(":user::probe09");
}

// ─── record (regression: exists via Record.wat; stays green after body→conforms?) ─

#[test]
fn probe_10_record_is_self_regression() {
    assert_true(":user::probe10");
}
