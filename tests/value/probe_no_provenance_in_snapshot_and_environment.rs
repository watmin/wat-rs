//! Standing probe — excursus 003 strike G item 4's gate.
//!
//! Item 4 retired `Provenance`/`TrackedValue` wholesale: `ValueSnapshot` lost its
//! `provenance` field (it carried `Unknown` for 503 of 506 construction sites —
//! `AUDIT-the-shape-of-an-error.md` F5) and `Environment`'s `BoundEntry` lost the
//! `binding_span` it existed only to feed into `lookup`'s `Provenance::SymbolBound`
//! re-derivation.
//!
//! The build itself is the primary gate (nothing named `Provenance`/`TrackedValue`
//! compiles any more). This probe is the "standing assertion" the item's gate
//! description asks for in addition: each of these two structs is destructured
//! with an EXHAUSTIVE field pattern (no `..`). If either struct ever grows a
//! `provenance` field again — or any other field — this probe fails to COMPILE
//! (a missing-field error on the pattern), not merely to assert false at runtime.
//! That is a tighter pin than a runtime check: it catches the shape changing at
//! all, not just a specific value showing up.
//!
//! Mutation: add `pub provenance: Provenance` (or any new field) to `ValueSnapshot`
//! or `BoundEntry` → this file fails to compile. RED.

use wat::value::{BoundEntry, Environment, Value, ValueSnapshot};

#[test]
fn value_snapshot_carries_no_provenance() {
    let snap = ValueSnapshot::of(&Value::i64(42));
    // Exhaustive field pattern — compiles iff ValueSnapshot has EXACTLY
    // {type_name, rendered}. A reintroduced `provenance` field breaks this build.
    let ValueSnapshot { type_name, rendered } = snap;
    assert_eq!(type_name, "wat::core::i64");
    assert_eq!(rendered, "42");
}

#[test]
fn bound_entry_carries_no_provenance() {
    let entry = BoundEntry { value: Value::i64(7) };
    // Exhaustive field pattern — compiles iff BoundEntry has EXACTLY {value}.
    // A reintroduced `binding_span` (or `provenance`) field breaks this build.
    let BoundEntry { value } = entry;
    assert!(matches!(value, Value::i64(7)));
}

#[test]
fn environment_lookup_returns_bare_value_not_a_tracked_pair() {
    // Environment::lookup takes only a name (no head_span — that existed only to
    // build Provenance::SymbolBound) and returns `Option<Value>` (no provenance
    // riding along). This call site pins that signature: it would not compile
    // against the old `lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue>`.
    let env = Environment::new().child().bind("x", Value::i64(5)).build();
    let looked_up: Option<Value> = env.lookup("x");
    assert!(matches!(looked_up, Some(Value::i64(5))));
}
