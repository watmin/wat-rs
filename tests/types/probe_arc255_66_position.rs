//! Stone 255.66 — `wat.type/` heads construct where the old head does, and type where a type is written.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn is_true(name: &str) {
    match call_beside_value(file!(), name) {
        Ok(Value::bool(true)) => {}
        other => panic!("{name} must be true; got {other:?}"),
    }
}

#[test]
fn both_spellings_equal_the_literal() {
    is_true(":user::lit-both");
}

#[test]
fn both_spellings_build_an_empty_vector() {
    is_true(":user::empty-both");
}

#[test]
fn a_type_position_accepts_the_new_head() {
    match call_beside_value(file!(), ":user::in-type") {
        Ok(Value::i64(7)) => {}
        other => panic!("type position must yield 7; got {other:?}"),
    }
}

#[test]
fn conj_and_nth_see_the_new_head() {
    match call_beside_value(file!(), ":user::conj-nth") {
        Ok(Value::i64(4)) => {}
        other => panic!("conj/nth must yield 4; got {other:?}"),
    }
}

#[test]
fn foldl_and_mapv_see_the_new_head() {
    match call_beside_value(file!(), ":user::fold") {
        Ok(Value::i64(3)) => {}
        other => panic!("foldl must yield 3; got {other:?}"),
    }
    is_true(":user::mapv-it");
}

#[test]
fn the_other_constructors_agree_across_spellings() {
    is_true(":user::hmap");
    is_true(":user::hset");
    is_true(":user::pvec");
    is_true(":user::plist");
    is_true(":user::ptuple");
    is_true(":user::pmap");
}

#[test]
fn a_macro_body_may_construct_with_the_new_head() {
    match call_beside_value(file!(), ":user::macro-ran") {
        Ok(Value::i64(1)) => {}
        other => panic!("macro must expand to 1; got {other:?}"),
    }
}
