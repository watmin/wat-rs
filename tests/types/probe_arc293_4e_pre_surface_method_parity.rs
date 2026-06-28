//! Arc 293.4e-pre — surface-method dispatch must handle methods with args BEYOND `self` (the disconfirming probe).
//!
//! THE GAP (found by an examinare probe before drawing 293.4e): the surface-method machinery (293.4b dispatch /
//! check-side call typing, broadened in 293.4d) was only ever exercised with `[self]`-only method members. A method
//! with a second arg (`make [self x]`) fails the arity check — `:t::Maker/make: expected 3 argument(s); got 2`
//! (self double-counted). The generic form `make<T>` is worse — `unknown callee :t::Maker/make`.
//!
//! This BLOCKS 293.4e (annihilate `defprotocol`): `:wat::spawn::Locus`'s `launch<S,R,St,Sh,Lu>` has 6 args + 5 type
//! params; migrating it to `defsurface` needs multi-arg + generic surface-method parity with arc-267's generic
//! protocol methods FIRST.
//!
//! RED at HEAD. GREEN at 293.4e-pre.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A surface method with a second arg (`make [self x]`) must dispatch with the right arity and return the arg.
#[test]
#[ignore = "RED at HEAD: arc-293.4e-pre (surface methods with args beyond self / generics) not built; \
            un-ignore when the strike lands GREEN — it unblocks 293.4e (annihilate defprotocol)"]
fn surface_method_with_args_beyond_self_dispatches() {
    let world = startup_beside(file!())
        .expect("293.4e-pre: a surface method `(make [self x] …)` must type-check + dispatch with correct arity");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch :t::Maker/make (self + one arg) to :t::Id/make")
    .value_owned();

    match got {
        Value::i64(n) => assert_eq!(n, 42, "the 2-arg surface method should return its second arg; got {n}"),
        other => panic!("expected i64 42 from the multi-arg surface-method dispatch; got {other:?}"),
    }
}
