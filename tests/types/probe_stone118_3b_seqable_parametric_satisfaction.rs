//! Stone 118.3-B — a concrete container satisfies a PARAMETRIC surface (`(Seqable :- [T])`).
//!
//! **The wat source is the co-located sibling fixture**
//! `probe_stone118_3b_seqable_parametric_satisfaction.wat`, driven via `call_beside_value` —
//! the repo's test-fixture scheme (never inlined as a Rust string). The two negative rows are
//! `…_wrong_element.wat.bad` and `…_no_edge.wat.bad`, beside it.
//!
//! `src/check.rs`'s `(Parametric actual, Parametric expected)` arm string-compared a registered
//! `extend-type` edge against the call site's rendered expected type (a fresh unification var,
//! `(Seqable :- [?454])`) — never equal, so NO concrete container could satisfy a parametric
//! surface bound. See docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/{BRIEF,EXPECTATIONS,
//! MEASURED}-118.3-B*.md.
//!
//! Stone 255.22 — the fixture tests the language's own `:wat::core::Seqable` (it used to test a
//! private `:t118b::Seqable` copy, with an `as-vec` method standing in for `into`). The edges it
//! exercises now DECLARE their parameter (`(extend-type :- [T] (Vector :- [T]) (Seqable :- [T]))`)
//! and are matched structurally by that binder. The old `bare_surface_*` row (a private,
//! NON-parametric `:t118b::BareSeqable`) is gone with its surface; the (Parametric actual, bare
//! surface) arm it guarded is exercised by every surface method call's receiver check, e.g. the
//! 255.22 hello-world rows (`probe_arc255_22_an_edge_declares_its_type_parameters.rs`).

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

const DIR: &str = "tests/types/probe_stone118_3b_seqable_parametric_satisfaction";

fn expect_i64(fn_name: &str) -> i64 {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::i64(n) => n,
        other => panic!("{fn_name}: expected i64, got {other:?}"),
    }
}

/// The `.wat.bad` beside this file must be refused at CHECK, naming the parameter it refused.
fn refused_at_check(suffix: &str, needle: &str) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must be refused"));
    let StartupError::Check(errs) = &err else {
        panic!("{path}: expected a check-time refusal, got {err:?}");
    };
    let rendered = format!("{errs:?}");
    assert!(rendered.contains(needle), "{path}: refusal must name {needle:?}; got {rendered}");
}

/// Row 1 — the PARAMETRIC surface `(Seqable :- [T])` is satisfied by all four containers.
#[test]
fn parametric_surface_dispatches_vector() {
    assert_eq!(expect_i64(":t::param-vector"), 3);
}

#[test]
fn parametric_surface_dispatches_persistent_vector() {
    assert_eq!(expect_i64(":t::param-persistent-vector"), 4);
}

#[test]
fn parametric_surface_dispatches_list() {
    assert_eq!(expect_i64(":t::param-list"), 5);
}

#[test]
fn parametric_surface_dispatches_stream() {
    assert_eq!(expect_i64(":t::param-stream"), 2);
}

/// Row 2 — a CONCRETE instantiation `(Seqable :- [i64])` is satisfied by all four containers,
/// and the element type reaches the body (`+` on each element).
#[test]
fn concrete_instantiation_accepts_all_four() {
    assert_eq!(expect_i64(":t::sum-vector"), 6);
    assert_eq!(expect_i64(":t::sum-persistent-vector"), 10);
    assert_eq!(expect_i64(":t::sum-list"), 15);
    assert_eq!(expect_i64(":t::sum-stream"), 30);
}

// rune:lint(no-inlined-wat) — the `"(:wat::core::… :- […])"` literals below are golden COMPARISON
// text for the refusal's rendered type (a parametric type renders as a real `(Head :- [args])`
// form, so the reader happens to parse it); nothing here builds, evals, or runs a wat program from
// a string — every program is a `.wat`/`.wat.bad` fixture.
/// Negative — the swap-gate: `(Vector :- [String])` is not a `(Seqable :- [i64])`.
#[test]
fn a_wrong_element_type_is_refused() {
    refused_at_check("wrong_element", "(:wat::core::Vector :- [:wat::core::String])");
}

/// Negative — a family with no edge to the surface offers nothing.
#[test]
fn a_family_with_no_edge_is_refused() {
    refused_at_check("no_edge", "(:wat::core::HashMap :- [:wat::core::String :wat::core::i64])");
}
