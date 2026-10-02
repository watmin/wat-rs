//! Stone 255.52 — `(Vector :- [T])` is a `Mark` when `T` is a `Mark`.
// rune:lint(no-inlined-wat) — the assertion strings are the checker's rendered types, compared exactly; the programs live in the co-located fixtures.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn returns_one(name: &str) {
    match call_beside_value(file!(), name) {
        Ok(Value::i64(1)) => {}
        other => panic!("{name} is a Mark; got {other:?}"),
    }
}

#[test]
fn a_vector_of_a_member_is_admitted() {
    returns_one(":user::vec-in");
}

#[test]
fn a_vector_of_a_vector_of_a_member_is_admitted() {
    returns_one(":user::vec-nested");
}

#[test]
fn a_bounded_fn_accepts_a_vector_of_a_member() {
    returns_one(":user::f-in");
}

#[test]
fn an_unbounded_vector_edge_still_admits_any_element() {
    returns_one(":user::any-out");
}

#[test]
fn a_vector_of_a_non_member_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_52_conditional_membership_out.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MembershipBound { argument, surface, param, bound, got, slot }
            if argument == "(wat.type/Vector :- [:u::Out])"
            && surface == ":u::Mark"
            && param == "T"
            && bound == ":u::Mark"
            && got == ":u::Out"
            && slot.is_none()
    );
}

#[test]
fn a_nested_vector_of_a_non_member_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_52_conditional_membership_nested.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MembershipBound { argument, surface, param, bound, got, slot }
            if argument == "(wat.type/Vector :- [(wat.type/Vector :- [:u::Out])])"
            && surface == ":u::Mark"
            && param == "T"
            && bound == ":u::Mark"
            && got == ":u::Out"
            && slot.is_none()
    );
}

#[test]
fn a_bounded_fn_refuses_a_vector_of_a_non_member() {
    let result = startup_from_file("tests/types/probe_arc255_52_conditional_membership_f.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::BoundNotSatisfied { function, param, bound, got }
            if function == ":u::f"
            && param == "T"
            && bound == ":u::Mark"
            && got == "(wat.type/Vector :- [:u::Out])"
    );
}

#[test]
fn a_bound_that_names_an_undeclared_type_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_52_conditional_membership_free.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::EdgeFreeTypeName { name, slot, .. }
                if name == ":u::NoSuch" && slot == "bound")
    );
}
