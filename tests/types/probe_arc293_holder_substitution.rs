//! Arc 293 — holder-lattice Liskov substitution probe.
//!
//! Verifies the lattice rule: `HolonRecord <: Record <: Value`, with `Struct` on a
//! separate branch.  Widening (holon → record) must be accepted; narrowing (record →
//! holon) must be rejected even when the fields match.  Struct is disjoint from Record.

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
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn core_record_rejected_where_holon_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c4_bad.wat");
    // The rejection must be the HOLDER mismatch — not an incidental error. The fields are
    // identical to a holon's, so the only thing that can fail is `:geo::Pt` ↛ `:wat::holon::Record`.
    let err = format!("{:?}", world.err());
    assert_eq!(err, r##"Some(Check(CheckErrors([CheckError { span: Span { file: "tests/types/probe_arc293_holder_substitution_c4_bad.wat", line: 6, col: 20, end_line: 6, end_col: 34 }, kind: TypeMismatch { callee: ":u::wants-holon", param: "#1", expected: ":wat::holon::Record", got: ":geo::Pt" } }])))"##);
}

/// Case 5 — a STRUCT is not a record (separate branch): passing `:geo::SPt` where
/// `:wat::core::Record` is wanted must be rejected.
#[test]
fn struct_rejected_where_record_wanted() {
    let world = startup_from_file("tests/types/probe_arc293_holder_substitution_c5_bad.wat");
    assert!(
        world.is_err(),
        "a struct must NOT satisfy :wat::core::Record (struct is a separate branch of the holder lattice)"
    );
}
