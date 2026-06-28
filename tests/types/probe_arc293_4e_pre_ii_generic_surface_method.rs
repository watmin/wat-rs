//! Arc 293.4e-pre.ii — generic surface method members (the disconfirming probe).
//!
//! THE GAP: `parse_method_member_sig` (src/types/surface.rs) hardcodes `type_params: vec![]` and does NOT split the
//! `<T>` off the method name, so a generic member `(make<T> [self … x <- :T] -> :T)` is stored with name `"make<T>"`.
//! The call `:t::Maker/make` is `"make"` → no match → `unknown callee`. The protocol path splits via
//! `split_name_and_type_params`; the surface path must reach the same parity (arc-267 generic protocol methods).
//!
//! RED at HEAD. GREEN at 293.4e-pre.ii — unblocks the `:wat::spawn::Locus` migration (its `launch<S,R,St,Sh,Lu>`
//! is generic), i.e. the `defprotocol` annihilation (293.4e).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A generic surface method `(make<T> …)` dispatched on a record, instantiating `T = i64`, returns its arg (42).
#[test]
fn generic_surface_method_dispatches_with_type_params() {
    let world = startup_beside(file!())
        .expect("293.4e-pre.ii: a generic surface method `(make<T> …)` must type-check + dispatch");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch the generic :t::Maker/make (T=i64) to :t::Id/make")
    .value_owned();

    match got {
        Value::i64(n) => assert_eq!(n, 42, "the generic surface method should return its arg (T=i64); got {n}"),
        other => panic!("expected i64 42 from the generic surface-method dispatch; got {other:?}"),
    }
}
