//! Arc 296 — Vector<Surface> element structural satisfaction probe.
//!
//! Verifies that a record satisfying a surface may be passed as an element
//! to `(:wat::core::Vector :- [:Surface] <record>)` — the same structural satisfaction
//! path that function-parameter binding already accepts.
//!
//! RED at HEAD: `infer_list_constructor` uses bare `unify` for element checks →
//! TypeMismatch {:expected ":g::E" :got ":g::Boom"} even though `:g::Boom` structurally
//! satisfies surface `:g::E`.
//!
//! GREEN after arc 296 fix: the element check detects a Surface `elem_ty` and
//! routes through `assignable` (structural satisfaction) instead of bare `unify`.

use wat::freeze::startup_beside;

#[test]
fn record_in_surface_vector_accepted() {
    // GREEN TARGET: (:g::Boom "x") is a valid element of Vector<:g::E> because :g::Boom
    // structurally satisfies :g::E (both have `msg <- :wat::core::String`).
    // RED AT HEAD: TypeMismatch expected :g::E got :g::Boom.
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "a record satisfying a surface must be accepted as a Vector<Surface> element; \
         got: {:?}",
        world.err()
    );
}
