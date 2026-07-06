//! Arc 296 — pure-surface field purity probe.
//!
//! Verifies that a `Record`-natured surface (`defsurface` with `:nature :wat::core::Record`)
//! is correctly classified as a PURE type by `is_pure_type`, so a pure aggregate (`defrecord`)
//! may declare a field typed by it (including the recursive `Vector<Surface>` case).
//!
//! RED at HEAD: `is_pure_type` returns `false` for any `TypeDef::Surface(_)` arm (stale stub
//! from before arc 293 added the mandatory nature). The checker raises
//! `ImpureFieldInPureAggregate { aggregate: ":probe::E$holon-record", field: "causes" … }`.
//!
//! GREEN after arc 296 fix: the Surface arm mirrors the Aggregate arm —
//! `s.nature.as_ref().map(|h| h.is_pure()).unwrap_or(false)` —
//! so a Record-natured surface is pure and the field is accepted.

use wat::freeze::startup_beside;

#[test]
fn record_natured_surface_is_pure_field_type() {
    // GREEN TARGET: :probe::Boom (defrecord) may carry `causes <- Vector<probe::E>` because
    // :probe::E is a Record-natured surface → pure.
    // RED AT HEAD: ImpureFieldInPureAggregate raised on the `causes` field during startup.
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "a Record-natured surface must be classified as a pure type; \
         `causes <- :wat::core::Vector<probe::E>` in a defrecord should be accepted; \
         got: {:?}",
        world.err()
    );
}
