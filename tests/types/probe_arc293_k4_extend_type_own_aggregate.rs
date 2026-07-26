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

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn extend_type_satisfies_a_surface_on_an_own_aggregate_type() {
    match call_beside_value(file!(), ":k4::demo") {
        Ok(Value::i64(25)) => {}
        other => panic!("expected 25 (3*3 + 4*4) via :k4::Located/mag2 dispatched to the own-type extend-type impl; got {other:?}"),
    }
}
