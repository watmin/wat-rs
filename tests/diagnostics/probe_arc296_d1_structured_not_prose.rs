//! Arc 296 D1 probe — embedded structured data travels as EDN, never prose.
//!
//! Verifies three acceptance contracts from DESIGN-296-D1:
//!
//! 1. `ReturnTypeMismatch`: `:remedies` is always a `Vector` (never a String,
//!    never absent). Empty remedies → `[]`; non-empty tested in-crate (see
//!    `src/remedy/mod.rs` `#[cfg(test)]` block — Remedy is pub(crate)).
//!
//! 2. `LoadError::Fetch` with `NotFound`: `:cause` is a `#wat.kernel/NotFound`
//!    tagged map (NOT a prose String like "load: file not found: /path").
//!
//! 3. `NoMatchingClauseAtCallSite`: `:called-arg-types` is a `Vector` (NOT a
//!    comma-joined String), and `:attempted-clauses` is present and non-nil
//!    (was previously DROPPED).
//!
//! RED → GREEN transitions:
//! - Probe 1 (empty remedies): RED = `:remedies` absent; GREEN = `:remedies []`
//! - Probe 2 (Fetch cause): RED = `:cause` is a String; GREEN = `:cause` tagged
//! - Probe 3 (NoMatchingClause): RED = `:called-arg-types` String + no `:attempted-clauses`;
//!   GREEN = both structured.

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind};
use wat::span::Span;
use wat::edn::contract::ToEdn;
use wat_edn::OwnedValue;

fn make_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 1)
}

// ─── Probe 1 — ReturnTypeMismatch :remedies is a Vector (even when empty) ────
//
// Integration tests cannot construct `Remedy` (pub(crate)), so the non-empty
// case lives in src/remedy/mod.rs #[cfg(test)]. This probe covers the
// structural contract: the field is always a Vector, never a String or absent.

#[test]
fn probe_1_return_type_mismatch_remedies_field_is_vector_not_prose() {
    // Empty remedies: before the fix, :remedies is NOT emitted at all;
    // after the fix, :remedies is emitted as [].
    let err = CheckError {
        span: make_span(),
        kind: CheckErrorKind::ReturnTypeMismatch {
            function: ":user::main".into(),
            expected: ":wat::core::nil".into(),
            got: ":wat::core::String".into(),
            remedies: vec![], // empty — constructable from outside crate
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    // :remedies must be a Vector field (even when empty); no prose "did you mean" blob.
    wat::assert_edn_matches_file!(s.clone(), "probe_arc296_d1_structured_not_prose__return_type_mismatch_remedies.edn", "ReturnTypeMismatch must always emit :remedies [] (never absent)");

    // The :remedies value must be a Vector, not a String.
    if let OwnedValue::Tagged(_, body) = &edn {
        if let OwnedValue::Map(fields) = body.as_ref() {
            let remedies_field = fields.iter().find(|(k, _)| {
                matches!(k, OwnedValue::Keyword(kw) if kw.name() == "remedies")
            });
            let (_, remedies_val) = remedies_field
                .expect("`:remedies` field must be present in ReturnTypeMismatch EDN");
            assert!(
                matches!(remedies_val, OwnedValue::Vector(_)),
                "`:remedies` must be OwnedValue::Vector, got: {:?}",
                remedies_val
            );
        }
    }

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 2 — LoadError::Fetch :cause is a tagged map, NOT a String ─────────
//
// Before fix: LoadFetchError serialized via .to_string() → prose String.
// After fix:  LoadFetchError.to_edn() → #wat.kernel/NotFound {:path "…"}.

#[test]
fn probe_2_load_fetch_error_cause_is_tagged_not_string() {
    use wat::{LoadError, LoadErrorKind, LoadFetchError};

    let err = LoadError::new(
        wat::rust_caller_span!(),
        LoadErrorKind::Fetch(LoadFetchError::NotFound("/no/such/file.wat".into())),
    );

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);
    eprintln!("=== probe_2 LoadError::Fetch(NotFound): {}", s);

    // Must NOT contain the prose form of LoadFetchError::Display.
    // rune:lint(loose-assert) — `s` is the EDN of a LoadError whose `:span` was set via `wat::rust_caller_span!()` at line 91 of this file; the span embeds the absolute host filesystem path to this test source file. The full EDN string varies by host. Targeted absence of the prose fallback form is the real contract.
    assert!(
        !s.contains("\"load:"),
        "`:cause` must NOT be a prose string 'load: file not found …'; got: {}",
        s
    );

    // Must contain the structured tagged form.
    // rune:lint(loose-assert) — same as above: `s` contains an absolute host path in the `:span` field from `rust_caller_span!()`; full string varies by host. Targeted presence of the structured tag is the real contract.
    assert!(
        s.contains("wat.kernel/NotFound"),
        "`:cause` must be `#wat.kernel/NotFound`; got: {}",
        s
    );

    // The :cause OwnedValue must be Tagged, not String.
    if let OwnedValue::Tagged(_, body) = &edn {
        if let OwnedValue::Map(fields) = body.as_ref() {
            let cause_field = fields.iter().find(|(k, _)| {
                matches!(k, OwnedValue::Keyword(kw) if kw.name() == "cause")
            });
            let (_, cause_val) = cause_field
                .expect("`:cause` field must be present in LoadError::Fetch EDN");
            assert!(
                matches!(cause_val, OwnedValue::Tagged(..)),
                "`:cause` must be OwnedValue::Tagged (not String); got: {:?}",
                cause_val
            );
            // The tag must be NotFound.
            if let OwnedValue::Tagged(tag, _) = cause_val {
                assert_eq!(
                    tag.name(), "NotFound",
                    "`:cause` tag must be 'NotFound'; got: {:?}",
                    tag.name()
                );
            }
        }
    }

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 3 — NoMatchingClauseAtCallSite: Vector types + attempted-clauses ──
//
// Before fix: :called-arg-types is comma-joined String; :attempted-clauses dropped.
// After fix:  :called-arg-types is a Vector; :attempted-clauses is a Vector.

#[test]
fn probe_3_no_matching_clause_at_call_site_is_structured() {
    let err = CheckError {
        span: make_span(),
        kind: CheckErrorKind::NoMatchingClauseAtCallSite {
            name: ":user::greet".into(),
            called_arity: 1,
            called_arg_types: vec![":wat::core::i64".into()],
            attempted_clauses: vec![
                (1, vec![":wat::core::String".into()]),
            ],
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    // :called-arg-types must be a Vector; :attempted-clauses must be present (was DROPPED before fix).
    wat::assert_edn_matches_file!(s.clone(), "probe_arc296_d1_structured_not_prose__no_matching_clause_at_call_site.edn", "NoMatchingClauseAtCallSite must emit structured :called-arg-types Vector + :attempted-clauses");

    // Detailed structural checks on the OwnedValue.
    if let OwnedValue::Tagged(_, body) = &edn {
        if let OwnedValue::Map(fields) = body.as_ref() {
            // :called-arg-types must be a Vector (not a comma-joined String).
            let cat_field = fields.iter().find(|(k, _)| {
                matches!(k, OwnedValue::Keyword(kw) if kw.name() == "called-arg-types")
            });
            let (_, cat_val) = cat_field
                .expect("`:called-arg-types` must be present");
            assert!(
                matches!(cat_val, OwnedValue::Vector(_)),
                "`:called-arg-types` must be a Vector, got: {:?}",
                cat_val
            );

            // :attempted-clauses must be a Vector and non-nil.
            let ac_field = fields.iter().find(|(k, _)| {
                matches!(k, OwnedValue::Keyword(kw) if kw.name() == "attempted-clauses")
            });
            let (_, ac_val) = ac_field
                .expect("`:attempted-clauses` must be present");
            assert!(
                matches!(ac_val, OwnedValue::Vector(_)),
                "`:attempted-clauses` must be a Vector, got: {:?}",
                ac_val
            );

            // Each clause element must have :arity and :param-types.
            if let OwnedValue::Vector(clauses) = ac_val {
                assert!(!clauses.is_empty(), "attempted-clauses must be non-empty");
                let clause_str = wat_edn::write(&clauses[0]);
                wat::assert_edn_matches_file!(clause_str, "probe_arc296_d1_structured_not_prose__clause_element.edn", "clause element must have :arity and :param-types");
            }
        }
    }

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}
