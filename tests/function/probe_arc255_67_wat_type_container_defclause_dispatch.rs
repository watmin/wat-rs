//! arc 255.67 — see the co-located .wat's header for the bug this guards.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn is_true(name: &str) {
    match call_beside_value(file!(), name) {
        Ok(Value::bool(true)) => {}
        other => panic!("{name} must be true; got {other:?}"),
    }
}

#[test]
fn vector_clause_dispatches() {
    is_true(":user::vector-clause-dispatches");
}

#[test]
fn list_clause_dispatches() {
    is_true(":user::list-clause-dispatches");
}
