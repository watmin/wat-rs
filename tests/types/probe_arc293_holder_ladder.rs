//! RED probe — arc 293 K1a: the contravariant nature ladder for surface satisfaction.
//!
//! A required `:nature` is a FLOOR, not an exact kind: `:nature :Struct` accepts struct+record+holon,
//! `:nature :Record` accepts record+holon, `:nature :HolonRecord` accepts holon only.
//!
//! RED at HEAD: `check.rs:14698` does `agg_nature == req` (EXACT match) — a record is rejected by a
//! `:nature :Struct` surface, a holon by a `:nature :Record` surface, so the world fails to type-check.
//! GREEN once satisfaction uses `agg_nature.rank() >= req.rank()` (Struct -1 < Record 0 < HolonRecord +1).

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn nature_bound_satisfaction_is_a_contravariant_ladder() {
    match call_beside(file!(), ":lad::demo") {
        Ok(Value::String(s)) if &*s == "alice @ 100" => {}
        other => panic!("expected \"alice @ 100\" via the nature ladder; got {other:?}"),
    }
}
