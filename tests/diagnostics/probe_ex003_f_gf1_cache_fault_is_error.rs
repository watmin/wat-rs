//! Excursus 003 strike F, GF1 — `:wat::cache::Fault` conforms to `:wat::core::Error`.
//!
//! AUDIT-the-shape-of-an-error.md F8 / RULING 2026-09-27 item 5: `:wat::cache::Fault`
//! shares the floor's `Fault` name but used to carry no `location`. It now conforms:
//! `location <- :wat::core::Span` is added, minted at each lift site inside
//! `wat/cache.wat` (`Lru/new`, `Lru/put`) — the span in hand where the raw Rust tuple
//! becomes a wat error.
//!
//! Drives a REAL producer (co-located fixture `:user::verify`: `Lru/new` with capacity
//! 0) through the real entry point. The fixture proves conformance itself by passing
//! the fault to a `[e <- :wat::core::Error]`-typed function and round-tripping it
//! through `edn::write`/`edn::read`.
//!
//! Mutation (recorded, not re-encoded here): drop `location` from `:wat::cache::Fault`'s
//! defrecord — RED, the same cascade as the sqlite sibling (GB2's precedent): the
//! fixture's `[e <- :wat::core::Error]` param stops structurally accepting the
//! two-field-short record and the trivial program fails to freeze at all.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn cache_fault_satisfies_error_and_carries_a_real_location() {
    let v = call_beside_value(file!(), ":user::verify").expect(
        "a real :wat::cache::Fault must satisfy [e <- :wat::core::Error], \
         round-trip through edn::write/edn::read, and expose a :location",
    );
    let s = match v {
        Value::String(s) => s.to_string(),
        other => panic!("expected Value::String; got {other:?}"),
    };
    // Byte-identical per docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden' (a scalar
    // gets a byte-identical assert_eq!, never a loose contains/starts_with/ends_with check).
    // `wat/cache.wat:135:31` is the `(:wat::kernel::here)` call inside `Lru/new`'s lift arm —
    // a fixed source coordinate, not a value that varies per run.
    assert_eq!(s, "wat/cache.wat:135:31 :: capacity must be positive; got 0");
}
