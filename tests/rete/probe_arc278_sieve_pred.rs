//! Arc 278 Stone 2 — the `sieve-pred` capture macro (`wat/query.wat`). Models the `ast_to_source`
//! probe (`tests/rete/probe_arc278_ast_to_source.rs`): the user writes a REAL
//! `(fn [log <- :T] -> :bool …)`; `sieve-pred` captures it, `ast->source`s it into a
//! `:wat::query::Sieve::Predicate` String field — the user never types a string.
//!
//! Two assertions, each a co-located `.wat` entry:
//! 1. the `pred` string contains `"::"` verbatim (organic UX — a captured fn-form, not hand text).
//! 2. it round-trips through `read-string` back to the SAME fn-form the user wrote.
//!
//! Run: cargo test --release -p wat sieve_pred

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn classify(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

/// `sieve-pred` captures a real `(fn …)` and prints its verbatim `::`-source into the
/// `Sieve::Predicate` field — never a hand-typed string.
#[test]
fn sieve_pred_captures_verbatim_colon_colon_source() {
    assert!(
        classify(":user::sieve-pred-contains-double-colon"),
        "sieve-pred's captured Sieve::Predicate/pred string must contain verbatim `::` source"
    );
}

/// The captured source round-trips: `read-string(pred)` reproduces the SAME fn-form the user wrote.
#[test]
fn sieve_pred_round_trips_through_read_string() {
    assert!(
        classify(":user::sieve-pred-round-trips"),
        "read-string(Sieve::Predicate/pred) must reproduce the user's original fn-form"
    );
}
