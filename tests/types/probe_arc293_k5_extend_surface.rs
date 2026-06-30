//! RED probe — arc 293 K5: `extend-surface` (the LAST tool — default method impls over a surface).
//!
//! A wat `defmacro`: the user writes a TYPELESS body; the macro reads the surface's declared method
//! sigs (via a new pure expand-time reflection seam — the one substrate dependency the model names),
//! fills the types, and expands to `extend-type`. Option A (co-design): the default attaches to ALL
//! THREE backing tiers (`$struct` / `$core-record` / `$holon-record`) — a to-record'd value at any
//! tier inherits the default for free.
//!
//! RED at HEAD: `extend-surface` is unbound (no macro, no surface-method-sig reflection seam) — the
//! world does not expand/load. GREEN after K5.
//!
//! STRIKE-READY: committed `#[ignore]`'d (RED) so the floor stays 0; un-ignore when K5 lands.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
#[ignore = "arc 293 K5 — RED until extend-surface (the defmacro + the surface-method-sig reflection seam) lands"]
fn extend_surface_default_attaches_to_all_three_backing_tiers() {
    let world = startup_beside(file!())
        .expect("extend-surface must fill types from the surface and register the default on all three backing tiers");
    let ast = wat::parse_one!("(:k5::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(44)) => {}
        other => panic!("expected 44 (8+13+23) — the extend-surface default firing on $struct/$core-record/$holon-record; got {other:?}"),
    }
}
