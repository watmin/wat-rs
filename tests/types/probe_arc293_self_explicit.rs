//! RED probe — arc 293 K0c: the self-reference cycle-guard.
//!
//! A surface method's `self` is a normal typed binder `self <- :TheSurface` (the surface names itself).
//! At HEAD the satisfaction check recurses on the self-type (`surface.rs:83` re-enters satisfaction of
//! the surface) and STACK-OVERFLOWS. GREEN after K0c: position 0 (self) is skipped in the arg-type
//! comparison — self is the receiver, tautologically the surface, never re-checked.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn explicit_self_typed_as_the_surface_does_not_recurse() {
    let world = startup_beside(file!())
        .expect("a surface method with explicit `self <- :TheSurface` must type-check, not stack-overflow");
    let ast = wat::parse_one!("(:se::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) if &*s == "bob" => {}
        other => panic!("expected \"bob\" via explicit-self satisfaction; got {other:?}"),
    }
}
