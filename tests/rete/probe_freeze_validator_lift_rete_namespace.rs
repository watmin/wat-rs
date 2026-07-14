//! Arc 294 9a follow-on — the `defrule` freeze wall was lifted off a hardcoded call in
//! `build_env` step 7.8 into a pluggable `FreezeValidator` extension point
//! (`src/freeze/validator.rs`), drained via `inventory::iter` — mirrors the existing
//! `RestrictionEntry` drain in the same fn (`src/restriction_entry.rs`). `validate_rete_rules`
//! itself is now the FIRST registered consumer (`src/rete/validate.rs`'s `inventory::submit!`);
//! its validation logic is unchanged, only its caller moved.
//!
//! This is the end-to-end proof, run through the FULL public [`startup_beside`] pipeline (not
//! a crate-internal `build_env` call, which the co-located unit tests in
//! `src/rete/validate.rs` already exercise directly): a corrupt `defrule` still freezes as
//! `StartupError::Validator`, and the boxed error's `to_edn()` STILL tags
//! `#wat.rete/MalformedClause` — the load-bearing property this lift depends on. A
//! `Box<dyn FreezeValidatorError>` is a generic carrier; it must not erase or re-tag the
//! concrete validator's own namespace.
//!
//! Fixture: `probe_freeze_validator_lift_rete_namespace.wat` (co-located) — the same
//! bare-keyword `:celsius` corruption `src/rete/validate.rs`'s
//! `corrupt_when_clause_is_a_located_error` test carries.

use wat::freeze::{startup_beside, StartupError};

#[test]
fn corrupt_defrule_through_full_pipeline_surfaces_as_validator_error_with_rete_namespace_intact() {
    let err = startup_beside(file!())
        .err()
        .expect("the injected bare-keyword clause must be a located freeze error");
    let boxed = match err {
        StartupError::Validator(e) => e,
        other => {
            panic!("expected StartupError::Validator (the FreezeValidator drain fired); got {other:?}")
        }
    };
    let edn = wat_edn::write(&boxed.to_edn());
    assert!(
        edn.contains("wat.rete/MalformedClause"),
        "boxed error must still tag #wat.rete/MalformedClause through dynamic dispatch \
         (the rete wall's own namespace, preserved through the generic box); got: {edn}"
    );
    assert!(
        edn.contains("alert::unattended"),
        "the located error must still name the offending rule; got: {edn}"
    );
}
