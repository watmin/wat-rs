//! Arc 293.4e-pre.iii — a bare-binder `extend-type` impl on a surface inherits the surface method's types.
//!
//! THE GENERIC SHAPE: a `make<T>` surface method, a BARE impl `(make [self x] (:t::Box x))` whose body wraps the
//! `:T`-typed arg in a `:t::Box<T>` — the `:wat::spawn::Locus` `launch<S,R,St,Sh,Lu>` shape. For the body to
//! type-check, the impl must inherit the surface member's sig (self → the extending type, `x : T`, ret `Box<T>`,
//! type-params carried), since the bare binders carry no annotations.
//!
//! STATUS (grounded 2026-06-30): the capability is PRESENT on current HEAD — it landed via 293.4e-pre.ii
//! (`c62a817c`, generic surface-method call-site instantiation) + the Clause→ArgSpec heresy fix (`7d983012`),
//! both after this probe's original "RED at HEAD" framing. So the probe is GREEN and un-ignored. (It was briefly
//! RED only on a wrong assertion: the body wraps `x` (42), not the Id's tag.)

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A bare generic extend-impl whose body wraps the `:T` arg in `Box<T>` type-checks + dispatches.
#[test]
fn extend_impl_inherits_surface_method_types() {
    let world = startup_beside(file!())
        .expect("293.4e-pre.iii: a surface extend-impl with a bare generic body must type-check (inherit the surface sig)");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch :t::Maker/make to the :t::Id extend-impl and read the :t::Box")
    .value_owned();

    match got {
        // (:t::Maker/make (:t::Id 7) 42) → (:t::Box 42) → (:t::Box/v …) = 42.
        Value::i64(n) => assert_eq!(n, 42, "the extend-impl body (Box wrapping x=42) should yield 42; got {n}"),
        other => panic!("expected i64 42 from the bare generic extend-impl dispatch; got {other:?}"),
    }
}
