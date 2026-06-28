//! Arc 293.4b — the generated surface dispatcher (the disconfirming probe).
//!
//! THE GAP this isolates: a call `(:t::Shape/area s)` — where `:t::Shape` is a surface with a method member
//! `area` — must DISPATCH on `s`'s runtime type to that type's `:T/area` defn. Two satisfiers (Circle, Square)
//! back `area` differently; one polymorphic `describe` must route each to the right impl.
//!
//! RED at HEAD (post-293.4a) — method members parse + satisfy, but `:t::Shape/area` has no dispatcher; the call
//! head resolves as UnknownFunction at check time and the program fails to type-check.
//!
//! GREEN at 293.4b — a `:Surface/method` head dispatches on the receiver's concrete type to `:<T>/<method>`
//! (LIFT the arc-232 protocol dispatch shape at `src/runtime.rs:5101`, routing to the plain `defn :T/method`,
//! NOT an `extend:<P>:<T>` impl). Plus the check-side call typing (mirror `src/check.rs:5789`).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// `(:t::Shape/area s)` routes by `s`'s runtime type: Circle → π·r² ≈ 12.566, Square → s² = 9.0.
#[test]
#[ignore = "RED at HEAD: arc-293.4b (the generated surface dispatcher :Surface/method) not built; \
            un-ignore when the strike lands GREEN"]
fn surface_method_dispatches_by_runtime_type() {
    let world =
        startup_beside(file!()).expect("293.4b: the :Shape/area surface dispatcher must type-check");

    let circle = eval_in_frozen(
        &wat::parse_one!("(:t::circle-area)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("circle-area must dispatch to :t::Circle/area")
    .value_owned();

    let square = eval_in_frozen(
        &wat::parse_one!("(:t::square-area)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("square-area must dispatch to :t::Square/area")
    .value_owned();

    let (c, s) = match (circle, square) {
        (Value::f64(c), Value::f64(s)) => (c, s),
        other => panic!("expected two f64 areas from the dispatcher; got {other:?}"),
    };

    // Routing by runtime type → the two satisfiers' own impls → distinct areas.
    assert!(
        (c - 12.566_36).abs() < 1e-3,
        "Circle area via :Shape/area should dispatch to :t::Circle/area (π·2² ≈ 12.566); got {c}"
    );
    assert!(
        (s - 9.0).abs() < 1e-9,
        "Square area via :Shape/area should dispatch to :t::Square/area (3² = 9.0); got {s}"
    );
}
