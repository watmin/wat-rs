//! Arc 293.3-records (the def-unification, strike 1) — DISCONFIRMING PROBE:
//! a RECORD structurally satisfies a `defsurface`.
//!
//! 293.3-core made STRUCTS satisfy surfaces (`StructDef.fields` is typed). Records carry
//! their field types in `RecordDef.field_types` — but `:wat::core::defrecord` emits the
//! *string-literal* `recordtype` form (`["name" ...]`), leaving `field_types = None`, so a
//! record cannot satisfy a surface even though it has the members. (`recordtype` ALREADY
//! parses the typed form `[name <- :type ...]` → `field_types = Some` — `types.rs:2161`.)
//!
//! This is strike 1 of the base-struct unification (R2): make `:wat::core::defrecord` +
//! `:wat::holon::defrecord` emit the TYPED `recordtype` form so `field_types = Some`, and
//! add a Record arm to `assignable` mirroring the Struct arm. Then core AND holon records
//! satisfy surfaces by the same width-match as structs — and `RecordDef` now carries the
//! same typed-field data as `StructDef`, the precondition for the eventual `AggregateDef` merge.
//!
//! RED at HEAD: a core/holon record passed where a `:geo::Shape` is wanted fails to
//! type-check (`field_types = None` + no Record arm). GREEN when 293.3-records lands.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// A CORE record (`:wat::core::defrecord`) that HAS the surface's members satisfies it
/// structurally (width subtyping — the extra `radius` is fine).
#[test]
fn core_record_structurally_satisfies_a_defsurface() {
    let src = r#"
        (:wat::core::defsurface :geo::Shape [color <- :wat::core::String])
        (:wat::core::defrecord :geo::Circle [color <- :wat::core::String  radius <- :wat::core::f64])
        (:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
          "ok")
        (:wat::core::defn :user::main [] -> :wat::core::String
          (:geo::describe (:geo::Circle "red" 2.0)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "a core record with the surface's members should satisfy it; got: {:?}",
        world.err()
    );
}

/// THE R2 HEADLINE: a HOLON record (`:wat::holon::defrecord`) satisfies a core surface —
/// because it carries the same `(class, fields)` as a core record, plus a hologram.
#[test]
fn holon_record_structurally_satisfies_a_core_surface() {
    let src = r#"
        (:wat::core::defsurface :geo::Shape [color <- :wat::core::String])
        (:wat::holon::defrecord :geo::HCircle [color <- :wat::core::String  radius <- :wat::core::f64])
        (:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
          "ok")
        (:wat::core::defn :user::main [] -> :wat::core::String
          (:geo::describe (:geo::HCircle "red" 2.0)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "a holon record satisfies the surface of a core record (R2); got: {:?}",
        world.err()
    );
}

/// GUARD (passes both before AND after — the surface is a real lower bound, not a stamp):
/// a record MISSING a surface member is rejected.
#[test]
fn record_missing_a_surface_member_is_rejected() {
    let src = r#"
        (:wat::core::defsurface :geo::Shape [color <- :wat::core::String])
        (:wat::core::defrecord :geo::Bare [other <- :wat::core::i64])
        (:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
          "ok")
        (:wat::core::defn :user::main [] -> :wat::core::String
          (:geo::describe (:geo::Bare 5)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_err(),
        "a record lacking `color` must NOT satisfy :geo::Shape"
    );
}
