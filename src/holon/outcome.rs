//! Outcome-enum constructors for the holon VSA algebra: `CosineOutcome`,
//! `DotOutcome`, `VectorDecodeOutcome`, `CombineOutcome`, `DegenerateSide`,
//! the value-in measurement functions that build `CosineOutcome`/`DotOutcome`
//! (`cosine_outcome_from_values`, `dot_outcome_from_values`, and their shared
//! `pair_values_to_vectors` plumbing), plus the shared rete-Fallback
//! projection/classification (`HolonReteProject`/`project_holon_rete_fallback`,
//! `FallbackVerdict`/`classify_fallback_outcome`) these outcomes feed. Functions
//! lifted out of `runtime.rs` — see `src/holon/mod.rs` for the doctrine.

// `no_field_names`/`builtin_enum_variant_names` and `program_dim`/
// `require_encoding_ctx` are genuinely defined in `crate::runtime` (not
// facade re-exports of `crate::value` types — see STOP-2): the first pair
// is the generic `Value`/`EnumValue` field-name machinery shared by ten and
// seven OTHER impl homes (not holon's to own); the second pair is ambient
// program config (dimension, encoding context) that stays in `runtime.rs`.
use crate::runtime::{
    builtin_enum_variant_names, no_field_names, program_dim, require_encoding_ctx,
};

// `to_holon_inner` is `crate::holon::ast::to_holon_inner`, re-exported at
// `crate::holon` (the `ast` submodule itself is private) — the canonical
// path, not a facade.
use crate::holon::to_holon_inner;

use crate::span::Span;
use crate::value::{
    EnumValue, EvalBreak, HolonForm, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use holon;
use holon::{encode, Similarity, DEGENERATE_EPSILON};
use std::sync::Arc;

/// Arc 052 — polymorphic-input helper for cosine/dot. Accepts
/// HolonAST or Vector in either position; returns a (Vector, Vector)
/// pair at a consistent d.
///
/// Dimension-resolution rule:
/// - Both Vector → dims must match; returns the shared dim.
/// - Both AST → dim from ambient `pick_d_for_pair` (arc 037 router).
/// - Mixed (one AST, one Vector) → use the Vector's dim; encode the
///   AST at that dim.
///
/// Cross-dim Vector pairs error with `TypeMismatch`. There's no
/// auto-promotion: a raw Vector at d=10000 has no source AST to
/// re-encode at d=4096; the caller must produce matching-dim inputs.
/// The measured outcome of resolving a `(Value, Value)` pair to a
/// same-dimension `(Vector, Vector)` pair. Arc 278 the cosine outcome wall
/// (`BRIEF-cosine-outcome-wall.md`) — a dimension disagreement between two
/// operands used to make this helper RAISE a `TypeMismatch`, uncatchable,
/// unwinding past every caller alike. It is now a domain fact this enum
/// carries, so each of the five callers (`cosine`, `dot`, `coincident?`,
/// `presence?` — which never reaches this helper at all, see its own
/// comment — and `coincident-explain`) decides for itself what to DO with a
/// mismatch, per its own return-shape contract. A value that is neither
/// Vector nor HolonAST/Record still raises via this function's `Err` arm —
/// that is a call-site type bug, not a domain hole this wall covers.
pub(crate) enum PairedVectors {
    Paired(holon::Vector, holon::Vector),
    DimensionMismatch { expected: i64, got: i64 },
}

pub(crate) fn pair_values_to_vectors(
    op: &'static str,
    a: Value,
    b: Value,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<PairedVectors, EvalBreak> {
    let ctx = require_encoding_ctx(op, sym, list_span)?;
    // Arc 234 Stone 234.5 — D3: normalize Value::Aggregate(HolonRecord) → HolonAST before dispatch.
    // HolonRecord carries a pre-built hologram; coerce both sides so the existing
    // HolonAST arms handle them. Vector and HolonAST pass through unchanged.
    //
    // Arc 294.a — widen: any other value is lifted via to_holon_inner (which accepts
    // any EdnRepresentable value — plain maps, vectors, scalars, etc. — and errors
    // honestly on non-EDN types like Struct or resources). The base-record reject
    // ("has no holon flavor") dies into to_holon_inner's own honest error for now;
    // to_holon_inner must be extended to lift base records (STOP-1 gap, 294.a report).
    // Arc 293.R2.1 — HolonRecord exposes hologram; else lift via to_holon_inner.
    let normalize_for_cosine = |v: Value, span: &Span| -> Result<Value, EvalBreak> {
        match v {
            Value::Aggregate(ref a) => match &a.holon {
                HolonForm::Hologram(h) => Ok(Value::holon__HolonAST(h.clone())),
                HolonForm::Empty => to_holon_inner(v, span),
            },
            Value::holon__HolonAST(h) => Ok(Value::holon__HolonAST(h)),
            Value::Vector(v) => Ok(Value::Vector(v)),
            other => to_holon_inner(other, span),
        }
    };
    let a = normalize_for_cosine(a, list_span)?;
    let b = normalize_for_cosine(b, list_span)?;
    match (a, b) {
        (Value::Vector(va), Value::Vector(vb)) => {
            if va.dimensions() != vb.dimensions() {
                return Ok(PairedVectors::DimensionMismatch {
                    expected: va.dimensions() as i64,
                    got: vb.dimensions() as i64,
                });
            }
            Ok(PairedVectors::Paired(
                va.as_ref().clone(),
                vb.as_ref().clone(),
            ))
        }
        (Value::Vector(va), Value::holon__HolonAST(b)) => {
            let d = va.dimensions();
            let enc = ctx.encoders.get(d);
            let vb = encode(&b, &enc.vm, &enc.scalar);
            Ok(PairedVectors::Paired(va.as_ref().clone(), vb))
        }
        (Value::holon__HolonAST(a), Value::Vector(vb)) => {
            let d = vb.dimensions();
            let enc = ctx.encoders.get(d);
            let va = encode(&a, &enc.vm, &enc.scalar);
            Ok(PairedVectors::Paired(va, vb.as_ref().clone()))
        }
        (Value::holon__HolonAST(a), Value::holon__HolonAST(b)) => {
            let d = program_dim(op, sym, list_span)?;
            let enc = ctx.encoders.get(d);
            let va = encode(&a, &enc.vm, &enc.scalar);
            let vb = encode(&b, &enc.vm, &enc.scalar);
            Ok(PairedVectors::Paired(va, vb))
        }
        (a, _) => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::HolonAST, wat::core::Record, or wat::holon::Vector",
                got: Box::new(ValueSnapshot::of(&a)),
                // arc 138: no per-arg AST span (takes a Value pair) — list_span (the call site) used instead
            },
        )
        .into()),
    }
}

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


/// Value-in cosine. Shared by `eval_algebra_cosine` (AST eval) and native
/// `apply_op` (compiled `where`). One measurement; two mouths.
pub(crate) fn cosine_outcome_from_values(
    a: Value,
    b: Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let (vt, vr) = match pair_values_to_vectors(":wat::holon::cosine", a, b, sym, list_span)? {
        PairedVectors::DimensionMismatch { expected, got } => {
            return Ok(cosine_outcome_dimension_mismatch(expected, got));
        }
        PairedVectors::Paired(vt, vr) => (vt, vr),
    };
    // Arc 278 the cosine outcome wall — face the zero-magnitude case BEFORE
    // calling `Similarity::cosine`, rather than letting holon-rs's guarded
    // `0.0` sail through as data. `holon::Vector::norm()` is pub for exactly
    // this: test each operand's own norm, decide which side (if any) is
    // degenerate, and answer with the fact instead of a fabricated measurement.
    let (na, nb) = (vt.norm(), vr.norm());
    let side = match (na < DEGENERATE_EPSILON, nb < DEGENERATE_EPSILON) {
        (true, true) => Some(degenerate_side_both()),
        (true, false) => Some(degenerate_side_target()),
        (false, true) => Some(degenerate_side_reference()),
        (false, false) => None,
    };
    match side {
        Some(side) => Ok(cosine_outcome_degenerate(side)),
        None => {
            // Clamp to [-1, 1]: cosine similarity is mathematically bounded to
            // this range, but floating-point arithmetic can produce values
            // slightly outside (e.g., 1.0000000000000002 for identical
            // vectors). Clamping is the honest substrate-level fix — the VSA
            // semantics are defined on [-1, 1].
            Ok(cosine_outcome_similarity(
                Similarity::cosine(&vt, &vr).clamp(-1.0, 1.0),
            ))
        }
    }
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


/// Value-in `dot`. Shared by AST eval and native `apply_op`.
pub(crate) fn dot_outcome_from_values(
    a: Value,
    b: Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let (vx, vy) = match pair_values_to_vectors(":wat::holon::dot", a, b, sym, list_span)? {
        PairedVectors::DimensionMismatch { expected, got } => {
            return Ok(dot_outcome_dimension_mismatch(expected, got));
        }
        PairedVectors::Paired(vx, vy) => (vx, vy),
    };
    Ok(dot_outcome_computed(Similarity::dot(&vx, &vy)))
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

/// What a fallback-carrying op's outcome MEANS for this row: use the value, or take
/// the caller's `:undefined` expression.
///
/// The recursion is deliberately NOT in here. Each caller reaches its fallback
/// expression differently — `eval_inner` in the core evaluator, `exec_dim` in the
/// where-tree, `exec` in the compiled-expression walk — and that is the caller's own
/// business. What must NOT differ, and used to, is the CLASSIFICATION.
pub(crate) enum FallbackVerdict {
    /// Use this value as the op's result.
    Value(Value),
    /// This row reached its undefined point — evaluate the caller's `:undefined` arg.
    UseFallback,
}

/// THE one classification of a fallback-carrying op's outcome. Was written by hand
/// three times (`runtime.rs`, `where_tree.rs`'s `exec_dim`, `expr_ir.rs`'s `exec`),
/// and the copies DIVERGED: only this one guarded on the row's declared `ret`, so a
/// generic-`ret` row (`get`/`first`, `ret: Var("T")`) returning a non-finite float
/// took the fallback in the rete paths and not here — native answering `1` where the
/// `$oracle` answered `0`, on a total op. Gated by
/// `tests/rete/probe_arc278_fallback_generic_ret`.
///
/// A fallback-carrying op is TOTAL, so it must face EVERY way its core op reaches its
/// undefined point, and the families reach it differently:
///
/// 1. **A non-finite `f64`, and ONLY when the row DECLARES `ret: F64`.** The f64
///    arithmetic family fails by RETURNING — `eval_f64_arith` is raw IEEE 754 with no
///    overflow guard, so a domain failure surfaces as an `Ok` holding NaN or ±Inf,
///    never an `Err`. Decided from the row's declared `ret`, NEVER by sniffing the
///    runtime value's type: a value-sniff silently changes behaviour for any row that
///    returns a float for a non-arithmetic reason, and six such rows already exist.
///    `!is_finite()` is exactly the predicate — true for NaN, +Inf, -Inf and nothing
///    else; ordinary finites, `-0.0` and subnormals pass through.
/// 2. **`Option::None`** — an op that reports absence by `Option` (`get` today; any
///    future `(Option :- [T])`-returning verb). `Value::Option` is NOT `Value::Enum`,
///    so the holon projection below cannot fire on it; it needs its own arm.
/// 3. **A holon outcome enum** whose variant means degenerate/mismatch. `Scalar`
///    unwraps to its number and `NotHolon` passes the value through untouched — only
///    the middle case is a fallback.
/// 4. **`IntegerOverflow` / `DivisionByZero`** — the i64 arithmetic family fails by
///    RAISING. With args already checked as (i64, i64) this is EXHAUSTIVE for that
///    family, not a catch-all: a type or arity error is a caller bug and propagates.
/// 5. **`MalformedForm` whose head is this op's own `core_name`** — the sequence
///    accessors (`first`, `eval_positional_accessor`) fail by raising on an empty
///    container. The head test is what keeps this narrow: a `MalformedForm` raised
///    DEEPER carries that callee's head, and the `:undefined`-marker check raises with
///    the RETE name, so both stay structurally distinguishable and both propagate.
///
/// Everything else propagates. Widening any of the five turns a real error into a
/// silently-substituted value, which is the one failure this shape exists to prevent.
pub(crate) fn classify_fallback_outcome(
    outcome: Result<Value, EvalBreak>,
    ret: &crate::rete::vocabulary::ParamType,
    core_name: &str,
    holon_name: &str,
    span: &Span,
) -> Result<FallbackVerdict, EvalBreak> {
    match outcome {
        Ok(Value::f64(x))
            if matches!(ret, crate::rete::vocabulary::ParamType::F64) && !x.is_finite() =>
        {
            Ok(FallbackVerdict::UseFallback)
        }
        Ok(Value::Option(opt)) => match opt.as_ref() {
            Some(v) => Ok(FallbackVerdict::Value(v.clone())),
            None => Ok(FallbackVerdict::UseFallback),
        },
        Ok(v) => match project_holon_rete_fallback(&v, holon_name, span)? {
            HolonReteProject::Scalar(x) => Ok(FallbackVerdict::Value(Value::f64(x))),
            HolonReteProject::Fallback => Ok(FallbackVerdict::UseFallback),
            HolonReteProject::NotHolon => Ok(FallbackVerdict::Value(v)),
        },
        Err(EvalBreak::Diagnostic(e))
            if matches!(
                e.kind(),
                RuntimeErrorKind::IntegerOverflow { .. } | RuntimeErrorKind::DivisionByZero
            ) =>
        {
            Ok(FallbackVerdict::UseFallback)
        }
        Err(EvalBreak::Diagnostic(e))
            if matches!(
                e.kind(),
                RuntimeErrorKind::MalformedForm { head, .. } if head.as_str() == core_name
            ) =>
        {
            Ok(FallbackVerdict::UseFallback)
        }
        Err(e) => Err(e),
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

