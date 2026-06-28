//! Arc 293.0 — THE ACCEPTANCE DEMO (the arc's final GREEN gate / R1 FORMA SOLA SUFFICIT fulfillment).
//!
//! This is the program DESIGN.md § "WHAT THE ARC DELIVERS" promises goes GREEN. It is the gate for R1
//! (the structural-surface / methods-are-accessors / Expression-Problem realization). When this passes,
//! arc 293's thesis is not just built — it is *demonstrated*: a foreign built-in (holon Vector) taught to
//! satisfy a user surface it never declared, dispatch routing by runtime shape, fields and methods backing
//! the same accessor interchangeably.
//!
//! RED at HEAD — and the RED names exactly what 293.4 (methods-are-accessors) must build:
//!   1. `defsurface` is FIELD-only today (`src/types/surface.rs`: members are `(name, TypeExpr)` pairs);
//!      the METHOD members `(area [self] -> :f64)` do not parse.
//!   2. The generated single-dispatch dispatcher `:geo::Shape/area s` (route by `s`'s runtime type to
//!      `:T/area`, reusing arc 232's extract-classifier + apply) does not exist.
//!   3. `extend-type` is still the arc-232 subtype-edge form, not the foreign-accessor adapter that adds
//!      `:wat::holon::Vector/color` etc. (the monkeypatch).
//!
//! The deep point (DESIGN): look at `color`. The surface only requires "expose `:T/color -> :String`."
//! Circle backs it with a FIELD (free accessor); the Vector backs it with a METHOD. Field-vs-method is the
//! satisfier's private choice — the interface sees only an accessor. That is "methods are accessors," and
//! it dissolves the field/method seam end to end.
//!
//! GREEN when 293.4 lands: method members in `defsurface`, the generated dispatcher, `extend-type` as the
//! typed foreign-accessor adapter (`defprotocol` annihilated, its live spawn/service users migrated).
//! `#[ignore]`'d STRIKE-READY; un-ignore when the demo runs.

use wat::freeze::startup_beside;

/// The full acceptance program. Uses the FINAL names: `defsurface` (not the historical `definterface`),
/// `defrecord`/`holon::defrecord` (landed 293.2-rename), `extend-type` (the demoted foreign adapter).
#[test]
#[ignore = "RED at HEAD: arc-293.4 (methods-are-accessors + dispatcher + extend-type adapter) not built; \
            un-ignore when the acceptance demo runs — the arc's final GREEN gate / R1 FORMA SOLA SUFFICIT"]
fn shape_demo_fields_and_methods_and_the_monkeypatch() {
    // GREEN TARGET: the program type-checks and (:geo::demo) yields
    //   "red circle(r=2.0) area=12.56636  |  blue square(s=3.0) area=9.0  |  grey vector[3] area=3.0"
    // RED AT HEAD: method members in defsurface / the dispatcher / extend-type-as-adapter are unbuilt.
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "the acceptance demo (Shape/Circle/Square + holon-Vector monkeypatch) must type-check; got: {:?}",
        world.err()
    );
}
