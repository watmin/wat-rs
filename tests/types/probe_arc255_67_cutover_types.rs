//! Stone 255.67 — cutover 2/7. Every one of the 24 `wat.type/` hard primitives resolves in a
//! type position, driven from the beside fixture `probe_arc255_67_cutover_types.wat`.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn is_true(name: &str) {
    match call_beside_value(file!(), name) {
        Ok(Value::bool(true)) => {}
        other => panic!("{name} must be true; got {other:?}"),
    }
}

#[test]
fn scalars_round_trip() {
    is_true(":user::scalars-round-trip");
}

#[test]
fn value_annotation_resolves() {
    is_true(":user::value-round-trips");
}

#[test]
fn fn_annotation_resolves() {
    is_true(":user::fn-annotation-resolves");
}

#[test]
fn bytes_round_trips() {
    is_true(":user::bytes-round-trips");
}

#[test]
fn ast_round_trips() {
    is_true(":user::ast-round-trips");
}

#[test]
fn record_and_struct_values() {
    is_true(":user::record-value");
    is_true(":user::struct-value");
}

#[test]
fn every_container_constructor_in_value_position() {
    is_true(":user::vector-value");
    is_true(":user::hashmap-value");
    is_true(":user::hashset-value");
    is_true(":user::list-value");
    is_true(":user::tuple-value");
    is_true(":user::persistentvector-value");
    is_true(":user::persistentmap-value");
}
