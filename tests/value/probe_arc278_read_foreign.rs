//! Stone A RED gate — `:wat::edn::read-foreign`, the dynamic EDN decode (the keystone).
//!
//! The consumer LACKS a tag's type, so `read-foreign` reconstructs an unknown tag as a
//! self-describing DYNAMIC value (ForeignRecord / ForeignVariant) instead of raising
//! `UnknownTag` — and it is RECURSIVE: a foreign record CONTAINING a foreign variant field
//! decodes all the way down. STRICT `read` on the SAME input still raises (the
//! no-hidden-failures floor, R41 EGO SVM LEX, is untouched).
//!
//! RED at HEAD `a5a48aa1`: `:wat::edn::read-foreign` + `ForeignRecord`/`ForeignVariant` +
//! their accessors do not exist, so the co-located fixture will not freeze and `call_beside_value`
//! fails on exactly that gap. GREEN when Stone A lands.
//!
//! The two asserts are the campaign's `DESIGN-STONE-A-read-foreign.md` acceptance:
//!   (1) a foreign record containing a foreign variant field round-trips through read-foreign;
//!   (2) strict `read` on the same input still errors UnknownTag.

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeErrorKind, Value};

/// (1) read-foreign reconstructs the nested-unknown EDN and navigates to the NESTED variant.
/// `:my::compute` returns the `:kind` field's variant name — proving read-foreign built a
/// ForeignRecord, `get` reached `:kind`, that field is itself a ForeignVariant (recursion),
/// and its variant is `:Click`. Exact assertion (the loose form can mask a bug; the exact one
/// unmasks it — arc-296 lesson): the whole recursive path must resolve to EXACTLY `:Click`.
#[test]
fn read_foreign_reconstructs_nested_foreign_variant() {
    let v = call_beside_value(file!(), ":my::compute")
        .expect("read-foreign navigation should return the nested variant name");
    assert_eq!(
        v,
        Value::wat__core__keyword(Arc::new(":Click".to_string())),
        "read-foreign should navigate #some.unknown/Rec {{:kind #some.unknown.Kind/Click [42]}} \
         down to the nested ForeignVariant and yield its variant keyword :Click"
    );
}

/// `ForeignRecord/get` of an absent key is `None`, never a raise — HashMap/get's contract.
#[test]
fn foreign_record_get_miss_is_none() {
    let v = call_beside_value(file!(), ":my::missing-field-is-none")
        .expect("missing-field get should return bool, not raise");
    assert_eq!(
        v,
        Value::bool(true),
        "ForeignRecord/get of an absent key must be None"
    );
}

/// Junk EDN is `:Malformed`, never a raise — `read-json`'s contract. Totality.
#[test]
fn read_foreign_malformed_does_not_raise() {
    let v = call_beside_value(file!(), ":my::malformed-is-malformed")
        .expect("malformed EDN must return ReadForeignOutcome::Malformed, not raise");
    assert_eq!(
        v,
        Value::bool(true),
        "read-foreign of junk must be :Malformed"
    );
}

/// (2) STRICT read on the SAME input STILL raises — the no-hidden-failures floor held (R41).
/// `expect_err` proves it raised (did not silently decode); the `matches!` pins it to the
/// read verb's malformed-form raise (exact kind, not a loose message substring).
#[test]
fn strict_read_still_errors_on_the_unknown_tag() {
    let err = call_beside_value(file!(), ":my::strict-errors")
        .expect_err("strict edn::read on an unknown tag must still raise, not decode");
    assert!(
        matches!(err.kind(), RuntimeErrorKind::MalformedForm { .. }),
        "strict read must raise a MalformedForm carrying the unknown-tag reason (the floor); \
         got kind {:?}",
        err.kind()
    );
}
