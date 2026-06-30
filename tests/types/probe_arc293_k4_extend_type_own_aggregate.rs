//! Lock-in regression — arc 293 K4: `extend-type` is the GENERAL per-type satisfaction door.
//!
//! It binds method impls for ANY type, your OWN aggregates included — not just foreign builtins.
//! The "foreign-only adapter / monkeypatch / demoted" framing was DOCTRINAL: 293.4c built the
//! registration generically (it registers `:T/method` for any T, never gated to foreign), and
//! 293.4e-pre.i gave it the one canonical `ArgSpec`. K4 (the "un-demote") was therefore already
//! true in the substrate; this test PROVES it and GUARDS it.
//!
//! Why it matters: K5's `extend-surface` expands to `(extend-type S$record S …)`, and `S$record`
//! is an OWN aggregate (K2/K3-emitted) — so extend-type-on-own-types is K5's load-bearing seam.
//!
//! GREEN at HEAD (no RED phase — this is a lock-in regression, NOT a disconfirming probe, so it is
//! committed un-ignored and joins the floor immediately).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn extend_type_satisfies_a_surface_on_an_own_aggregate_type() {
    let world = startup_beside(file!())
        .expect("extend-type must bind method impls on an OWN aggregate type (the general per-type door)");
    let ast = wat::parse_one!("(:k4::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(25)) => {}
        other => panic!("expected 25 (3*3 + 4*4) via :k4::Located/mag2 dispatched to the own-type extend-type impl; got {other:?}"),
    }
}
