//! Excursus 003 strike F, GF1 — `:wat::query::Fault` conforms to `:wat::core::Error`.
//!
//! AUDIT-the-shape-of-an-error.md F8 / RULING 2026-09-27 item 5: `:wat::query::Fault`
//! shares the floor's `Fault` name but used to carry no `location`, so it could not be a
//! cause, a `Failure.error`, or anything the envelope carries. It now conforms:
//! `location <- :wat::core::Span` is added. Every construction mints `:location` via
//! `(:wat::kernel::error-site)` EXCEPT `wat/query/sqlite-store.wat`'s `lift-fault`, which
//! PROPAGATES the originating `:wat::sqlite::Fault`'s own `:location` instead of minting
//! a fresh one — this test drives exactly that path.
//!
//! Strike F2 (BRIEF-shape-strike-F2-one-location-rule.md, GF2a): the ORIGIN
//! (`:wat::sqlite::classify`) now derives a user-source location rather than minting
//! `wat/sqlite.wat`'s own line, so the location `lift-fault` propagates is ALSO this
//! fixture's own line — propagation was always "equally honest" as a fresh F2 mint would
//! be, never "more honest" (the pre-F2 framing, when a fresh mint still meant the
//! stdlib's own line).
//!
//! Drives a REAL producer (co-located fixture `:user::verify`: a real sqlite open failure
//! narrowed through `lift-fault`) through the real entry point. The fixture proves
//! conformance by passing the value to a `[e <- :wat::core::Error]`-typed function and
//! round-tripping it through `edn::write`/`edn::read`, and proves PROPAGATION (not a fresh
//! mint) by asserting the location names THIS fixture's own file (the real origin, now
//! user-derived), never `wat/query/sqlite-store.wat` (`lift-fault`'s own call site).
//!
//! Mutations (recorded, not re-encoded here):
//! - drop `location` from `:wat::query::Fault`'s defrecord in wat/query.wat — RED, the
//!   same cascade as the sqlite/cache siblings: the fixture's `[e <- :wat::core::Error]`
//!   param stops structurally accepting the one-field-short record and the trivial
//!   program fails to freeze at all.
//! - GF2a: restore `(:wat::kernel::here)` at `:wat::sqlite::classify`'s own mint site in
//!   `wat/sqlite.wat` (the real origin `lift-fault` propagates) — RED, the propagated
//!   `:location` goes back to naming `wat/sqlite.wat` instead of this fixture's own file.

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
    // Excursus 003 F2 (GF2a): the fixture's OWN line, propagated by `lift-fault` from the
    // origin `:wat::sqlite::classify`'s now user-derived `:location` — never
    // `wat/sqlite.wat`'s mint-site line, and never `lift-fault`'s own narrowing call site.
    assert_eq!(
        s,
        "tests/diagnostics/probe_ex003_f_gf1_query_fault_is_error.wat:20:22 :: unable to open database file: /nonexistent-dir-arc278-strike-f-query/x.db",
        "lift-fault must PROPAGATE the original sqlite::Fault's :location, which F2 now \
         derives to the user's own call site, never wat/sqlite.wat's mint-site line nor \
         lift-fault's own narrowing call site"
    );
}
