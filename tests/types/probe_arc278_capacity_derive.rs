//! Arc 278 capacity stone 1 — `:wat::telemetry::framing-floor-of` re-derives the framing floor
//! from a record type's LIVE field set (disconfirming probe).
//!
//! RED at HEAD: `wat/telemetry.wat` does not yet ship `framing-floor-of`, so the co-located
//! fixture's top-level `def`s referencing it fail to freeze ("unknown callee"). GREEN once the
//! derive ships: the fixture's own top-level `assert-true` bindings (evaluated eagerly at freeze,
//! same as any top-level `def`) prove RecB (RecA + one extra fixed `i64` field) yields a strictly
//! larger floor than RecA — adaptivity — and that the real `Log` floor is `>= 56`. A failing
//! assertion panics during its def's own evaluation, so it surfaces as a freeze error exactly like
//! the RED "unknown callee" does: `world.is_ok()` is the single honest gate for both.
use wat::freeze::startup_beside;

#[test]
fn framing_floor_of_rederives_when_a_fixed_field_is_added() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        ":wat::telemetry::framing-floor-of must exist and re-derive the framing floor from a \
         record type's live field set — RecB (RecA + one extra fixed i64 field) must yield a \
         strictly larger floor than RecA, and :wat::telemetry::Log's floor must be >= 56; got: {:?}",
        world.err()
    );
}
