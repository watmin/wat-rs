//! RED probe — arc 293 K3: the THREE projection verbs (`to-struct` / core `to-record` / holon `to-record`).
//!
//! Projection is a FREE EXPLICIT tier choice — the surface's `:holder` governs *satisfaction* (who may be
//! passed to a `[x <- :S]` slot), NOT *projection* (what tier a `to-record` builds). One shared extraction
//! reads S's attributes off the satisfier; the three verbs differ only in the target holder, and a surface
//! emits all three backing records (`:S$struct` / `:S$core-record` / `:S$holon-record`).
//!
//! RED at HEAD: `to-struct`/`to-record` are unbound, and K2 emits only `:S$record` (not the triple) — so the
//! three projections and their `$struct`/`$core-record`/`$holon-record` accessors fail to resolve and the
//! world does not type-check. GREEN after K3.
//!
//! STRIKE-READY: committed `#[ignore]`'d (RED) so the floor stays 0; un-ignore when K3 lands.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
#[ignore = "arc 293 K3 — RED until the three projection verbs (to-struct / to-record core+holon) land"]
fn three_projection_verbs_materialize_a_surface_at_each_tier() {
    let world = startup_beside(file!())
        .expect("the three projection verbs must emit + populate :S$struct / :S$core-record / :S$holon-record");
    let ast = wat::parse_one!("(:k3::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(10)) => {}
        other => panic!("expected 10 (3+4+3) from the three projected backing records; got {other:?}"),
    }
}
