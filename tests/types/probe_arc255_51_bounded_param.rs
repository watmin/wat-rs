//! Stone 255.51 — `[T :< X]` on fn, and therefore on defn.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

#[test]
fn a_member_of_the_bound_is_admitted() {
    match call_beside_value(file!(), ":user::admitted") {
        Ok(Value::i64(1)) => {}
        other => panic!("In joins Mark by an edge, so pass accepts it; got {other:?}"),
    }
}

#[test]
fn an_anonymous_fn_uses_the_same_bound() {
    match call_beside_value(file!(), ":user::anon") {
        Ok(Value::i64(1)) => {}
        other => panic!("an anonymous fn with [T :< Mark] accepts In; got {other:?}"),
    }
}

#[test]
fn a_record_without_the_edge_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_51_bounded_param_out.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::BoundNotSatisfied { function, param, bound, got }
            if function == ":u::pass"
            && param == "T"
            && bound == ":u::Mark"
            && got == ":u::Out"
    );
}

#[test]
fn an_unbounded_parameter_is_still_refused() {
    let result = startup_from_file("tests/types/probe_arc255_51_bounded_param_unbounded.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":u::takes-mark"
            && param == "#1"
            && expected == ":u::Mark"
            && got == ":T"
    );
}

#[test]
fn a_bounded_variable_left_unresolved_is_refused() {
    let result = startup_from_file("tests/types/probe_arc255_51_bounded_param_unresolved.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::BoundUnresolved { function, param, bound }
            if function == ":u::bad"
            && param == "T"
            && bound == ":u::Mark"
    );
}

#[test]
fn a_fn_binder_entry_that_is_not_a_bound_names_the_entry() {
    let result = startup_from_file("tests/types/probe_arc255_51_binder_fn_malformed.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Runtime(err)
            if matches!(err.kind(), wat::runtime::RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defn"
                && reason == "binder entry must be a bare name or [Name :< Type]; got [T :- :u::Mark]")
    );
}

#[test]
fn a_fn_binder_list_entry_names_the_entry() {
    let result = startup_from_file("tests/types/probe_arc255_51_binder_fn_list.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Runtime(err)
            if matches!(err.kind(), wat::runtime::RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defn"
                && reason == "binder entry must be a bare name or [Name :< Type]; got (T <- :u::Mark)")
    );
}

#[test]
fn a_method_binder_entry_that_is_not_a_bound_names_the_entry() {
    let result = startup_from_file("tests/types/probe_arc255_51_binder_method_malformed.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == ":wat::core::defsurface"
                && reason == "method member `go`: binder entry must be a bare name or [Name :< Type]; got [T :- :u::Mark]")
    );
}

#[test]
fn an_extend_type_keyword_entry_names_the_entry() {
    let result = startup_from_file("tests/types/probe_arc255_51_binder_extend_keyword.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "binder entry must be a bare name or [Name :< Type]; got :u::Mark")
    );
}

#[test]
fn an_extend_type_bounded_entry_names_the_entry() {
    let result = startup_from_file("tests/types/probe_arc255_51_binder_extend_bounded.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "extend-type does not accept a bounded type parameter yet (conditional membership is a later stone); entry [T :< :u::Mark]")
    );
}
