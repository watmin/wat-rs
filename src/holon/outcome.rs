//! Outcome-enum constructors for the holon VSA algebra: `CosineOutcome`,
//! `DotOutcome`, `VectorDecodeOutcome`, `CombineOutcome`, `DegenerateSide`,
//! plus the shared rete-Fallback projection (`HolonReteProject`) these
//! outcomes feed. Pure functions lifted out of `runtime.rs` per Stone
//! HOME-8 — see `src/holon/mod.rs` for the two-layer doctrine.

use crate::runtime::{
    builtin_enum_variant_names, no_field_names, EnumValue, EvalBreak, RuntimeError,
    RuntimeErrorKind, Value,
};
use crate::span::Span;
use holon;
use std::sync::Arc;

/// Arc 278 the cosine outcome wall — type path of `:wat::holon::DegenerateSide`
/// (registered in `types.rs`). Diagnostic payload for `CosineOutcome::Degenerate`.
const DEGENERATE_SIDE_TYPE: &str = ":wat::holon::DegenerateSide";


/// `DegenerateSide::Target []` — the `target` (first) operand is a
/// zero-magnitude vector.
pub(crate) fn degenerate_side_target() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: DEGENERATE_SIDE_TYPE.into(),
        variant_name: "Target".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}


/// `DegenerateSide::Reference []` — the `reference` (second) operand is a
/// zero-magnitude vector.
pub(crate) fn degenerate_side_reference() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: DEGENERATE_SIDE_TYPE.into(),
        variant_name: "Reference".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}


/// `DegenerateSide::Both []` — both operands are zero-magnitude vectors.
pub(crate) fn degenerate_side_both() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: DEGENERATE_SIDE_TYPE.into(),
        variant_name: "Both".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}


/// Arc 278 the cosine outcome wall — type path of `:wat::holon::CosineOutcome`
/// (registered in `types.rs`).
const COSINE_OUTCOME_TYPE: &str = ":wat::holon::CosineOutcome";


/// `CosineOutcome::Similarity [similarity <- f64]` — the happy path, the raw
/// cosine clamped to `[-1, 1]`.
pub(crate) fn cosine_outcome_similarity(similarity: f64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COSINE_OUTCOME_TYPE.into(),
        variant_name: "Similarity".into(),
        names: builtin_enum_variant_names(COSINE_OUTCOME_TYPE, "Similarity"),
        fields: vec![Value::f64(similarity)],
    }))
}


/// `CosineOutcome::Degenerate [side <- DegenerateSide]` — one operand (or
/// both) is a zero-magnitude vector, so a direction — and therefore a cosine
/// — is undefined. Was the guarded `0.0` in `Similarity::cosine`, which reads
/// as "orthogonal, unrelated" in cosine's own codomain — a fabricated
/// answer, not a result.
pub(crate) fn cosine_outcome_degenerate(side: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COSINE_OUTCOME_TYPE.into(),
        variant_name: "Degenerate".into(),
        names: builtin_enum_variant_names(COSINE_OUTCOME_TYPE, "Degenerate"),
        fields: vec![side],
    }))
}


/// `CosineOutcome::DimensionMismatch [expected <- i64  got <- i64]` — the two
/// operands disagree in dimension. Was `pair_values_to_vectors`'s
/// `TypeMismatch` raise, now a domain fact.
pub(crate) fn cosine_outcome_dimension_mismatch(expected: i64, got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COSINE_OUTCOME_TYPE.into(),
        variant_name: "DimensionMismatch".into(),
        names: builtin_enum_variant_names(COSINE_OUTCOME_TYPE, "DimensionMismatch"),
        fields: vec![Value::i64(expected), Value::i64(got)],
    }))
}


/// Arc 278 the cosine outcome wall — type path of `:wat::holon::DotOutcome`
/// (registered in `types.rs`). Sibling to `CosineOutcome`, not a reuse: `dot`
/// performs no division (`Similarity::dot` sums `i8 × i8` products, bounded
/// by `d × 127²` — reaching ±Inf needs `d ≈ 10³⁰⁴`, closed), so a
/// zero-magnitude operand yields an HONEST `0.0` and `dot` gets no
/// `Degenerate` arm to construct.
const DOT_OUTCOME_TYPE: &str = ":wat::holon::DotOutcome";


/// `DotOutcome::Computed [product <- f64]` — the happy path.
pub(crate) fn dot_outcome_computed(product: f64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: DOT_OUTCOME_TYPE.into(),
        variant_name: "Computed".into(),
        names: builtin_enum_variant_names(DOT_OUTCOME_TYPE, "Computed"),
        fields: vec![Value::f64(product)],
    }))
}


/// `DotOutcome::DimensionMismatch [expected <- i64  got <- i64]` — the two
/// operands disagree in dimension. Was `pair_values_to_vectors`'s
/// `TypeMismatch` raise, now a domain fact — the same fact
/// `CosineOutcome::DimensionMismatch` carries, reached through the same
/// shared guard.
pub(crate) fn dot_outcome_dimension_mismatch(expected: i64, got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: DOT_OUTCOME_TYPE.into(),
        variant_name: "DimensionMismatch".into(),
        names: builtin_enum_variant_names(DOT_OUTCOME_TYPE, "DimensionMismatch"),
        fields: vec![Value::i64(expected), Value::i64(got)],
    }))
}


/// Shared rete Fallback projection of holon outcome enums.
/// `dispatch_rete_op` and native `CallFallback` both face the third failure
/// mode through this one arm — happy payload becomes `f64`, every other
/// named variant takes the caller's `:undefined`. The two enums do NOT
/// share variant/field names (`Similarity`/`similarity` vs `Computed`/`product`;
/// only cosine has `Degenerate`), so each is named explicitly. No `_`
/// wildcard on a recognised type_path.
pub(crate) enum HolonReteProject {
    /// Not a holon outcome enum — caller continues with other Fallback modes.
    NotHolon,
    Scalar(f64),
    Fallback,
}


pub(crate) fn project_holon_rete_fallback(
    v: &Value,
    head: &str,
    span: &Span,
) -> Result<HolonReteProject, EvalBreak> {
    match v {
        Value::Enum(ev) if ev.type_path == COSINE_OUTCOME_TYPE => {
            match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                ("Similarity", [Value::f64(similarity)]) => {
                    Ok(HolonReteProject::Scalar(*similarity))
                }
                ("Degenerate", [_]) | ("DimensionMismatch", [_, _]) => {
                    Ok(HolonReteProject::Fallback)
                }
                (variant, fields) => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: format!(
                            "rete Fallback arm's holon mode has no route for CosineOutcome::{variant} ({} field(s)) — add one before shipping this shape",
                            fields.len()
                        ),
                    },
                )
                .into()),
            }
        }
        Value::Enum(ev) if ev.type_path == DOT_OUTCOME_TYPE => {
            match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                ("Computed", [Value::f64(product)]) => Ok(HolonReteProject::Scalar(*product)),
                ("DimensionMismatch", [_, _]) => Ok(HolonReteProject::Fallback),
                (variant, fields) => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: format!(
                            "rete Fallback arm's holon mode has no route for DotOutcome::{variant} ({} field(s)) — add one before shipping this shape",
                            fields.len()
                        ),
                    },
                )
                .into()),
            }
        }
        _ => Ok(HolonReteProject::NotHolon),
    }
}


/// Arc 278 the dimension-heresy strike — the type path of `bytes-vector`'s
/// matchable decode outcome enum (`:wat::holon::VectorDecodeOutcome`,
/// registered in `types.rs`).
const VECTOR_DECODE_OUTCOME_TYPE: &str = ":wat::holon::VectorDecodeOutcome";


/// `VectorDecodeOutcome::Decoded [vector <- Vector]` — the happy path.
pub(crate) fn vector_decode_outcome_decoded(v: holon::Vector) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: VECTOR_DECODE_OUTCOME_TYPE.into(),
        variant_name: "Decoded".into(),
        names: builtin_enum_variant_names(VECTOR_DECODE_OUTCOME_TYPE, "Decoded"),
        fields: vec![Value::Vector(Arc::new(v))],
    }))
}


/// `VectorDecodeOutcome::DimensionMismatch [expected <- i64  got <- i64]` —
/// the wire header's dim disagrees with this program's constant `dim-count`.
pub(crate) fn vector_decode_outcome_dimension_mismatch(expected: i64, got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: VECTOR_DECODE_OUTCOME_TYPE.into(),
        variant_name: "DimensionMismatch".into(),
        names: builtin_enum_variant_names(VECTOR_DECODE_OUTCOME_TYPE, "DimensionMismatch"),
        fields: vec![Value::i64(expected), Value::i64(got)],
    }))
}


/// `VectorDecodeOutcome::TruncatedHeader [got <- i64]` — fewer than the
/// 4-byte dim header. No `expected` field — 4 is a protocol constant, not a
/// per-call datum.
pub(crate) fn vector_decode_outcome_truncated_header(got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: VECTOR_DECODE_OUTCOME_TYPE.into(),
        variant_name: "TruncatedHeader".into(),
        names: builtin_enum_variant_names(VECTOR_DECODE_OUTCOME_TYPE, "TruncatedHeader"),
        fields: vec![Value::i64(got)],
    }))
}


/// `VectorDecodeOutcome::LengthMismatch [expected <- i64  got <- i64]` — the
/// header's dim parsed fine, but the data bytes don't match `ceil(dim/4)`.
pub(crate) fn vector_decode_outcome_length_mismatch(expected: i64, got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: VECTOR_DECODE_OUTCOME_TYPE.into(),
        variant_name: "LengthMismatch".into(),
        names: builtin_enum_variant_names(VECTOR_DECODE_OUTCOME_TYPE, "LengthMismatch"),
        fields: vec![Value::i64(expected), Value::i64(got)],
    }))
}


/// `VectorDecodeOutcome::InvalidCell [at <- i64]` — a 2-bit cell decoded to
/// the reserved `0b11` pattern at cell index `at`.
pub(crate) fn vector_decode_outcome_invalid_cell(at: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: VECTOR_DECODE_OUTCOME_TYPE.into(),
        variant_name: "InvalidCell".into(),
        names: builtin_enum_variant_names(VECTOR_DECODE_OUTCOME_TYPE, "InvalidCell"),
        fields: vec![Value::i64(at)],
    }))
}


/// Arc 278 the dimension-heresy strike, part 2 — the type path of the
/// matchable outcome enum shared by `vector-bind` / `vector-bundle` /
/// `vector-blend` (`:wat::holon::CombineOutcome`, registered in `types.rs`).
/// ONE shared enum, not three per-verb siblings — the three verbs' outcome
/// spaces are identical (`[expected, got]` on disagreement), unlike
/// `RecvOutcome`/`SendOutcome`/`TrySendOutcome` whose split is earned by a
/// genuine shape difference.
const COMBINE_OUTCOME_TYPE: &str = ":wat::holon::CombineOutcome";


/// `CombineOutcome::Combined [vector <- Vector]` — the happy path (bind's
/// XOR-compose / bundle's superposition / blend's weighted linear
/// combination — one shape of success across all three verbs).
pub(crate) fn combine_outcome_combined(v: holon::Vector) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COMBINE_OUTCOME_TYPE.into(),
        variant_name: "Combined".into(),
        names: builtin_enum_variant_names(COMBINE_OUTCOME_TYPE, "Combined"),
        fields: vec![Value::Vector(Arc::new(v))],
    }))
}


/// `CombineOutcome::DimensionMismatch [expected <- i64  got <- i64]` — the
/// operands disagree. Deliberately the same variant name as
/// `VectorDecodeOutcome::DimensionMismatch` (one fact, two routes) — but
/// neither vector here is "foreign": both are ordinary in-program values
/// that simply disagree.
pub(crate) fn combine_outcome_dimension_mismatch(expected: i64, got: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COMBINE_OUTCOME_TYPE.into(),
        variant_name: "DimensionMismatch".into(),
        names: builtin_enum_variant_names(COMBINE_OUTCOME_TYPE, "DimensionMismatch"),
        fields: vec![Value::i64(expected), Value::i64(got)],
    }))
}


::wat_source_derive::wat_field_names_from!(
    COINCIDENT_EXPLANATION_FIELDS,
    "wat/holon.wat",
    ":wat::holon::CoincidentExplanation"
);
pub(crate) fn coincident_explanation_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(COINCIDENT_EXPLANATION_FIELDS))
        .clone()
}

