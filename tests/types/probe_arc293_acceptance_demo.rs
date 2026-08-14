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

/// The full acceptance program. Uses the FINAL names: `defsurface` (not the historical `definterface`),
/// `defrecord`/`holon::defrecord` (landed 293.2-rename), `extend-type` (the demoted foreign adapter).
#[test]
fn shape_demo_fields_and_methods_and_the_monkeypatch() {
    // GREEN at 293.4d: field + method surface members dispatch + extend-type foreign adapter.
    // GREEN TARGET: the program type-checks and (:geo::demo) yields
    //   "red circle(r=2.0) area=12.56636  |  blue square(s=3.0) area=9.0  |  grey vector[3] area=3.0"
    use wat::freeze::call_beside_value;
    use wat::runtime::Value;

    let got = call_beside_value(file!(), ":geo::demo").expect("(:geo::demo) must evaluate");

    match got {
        Value::String(s) => assert_eq!(
            &*s,
            // ⛔ RESTORED to the DESIGN's promise, 2026-08-14 (stone 279.2). This row used to
            // assert `r=2` / `area=9` / `area=3` — Rust's `f64` Display, which drops the trailing
            // `.0` — and `SCORE-293.4d.md:30` logged the gap as an honest delta "in f64 Display
            // only". It was not cosmetic. **A float is not an int** (builder, 2026-08-14): `2`
            // reads back as an `i64`, so the old render was TYPE-LOSSY, and EDN requires the
            // decimal point for exactly that reason. `str` now routes through the EDN encoder
            // (279.2), which renders `2.0` — so the deviation closed itself and the GREEN TARGET
            // in this test's own header comment, four lines up, is what the code now produces.
            // Clojure/Ruby/Python all render `2.0`; Rust's Display was the outlier `str` inherited.
            "red circle(r=2.0) area=12.56636  |  blue square(s=3.0) area=9.0  |  grey vector[3] area=3.0",
            "the demo must match the DESIGN's promise — a float renders as a float (`2.0`, not `2`); got {s:?}"
        ),
        other => panic!("expected a String from (:geo::demo); got {other:?}"),
    }
}
