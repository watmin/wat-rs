//! RED probe — arc 293 K5: `extend-surface` (the LAST tool — default method impls over a surface).
//!
//! A wat `defmacro`: the user writes a TYPELESS method body; the macro emits one `extend-type` per PAIR
//! backing tier (`$core-record` + `$holon-record`), forwarding the typeless body. `extend-type` already
//! fills the method's types from the surface (the 293.4e-pre.iii capability, present on HEAD), so the
//! macro needs NO reflection seam — it is purely syntactic. Per the K5 decision (option A): the default
//! attaches to BOTH pair tiers, so a `to-record`'d value at either tier inherits it for free.
//!
//! RED at HEAD: `extend-surface` is unbound (no macro) — the world does not expand/load; the backing
//! records never get `dbl`, so they don't satisfy `:k5::HasX` and `:k5::HasX/dbl` rejects them.
//! GREEN after K5.
//!
//! STRIKE-READY: committed `#[ignore]`'d (RED) so the floor stays 0; un-ignore when K5 lands.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
#[ignore = "arc 293 K5 — RED until extend-surface (the defmacro emitting extend-type per pair tier) lands"]
fn extend_surface_default_rides_both_pair_tiers() {
    let world = startup_beside(file!())
        .expect("extend-surface must emit extend-type per pair tier so the default `dbl` rides $core-record + $holon-record");
    let ast = wat::parse_one!("(:k5::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(84)) => {}
        other => panic!("expected 84 (42 core + 42 holon) — the extend-surface default on both pair backing tiers; got {other:?}"),
    }
}
