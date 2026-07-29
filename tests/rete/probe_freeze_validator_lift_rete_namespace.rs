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
        .expect_err("the injected bare-keyword clause must be a located freeze error");
    let boxed = match err {
        StartupError::Validator(e) => e,
        other => {
            panic!("expected StartupError::Validator (the FreezeValidator drain fired); got {other:?}")
        }
    };
    let edn = wat_edn::write(&boxed.to_edn());

    // Assert the PARSED tag, not a substring of the rendering. The claim IS the tag — "a
    // Box<dyn FreezeValidatorError> must not erase or re-tag the concrete validator's own
    // namespace" — so state it exactly: `contains` would also pass if the tag were merely
    // MENTIONED inside some other structure the box had wrapped it in, which is precisely
    // the failure this probe exists to catch.
    let (tag, fields) = first_tagged_error(&edn);
    assert_eq!(
        tag,
        wat_edn::Tag::ns("wat.rete", "MalformedClause"),
        "boxed error must still tag #wat.rete/MalformedClause through dynamic dispatch \
         (the rete wall's own namespace, preserved through the generic box); got: {edn}"
    );
    assert_eq!(
        field_str(&fields, "rule"),
        "alert::unattended",
        "the located error must still name the offending rule; got: {edn}"
    );
}

/// The boxed error's rendered EDN, parsed: the FIRST error's variant tag + field map.
///
/// The boxed error carries no `Any` bound (a multi-consumer registry has no reason to let a
/// caller downcast to one validator's concrete type), so the wire EDN is the only face
/// available — but parsing it lets the assertions be EXACT. A whole-blob golden is the wrong
/// tool: each error carries a `:span` into the co-located fixture, and the batch is a
/// `#wat.rete/ReteCheckErrors` wrapper whose shape is not this probe's subject.
fn first_tagged_error(edn: &str) -> (wat_edn::Tag, Vec<(wat_edn::OwnedValue, wat_edn::OwnedValue)>) {
    use wat_edn::{Keyword, OwnedValue, Tag};
    let parsed = wat_edn::parse_owned(edn).expect("the wall's error face must be EDN");
    let errors = match parsed {
        OwnedValue::Tagged(tag, body) => {
            assert_eq!(tag, Tag::ns("wat.rete", "ReteCheckErrors"), "outer batch tag");
            match *body {
                OwnedValue::Map(m) => m
                    .into_iter()
                    .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new("errors")))
                    .map(|(_, v)| v)
                    .expect("the batch must carry :errors"),
                other => panic!("expected a map body; got {other:?}"),
            }
        }
        other => panic!("expected a tagged #wat.rete/ReteCheckErrors batch; got {other:?}"),
    };
    let first = match errors {
        OwnedValue::Vector(mut xs) if !xs.is_empty() => xs.remove(0),
        other => panic!("expected a non-empty :errors vector; got {other:?}"),
    };
    match first {
        OwnedValue::Tagged(tag, body) => match *body {
            OwnedValue::Map(m) => (tag, m),
            other => panic!("expected a map body; got {other:?}"),
        },
        other => panic!("expected a tagged error; got {other:?}"),
    }
}

/// Read one field of a parsed error as a String.
fn field_str(fields: &[(wat_edn::OwnedValue, wat_edn::OwnedValue)], name: &str) -> String {
    use wat_edn::{Keyword, OwnedValue};
    let v = fields
        .iter()
        .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new(name)))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("the error must carry :{name}"));
    match v {
        OwnedValue::String(s) => s.to_string(),
        other => panic!(":{name} must be a String; got {other:?}"),
    }
}
