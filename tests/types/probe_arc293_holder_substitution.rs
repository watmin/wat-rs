//! Arc 293 — nature-lattice Liskov substitution probe.
//!
//! Verifies the lattice rule: `HolonRecord <: Record <: Value`, with `Struct` on a
//! separate branch.  Widening (holon → record) must be accepted; narrowing (record →
//! holon) must be rejected even when the fields match.  Struct is disjoint from Record.

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

/// Case 1 — a core record IS a record: passing `:geo::Pt` where `:wat::core::Record` is
/// wanted must succeed.
#[test]
fn core_record_accepted_where_record_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c1.wat");
    assert!(
        world.is_ok(),
        "a core record should be accepted where :wat::core::Record is wanted; got: {:?}",
        world.err()
    );
}

/// Case 2 — THE WIDEN: a holon record IS a record (holon <: record <: value).
/// Passing `:geo::HPt` where `:wat::core::Record` is wanted must succeed.
#[test]
fn holon_record_accepted_where_record_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c2.wat");
    assert!(
        world.is_ok(),
        "a holon record should widen to :wat::core::Record (holon <: record); got: {:?}",
        world.err()
    );
}

/// Case 3 — a holon record IS a holon: passing `:geo::HPt` where `:wat::holon::Record`
/// is wanted must succeed (same-kind, not a substitution question at all).
#[test]
fn holon_record_accepted_where_holon_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c3.wat");
    assert!(
        world.is_ok(),
        "a holon record should be accepted where :wat::holon::Record is wanted; got: {:?}",
        world.err()
    );
}

/// Case 4 — THE NARROW (forbidden): a CORE record may NOT be passed where a
/// `:wat::holon::Record` is wanted — even though the fields are identical.
/// Holon-ness is CATEGORICAL (carries `holon_form`/VSA capability), not structural.
#[test]
fn core_record_rejected_where_holon_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c4.wat.bad");
    // The rejection must be the HOLDER mismatch — not an incidental error. The fields are
    // identical to a holon's, so the only thing that can fail is `:geo::Pt` ↛ `:wat::holon::Record`.
    let err = world.expect_err("expected startup failure; got Ok").to_string();
    wat::assert_edn_matches_file!(err, "probe_arc293_holder_substitution__core_record_rejected_where_holon_wanted.edn", "core record narrowed to :wat::holon::Record must be rejected: TypeMismatch");
}

/// Case 5 — a STRUCT is not a record (separate branch): passing `:geo::SPt` where
/// `:wat::core::Record` is wanted must be rejected.
#[test]
fn struct_rejected_where_record_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c5.wat.bad");
    // The set also carries an incidental MalformedForm (arc294 kwargs-construct retirement, a
    // side effect of the fixture's bare-positional constructor syntax) — the substitution
    // rule under test is the TypeMismatch below (set membership, per `assert_check_error_present!`).
    wat::assert_startup_error!(world, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":u::wants-record"
            && param == "#1"
            && expected == ":wat::core::Record"
            && got == ":geo::SPt"
    );
}
