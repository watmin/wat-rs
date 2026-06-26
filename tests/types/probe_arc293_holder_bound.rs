//! Arc 293 — DISCONFIRMING PROBE for the `:holder` SURFACE BOUND (R3's `foobar` form).
//!
//! The landed surface (293.3-core/records) is PURELY STRUCTURAL: a `defsurface` is a set of
//! members, and any aggregate with those members satisfies it (width subtyping), REGARDLESS of
//! its holder. That is correct for the structural axis — but R3 (DESIGN § THE HOLDER × SURFACE
//! MODEL) adds the orthogonal CATEGORICAL axis: a surface may carry an optional `:holder` bound
//! that is enforced HARD. Holon-ness, like edn-portability, is a capability a structural shape
//! CANNOT fake — a core record with the same fields as a holon is still not a holon (no
//! `holon_form`). So a surface that requires `:holder :holon-record` must REJECT a core record
//! even when its fields match.
//!
//! THE GAP (RED at HEAD):
//!   1. `SurfaceDef` has no `holder` field (`types.rs:233`).
//!   2. `parse_defsurface` requires exactly `args.len() == 2` (name + member-vector); the
//!      `:holder :holon-record` clause yields 4 args → MalformedDecl ("got 4 args after head").
//!   3. The `assignable` surface arm (`check.rs:14229`) width-matches fields only — it never
//!      checks the holder.
//!
//! GREEN when the additive `:holder` layer lands: `SurfaceDef { holder: Option<Holder> }`,
//! `parse_defsurface` accepts the optional `:holder <kw>` clause, and the `assignable` arm
//! enforces `surf.holder == Some(h) ⇒ agg.holder == h` (categorical, hard).
//!
//! NOTE on the trap: the `:holder` clause makes `defsurface` a >2-arg form, so EVERY test here
//! errors with `MalformedDecl` at HEAD. The accept-case disconfirms cleanly (`is_ok` is false at
//! HEAD). The reject-case must assert on the HOLDER-MISMATCH REASON (the error cites the surface
//! `:env::Holon`), not merely `is_err` — else it would pass at HEAD for the wrong reason (a parse
//! error, not a categorical rejection). Mirrors `probe_arc293_holder_substitution.rs` case 4.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// THE ACCEPT CASE — a holon record satisfies a `:holder :holon-record` surface.
/// `:env::HEnv` is a holon record with the `slot` member, so it satisfies `:env::Holon`
/// both structurally (has `slot`) AND categorically (holder == HolonRecord).
#[test]
fn holon_record_satisfies_holder_bound_surface() {
    let src = r#"
        (:wat::core::defsurface :env::Holon
          :holder :holon-record
          [slot <- :wat::core::i64])
        (:wat::holon::Record::def :env::HEnv [slot <- :wat::core::i64])
        (:wat::core::defn :env::wants-holon [x <- :env::Holon] -> :wat::core::bool
          true)
        (:wat::core::defn :user::main [] -> :wat::core::bool
          (:env::wants-holon (:env::HEnv 1)))
    "#;
    // GREEN TARGET: HEnv is a holon record with `slot` ⇒ satisfies the holder-bound surface.
    // RED AT HEAD: `:holder` makes defsurface a >2-arg form ⇒ MalformedDecl, startup errors.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "a holon record should satisfy a :holder :holon-record surface it structurally fits; got: {:?}",
        world.err()
    );
}

/// THE REJECT CASE (the categorical bound) — a CORE record with the SAME members is REJECTED.
/// `:env::CEnv` has `slot` (structural match passes) but its holder is `Record`, not
/// `HolonRecord`, so the `:holder :holon-record` bound must reject it. The rejection must CITE
/// the surface (a holder mismatch), NOT be the incidental MalformedDecl parse error of HEAD.
#[test]
fn core_record_rejected_by_holon_holder_bound() {
    let src = r#"
        (:wat::core::defsurface :env::Holon
          :holder :holon-record
          [slot <- :wat::core::i64])
        (:wat::Record::def :env::CEnv [slot <- :wat::core::i64])
        (:wat::core::defn :env::wants-holon [x <- :env::Holon] -> :wat::core::bool
          true)
        (:wat::core::defn :user::main [] -> :wat::core::bool
          (:env::wants-holon (:env::CEnv 1)))
    "#;
    // GREEN TARGET: startup FAILS for the HOLDER mismatch — CEnv's fields match but it is a core
    // record, and `:holder :holon-record` is categorical. The error cites the surface `:env::Holon`.
    // RED AT HEAD: the error is a MalformedDecl on the `:holder` clause, which does NOT cite the
    // surface — so this assertion fails at HEAD (disconfirms on exactly the gap, not the parse).
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    // `world.err()` is `None` for an Ok world ⇒ formats as "None" ⇒ fails the cite check (an Ok
    // is wrong). A built holder-rejection cites `:env::Holon`; HEAD's MalformedDecl does not.
    let err = format!("{:?}", world.err());
    assert!(
        err.contains("env::Holon"),
        "a core record must NOT satisfy a :holder :holon-record surface (fields match but \
         holon-ness is categorical), and the rejection must cite the surface :env::Holon, \
         not be the incidental parse error. got: {err}"
    );
}
