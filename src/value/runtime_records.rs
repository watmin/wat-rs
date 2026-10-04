//! Excursus 003 envelope step 3a — `RuntimeError::to_record`: convert a
//! `RuntimeError` into a value of its declared `:wat::runtime::<Kind>` record
//! (`wat/runtime-errors.wat`), instead of the untyped tagged EDN the
//! `#[derive(ToEdn)]` on `RuntimeErrorKind` still writes (unchanged — that
//! derive is the wire until step 3b).
//!
//! Field names for every generated record/enum-variant come from the wat
//! declaration itself (`wat_field_names_from!` / `wat_enum_field_names_from!`),
//! never hand-typed — same discipline as `crate::runtime::fault_from_runtime_error`
//! / `value_from_span`, which this module reuses rather than re-implements.
//!
//! `to_record`'s match is EXHAUSTIVE with no `_` arm: a new `RuntimeErrorKind`
//! variant with no matching arm here fails to compile (G1's build-time half —
//! see the gate's own test for the runtime half).

use std::sync::Arc;

use crate::value::{
    AggregateValue, ClauseAttempt, ClauseFailureReason, EnumValue, ReteCeiling,
    RuntimeError, RuntimeErrorKind, Value, ValueSnapshot,
};
use crate::edn::contract::WatError;

/// Shared boilerplate: a `wat_field_names_from!` const plus the
/// `OnceLock`-cached `Arc<Vec<String>>` accessor every construction site
/// wants. The NAMES still come from the `.wat` declaration (the macro
/// argument); this only avoids retyping the caching dance per record.
macro_rules! record_names_fn {
    ($fn_name:ident, $const_name:ident, $wat_path:literal, $type_path:literal) => {
        ::wat_source_derive::wat_field_names_from!($const_name, $wat_path, $type_path);
        fn $fn_name() -> Arc<Vec<String>> {
            static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
            N.get_or_init(|| crate::value::value::names_arc_from_static($const_name)).clone()
        }
    };
}

/// Same shared boilerplate, for one TAGGED variant of a `defenum`
/// (`wat_enum_field_names_from!` — the field list is per-variant, not
/// per-type; see that macro's own header in `crates/wat-source-derive`).
macro_rules! variant_names_fn {
    ($fn_name:ident, $const_name:ident, $wat_path:literal, $type_path:literal, $variant:literal) => {
        ::wat_source_derive::wat_enum_field_names_from!($const_name, $wat_path, $type_path, $variant);
        fn $fn_name() -> Arc<Vec<String>> {
            static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
            N.get_or_init(|| crate::value::value::names_arc_from_static($const_name)).clone()
        }
    };
}

// ─── ValueSnapshot ──────────────────────────────────────────────────────────
//
// Excursus 003 strike C: `provenance` left `ValueSnapshot` per the 2026-09-27
// ruling item 4. `:wat::runtime::Provenance` lost its only holder and is
// retired (wat/runtime-errors.wat; registration dropped from src/types.rs)
// — `provenance_value`/the three `variant_names_fn!` rows it fed went with it.

record_names_fn!(value_snapshot_names, VALUE_SNAPSHOT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ValueSnapshot");

fn value_snapshot_value(snap: &ValueSnapshot) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::runtime::ValueSnapshot".to_string(),
        value_snapshot_names(),
        Arc::new(vec![
            Value::String(Arc::new(snap.type_name.to_string())),
            Value::String(Arc::new(snap.rendered.clone())),
        ]),
    )))
}

// ─── ClauseFailureReason / ClauseAttempt (`:wat::kernel::`, builder's ruling) ─

variant_names_fn!(clause_failure_reason_arity_mismatch_names, CLAUSE_FAILURE_REASON_ARITY_MISMATCH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::ClauseFailureReason", "ArityMismatch");
variant_names_fn!(clause_failure_reason_arg_type_mismatch_names, CLAUSE_FAILURE_REASON_ARG_TYPE_MISMATCH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::ClauseFailureReason", "ArgTypeMismatch");
record_names_fn!(clause_attempt_names, CLAUSE_ATTEMPT_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::ClauseAttempt");

fn clause_failure_reason_value(reason: &ClauseFailureReason) -> Value {
    match reason {
        ClauseFailureReason::ArityMismatch { expected, got } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::ClauseFailureReason".to_string(),
            variant_name: "ArityMismatch".to_string(),
            names: clause_failure_reason_arity_mismatch_names(),
            fields: vec![Value::i64(*expected as i64), Value::i64(*got as i64)],
        })),
        ClauseFailureReason::ArgTypeMismatch { position, expected, got } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::ClauseFailureReason".to_string(),
            variant_name: "ArgTypeMismatch".to_string(),
            names: clause_failure_reason_arg_type_mismatch_names(),
            fields: vec![
                Value::i64(*position as i64),
                Value::String(Arc::new(expected.clone())),
                Value::String(Arc::new(got.clone())),
            ],
        })),
        // Unit variant — no fields, so no wat_enum_field_names_from! call needed
        // (there is no field vector for it to read).
        ClauseFailureReason::GuardFalse => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::ClauseFailureReason".to_string(),
            variant_name: "GuardFalse".to_string(),
            names: Arc::new(Vec::new()),
            fields: Vec::new(),
        })),
    }
}

fn clause_attempt_value(attempt: &ClauseAttempt) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::ClauseAttempt".to_string(),
        clause_attempt_names(),
        Arc::new(vec![
            Value::i64(attempt.clause_index as i64),
            Value::i64(attempt.declared_arity as i64),
            Value::Vec(Arc::new(
                attempt.declared_arg_types.iter().map(|s| Value::String(Arc::new(s.clone()))).collect(),
            )),
            clause_failure_reason_value(&attempt.failure_reason),
        ]),
    )))
}

// ─── HashError / HashErrorKind (`:wat::kernel::`, excursus 003 strike B2) ────
//
// `HashError` is now a genuine `:wat::core::Error`-satisfying record (floor
// `message`/`location` plus a `kind` field typed `:wat::kernel::HashErrorKind`,
// itself a `defenum`) — replaces B1's interim `:wat::core::Fault` stand-in at
// `EvalVerificationFailed.cause`. See [`hash_error_value`]'s own doc comment, below.

record_names_fn!(hash_error_names, HASH_ERROR_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashError");
variant_names_fn!(hash_error_kind_unsupported_algorithm_names, HASH_ERROR_KIND_UNSUPPORTED_ALGORITHM_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "UnsupportedAlgorithm");
variant_names_fn!(hash_error_kind_mismatch_names, HASH_ERROR_KIND_MISMATCH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "Mismatch");
variant_names_fn!(hash_error_kind_unsupported_signature_algorithm_names, HASH_ERROR_KIND_UNSUPPORTED_SIGNATURE_ALGORITHM_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "UnsupportedSignatureAlgorithm");
variant_names_fn!(hash_error_kind_invalid_base64_names, HASH_ERROR_KIND_INVALID_BASE64_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "InvalidBase64");
variant_names_fn!(hash_error_kind_invalid_signature_length_names, HASH_ERROR_KIND_INVALID_SIGNATURE_LENGTH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "InvalidSignatureLength");
variant_names_fn!(hash_error_kind_invalid_pub_key_length_names, HASH_ERROR_KIND_INVALID_PUB_KEY_LENGTH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "InvalidPubKeyLength");
variant_names_fn!(hash_error_kind_invalid_pub_key_names, HASH_ERROR_KIND_INVALID_PUB_KEY_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "InvalidPubKey");
variant_names_fn!(hash_error_kind_signature_mismatch_names, HASH_ERROR_KIND_SIGNATURE_MISMATCH_FIELDS, "wat/kernel/diagnostics.wat", ":wat::kernel::HashErrorKind", "SignatureMismatch");

fn hash_error_kind_value(kind: &crate::hash::HashErrorKind) -> Value {
    use crate::hash::HashErrorKind;
    match kind {
        HashErrorKind::UnsupportedAlgorithm { algo } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "UnsupportedAlgorithm".to_string(),
            names: hash_error_kind_unsupported_algorithm_names(),
            fields: vec![Value::String(Arc::new(algo.clone()))],
        })),
        HashErrorKind::Mismatch { algo, expected, actual } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "Mismatch".to_string(),
            names: hash_error_kind_mismatch_names(),
            fields: vec![
                Value::String(Arc::new(algo.clone())),
                Value::String(Arc::new(expected.clone())),
                Value::String(Arc::new(actual.clone())),
            ],
        })),
        HashErrorKind::UnsupportedSignatureAlgorithm { algo } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "UnsupportedSignatureAlgorithm".to_string(),
            names: hash_error_kind_unsupported_signature_algorithm_names(),
            fields: vec![Value::String(Arc::new(algo.clone()))],
        })),
        HashErrorKind::InvalidBase64 { field, reason } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "InvalidBase64".to_string(),
            names: hash_error_kind_invalid_base64_names(),
            fields: vec![
                Value::String(Arc::new((*field).to_string())),
                Value::String(Arc::new(reason.clone())),
            ],
        })),
        HashErrorKind::InvalidSignatureLength { algo, expected, got } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "InvalidSignatureLength".to_string(),
            names: hash_error_kind_invalid_signature_length_names(),
            fields: vec![
                Value::String(Arc::new(algo.clone())),
                Value::i64(*expected as i64),
                Value::i64(*got as i64),
            ],
        })),
        HashErrorKind::InvalidPubKeyLength { algo, expected, got } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "InvalidPubKeyLength".to_string(),
            names: hash_error_kind_invalid_pub_key_length_names(),
            fields: vec![
                Value::String(Arc::new(algo.clone())),
                Value::i64(*expected as i64),
                Value::i64(*got as i64),
            ],
        })),
        HashErrorKind::InvalidPubKey { algo, reason } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "InvalidPubKey".to_string(),
            names: hash_error_kind_invalid_pub_key_names(),
            fields: vec![
                Value::String(Arc::new(algo.clone())),
                Value::String(Arc::new(reason.clone())),
            ],
        })),
        HashErrorKind::SignatureMismatch { algo } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::kernel::HashErrorKind".to_string(),
            variant_name: "SignatureMismatch".to_string(),
            names: hash_error_kind_signature_mismatch_names(),
            fields: vec![Value::String(Arc::new(algo.clone()))],
        })),
    }
}

/// Build the REAL `:wat::kernel::HashError` record Value — floor
/// (`message`/`location`) plus `kind` (the nested dotted-tagged enum value).
/// Used by `to_record`'s `EvalVerificationFailed` arm in place of B1's
/// interim `Fault` stand-in (closed by strike B2 item 3; see
/// [`macro_error_value`] for `MacroExpansionFailed`'s own closure, B3 item 1).
fn hash_error_value(err: &crate::hash::HashError) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::HashError".to_string(),
        hash_error_names(),
        Arc::new(vec![
            Value::String(Arc::new(err.message.clone())),
            crate::runtime::value_from_span(err.location.clone()),
            hash_error_kind_value(&err.kind),
        ]),
    )))
}

// ─── ReteCeilingKind ──────────────────────────────────────────────────────────

variant_names_fn!(rete_ck_session_memory_exceeded_names, RETE_CK_SESSION_MEMORY_EXCEEDED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeilingKind", "SessionMemoryCeilingExceeded");
variant_names_fn!(rete_ck_session_memory_exceeded_on_insert_names, RETE_CK_SESSION_MEMORY_EXCEEDED_ON_INSERT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeilingKind", "SessionMemoryCeilingExceededOnInsert");
variant_names_fn!(rete_ck_rule_set_may_not_terminate_names, RETE_CK_RULE_SET_MAY_NOT_TERMINATE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeilingKind", "RuleSetMayNotTerminate");
variant_names_fn!(rete_ck_fixpoint_round_cap_exceeded_names, RETE_CK_FIXPOINT_ROUND_CAP_EXCEEDED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeilingKind", "FixpointRoundCapExceeded");

fn rete_ceiling_kind_value(ceiling: &ReteCeiling) -> Value {
    match ceiling {
        ReteCeiling::SessionMemoryCeilingExceeded { limit, used, rounds } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::ReteCeilingKind".to_string(),
            variant_name: "SessionMemoryCeilingExceeded".to_string(),
            names: rete_ck_session_memory_exceeded_names(),
            fields: vec![Value::i64(*limit as i64), Value::i64(*used as i64), Value::i64(*rounds as i64)],
        })),
        ReteCeiling::SessionMemoryCeilingExceededOnInsert { limit, used, staged } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::ReteCeilingKind".to_string(),
            variant_name: "SessionMemoryCeilingExceededOnInsert".to_string(),
            names: rete_ck_session_memory_exceeded_on_insert_names(),
            fields: vec![Value::i64(*limit as i64), Value::i64(*used as i64), Value::i64(*staged as i64)],
        })),
        ReteCeiling::RuleSetMayNotTerminate { rule, fact_type } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::ReteCeilingKind".to_string(),
            variant_name: "RuleSetMayNotTerminate".to_string(),
            names: rete_ck_rule_set_may_not_terminate_names(),
            fields: vec![Value::String(Arc::new(rule.clone())), Value::String(Arc::new(fact_type.clone()))],
        })),
        ReteCeiling::FixpointRoundCapExceeded { cap, still_deriving } => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::ReteCeilingKind".to_string(),
            variant_name: "FixpointRoundCapExceeded".to_string(),
            names: rete_ck_fixpoint_round_cap_exceeded_names(),
            fields: vec![Value::i64(*cap as i64), Value::i64(*still_deriving as i64)],
        })),
    }
}

// ─── The 33 RuntimeErrorKind records ──────────────────────────────────────────

record_names_fn!(unbound_symbol_names, UNBOUND_SYMBOL_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnboundSymbol");
record_names_fn!(unknown_function_names, UNKNOWN_FUNCTION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnknownFunction");
record_names_fn!(not_value_dispatchable_names, NOT_VALUE_DISPATCHABLE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NotValueDispatchable");
record_names_fn!(not_callable_names, NOT_CALLABLE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NotCallable");
record_names_fn!(type_mismatch_names, TYPE_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::TypeMismatch");
record_names_fn!(arity_mismatch_names, ARITY_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ArityMismatch");
record_names_fn!(bad_condition_names, BAD_CONDITION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::BadCondition");
record_names_fn!(malformed_form_names, MALFORMED_FORM_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MalformedForm");
record_names_fn!(division_by_zero_names, DIVISION_BY_ZERO_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::DivisionByZero");
record_names_fn!(integer_overflow_names, INTEGER_OVERFLOW_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::IntegerOverflow");
record_names_fn!(duplicate_define_names, DUPLICATE_DEFINE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::DuplicateDefine");
record_names_fn!(reserved_prefix_names, RESERVED_PREFIX_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReservedPrefix");
record_names_fn!(unreachable_clause_names, UNREACHABLE_CLAUSE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnreachableClause");
record_names_fn!(unnamespaced_name_names, UNNAMESPACED_NAME_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnnamespacedName");
record_names_fn!(dotted_name_names, DOTTED_NAME_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::DottedName");
record_names_fn!(declaration_in_expression_position_names, DECLARATION_IN_EXPRESSION_POSITION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::DeclarationInExpressionPosition");
record_names_fn!(eval_forbids_mutation_form_names, EVAL_FORBIDS_MUTATION_FORM_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EvalForbidsMutationForm");
record_names_fn!(user_main_missing_names, USER_MAIN_MISSING_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UserMainMissing");
record_names_fn!(eval_verification_failed_names, EVAL_VERIFICATION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EvalVerificationFailed");
record_names_fn!(rete_ceiling_names, RETE_CEILING_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeiling");
record_names_fn!(macro_expansion_failed_names, MACRO_EXPANSION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MacroExpansionFailed");
record_names_fn!(pattern_match_failed_names, PATTERN_MATCH_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::PatternMatchFailed");
record_names_fn!(effectful_in_step_names, EFFECTFUL_IN_STEP_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EffectfulInStep");
record_names_fn!(no_step_rule_names, NO_STEP_RULE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoStepRule");
record_names_fn!(assertion_failed_names, ASSERTION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::AssertionFailed");
record_names_fn!(service_not_running_names, SERVICE_NOT_RUNNING_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ServiceNotRunning");
record_names_fn!(edn_coerce_mismatch_names, EDN_COERCE_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EdnCoerceMismatch");
record_names_fn!(unknown_field_names, UNKNOWN_FIELD_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnknownField");
record_names_fn!(no_matching_clause_names, NO_MATCHING_CLAUSE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoMatchingClause");
record_names_fn!(postcondition_failed_names, POSTCONDITION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::PostconditionFailed");
record_names_fn!(macro_abort_names, MACRO_ABORT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MacroAbort");
record_names_fn!(write_stopped_names, WRITE_STOPPED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::WriteStopped");
record_names_fn!(rete_defn_axis_violation_names, RETE_DEFN_AXIS_VIOLATION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteDefnAxisViolation");
record_names_fn!(rete_defn_recursive_names, RETE_DEFN_RECURSIVE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteDefnRecursive");

/// Excursus 003 strike A (F2) — build a bare `:wat::runtime::AssertionFailed`
/// `Value::Aggregate(Record)` directly from an assertion's own message/location/
/// actual/expected, NOT from a `RuntimeErrorKind` (the `AssertionFailed` arm of
/// `to_record` above builds one from that path; this is the panic path's own
/// builder). This is now the natural home for an unhandled `assertion-failed!` /
/// `option`/`result::expect` panic's `:wat::kernel::Failure.error`: "an assertion is
/// the same concept whichever path raised it" (BRIEF-shape-strike-A-one-death-
/// shape.md) — the record already carries `actual`/`expected` as its own fields, so
/// `Failure` no longer needs to duplicate them. `causes` is empty (an assertion names
/// no nested cause of its own).
pub(crate) fn assertion_failed_value(
    message: String,
    location: crate::span::Span,
    actual: Option<String>,
    expected: Option<String>,
) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::runtime::AssertionFailed".to_string(),
        assertion_failed_names(),
        Arc::new(vec![
            Value::String(Arc::new(message)),
            crate::runtime::value_from_span(location),
            Value::Option(Arc::new(actual.map(|s| Value::String(Arc::new(s))))),
            Value::Option(Arc::new(expected.map(|s| Value::String(Arc::new(s))))),
        ]),
    )))
}

/// Build the REAL `:wat::macro::<Kind>` record `MacroExpansionFailed.cause` holds
/// (excursus 003 strike B3, item 1 — closes B1's interim-`Fault` gap, the last
/// caller of the retired `single_cause_fault`).
///
/// `MacroError::to_record` can't thread a `TypeEnv`/`SymbolTable` through to
/// strict-decode with — `to_record`'s whole call graph (`kernel/error.rs`,
/// `process/died.rs`, both peer-death paths) has none in hand, unlike the four
/// item-5 wrapper sites, which run inside a call that already holds one. B1
/// measured that gap correctly; what it missed is that `to_record` doesn't need
/// a *caller-supplied* registry to decode a BUILTIN record — every error record
/// has been one since the sweep (`AUDIT-the-shape-of-an-error.md`, Strike B1
/// landed), so this decodes `MacroError`'s own wire form (`error_edn()`,
/// `WatError`, `src/macros/error_edn.rs`) against the PROCESS-WIDE builtins
/// registry (`TypeEnv::with_builtins()` — the same registry
/// `src/runtime.rs::builtin_enum_variant_names`'s `OnceLock` caches) instead of
/// hand-building a second copy of the 16-`MacroErrorKind`-variant shape the way
/// [`hash_error_value`] does for the single-kind `HashError`. The decode is
/// already mutation-proven for every declared kind, including both
/// nested-cause variants (`ProgramBodyEvalFailed`, `MacroEvalRuntimeFailed`):
/// `excursus_003_s3_gates::g_strict_every_declared_kind_decodes_typed`
/// (`src/macros/error_edn.rs`).
fn macro_error_value(cause: &crate::macros::MacroError) -> Value {
    let wire = wat_edn::write(&cause.error_edn());
    let types = crate::types::TypeEnv::with_builtins();
    crate::edn::render::decode_trusted_wire(&wire, Some(&types), None).unwrap_or_else(|e| {
        panic!(
            "macro_error_value: MacroError::error_edn() must decode typed against \
             TypeEnv::with_builtins() (every error record is a builtin) — got {e:?}; wire: {wire}"
        )
    })
}

/// `EdnCoerceMismatch.path`'s wire shape is a `Vector` of dot-path segments
/// (`#[to_edn(via = crate::edn::error::edn_path_segments)]`), not the bare
/// `String` the Rust field holds. Reuses the SAME split primitive
/// `edn_path_segments` calls (`wat_reader::identifier::dot_path_segments`) —
/// not a second copy of the split.
fn path_segments_value(path: &str) -> Value {
    Value::Vec(Arc::new(
        wat_reader::identifier::dot_path_segments(path)
            .into_iter()
            .map(|s| Value::String(Arc::new(s.to_string())))
            .collect(),
    ))
}

impl RuntimeError {
    /// Convert this error into a value of its declared `:wat::runtime::<Kind>`
    /// record (`wat/runtime-errors.wat`). Ships NO envelope change (excursus
    /// 003 step 3a) — `LociDiedError` and the wire (`RuntimeErrorKind`'s
    /// `#[derive(ToEdn)]`, `WatError::error_edn`) are untouched; this is the
    /// value step 3b will put into `Failure`.
    ///
    /// Exhaustive `match`, no `_` arm: a new `RuntimeErrorKind` variant with
    /// no arm here fails to compile.
    pub fn to_record(&self) -> Value {
        let floor_message = Value::String(Arc::new(self.message()));
        let floor_location = crate::runtime::value_from_span(self.span().clone());
        match self.kind() {
            RuntimeErrorKind::UnboundSymbol(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UnboundSymbol".to_string(),
                unbound_symbol_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::UnknownFunction(path) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UnknownFunction".to_string(),
                unknown_function_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(path.clone()))]),
            ))),
            RuntimeErrorKind::NotValueDispatchable { name } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NotValueDispatchable".to_string(),
                not_value_dispatchable_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::NotCallable { got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NotCallable".to_string(),
                not_callable_names(),
                Arc::new(vec![floor_message, floor_location, value_snapshot_value(got)]),
            ))),
            RuntimeErrorKind::TypeMismatch { op, expected, got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::TypeMismatch".to_string(),
                type_mismatch_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(op.clone())),
                    Value::String(Arc::new((*expected).to_string())),
                    value_snapshot_value(got),
                ]),
            ))),
            RuntimeErrorKind::ArityMismatch { op, expected, got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ArityMismatch".to_string(),
                arity_mismatch_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(op.clone())),
                    Value::i64(*expected as i64),
                    Value::i64(*got as i64),
                ]),
            ))),
            RuntimeErrorKind::BadCondition { got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::BadCondition".to_string(),
                bad_condition_names(),
                Arc::new(vec![floor_message, floor_location, value_snapshot_value(got)]),
            ))),
            RuntimeErrorKind::MalformedForm { head, reason } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::MalformedForm".to_string(),
                malformed_form_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(head.clone())),
                    Value::String(Arc::new(reason.clone())),
                ]),
            ))),
            RuntimeErrorKind::DivisionByZero => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DivisionByZero".to_string(),
                division_by_zero_names(),
                Arc::new(vec![floor_message, floor_location]),
            ))),
            RuntimeErrorKind::IntegerOverflow { op, a, b } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::IntegerOverflow".to_string(),
                integer_overflow_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(op.clone())),
                    Value::i64(*a),
                    Value::i64(*b),
                ]),
            ))),
            RuntimeErrorKind::DuplicateDefine(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DuplicateDefine".to_string(),
                duplicate_define_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::ReservedPrefix(prefix) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReservedPrefix".to_string(),
                reserved_prefix_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(prefix.clone()))]),
            ))),
            RuntimeErrorKind::UnreachableClause { name, clause_index, subsumed_by, declared_arg_types } => {
                Value::Aggregate(Arc::new(AggregateValue::record(
                    "wat::runtime::UnreachableClause".to_string(),
                    unreachable_clause_names(),
                    Arc::new(vec![
                        floor_message,
                        floor_location,
                        Value::String(Arc::new(name.clone())),
                        Value::i64(*clause_index as i64),
                        Value::i64(*subsumed_by as i64),
                        Value::Vec(Arc::new(declared_arg_types.iter().map(|s| Value::String(Arc::new(s.clone()))).collect())),
                    ]),
                )))
            }
            RuntimeErrorKind::UnnamespacedName(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UnnamespacedName".to_string(),
                unnamespaced_name_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::DottedName(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DottedName".to_string(),
                dotted_name_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::DeclarationInExpressionPosition(head) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DeclarationInExpressionPosition".to_string(),
                declaration_in_expression_position_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(head.clone()))]),
            ))),
            RuntimeErrorKind::EvalForbidsMutationForm { head } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EvalForbidsMutationForm".to_string(),
                eval_forbids_mutation_form_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(head.clone()))]),
            ))),
            RuntimeErrorKind::UserMainMissing => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UserMainMissing".to_string(),
                user_main_missing_names(),
                Arc::new(vec![floor_message, floor_location]),
            ))),
            RuntimeErrorKind::EvalVerificationFailed { err } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EvalVerificationFailed".to_string(),
                eval_verification_failed_names(),
                Arc::new(vec![floor_message, floor_location, hash_error_value(err)]),
            ))),
            RuntimeErrorKind::ReteCeiling(ceiling) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReteCeiling".to_string(),
                rete_ceiling_names(),
                Arc::new(vec![floor_message, floor_location, rete_ceiling_kind_value(ceiling)]),
            ))),
            // Field order matches the wat declaration (`wat/runtime-errors.wat`):
            // [message location op cause] — the codemod that added `cause` (excursus 003
            // strike B1) appended it AFTER the pre-existing `op` field, not before.
            RuntimeErrorKind::MacroExpansionFailed { op, cause } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::MacroExpansionFailed".to_string(),
                macro_expansion_failed_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(op.clone())),
                    macro_error_value(cause),
                ]),
            ))),
            RuntimeErrorKind::PatternMatchFailed { value_type } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::PatternMatchFailed".to_string(),
                pattern_match_failed_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new((*value_type).to_string()))]),
            ))),
            RuntimeErrorKind::EffectfulInStep { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EffectfulInStep".to_string(),
                effectful_in_step_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::NoStepRule { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NoStepRule".to_string(),
                no_step_rule_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::ServiceNotRunning { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ServiceNotRunning".to_string(),
                service_not_running_names(),
                Arc::new(vec![floor_message, floor_location, Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::EdnCoerceMismatch { op, expected, got, path } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EdnCoerceMismatch".to_string(),
                edn_coerce_mismatch_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(op.clone())),
                    Value::String(Arc::new((**expected).clone())),
                    Value::String(Arc::new((**got).clone())),
                    path_segments_value(path),
                ]),
            ))),
            RuntimeErrorKind::UnknownField { record_class, field, available } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UnknownField".to_string(),
                unknown_field_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(record_class.clone())),
                    Value::String(Arc::new(field.clone())),
                    Value::Vec(Arc::new(available.iter().map(|s| Value::String(Arc::new(s.clone()))).collect())),
                ]),
            ))),
            RuntimeErrorKind::NoMatchingClause { name, called_arity, called_args, attempted_clauses } => {
                Value::Aggregate(Arc::new(AggregateValue::record(
                    "wat::runtime::NoMatchingClause".to_string(),
                    no_matching_clause_names(),
                    Arc::new(vec![
                        floor_message,
                        floor_location,
                        Value::String(Arc::new(name.clone())),
                        Value::i64(*called_arity as i64),
                        Value::Vec(Arc::new(called_args.iter().map(value_snapshot_value).collect())),
                        Value::Vec(Arc::new(attempted_clauses.iter().map(clause_attempt_value).collect())),
                    ]),
                )))
            }
            RuntimeErrorKind::PostconditionFailed {
                defclause_name,
                clause_index,
                ensure_expr_snapshot,
                returned_value,
                ensure_span,
            } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::PostconditionFailed".to_string(),
                postcondition_failed_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(defclause_name.clone())),
                    Value::i64(*clause_index as i64),
                    Value::String(Arc::new(ensure_expr_snapshot.clone())),
                    value_snapshot_value(returned_value),
                    crate::runtime::value_from_span((**ensure_span).clone()),
                ]),
            ))),
            RuntimeErrorKind::MacroAbort { .. } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::MacroAbort".to_string(),
                macro_abort_names(),
                Arc::new(vec![floor_message, floor_location]),
            ))),
            RuntimeErrorKind::WriteStopped => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::WriteStopped".to_string(),
                write_stopped_names(),
                Arc::new(vec![floor_message, floor_location]),
            ))),
            RuntimeErrorKind::ReteDefnAxisViolation { name, axis, head } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReteDefnAxisViolation".to_string(),
                rete_defn_axis_violation_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(name.clone())),
                    Value::String(Arc::new((*axis).to_string())),
                    Value::String(Arc::new(head.clone())),
                ]),
            ))),
            RuntimeErrorKind::ReteDefnRecursive { name, head } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReteDefnRecursive".to_string(),
                rete_defn_recursive_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    Value::String(Arc::new(name.clone())),
                    Value::String(Arc::new(head.clone())),
                ]),
            ))),
        }
    }
}
