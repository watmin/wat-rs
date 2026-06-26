//! Arc 293 — holder-lattice Liskov substitution probe.
//!
//! Verifies the lattice rule: `HolonRecord <: Record <: Value`, with `Struct` on a
//! separate branch.  Widening (holon → record) must be accepted; narrowing (record →
//! holon) must be rejected even when the fields match.  Struct is disjoint from Record.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Case 1 — a core record IS a record: passing `:geo::Pt` where `:wat::Record` is
/// wanted must succeed.
#[test]
fn core_record_accepted_where_record_wanted() {
    let src = r#"
        (:wat::core::defrecord :geo::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:u::wants-record (:geo::Pt 1 2))
          nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "a core record should be accepted where :wat::Record is wanted; got: {:?}",
        world.err()
    );
}

/// Case 2 — THE WIDEN: a holon record IS a record (holon <: record <: value).
/// Passing `:geo::HPt` where `:wat::Record` is wanted must succeed.
#[test]
fn holon_record_accepted_where_record_wanted() {
    let src = r#"
        (:wat::holon::defrecord :geo::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:u::wants-record (:geo::HPt 1 2))
          nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "a holon record should widen to :wat::Record (holon <: record); got: {:?}",
        world.err()
    );
}

/// Case 3 — a holon record IS a holon: passing `:geo::HPt` where `:wat::holon::Record`
/// is wanted must succeed (same-kind, not a substitution question at all).
#[test]
fn holon_record_accepted_where_holon_wanted() {
    let src = r#"
        (:wat::holon::defrecord :geo::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:u::wants-holon (:geo::HPt 1 2))
          nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
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
    let src = r#"
        (:wat::core::defrecord :geo::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:u::wants-holon (:geo::Pt 1 2))
          nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    // The rejection must be the HOLDER mismatch — not an incidental error. The fields are
    // identical to a holon's, so the only thing that can fail is `:geo::Pt` ↛ `:wat::holon::Record`.
    let err = format!("{:?}", world.err());
    assert!(
        err.contains("holon::Record") && (err.contains("geo::Pt") || err.contains("Mismatch")),
        "a core record must NOT narrow to :wat::holon::Record (fields match but holon-ness is \
         categorical); and the rejection must CITE the holder, not be incidental. got: {err}"
    );
}

/// Case 5 — a STRUCT is not a record (separate branch): passing `:geo::SPt` where
/// `:wat::Record` is wanted must be rejected.
#[test]
fn struct_rejected_where_record_wanted() {
    let src = r#"
        (:wat::core::defstruct :geo::SPt [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:u::wants-record (:geo::SPt/new 1 2))
          nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_err(),
        "a struct must NOT satisfy :wat::Record (struct is a separate branch of the holder lattice)"
    );
}
