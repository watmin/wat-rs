//! Excursus 003 strike F, GF1 — `:wat::query::Fault` conforms to `:wat::core::Error`.
//!
//! AUDIT-the-shape-of-an-error.md F8 / RULING 2026-09-27 item 5: `:wat::query::Fault`
//! shares the floor's `Fault` name but used to carry no `location`, so it could not be a
//! cause, a `Failure.error`, or anything the envelope carries. It now conforms:
//! `location <- :wat::core::Span` is added. Every construction mints `:location` at its
//! own call site via `(:wat::kernel::here)` EXCEPT `wat/query/sqlite-store.wat`'s
//! `lift-fault`, which PROPAGATES the originating `:wat::sqlite::Fault`'s own `:location`
//! instead of minting a fresh one — this test drives exactly that path.
//!
//! Drives a REAL producer (co-located fixture `:user::verify`: a real sqlite open failure
//! narrowed through `lift-fault`) through the real entry point. The fixture proves
//! conformance by passing the value to a `[e <- :wat::core::Error]`-typed function and
//! round-tripping it through `edn::write`/`edn::read`, and proves PROPAGATION (not a fresh
//! mint) by asserting the location names `wat/sqlite.wat` (the original `classify` call
//! site), never `wat/query/sqlite-store.wat` (`lift-fault`'s own call site).
//!
//! Mutation (recorded, not re-encoded here): drop `location` from `:wat::query::Fault`'s
//! defrecord in wat/query.wat — RED, the same cascade as the sqlite/cache siblings: the
//! fixture's `[e <- :wat::core::Error]` param stops structurally accepting the
//! one-field-short record and the trivial program fails to freeze at all.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn query_fault_satisfies_error_and_propagates_the_real_origin_location() {
    let v = call_beside_value(file!(), ":user::verify").expect(
        "a real :wat::query::Fault must satisfy [e <- :wat::core::Error], \
         round-trip through edn::write/edn::read, and expose a :location",
    );
    let s = match v {
        Value::String(s) => s.to_string(),
        other => panic!("expected Value::String; got {other:?}"),
    };
    assert_eq!(
        s,
        "wat/sqlite.wat:91:29 :: unable to open database file: /nonexistent-dir-arc278-strike-f-query/x.db",
        "lift-fault must PROPAGATE the original sqlite::Fault's :location \
         (the real `classify` call site), never mint a fresh one at its own call site"
    );
}
