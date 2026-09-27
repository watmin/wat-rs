//! Stone 255.56 — `<` and `=` ask the declared classes.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

#[test]
fn a_newtype_orders_through_less_than() {
    match call_beside_value(file!(), ":user::ord-nt") {
        Ok(Value::bool(true)) => {}
        other => panic!("(< T 1 T 2) must be true; got {other:?}"),
    }
}

#[test]
fn nil_is_equatable_and_not_orderable() {
    match call_beside_value(file!(), ":user::eq-nil") {
        Ok(Value::bool(true)) => {}
        other => panic!("(= nil nil) must be true; got {other:?}"),
    }
    let result = startup_from_file("tests/types/probe_arc255_56_operators_ord_nil.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, got, .. }
            if callee == ":wat::core::<" && got == ":wat::core::nil"
    );
}

#[test]
fn a_struct_is_not_equatable() {
    let result = startup_from_file("tests/types/probe_arc255_56_operators_struct.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, expected, got, .. }
            if callee == ":wat::core::="
            && expected == ":wat::core::Equatable"
            && got == ":u::St"
    );
}

#[test]
fn an_impure_enum_is_not_equatable() {
    let result = startup_from_file("tests/types/probe_arc255_56_operators_impure.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, expected, got, .. }
            if callee == ":wat::core::="
            && expected == ":wat::core::Equatable"
            && got == ":u::Imp"
    );
}

#[test]
fn an_unresolved_operand_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_56_operators_unresolved.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, expected, got, .. }
            if callee == ":wat::core::="
            && expected == "a resolved type; the operand is unresolved"
            && got == "_"
    );
}

#[test]
fn numeric_cross_is_admitted() {
    match call_beside_value(file!(), ":user::num") {
        Ok(Value::bool(_)) => {}
        other => panic!("(< 1 2.0) must check and return a bool; got {other:?}"),
    }
}

#[test]
fn eq_generic_refuses_a_function() {
    let result = startup_from_file("tests/types/probe_arc255_56_eq_generic_fn.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::BoundNotSatisfied { function, param, bound, got }
            if function == ":user::eq-generic"
            && param == "T"
            && bound == ":wat::core::Equatable"
            && *got == {
                let mut rendered = String::from(":wat::core::i64 :-> :wat::core::i64");
                rendered.insert(0, '[');
                rendered.push(']');
                rendered
            }
    );
}

#[test]
fn an_enum_compares_with_its_variant() {
    match call_beside_value(file!(), ":user::variant") {
        Ok(Value::bool(true)) => {}
        other => panic!("(= variant enum) must be true; got {other:?}"),
    }
}
