//! Arc 255.77 — cutover stone 3, "the UUID type goes home". `wat.uuid/UUID` checks and runs in
//! a HEADER (fn param + return type) and as a SET ELEMENT (`HashSet<wat.uuid/UUID>`, which
//! requires Equatable + Hashable for membership dedup). The co-located fixture's own top-level
//! `def`s (evaluated eagerly at freeze, same mechanism `probe_arc278_capacity_derive.wat` uses)
//! assert both; a failing assertion panics during its def's own evaluation, surfacing as a
//! freeze error — so `world.is_ok()` is the single, honest gate.
use wat::freeze::startup_beside;

#[test]
fn uuid_home_type_checks_and_runs_in_a_header_and_as_a_set_element() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "wat.uuid/UUID must check and run as a fn header's param+return type, and \
         HashSet<wat.uuid/UUID> must dedup two equal uuids to one member; got: {:?}",
        world.err()
    );
}
