//! Stone 255.53 — a tuple is a Mark when each element is.
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
fn a_pair_of_members_is_admitted() {
    returns_one(":user::pair");
}

#[test]
fn a_triple_of_members_is_admitted() {
    returns_one(":user::triple");
}

#[test]
fn a_nested_tuple_of_members_is_admitted() {
    returns_one(":user::nested");
}

#[test]
fn a_vector_of_a_member_tuple_is_admitted() {
    returns_one(":user::vec-of-pair");
}

#[test]
fn a_pair_with_a_non_member_names_slot_2() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_out.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MembershipBound { argument, surface, param, bound, got, slot }
            if argument == ":(u::In,u::Out)"
            && surface == ":u::Mark"
            && param == "Ts"
            && bound == ":u::Mark"
            && got == ":u::Out"
            && *slot == Some(2)
    );
}

#[test]
fn a_nested_non_member_names_the_inner_slot() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_nested.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MembershipBound { argument, surface, param, bound, got, slot }
            if argument == ":(u::In,(u::In,u::Out))"
            && surface == ":u::Mark"
            && param == "Ts"
            && bound == ":u::Mark"
            && got == ":u::Out"
            && *slot == Some(2)
    );
}

#[test]
fn a_fn_binder_refuses_the_repeated_marker() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_fn.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Runtime(err)
            if matches!(err.kind(), wat::runtime::RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defn"
                && reason == "`:..` is only legal on an extend-type binder; got [Ts :..]")
    );
}

#[test]
fn a_repeated_marker_that_is_not_last_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_not_last.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "`:..` must be the last binder entry; got [[Ts :< :u::Mark] :.. U]")
    );
}

#[test]
fn a_leading_repeated_marker_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_leading.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "`:..` must follow a binder entry; got [:.. Ts]")
    );
}

#[test]
fn a_tuple_slot_that_is_not_the_repeated_binder_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_53_tuple_member_wrong_name.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "`(Tuple :- [U :..])` repeats `U`; the repeated binder entry is `Ts`")
    );
}
