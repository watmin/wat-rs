//! Arc 294.a — DISCONFIRMING PROBE for DIRECT-EDN MEASUREMENT.
//!
//! 294 thesis (R2's letting-go, R5's `coincident?`): EDN is canonical; the holon
//! measurement surface should ride on the DATA, not force you to name the derivation.
//! Today it does the opposite — `cosine`/`coincident?`/`presence?`/`simhash` accept only
//! `:wat::holon::HolonAST | :wat::Record | :wat::holon::Vector`, so a plain EDN value must
//! be hand-lifted via `(:wat::holon::to-holon …)` before it can be measured. That manual
//! lift IS the inversion in user-facing miniature.
//!
//! 294.a widens the surface to accept ANY `EdnRepresentable` value, lifting internally via
//! `to_holon_inner` — so `(:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 3})` Just Works. The
//! Holder wall holds: a `Struct` (non-portable) still cannot measure.
//!
//! RED at HEAD: the type-check (`infer_polymorphic_holon_pair_to_f64`, `check.rs:12854`,
//! gate `is_holon_or_vector`) rejects plain EDN collections — `(cosine {…} {…})` fails with
//! `parameter #1 expects :wat::holon::HolonAST, :wat::Record, or :wat::holon::Vector; got
//! HashMap<…>`. GREEN when the surface accepts `EdnRepresentable` and lifts internally.
//! (Proven RED manually this session via `target/release/wat`.)

use wat::freeze::startup_from_file;

/// A plain EDN MAP measures directly — `(cosine {:a 1 :b 2} {:a 1 :b 3})`, no manual `to-holon`.
#[test]
fn edn_map_measures_directly() {
    // GREEN TARGET: the map is lifted internally and measured (a cosine in [-1, 1]).
    // RED AT HEAD: type-check rejects HashMap<keyword,i64> at parameter #1 of :wat::holon::cosine.
    let world = startup_from_file("tests/types/probe_arc294a_edn_measures_directly_map.wat");
    assert!(
        world.is_ok(),
        "a plain EDN map should measure directly via :wat::holon::cosine (no manual to-holon); got: {:?}",
        world.err()
    );
}

/// A plain EDN VECTOR measures directly — `(cosine [1 2 3] [1 2 4])`.
#[test]
fn edn_vec_measures_directly() {
    // GREEN TARGET: the vec is lifted internally and measured.
    // RED AT HEAD: type-check rejects Vector<i64> at parameter #1 of :wat::holon::cosine.
    let world = startup_from_file("tests/types/probe_arc294a_edn_measures_directly_vec.wat");
    assert!(
        world.is_ok(),
        "a plain EDN vec should measure directly via :wat::holon::cosine; got: {:?}",
        world.err()
    );
}
