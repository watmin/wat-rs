//! Arc 296 Strike 3a probe — `#[derive(ToEdn)]` on `TypeErrorKind` is
//! byte-identical to the deleted hand-written serializer.
//!
//! Asserts that `wat_edn::write(&err.to_edn())` equals the pre-derive
//! golden EDN string for every `TypeErrorKind` variant (18 variants ×
//! 2 span states = 36 assertions). SET-diff ∅.
//!
//! ## How the golden strings were derived
//!
//! The old hand-written `impl ToEdn for TypeError` (deleted in Strike 3a)
//! produced `edn_tag(variant, Map(fields_in_declaration_order ++ span_if_known))`.
//! Each golden string was constructed by tracing that exact code path for the
//! chosen field values and the two span states.
//!
//! ## What this proves
//!
//! - The derive generates the same variant tag (`#wat.kernel/<Name>`).
//! - Snake→kebab key conversion (`enum_name` → `:enum-name`, `field_ty` → `:field-ty`).
//! - `:span` is appended LAST by `splice_span` when known.
//! - Unknown spans produce no `:span` key (elide-when-unknown discipline).
//! - `Vec<Remedy>` serializes identically to the deleted `remedies_to_edn` call
//!   (empty slice → `[]`, same as `Vec<T>::to_edn()` with empty vec).
//! - `usize` fields (`expected`, `got`) serialize as integers, matching `edn_int(*n as i64)`.

use std::sync::Arc;
use wat::to_edn::ToEdn;
use wat::span::Span;
use wat::types::error::{TypeError, TypeErrorKind};

// ─── Shared span fixtures ─────────────────────────────────────────────────────

fn known_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn unknown_span() -> Span {
    Span::unknown()
}

fn write(err: &TypeError) -> String {
    wat_edn::write(&err.to_edn())
}

fn make(span: Span, kind: TypeErrorKind) -> TypeError {
    TypeError { span, kind }
}

// ─── 1. DuplicateType ────────────────────────────────────────────────────────

#[test]
fn probe_duplicate_type_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::DuplicateType { name: ":user::Foo".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/DuplicateType {:name ":user::Foo" :span {:file "test.wat" :line 1 :col 0}}"#,
        "DuplicateType with known span"
    );
}

#[test]
fn probe_duplicate_type_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::DuplicateType { name: ":user::Foo".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/DuplicateType {:name ":user::Foo"}"#,
        "DuplicateType with unknown span"
    );
}

// ─── 2. ReservedPrefix ───────────────────────────────────────────────────────

#[test]
fn probe_reserved_prefix_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::ReservedPrefix { name: ":wat::reserved".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ReservedPrefix {:name ":wat::reserved" :span {:file "test.wat" :line 1 :col 0}}"#,
        "ReservedPrefix with known span"
    );
}

#[test]
fn probe_reserved_prefix_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::ReservedPrefix { name: ":wat::reserved".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ReservedPrefix {:name ":wat::reserved"}"#,
        "ReservedPrefix with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedDecl {:head "struct" :reason "bad form" :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedDecl with known span"
    );
}

#[test]
fn probe_malformed_decl_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::MalformedDecl {
            head: "struct".to_string(),
            reason: "bad form".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedDecl {:head "struct" :reason "bad form"}"#,
        "MalformedDecl with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedName {:raw "bad-name" :reason "missing prefix" :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedName with known span"
    );
}

#[test]
fn probe_malformed_name_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::MalformedName {
            raw: "bad-name".to_string(),
            reason: "missing prefix".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedName {:raw "bad-name" :reason "missing prefix"}"#,
        "MalformedName with unknown span"
    );
}

// ─── 5. MalformedField ───────────────────────────────────────────────────────

#[test]
fn probe_malformed_field_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::MalformedField { reason: "bad field".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedField {:reason "bad field" :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedField with known span"
    );
}

#[test]
fn probe_malformed_field_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::MalformedField { reason: "bad field".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedField {:reason "bad field"}"#,
        "MalformedField with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedVariant {:enum-name "MyEnum" :offending "BadVariant" :reason "not a keyword" :remedies [] :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedVariant with known span"
    );
}

#[test]
fn probe_malformed_variant_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::MalformedVariant {
            enum_name: "MyEnum".to_string(),
            offending: "BadVariant".to_string(),
            reason: "not a keyword".to_string(),
            remedies: vec![],
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedVariant {:enum-name "MyEnum" :offending "BadVariant" :reason "not a keyword" :remedies []}"#,
        "MalformedVariant with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedTypeExpr {:raw ":bad" :reason "unknown type" :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedTypeExpr with known span"
    );
}

#[test]
fn probe_malformed_type_expr_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::MalformedTypeExpr {
            raw: ":bad".to_string(),
            reason: "unknown type".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedTypeExpr {:raw ":bad" :reason "unknown type"}"#,
        "MalformedTypeExpr with unknown span"
    );
}

// ─── 8. AnyBanned ────────────────────────────────────────────────────────────

#[test]
fn probe_any_banned_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::AnyBanned { raw: ":Any".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/AnyBanned {:raw ":Any" :span {:file "test.wat" :line 1 :col 0}}"#,
        "AnyBanned with known span"
    );
}

#[test]
fn probe_any_banned_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::AnyBanned { raw: ":Any".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/AnyBanned {:raw ":Any"}"#,
        "AnyBanned with unknown span"
    );
}

// ─── 9. CyclicAlias ──────────────────────────────────────────────────────────

#[test]
fn probe_cyclic_alias_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::CyclicAlias { name: ":A".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicAlias {:name ":A" :span {:file "test.wat" :line 1 :col 0}}"#,
        "CyclicAlias with known span"
    );
}

#[test]
fn probe_cyclic_alias_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::CyclicAlias { name: ":A".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicAlias {:name ":A"}"#,
        "CyclicAlias with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/AliasArityMismatch {:name ":Pair" :expected 2 :got 1 :span {:file "test.wat" :line 1 :col 0}}"#,
        "AliasArityMismatch with known span"
    );
}

#[test]
fn probe_alias_arity_mismatch_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::AliasArityMismatch {
            name: ":Pair".to_string(),
            expected: 2,
            got: 1,
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/AliasArityMismatch {:name ":Pair" :expected 2 :got 1}"#,
        "AliasArityMismatch with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/InnerColonInCompoundArg {:raw ":Vec<:String>" :offending ":String" :span {:file "test.wat" :line 1 :col 0}}"#,
        "InnerColonInCompoundArg with known span"
    );
}

#[test]
fn probe_inner_colon_in_compound_arg_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::InnerColonInCompoundArg {
            raw: ":Vec<:String>".to_string(),
            offending: ":String".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/InnerColonInCompoundArg {:raw ":Vec<:String>" :offending ":String"}"#,
        "InnerColonInCompoundArg with unknown span"
    );
}

// ─── 12. CyclicUnion ─────────────────────────────────────────────────────────

#[test]
fn probe_cyclic_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::CyclicUnion { name: ":MyUnion".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicUnion {:name ":MyUnion" :span {:file "test.wat" :line 1 :col 0}}"#,
        "CyclicUnion with known span"
    );
}

#[test]
fn probe_cyclic_union_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::CyclicUnion { name: ":MyUnion".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicUnion {:name ":MyUnion"}"#,
        "CyclicUnion with unknown span"
    );
}

// ─── 13. EmptyUnion ──────────────────────────────────────────────────────────

#[test]
fn probe_empty_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::EmptyUnion { name: ":Empty".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/EmptyUnion {:name ":Empty" :span {:file "test.wat" :line 1 :col 0}}"#,
        "EmptyUnion with known span"
    );
}

#[test]
fn probe_empty_union_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::EmptyUnion { name: ":Empty".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/EmptyUnion {:name ":Empty"}"#,
        "EmptyUnion with unknown span"
    );
}

// ─── 14. SingleMemberUnion ───────────────────────────────────────────────────

#[test]
fn probe_single_member_union_known_span() {
    let err = make(
        known_span(),
        TypeErrorKind::SingleMemberUnion { name: ":Single".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/SingleMemberUnion {:name ":Single" :span {:file "test.wat" :line 1 :col 0}}"#,
        "SingleMemberUnion with known span"
    );
}

#[test]
fn probe_single_member_union_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::SingleMemberUnion { name: ":Single".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/SingleMemberUnion {:name ":Single"}"#,
        "SingleMemberUnion with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/InvalidUnionMember {:union-name ":MyUnion" :member-form "fn" :reason "fn not allowed" :span {:file "test.wat" :line 1 :col 0}}"#,
        "InvalidUnionMember with known span"
    );
}

#[test]
fn probe_invalid_union_member_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::InvalidUnionMember {
            union_name: ":MyUnion".to_string(),
            member_form: "fn".to_string(),
            reason: "fn not allowed".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/InvalidUnionMember {:union-name ":MyUnion" :member-form "fn" :reason "fn not allowed"}"#,
        "InvalidUnionMember with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicSubtype {:child ":A" :parent ":B" :span {:file "test.wat" :line 1 :col 0}}"#,
        "CyclicSubtype with known span"
    );
}

#[test]
fn probe_cyclic_subtype_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::CyclicSubtype {
            child: ":A".to_string(),
            parent: ":B".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CyclicSubtype {:child ":A" :parent ":B"}"#,
        "CyclicSubtype with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ImpureFieldInPureAggregate {:aggregate ":user::MyRecord" :field "handle" :field-ty ":user::HandleStruct" :span {:file "test.wat" :line 1 :col 0}}"#,
        "ImpureFieldInPureAggregate with known span"
    );
}

#[test]
fn probe_impure_field_in_pure_aggregate_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::ImpureFieldInPureAggregate {
            aggregate: ":user::MyRecord".to_string(),
            field: "handle".to_string(),
            field_ty: ":user::HandleStruct".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ImpureFieldInPureAggregate {:aggregate ":user::MyRecord" :field "handle" :field-ty ":user::HandleStruct"}"#,
        "ImpureFieldInPureAggregate with unknown span"
    );
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
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ImpureVariantFieldInPureEnum {:enum-name ":user::MyEnum" :variant "WithHandle" :field "handle" :field-ty ":user::HandleStruct" :span {:file "test.wat" :line 1 :col 0}}"#,
        "ImpureVariantFieldInPureEnum with known span"
    );
}

#[test]
fn probe_impure_variant_field_in_pure_enum_unknown_span() {
    let err = make(
        unknown_span(),
        TypeErrorKind::ImpureVariantFieldInPureEnum {
            enum_name: ":user::MyEnum".to_string(),
            variant: "WithHandle".to_string(),
            field: "handle".to_string(),
            field_ty: ":user::HandleStruct".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/ImpureVariantFieldInPureEnum {:enum-name ":user::MyEnum" :variant "WithHandle" :field "handle" :field-ty ":user::HandleStruct"}"#,
        "ImpureVariantFieldInPureEnum with unknown span"
    );
}
