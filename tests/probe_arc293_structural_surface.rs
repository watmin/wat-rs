//! Arc 293.3 — DISCONFIRMING PROBE for STRUCTURAL SURFACE satisfaction (the riskiest new machinery).
//!
//! The model (arc 293): a function param `[s <- [color <- :String]]` declares a STRUCTURAL surface — "s
//! is anything that structurally exposes a `color -> :String` accessor." A record/struct satisfies it
//! AMBIENTLY by having the field/accessor (row-polymorphic width subtyping; no `:satisfies`, no `:parent`).
//!
//! RED at HEAD (the gap this probe isolates): `[name <- :type]` does NOT parse in type position — the
//! `[...]` bracket is fn-type-only (`parse_fn_type_bracket`, types.rs:2390, needs a `:->` arrow). So the
//! type parser rejects the structural surface before the checker ever runs. This probe proves the gap is
//! exactly there (the type parser), not somewhere downstream.
//!
//! GREEN when 293.3 lands: `TypeExpr::Surface` (a row variant) + the parser dual-read (`<-` triples →
//! Surface, `:->` → Fn) + a structural-match arm in `assignable` (the candidate type has ⊇ the surface's
//! members). Then `:geo::Circle` structurally satisfies `[color <- :String]` with no declaration.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
#[ignore = "arc 293.3 RED probe — structural surfaces do not parse in type position yet; run with --ignored to confirm the gap"]
fn structural_surface_field_satisfaction_is_ambient() {
    // Note: uses the CURRENT `:wat::Record::def` (the `defrecord` rename is 293.2). The new thing here is
    // ONLY the inline structural surface `[color <- :String]` in the `describe` param's type position.
    let src = r#"
        (:wat::Record::def :geo::Circle
          [color <- :wat::core::String  radius <- :wat::core::f64])

        (:wat::core::defn :geo::describe [s <- [color <- :wat::core::String]] -> :wat::core::String
          (:geo::Circle/color s))

        (:wat::core::defn :user::main [] -> :wat::core::String
          (:geo::describe (:geo::Circle "red" 2.0)))
    "#;
    // GREEN TARGET: startup succeeds — Circle ambiently satisfies the [color <- :String] surface.
    // RED AT HEAD: fails in the type parser (the structural surface is unparseable in type position).
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "structural surface [color <- :String] should type-check via width subtyping; got: {:?}",
        world.err()
    );
}
