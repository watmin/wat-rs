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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// The full acceptance program. Uses the FINAL names: `defsurface` (not the historical `definterface`),
/// `defrecord`/`holon::defrecord` (landed 293.2-rename), `extend-type` (the demoted foreign adapter).
#[test]
#[ignore = "RED at HEAD: arc-293.4 (methods-are-accessors + dispatcher + extend-type adapter) not built; \
            un-ignore when the acceptance demo runs — the arc's final GREEN gate / R1 FORMA SOLA SUFFICIT"]
fn shape_demo_fields_and_methods_and_the_monkeypatch() {
    let src = r#"
        ;; ── THE SURFACE — a set-of-accessor (fields AND methods, uniformly) ──
        (:wat::core::defsurface :geo::Shape
          [color <- :wat::core::String]              ; FIELD-style accessor  → :T/color -> :String
          (area  [self] -> :wat::core::f64)           ; METHOD accessor       → :T/area  [self] -> :f64
          (label [self] -> :wat::core::String))       ; METHOD accessor       → :T/label [self] -> :String

        ;; ── OWN TYPE #1 — Circle (core record). :geo::Circle/color is generated FREE by the field. ──
        (:wat::core::defrecord :geo::Circle [color <- :wat::core::String  radius <- :wat::core::f64])
        (:wat::core::defn :geo::Circle/area [self <- :geo::Circle] -> :wat::core::f64
          (:wat::core::f64::* 3.14159 (:wat::core::f64::* (:geo::Circle/radius self) (:geo::Circle/radius self))))
        (:wat::core::defn :geo::Circle/label [self <- :geo::Circle] -> :wat::core::String
          (:wat::core::str "circle(r=" (:geo::Circle/radius self) ")"))
        ;;  ⇒ Circle exposes color+area+label ⇒ STRUCTURALLY satisfies :geo::Shape. No declaration.

        ;; ── OWN TYPE #2 — Square. Same surface, different fields. ──
        (:wat::core::defrecord :geo::Square [color <- :wat::core::String  side <- :wat::core::f64])
        (:wat::core::defn :geo::Square/area [self <- :geo::Square] -> :wat::core::f64
          (:wat::core::f64::* (:geo::Square/side self) (:geo::Square/side self)))
        (:wat::core::defn :geo::Square/label [self <- :geo::Square] -> :wat::core::String
          (:wat::core::str "square(s=" (:geo::Square/side self) ")"))

        ;; ── THE MONKEYPATCH — teach a FOREIGN built-in (holon Vector) to be a Shape (you don't own it) ──
        (:wat::core::extend-type :wat::holon::Vector :geo::Shape
          (color [self] -> :wat::core::String "grey")
          (area  [self] -> :wat::core::f64 (:wat::core::i64::to-f64 (:wat::core::length self)))
          (label [self] -> :wat::core::String (:wat::core::str "vector[" (:wat::core::length self) "]")))

        ;; ── POLYMORPHIC CONSUMER — accepts ANY Shape; the dispatcher routes :T/<accessor> by runtime type ──
        (:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
          (:wat::core::str (:geo::Shape/color s) " " (:geo::Shape/label s) " area=" (:geo::Shape/area s)))

        (:wat::core::defn :geo::demo [] -> :wat::core::String
          (:wat::core::str
            (:geo::describe (:geo::Circle "red" 2.0))                  "  |  "
            (:geo::describe (:geo::Square "blue" 3.0))                 "  |  "
            (:geo::describe (:wat::core::Vector :wat::core::i64 10 20 30))))

        (:wat::core::defn :user::main [] -> :wat::core::String (:geo::demo))
    "#;
    // GREEN TARGET: the program type-checks and (:geo::demo) yields
    //   "red circle(r=2.0) area=12.56636  |  blue square(s=3.0) area=9.0  |  grey vector[3] area=3.0"
    // RED AT HEAD: method members in defsurface / the dispatcher / extend-type-as-adapter are unbuilt.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "the acceptance demo (Shape/Circle/Square + holon-Vector monkeypatch) must type-check; got: {:?}",
        world.err()
    );
}
