//! Arc 293.4d — a field member is an accessor too (the disconfirming probe).
//!
//! THE GAP this isolates: a FIELD member `color <- :String` called THROUGH the surface as `:Surface/color s`
//! must dispatch by `s`'s runtime type to the satisfier's `:T/color` accessor — exactly as a method member does
//! (293.4b). Field-vs-method is invisible at the call site; both are accessors.
//!
//! RED at HEAD (post-293.4c) — 293.4b generates a dispatcher only for METHOD members; `:t::Colored/color` (a field
//! member) is `UnknownCallee`.
//!
//! GREEN at 293.4d — every surface member (field or method) dispatches `:Surface/name s` → `:<T>/name`. Isolated to
//! a record here (the comprehensive case — a field member backed by a foreign extend-type METHOD — is the acceptance
//! demo).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// `(:t::Colored/color (:t::Ball "red" 2.0))` routes the FIELD member to `:t::Ball/color` (the auto field accessor).
#[test]
#[ignore = "RED at HEAD: arc-293.4d (field members are accessors too — :Surface/field dispatch) not built; \
            un-ignore when the strike lands GREEN"]
fn field_member_dispatches_through_the_surface() {
    let world = startup_beside(file!())
        .expect("293.4d: a field member called as :t::Colored/color must dispatch through the surface");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch the field member :t::Colored/color to :t::Ball/color")
    .value_owned();

    match got {
        Value::String(s) => assert_eq!(
            &*s, "red",
            "the field member :t::Colored/color, dispatched on a Ball, should read its color field; got {s:?}"
        ),
        other => panic!("expected the String \"red\" from the field-member accessor dispatch; got {other:?}"),
    }
}
