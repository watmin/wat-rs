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

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

fn run_bool(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
}

fn assert_true(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(true)) => {}
        other => panic!("expected conforms? true for `{}`; got {:?}", fn_name, other),
    }
}
fn assert_false(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected conforms? false for `{}`; got {:?}", fn_name, other),
    }
}

fn run_type(fn_name: &str) -> Result<String, StartupError> {
    match call_beside_value(file!(), fn_name).map_err(StartupError::from)? {
        Value::String(s) => Ok((*s).clone()),
        // Arc 296 Stone M: a wrong-shape return isn't a StartupError — the only caller
        // (`assert_type_is`) already panics on anything but a matching string.
        other => panic!("expected String; got {:?}", other),
    }
}

fn assert_type_is(fn_name: &str, expected: &str) {
    match run_type(fn_name) {
        Ok(s) if s == expected => {}
        other => panic!("expected type == {:?} from {}; got {:?}", expected, fn_name, other),
    }
}

// ─── enum (the confirmed break) ───────────────────────────────────────────────

#[test]
fn probe_01_enum_conforms_self() {
    assert_true(":user::probe01");
}

#[test]
fn probe_02_enum_not_conforms_other_enum() {
    assert_false(":user::probe02");
}

#[test]
fn probe_03_non_enum_not_conforms_enum() {
    assert_false(":user::probe03");
}

// ─── newtype ──────────────────────────────────────────────────────────────────

#[test]
fn probe_04_newtype_conforms_self() {
    assert_true(":user::probe04");
}

#[test]
fn probe_05_newtype_not_conforms_inner() {
    assert_false(":user::probe05");
}

// ─── struct (:wat::core::struct form) ──────────────────────────────────────────

#[test]
fn probe_06_struct_conforms_self() {
    assert_true(":user::probe06");
}

#[test]
fn probe_07_struct_not_conforms_other_struct() {
    assert_false(":user::probe07");
}

// ─── regression sentinels (must stay green) ────────────────────────────────────

#[test]
fn probe_08_record_conforms_self_regression() {
    assert_true(":user::probe08");
}

#[test]
fn probe_09_primitive_regression() {
    assert_true(":user::probe09");
}

// ─── the OTHER consumer of the one authority: :wat::core::type ─────────────────
// Proves the value→type extraction is fixed in ONE place that both `type` and
// conforms? ride. Pre-fix, `type` ALSO returns the generic kind for enum/newtype.

#[test]
fn probe_10_type_on_enum_is_declared_fqdn() {
    assert_type_is(":user::probe10", "my::Color");
}

#[test]
fn probe_11_type_on_newtype_is_declared_fqdn() {
    assert_type_is(":user::probe11", "my::Price");
}

#[test]
fn probe_12_type_on_struct_is_declared_fqdn() {
    assert_type_is(":user::probe12", "my::Point");
}
