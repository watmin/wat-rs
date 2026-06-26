//! Arc 293.3-core — DISCONFIRMING PROBE for STRUCTURAL SURFACE satisfaction (the keystone).
//!
//! The model (arc 293): `(:wat::core::defsurface :geo::Shape [color <- :String])` declares a STRUCTURAL
//! surface — "anything that structurally exposes a `color` field/accessor of type `:String`." A record
//! satisfies it AMBIENTLY by *having* the field (row-polymorphic width subtyping — no `:satisfies`, no
//! `:parent`, no declaration). So a `:geo::Circle` value is accepted wherever a `:geo::Shape` is expected.
//!
//! This is the MINIMAL keystone: prove SATISFACTION (the `assignable` structural field-match), not yet
//! reading-through-the-surface (the dispatcher, which rides 293.4 alongside methods).
//!
//! Uses the CURRENT record form `:wat::Record::def` (the `defrecord` rename is 293.2); the ONLY new things
//! are `defsurface` and the named `:geo::Shape` surface in type position (the keyword path — no parser
//! bracket change; the `[...]`-in-type-position fn-type bracket is the idealized-future syntax, untouched).
//!
//! RED at HEAD (the gap this isolates): `defsurface` is an unknown declaration head and `:geo::Shape` does
//! not resolve to a type. GREEN when 293.3-core lands: `defsurface` (over the existing `ArgSpec` parser) +
//! `TypeExpr::Surface` + the keyword resolution + the structural field-match arm in `assignable`.
//!
//! 293.3-core uses a `defstruct` candidate ON PURPOSE: a struct carries its field TYPES in the TypeEnv
//! (`StructDef.fields: Vec<(String, TypeExpr)>`), so the `assignable` match is pure-TypeEnv — no SymbolTable
//! entanglement. A `Record::def` record's field types live in its accessors (SymbolTable), NOT its RecordDef
//! (`field_types = None`), so SOUND record-matching rides 293.2 (which gives records their field types). The
//! same `assignable` Surface arm then serves both kinds.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn record_structurally_satisfies_a_defsurface() {
    let src = r#"
        (:wat::core::defsurface :geo::Shape
          [color <- :wat::core::String])

        (:wat::core::defstruct :geo::Circle
          [color <- :wat::core::String  radius <- :wat::core::f64])

        ;; accepts ANYTHING with the Shape surface; Circle has `color` ⇒ structurally satisfies it
        ;; (and `radius` is extra — width subtyping, fine).
        (:wat::core::defn :geo::accepts-shape [s <- :geo::Shape] -> :wat::core::bool
          true)

        (:wat::core::defn :user::main [] -> :wat::core::bool
          (:geo::accepts-shape (:geo::Circle/new "red" 2.0)))
    "#;
    // GREEN TARGET: Circle structurally satisfies :geo::Shape (width subtyping) ⇒ startup type-checks.
    // RED AT HEAD: defsurface is unknown; :geo::Shape does not resolve.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        ":geo::Circle should structurally satisfy :geo::Shape via field-match; got: {:?}",
        world.err()
    );
}

/// Arc 293.3-core negative gate: a struct MISSING the surface member must FAIL to type-check.
///
/// `:geo::Bare` has `other <- :i64` but NOT `color <- :String`, so it does NOT
/// satisfy `:geo::Shape`. Passing a `:geo::Bare` where `:geo::Shape` is expected must
/// be a type error — the surface is a real lower bound, not a rubber stamp.
#[test]
fn missing_surface_member_is_rejected() {
    let src = r#"
        (:wat::core::defsurface :geo::Shape
          [color <- :wat::core::String])

        (:wat::core::defstruct :geo::Bare
          [other <- :wat::core::i64])

        (:wat::core::defn :geo::accepts-shape [s <- :geo::Shape] -> :wat::core::bool
          true)

        (:wat::core::defn :user::main [] -> :wat::core::bool
          (:geo::accepts-shape (:geo::Bare/new 42)))
    "#;
    // GREEN TARGET: startup FAILS (type error) because :geo::Bare lacks `color`
    // and therefore does NOT structurally satisfy :geo::Shape.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_err(),
        ":geo::Bare (missing `color`) should NOT satisfy :geo::Shape; but startup succeeded"
    );
}
