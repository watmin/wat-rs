//! Arc 296 Strike 3a probe — `#[derive(ToEdn)]` on `TypeErrorKind` is
//! byte-identical to the deleted hand-written serializer.
//!
//! Asserts that `wat_edn::write(&err.to_edn())` equals the pre-derive
//! golden EDN string for every `TypeErrorKind` variant (18 variants,
//! one deterministic-span assertion each). SET-diff ∅.
//!
//! Arc 298.2 note: the former per-variant `*_unknown_span` tests proved the
//! elide-when-span-unknown branch of the hand-written serializer. That branch
//! was annihilated with `Span::unknown()` — there is now exactly one code path
//! (always emit `:span`), so a second span state would be a byte-for-byte
//! duplicate of the `*_known_span` golden. The redundant tests were deleted.
//!
//! ## How the golden strings were derived
//!
//! The old hand-written `impl ToEdn for TypeError` (deleted in Strike 3a)
//! produced `edn_tag(variant, Map(fields_in_declaration_order ++ span_if_known))`.
//! Each golden string was constructed by tracing that exact code path for the
//! chosen field values and a fixed `test.wat` span.
//!
//! ## What this proves
//!
//! - The derive generates the same variant tag (`#wat.type/<Name>`).
//! - Snake→kebab key conversion (`enum_name` → `:enum-name`, `field_ty` → `:field-ty`).
//! - `:span` is appended LAST by `splice_span` when known.
//! - `:span` is ALWAYS emitted (arc 298.2 retired the elide-when-unknown branch).
//! - `Vec<Remedy>` serializes identically to the deleted `remedies_to_edn` call
//!   (empty slice → `[]`, same as `Vec<T>::to_edn()` with empty vec).
//! - `usize` fields (`expected`, `got`) serialize as integers, matching `edn_int(*n as i64)`.

use std::sync::Arc;
use wat::edn::contract::ToEdn;
use wat::span::Span;
use wat::types::error::{TypeError, TypeErrorKind};

// ─── Shared span fixtures ─────────────────────────────────────────────────────

fn known_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn write(err: &TypeError) -> String {
    wat_edn::write(&err.to_edn())
}

fn make(span: Span, kind: TypeErrorKind) -> TypeError {
    TypeError::new(span, kind)
}

// ─── 1. DuplicateType ────────────────────────────────────────────────────────

#[test]
fn probe_duplicate_type_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::DuplicateType { name: ":user::Foo".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__duplicate_type.edn", "DuplicateType with known span");
}

// ─── 2. ReservedPrefix ───────────────────────────────────────────────────────

#[test]
fn probe_reserved_prefix_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::ReservedPrefix { name: ":wat::reserved".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__reserved_prefix.edn", "ReservedPrefix with known span");
}

// ─── 3. MalformedDecl ────────────────────────────────────────────────────────

#[test]
fn probe_malformed_decl_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedDecl {
            head: "struct".to_string(),
            reason: "bad form".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__malformed_decl.edn", "MalformedDecl with known span");
}

// ─── 4. MalformedName ────────────────────────────────────────────────────────

#[test]
fn probe_malformed_name_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedName {
            raw: "bad-name".to_string(),
            reason: "missing prefix".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__malformed_name.edn", "MalformedName with known span");
}

// ─── 5. MalformedField ───────────────────────────────────────────────────────

#[test]
fn probe_malformed_field_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedField { reason: "bad field".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__malformed_field.edn", "MalformedField with known span");
}

// ─── 6. MalformedVariant ─────────────────────────────────────────────────────
//
// `remedies: Vec<Remedy>` with empty vec: the derive calls `remedies.to_edn()`
// via `impl<T: ToEdn> ToEdn for Vec<T>`, producing `OwnedValue::Vector(vec![])`
// which serializes as `[]` — identical to the deleted `remedies_to_edn(&[])`.

#[test]
fn probe_malformed_variant_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedVariant {
            enum_name: "MyEnum".to_string(),
            offending: "BadVariant".to_string(),
            reason: "not a keyword".to_string(),
            remedies: vec![],
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__malformed_variant.edn", "MalformedVariant with known span");
}

// ─── 7. MalformedTypeExpr ────────────────────────────────────────────────────

#[test]
fn probe_malformed_type_expr_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedTypeExpr {
            raw: ":bad".to_string(),
            reason: "unknown type".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__malformed_type_expr.edn", "MalformedTypeExpr with known span");
}

// ─── 8. AnyBanned ────────────────────────────────────────────────────────────

#[test]
fn probe_any_banned_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::AnyBanned { raw: ":Any".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__any_banned.edn", "AnyBanned with known span");
}

// ─── 9. CyclicAlias ──────────────────────────────────────────────────────────

#[test]
fn probe_cyclic_alias_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::CyclicAlias { name: ":A".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__cyclic_alias.edn", "CyclicAlias with known span");
}

// ─── 10. AliasArityMismatch ──────────────────────────────────────────────────
//
// `expected: usize` and `got: usize` — derive calls `usize::to_edn()` which
// returns `OwnedValue::Integer(*self as i64)`, matching the deleted `edn_int(*n as i64)`.

#[test]
fn probe_alias_arity_mismatch_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::AliasArityMismatch {
            name: ":Pair".to_string(),
            expected: 2,
            got: 1,
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__alias_arity_mismatch.edn", "AliasArityMismatch with known span");
}

// ─── 11. InnerColonInCompoundArg ─────────────────────────────────────────────

#[test]
fn probe_inner_colon_in_compound_arg_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::InnerColonInCompoundArg {
            raw: ":Vec<:String>".to_string(),
            offending: ":String".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__inner_colon_in_compound_arg.edn", "InnerColonInCompoundArg with known span");
}

// ─── 12. CyclicUnion ─────────────────────────────────────────────────────────

#[test]
fn probe_cyclic_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::CyclicUnion { name: ":MyUnion".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__cyclic_union.edn", "CyclicUnion with known span");
}

// ─── 13. EmptyUnion ──────────────────────────────────────────────────────────

#[test]
fn probe_empty_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::EmptyUnion { name: ":Empty".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__empty_union.edn", "EmptyUnion with known span");
}

// ─── 14. SingleMemberUnion ───────────────────────────────────────────────────

#[test]
fn probe_single_member_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::SingleMemberUnion { name: ":Single".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__single_member_union.edn", "SingleMemberUnion with known span");
}

// ─── 15. InvalidUnionMember ──────────────────────────────────────────────────

#[test]
fn probe_invalid_union_member_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::InvalidUnionMember {
            union_name: ":MyUnion".to_string(),
            member_form: "fn".to_string(),
            reason: "fn not allowed".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__invalid_union_member.edn", "InvalidUnionMember with known span");
}

// ─── 16. CyclicSubtype ───────────────────────────────────────────────────────

#[test]
fn probe_cyclic_subtype_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::CyclicSubtype {
            child: ":A".to_string(),
            parent: ":B".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__cyclic_subtype.edn", "CyclicSubtype with known span");
}

// ─── 17. ImpureFieldInPureAggregate ──────────────────────────────────────────

#[test]
fn probe_impure_field_in_pure_aggregate_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::ImpureFieldInPureAggregate {
            aggregate: ":user::MyRecord".to_string(),
            field: "handle".to_string(),
            field_ty: ":user::HandleStruct".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__impure_field_in_pure_aggregate.edn", "ImpureFieldInPureAggregate with known span");
}

// ─── 18. ImpureVariantFieldInPureEnum ────────────────────────────────────────

#[test]
fn probe_impure_variant_field_in_pure_enum_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::ImpureVariantFieldInPureEnum {
            enum_name: ":user::MyEnum".to_string(),
            variant: "WithHandle".to_string(),
            field: "handle".to_string(),
            field_ty: ":user::HandleStruct".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3a_typeerror_derive_identical__impure_variant_field_in_pure_enum.edn", "ImpureVariantFieldInPureEnum with known span");
}

