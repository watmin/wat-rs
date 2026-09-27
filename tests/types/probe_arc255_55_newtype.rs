//! Stone 255.55 — a newtype is tagged, and ordered by its inner value.
//!
//! The ordering gate still uses the predicate, which does not admit a newtype.
//! The ordering rows call values_compare on the values the fixture builds.

use std::cmp::Ordering;

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn a_newtype_writes_as_a_tagged_scalar() {
    match call_beside_value(file!(), ":user::written") {
        Ok(Value::bool(true)) => {}
        other => panic!("tagged write did not match; got {other:?}"),
    }
}

#[test]
fn a_newtype_reads_back_equal_to_the_original() {
    match call_beside_value(file!(), ":user::round") {
        Ok(Value::bool(true)) => {}
        other => panic!("read-back was not equal; got {other:?}"),
    }
}

#[test]
fn printing_a_newtype_does_not_panic() {
    match call_beside_value(file!(), ":user::printed") {
        Ok(Value::bool(true)) => {}
        other => panic!("writer did not return the tagged form; got {other:?}"),
    }
}

#[test]
fn two_newtypes_of_the_same_class_compare_by_the_inner_value() {
    let one = call_beside_value(file!(), ":user::one").expect("inner 1");
    let two = call_beside_value(file!(), ":user::two").expect("inner 2");
    assert_eq!(wat::runtime::values_compare(&one, &two), Some(Ordering::Less));
}

#[test]
fn two_newtypes_of_different_classes_do_not_compare() {
    let one = call_beside_value(file!(), ":user::one").expect("first class");
    let other = call_beside_value(file!(), ":user::other").expect("second class");
    assert_eq!(wat::runtime::values_compare(&one, &other), None);
}
