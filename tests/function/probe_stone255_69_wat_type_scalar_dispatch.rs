//! STONE 255.69 — `constructor_head_key` (`src/types.rs`) generalized from a
//! hand-listed match over the seven container names to a registry-membership
//! check over `canonical_type_key`'s denoted key, so a scalar hard primitive
//! (`u8`, `char`) dispatches through the SAME door the five collection heads
//! already used. See the co-located `.wat`'s header for the bug this guards
//! (SCORE-STONE-255.68 § "The two scalar constructors").

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

#[test]
fn u8_constructs_its_own_value() {
    match call_beside_value(file!(), ":user::u8-dispatches") {
        Ok(Value::u8(65)) => {}
        other => panic!("(wat.type/u8 65) must run and yield Value::u8(65); got {other:?}"),
    }
}

#[test]
fn char_constructs_its_own_value() {
    match call_beside_value(file!(), ":user::char-dispatches") {
        Ok(Value::wat__core__Char('a')) => {}
        other => panic!("(wat.type/char \"a\") must run and yield Value::wat__core__Char('a'); got {other:?}"),
    }
}

#[test]
fn collection_head_still_dispatches() {
    match call_beside_value(file!(), ":user::vector-still-dispatches") {
        Ok(Value::wat__core__PersistentVector(v)) => {
            assert_eq!(v.len(), 3, "(wat.type/PersistentVector 1 2 3) must yield a length-3 vector");
        }
        other => panic!("(wat.type/PersistentVector 1 2 3) must still run; got {other:?}"),
    }
}

#[test]
fn unregistered_wat_type_head_still_refuses() {
    let err = startup_from_file("tests/function/probe_stone255_69_wat_type_scalar_dispatch.wat.bad")
        .expect_err("a wat.type/ head naming no registered type must fail to check");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — the diagnostic carries a file:line span that
    // varies with the fixture path; pin the two stable tokens of the refusal.
    assert!(
        msg.contains("UnresolvedReference") && msg.contains("Nope"),
        "expected UnresolvedReference naming Nope for (wat.type/Nope 1), got:\n{msg}"
    );
}
