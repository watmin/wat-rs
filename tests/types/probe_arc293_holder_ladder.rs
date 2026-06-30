//! RED probe — arc 293 K1a: the contravariant holder ladder for surface satisfaction.
//!
//! A required `:holder` is a FLOOR, not an exact kind: `:holder :Struct` accepts struct+record+holon,
//! `:holder :Record` accepts record+holon, `:holder :HolonRecord` accepts holon only.
//!
//! RED at HEAD: `check.rs:14698` does `agg_holder == req` (EXACT match) — a record is rejected by a
//! `:holder :Struct` surface, a holon by a `:holder :Record` surface, so the world fails to type-check.
//! GREEN once satisfaction uses `agg_holder.rank() >= req.rank()` (Struct -1 < Record 0 < HolonRecord +1).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn holder_bound_satisfaction_is_a_contravariant_ladder() {
    let world = startup_beside(file!())
        .expect("a record satisfies a :holder :Struct surface; a holon satisfies a :holder :Record surface");
    let ast = wat::parse_one!("(:lad::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) if &*s == "alice @ 100" => {}
        other => panic!("expected \"alice @ 100\" via the holder ladder; got {other:?}"),
    }
}
