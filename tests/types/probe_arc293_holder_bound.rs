//! Arc 293 — DISCONFIRMING PROBE for the `:nature` SURFACE BOUND (R3's `foobar` form).
//!
//! The landed surface (293.3-core/records) is PURELY STRUCTURAL: a `defsurface` is a set of
//! members, and any aggregate with those members satisfies it (width subtyping), REGARDLESS of
//! its nature. That is correct for the structural axis — but R3 (DESIGN § THE HOLDER × SURFACE
//! MODEL) adds the orthogonal CATEGORICAL axis: a surface may carry an optional `:nature` bound
//! that is enforced HARD. Holon-ness, like edn-portability, is a capability a structural shape
//! CANNOT fake — a core record with the same fields as a holon is still not a holon (no
//! `holon_form`). So a surface that requires `:nature :holon-record` must REJECT a core record
//! even when its fields match.
//!
//! THE GAP (RED at HEAD):
//!   1. `SurfaceDef` has no `nature` field (`types.rs:233`).
//!   2. `parse_defsurface` requires exactly `args.len() == 2` (name + member-vector); the
//!      `:nature :wat::holon::Record` clause yields 4 args → MalformedDecl ("got 4 args after head").
//!   3. The `assignable` surface arm (`check.rs:14229`) width-matches fields only — it never
//!      checks the nature.
//!
//! GREEN when the additive `:nature` layer lands: `SurfaceDef { nature: Option<Nature> }`,
//! `parse_defsurface` accepts the optional `:nature <nature-root-symbol>` clause, and the `assignable` arm
//! enforces `surf.nature == Some(h) ⇒ agg.nature == h` (categorical, hard).
//!
//! NOTE on the trap: the `:nature` clause makes `defsurface` a >2-arg form, so EVERY test here
//! errors with `MalformedDecl` at HEAD. The accept-case disconfirms cleanly (`is_ok` is false at
//! HEAD). The reject-case must assert on the NATURE-MISMATCH REASON (the error cites the surface
//! `:env::Holon`), not merely `is_err` — else it would pass at HEAD for the wrong reason (a parse
//! error, not a categorical rejection). Mirrors `probe_arc293_holder_substitution.rs` case 4.

use wat::freeze::startup_from_file;

/// THE ACCEPT CASE — a holon record satisfies a `:nature :wat::holon::Record` surface.
/// `:env::HEnv` is a holon record with the `slot` member, so it satisfies `:env::Holon`
/// both structurally (has `slot`) AND categorically (nature == HolonRecord).
#[test]
fn holon_record_satisfies_nature_bound_surface() {
    // GREEN TARGET: HEnv is a holon record with `slot` ⇒ satisfies the nature-bound surface.
    // RED AT HEAD: `:nature` makes defsurface a >2-arg form ⇒ MalformedDecl, startup errors.
    let world = startup_from_file("tests/types/probe_arc293_holder_bound_accept.wat");
    assert!(
        world.is_ok(),
        "a holon record should satisfy a :nature :holon-record surface it structurally fits; got: {:?}",
        world.err()
    );
}

/// THE REJECT CASE (the categorical bound) — a CORE record with the SAME members is REJECTED.
/// `:env::CEnv` has `slot` (structural match passes) but its nature is `Record`, not
/// `HolonRecord`, so the `:nature :wat::holon::Record` bound must reject it. The rejection must CITE
/// the surface (a nature mismatch), NOT be the incidental MalformedDecl parse error of HEAD.
#[test]
fn core_record_rejected_by_holon_nature_bound() {
    // GREEN TARGET: startup FAILS for the NATURE mismatch — CEnv's fields match but it is a core
    // record, and `:nature :holon-record` is categorical. The error cites the surface `:env::Holon`.
    // RED AT HEAD: the error is a MalformedDecl on the `:nature` clause, which does NOT cite the
    // surface — so this assertion fails at HEAD (disconfirms on exactly the gap, not the parse).
    let world = startup_from_file("tests/types/probe_arc293_holder_bound_reject.wat");
    // A built nature-rejection cites `:env::Holon`; HEAD's MalformedDecl does not.
    let err = format!("{:?}", world.expect_err("expected a NATURE-mismatch rejection, not Ok"));
    wat::assert_edn_matches_file!(
        err,
        "probe_arc293_holder_bound__core_record_rejected_by_holon_nature_bound.edn",
        "nature-mismatch rejection cites :env::Holon (surface) vs :env::CEnv (got)"
    );
}
