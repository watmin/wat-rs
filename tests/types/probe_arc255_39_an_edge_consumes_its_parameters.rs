//! Stone 255.39 — a surface parameter is consumed by the edge that binds it.
//!
//! Rows (fixtures beside this file):
//! - `edge_consumes` — featureless `(Owner :- [S R])` plus one implementing
//!   binder edge: accepted.
//! - `no_edge` — the same surface, no edge: `UnconsumedTypeParam`.
//! - `struct_unused` — a `defstruct` parameter no field uses: still
//!   `UnconsumedTypeParam`. Aggregates do not read edges.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::error::TypeErrorKind;

fn refusal(path: &str) -> (String, String) {
    let err = startup_from_file(path).expect_err(&format!("{path} must be refused"));
    let StartupError::Type(e) = err else {
        panic!("{path}: expected a registration-time type error, got {err:?}");
    };
    match e.kind() {
        TypeErrorKind::UnconsumedTypeParam { decl, param } => (decl.clone(), param.clone()),
        other => panic!("{path}: expected UnconsumedTypeParam, got {other:?}"),
    }
}

#[test]
fn an_implementing_edge_consumes_the_surface_parameter() {
    startup_from_file("tests/types/probe_arc255_39_edge_consumes.wat")
        .expect("a featureless surface bound by an implementing edge is accepted");
}

#[test]
fn a_featureless_surface_with_no_edge_is_refused() {
    let (decl, param) = refusal("tests/types/probe_arc255_39_no_edge.wat.bad");
    assert_eq!(decl, ":probe::Owner");
    assert_eq!(param, "S");
}

#[test]
fn an_unused_struct_parameter_is_still_refused() {
    let (decl, param) = refusal("tests/types/probe_arc255_39_struct_unused.wat.bad");
    assert_eq!(decl, ":probe::Unused");
    assert_eq!(param, "T");
}
