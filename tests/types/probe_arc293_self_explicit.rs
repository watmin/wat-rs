//! RED probe — arc 293 K0c: the self-reference cycle-guard.
//!
//! A surface method's `self` is a normal typed binder `self <- :TheSurface` (the surface names itself).
//! At HEAD the satisfaction check recurses on the self-type (`surface.rs:83` re-enters satisfaction of
//! the surface) and STACK-OVERFLOWS. GREEN after K0c: position 0 (self) is skipped in the arg-type
//! comparison — self is the receiver, tautologically the surface, never re-checked.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn explicit_self_typed_as_the_surface_does_not_recurse() {
    match call_beside_value(file!(), ":se::demo") {
        Ok(Value::String(s)) if &*s == "bob" => {}
        other => panic!("expected \"bob\" via explicit-self satisfaction; got {other:?}"),
    }
}
