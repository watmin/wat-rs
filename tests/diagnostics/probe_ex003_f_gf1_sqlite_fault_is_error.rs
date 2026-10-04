//! Excursus 003 strike F, GF1 — `:wat::sqlite::Fault` conforms to `:wat::core::Error`.
//!
//! AUDIT-the-shape-of-an-error.md F8 / RULING 2026-09-27 item 5: `:wat::sqlite::Fault`
//! shares the floor's `Fault` name but used to carry no `location`, so it could not be
//! a cause, a `Failure.error`, or anything the envelope carries. It now conforms:
//! `location <- :wat::core::Span` is added, minted at `:wat::sqlite::classify`'s own
//! call site (wat/sqlite.wat) — the span in hand where the raw Rust tuple becomes a
//! wat error.
//!
//! This test drives a REAL producer (co-located fixture `:user::verify`, which opens a
//! nonexistent directory and gets back `Error.Fatal {fault}`) through the real entry
//! point, never a hand-built `RuntimeError`. The fixture itself proves conformance by
//! passing the fault to a `[e <- :wat::core::Error]`-typed function and round-tripping
//! it through `edn::write`/`edn::read` — if either refused to type-check or decode,
//! `call_beside_value` below would return `Err`, not `Ok(Value::String(_))`.
//!
//! Mutation (recorded, not re-encoded here): drop `location` from `:wat::sqlite::Fault`'s
//! defrecord in wat/core.wat — RED, because the fixture's `[e <- :wat::core::Error]`
//! param no longer structurally accepts the two-field-short record and the trivial
//! program fails to freeze at all (the same cascade GB2's precedent documents).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn sqlite_fault_satisfies_error_and_carries_a_real_location() {
    let v = call_beside_value(file!(), ":user::verify").expect(
        "a real :wat::sqlite::Fault must satisfy [e <- :wat::core::Error], \
         round-trip through edn::write/edn::read, and expose a :location",
    );
    let s = match v {
        Value::String(s) => s.to_string(),
        other => panic!("expected Value::String; got {other:?}"),
    };
    // Byte-identical per docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden' (a scalar
    // gets a byte-identical assert_eq!, never a loose contains/starts_with/ends_with check).
    // `wat/sqlite.wat:91:29` is the `(:wat::kernel::here)` call inside `classify` — a fixed
    // source coordinate, not a value that varies per run.
    assert_eq!(
        s,
        "wat/sqlite.wat:91:29 :: unable to open database file: /nonexistent-dir-arc278-strike-f/x.db"
    );
}
