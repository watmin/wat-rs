//! Stone 255.48 — a featureless surface is a declared edge.
//!
//! Width subtyping stays for a surface that names members. An empty member
//! list admits nothing by structure.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

#[test]
fn a_declared_edge_admits_the_record() {
    match call_beside_value(file!(), ":user::admitted") {
        Ok(Value::i64(1)) => {}
        other => panic!("a declared edge must admit Item to Mark; got {other:?}"),
    }
}

#[test]
fn width_subtyping_still_admits_a_wider_record() {
    match call_beside_value(file!(), ":user::width") {
        Ok(Value::i64(1)) => {}
        other => panic!("a record with the named member must satisfy HasN; got {other:?}"),
    }
}

#[test]
fn an_undeclared_record_is_refused() {
    let result = startup_from_file(
        "tests/types/probe_arc255_48_featureless_surface_undeclared.wat.bad",
    );
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":probe::take"
            && param == "#1"
            && expected == ":probe::Mark"
            && got == ":probe::Item"
    );
}

#[test]
fn a_featureless_parametric_surface_refuses_an_undeclared_record() {
    let result = startup_from_file(
        "tests/types/probe_arc255_48_featureless_surface_parametric.wat.bad",
    );
    // rune:lint(no-inlined-wat) — the expected string below is the checker's rendered
    // TypeMismatch field, a `(Head :- [args])` form the reader happens to parse. Nothing
    // here builds or runs a wat program from that string; the program is the .wat.bad.
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":probe::take"
            && param == "#1"
            && expected == "(:probe::Tag :- [:wat::core::i64 :wat::core::i64])"
            && got == ":probe::Box"
    );
}
