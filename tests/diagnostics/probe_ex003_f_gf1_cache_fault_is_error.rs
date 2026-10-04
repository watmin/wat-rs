//! Excursus 003 strike F, GF1 — `:wat::cache::Fault` conforms to `:wat::core::Error`.
//!
//! AUDIT-the-shape-of-an-error.md F8 / RULING 2026-09-27 item 5: `:wat::cache::Fault`
//! shares the floor's `Fault` name but used to carry no `location`. It now conforms:
//! `location <- :wat::core::Span` is added, minted at each lift site inside
//! `wat/cache.wat` (`Lru/new`, `Lru/put`).
//!
//! Strike F2 (BRIEF-shape-strike-F2-one-location-rule.md, GF2a): the lift site mints
//! `:location` via `(:wat::kernel::error-site)`, not `(:wat::kernel::here)` — `here`
//! would carry `wat/cache.wat`'s OWN mint-site line (C-114's defect, in a RETURNED
//! value); `error-site` derives the SAME location D4 derives for a raised error, so
//! this fixture's own failing call locates at ITS OWN line, never the stdlib's.
//!
//! Drives a REAL producer (co-located fixture `:user::verify`: `Lru/new` with capacity
//! 0) through the real entry point. The fixture proves conformance itself by passing
//! the fault to a `[e <- :wat::core::Error]`-typed function and round-tripping it
//! through `edn::write`/`edn::read`.
//!
//! Mutations (recorded, not re-encoded here):
//! - drop `location` from `:wat::cache::Fault`'s defrecord — RED, the same cascade as
//!   the sqlite sibling (GB2's precedent): the fixture's `[e <- :wat::core::Error]`
//!   param stops structurally accepting the two-field-short record and the trivial
//!   program fails to freeze at all.
//! - GF2a: restore `(:wat::kernel::here)` at `Lru/new`'s own lift site in
//!   `wat/cache.wat` — RED, `:location` goes back to naming `wat/cache.wat` instead of
//!   this fixture's own file.

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
    // Excursus 003 F2 (GF2a): the fixture's OWN line — `:location` is derived by
    // `:wat::kernel::error-site` to the innermost user-source frame (this fixture's own
    // call to `Lru/new`), never `wat/cache.wat`'s mint-site line.
    assert_eq!(
        s,
        "tests/diagnostics/probe_ex003_f_gf1_cache_fault_is_error.wat:15:22 :: capacity must be positive; got 0"
    );
}
