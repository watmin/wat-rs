//! Arc 293.4e-pre.iii — extend-type impl on a surface inherits the surface method's types (the disconfirming probe).
//!
//! THE GAP: the 293.4c surface-extend scheme (check.rs:8954) is built from the bare impl clause's types (→ `nil`), not
//! the surface member's declared sig. The 293.4c probe used a CONSTANT body so it never bit; a typed body that uses
//! `self` and returns a non-nil value fails (`ReturnTypeMismatch expected :nil got <T>`; `self: :()`). This is the
//! `:wat::spawn::Locus` `launch` shape → it BLOCKS the `defprotocol` annihilation (293.4e).
//!
//! RED at HEAD. GREEN at 293.4e-pre.iii — the extend-impl scheme inherits the surface method's sig (self → the
//! extending type, args + ret from the member, type-params carried).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A bare extend-impl whose body uses `self` (typed) and returns a non-nil value type-checks + dispatches.
#[test]
#[ignore = "RED at HEAD: arc-293.4e-pre.iii (extend-type-for-surface impl inherits the surface method's types) \
            not built; un-ignore when GREEN — it unblocks the Locus migration / defprotocol annihilation (293.4e)"]
fn extend_impl_inherits_surface_method_types() {
    let world = startup_beside(file!())
        .expect("293.4e-pre.iii: a surface extend-impl with a typed body must type-check (inherit the surface sig)");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch :t::Maker/make to the :t::Id extend-impl and read the :t::Box")
    .value_owned();

    match got {
        Value::i64(n) => assert_eq!(n, 7, "the extend-impl body (Box wrapping the Id's tag) should yield 7; got {n}"),
        other => panic!("expected i64 7 from the typed extend-impl dispatch; got {other:?}"),
    }
}
