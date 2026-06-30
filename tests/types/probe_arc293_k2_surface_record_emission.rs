//! RED probe — arc 293 K2: a `defsurface` emits its backing record `:S$record`.
//!
//! A surface's `:features` ATTRIBUTES (Field members; methods excluded — a record holds no functions)
//! become a concrete, registered `AggregateDef` named `:S$record`, holder = the surface's `:holder`.
//! This is `to-record`'s (K3) return type. `$` is a legal keyword char (confirmed empirically).
//!
//! RED at HEAD: `defsurface` emits only the SurfaceDef; `:k2::Pt$record` does not exist, so neither
//! its ctor nor its accessors resolve and the world fails to type-check. GREEN after K2.
//!
//! STRIKE-READY: committed `#[ignore]`'d (RED) so the floor stays 0; un-ignore when K2 lands.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[ignore = "arc 293 K2 — defsurface does not yet emit the :S$record backing type; un-ignore when K2 lands"]
#[test]
fn defsurface_emits_a_backing_record_from_its_attributes() {
    let world = startup_beside(file!())
        .expect("a defsurface must emit a concrete `:S$record` record from its `:features` attributes");
    let ast = wat::parse_one!("(:k2::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(7)) => {}
        other => panic!("expected 7 (3+4) from the emitted :k2::Pt$record; got {other:?}"),
    }
}
