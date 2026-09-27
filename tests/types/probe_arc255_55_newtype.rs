//! Stone 255.55 — a newtype is tagged, and ordered by its inner value.
//!
//! Same-class ordering goes through `<`. Different classes are refused at
//! the check, and the runtime arm is covered beside `values_compare`.

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
    match call_beside_value(file!(), ":user::ord") {
        Ok(Value::bool(true)) => {}
        other => panic!("(< T 1 T 2) must be true; got {other:?}"),
    }
}
