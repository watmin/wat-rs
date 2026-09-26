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
    AggregateValue, ClauseAttempt, ClauseFailureReason, EnumValue, Provenance, ReteCeiling,
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

// ─── ValueSnapshot / Provenance ───────────────────────────────────────────────

record_names_fn!(value_snapshot_names, VALUE_SNAPSHOT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ValueSnapshot");
variant_names_fn!(provenance_literal_names, PROVENANCE_LITERAL_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::Provenance", "Literal");
variant_names_fn!(provenance_symbol_bound_names, PROVENANCE_SYMBOL_BOUND_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::Provenance", "SymbolBound");
variant_names_fn!(provenance_runtime_built_names, PROVENANCE_RUNTIME_BUILT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::Provenance", "RuntimeBuilt");

fn provenance_value(prov: &Provenance) -> Option<Value> {
    match prov {
        Provenance::Unknown => None,
        Provenance::Literal { span } => Some(Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::Provenance".to_string(),
            variant_name: "Literal".to_string(),
            names: provenance_literal_names(),
            fields: vec![crate::runtime::value_from_span(span.clone())],
        }))),
        Provenance::SymbolBound { binding_span, head_span } => Some(Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::Provenance".to_string(),
            variant_name: "SymbolBound".to_string(),
            names: provenance_symbol_bound_names(),
            fields: vec![
                crate::runtime::value_from_span(binding_span.clone()),
                crate::runtime::value_from_span(head_span.clone()),
            ],
        }))),
        Provenance::RuntimeBuilt { producer, call_span } => Some(Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::Provenance".to_string(),
            variant_name: "RuntimeBuilt".to_string(),
            names: provenance_runtime_built_names(),
            fields: vec![
                Value::String(Arc::new((*producer).to_string())),
                crate::runtime::value_from_span(call_span.clone()),
            ],
        }))),
    }
}

fn value_snapshot_value(snap: &ValueSnapshot) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::runtime::ValueSnapshot".to_string(),
        value_snapshot_names(),
        Arc::new(vec![
            Value::String(Arc::new(snap.type_name.to_string())),
            Value::String(Arc::new(snap.rendered.clone())),
            Value::Option(Arc::new(provenance_value(&snap.provenance))),
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

// ─── The 40 RuntimeErrorKind records ──────────────────────────────────────────

record_names_fn!(unbound_symbol_names, UNBOUND_SYMBOL_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnboundSymbol");
record_names_fn!(unknown_function_names, UNKNOWN_FUNCTION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnknownFunction");
record_names_fn!(not_value_dispatchable_names, NOT_VALUE_DISPATCHABLE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NotValueDispatchable");
record_names_fn!(not_callable_names, NOT_CALLABLE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NotCallable");
record_names_fn!(type_mismatch_names, TYPE_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::TypeMismatch");
record_names_fn!(arity_mismatch_names, ARITY_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ArityMismatch");
record_names_fn!(bad_condition_names, BAD_CONDITION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::BadCondition");
record_names_fn!(malformed_form_names, MALFORMED_FORM_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MalformedForm");
record_names_fn!(param_shadows_builtin_names, PARAM_SHADOWS_BUILTIN_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ParamShadowsBuiltin");
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
record_names_fn!(channel_disconnected_names, CHANNEL_DISCONNECTED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ChannelDisconnected");
record_names_fn!(rete_ceiling_names, RETE_CEILING_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteCeiling");
record_names_fn!(no_encoding_ctx_names, NO_ENCODING_CTX_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoEncodingCtx");
record_names_fn!(no_source_loader_names, NO_SOURCE_LOADER_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoSourceLoader");
record_names_fn!(no_macro_registry_names, NO_MACRO_REGISTRY_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoMacroRegistry");
record_names_fn!(macro_expansion_failed_names, MACRO_EXPANSION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MacroExpansionFailed");
record_names_fn!(pattern_match_failed_names, PATTERN_MATCH_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::PatternMatchFailed");
record_names_fn!(effectful_in_step_names, EFFECTFUL_IN_STEP_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EffectfulInStep");
record_names_fn!(no_step_rule_names, NO_STEP_RULE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoStepRule");
record_names_fn!(assertion_failed_names, ASSERTION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::AssertionFailed");
record_names_fn!(sandbox_scope_leak_names, SANDBOX_SCOPE_LEAK_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::SandboxScopeLeak");
record_names_fn!(service_not_running_names, SERVICE_NOT_RUNNING_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ServiceNotRunning");
record_names_fn!(edn_coerce_mismatch_names, EDN_COERCE_MISMATCH_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::EdnCoerceMismatch");
record_names_fn!(unknown_field_names, UNKNOWN_FIELD_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::UnknownField");
record_names_fn!(no_matching_clause_names, NO_MATCHING_CLAUSE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::NoMatchingClause");
record_names_fn!(postcondition_failed_names, POSTCONDITION_FAILED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::PostconditionFailed");
record_names_fn!(macro_abort_names, MACRO_ABORT_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::MacroAbort");
record_names_fn!(write_stopped_names, WRITE_STOPPED_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::WriteStopped");
record_names_fn!(rete_defn_axis_violation_names, RETE_DEFN_AXIS_VIOLATION_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteDefnAxisViolation");
record_names_fn!(rete_defn_recursive_names, RETE_DEFN_RECURSIVE_FIELDS, "wat/runtime-errors.wat", ":wat::runtime::ReteDefnRecursive");

/// An empty `(Vector :- [Error])` — every variant's `causes` except the two
/// that wrap a nested error (`EvalVerificationFailed`, `MacroExpansionFailed`).
fn empty_causes() -> Value {
    Value::Vec(Arc::new(Vec::new()))
}

/// A nested ERROR (rule 2 of the brief this file implements) becomes ONE
/// `:wat::core::Fault` in `causes` — its `Display` text as the Fault's
/// message, and the OUTER `RuntimeError`'s own raising-site span as the
/// Fault's location (neither `HashError` nor the reused-uniformly
/// `MacroError` path carries a location `to_record` reads independently
/// here). The wrapped error's own kind-specific shape (which `HashError`
/// variant; `MacroError`'s own richer `WatError` location/causes) is OUT OF
/// SCOPE for this step — named, not papered over, in the strike report.
fn single_cause_fault(message: String, span: &crate::span::Span) -> Value {
    Value::Vec(Arc::new(vec![crate::runtime::fault_value(message, Some(span.clone()))]))
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
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::UnknownFunction(path) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UnknownFunction".to_string(),
                unknown_function_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(path.clone()))]),
            ))),
            RuntimeErrorKind::NotValueDispatchable { name } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NotValueDispatchable".to_string(),
                not_value_dispatchable_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::NotCallable { got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NotCallable".to_string(),
                not_callable_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), value_snapshot_value(got)]),
            ))),
            RuntimeErrorKind::TypeMismatch { op, expected, got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::TypeMismatch".to_string(),
                type_mismatch_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
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
                    empty_causes(),
                    Value::String(Arc::new(op.clone())),
                    Value::i64(*expected as i64),
                    Value::i64(*got as i64),
                ]),
            ))),
            RuntimeErrorKind::BadCondition { got } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::BadCondition".to_string(),
                bad_condition_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), value_snapshot_value(got)]),
            ))),
            RuntimeErrorKind::MalformedForm { head, reason } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::MalformedForm".to_string(),
                malformed_form_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
                    Value::String(Arc::new(head.clone())),
                    Value::String(Arc::new(reason.clone())),
                ]),
            ))),
            RuntimeErrorKind::ParamShadowsBuiltin(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ParamShadowsBuiltin".to_string(),
                param_shadows_builtin_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::DivisionByZero => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DivisionByZero".to_string(),
                division_by_zero_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes()]),
            ))),
            RuntimeErrorKind::IntegerOverflow { op, a, b } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::IntegerOverflow".to_string(),
                integer_overflow_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
                    Value::String(Arc::new(op.clone())),
                    Value::i64(*a),
                    Value::i64(*b),
                ]),
            ))),
            RuntimeErrorKind::DuplicateDefine(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DuplicateDefine".to_string(),
                duplicate_define_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::ReservedPrefix(prefix) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReservedPrefix".to_string(),
                reserved_prefix_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(prefix.clone()))]),
            ))),
            RuntimeErrorKind::UnreachableClause { name, clause_index, subsumed_by, declared_arg_types } => {
                Value::Aggregate(Arc::new(AggregateValue::record(
                    "wat::runtime::UnreachableClause".to_string(),
                    unreachable_clause_names(),
                    Arc::new(vec![
                        floor_message,
                        floor_location,
                        empty_causes(),
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
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::DottedName(name) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DottedName".to_string(),
                dotted_name_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(name.clone()))]),
            ))),
            RuntimeErrorKind::DeclarationInExpressionPosition(head) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::DeclarationInExpressionPosition".to_string(),
                declaration_in_expression_position_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(head.clone()))]),
            ))),
            RuntimeErrorKind::EvalForbidsMutationForm { head } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EvalForbidsMutationForm".to_string(),
                eval_forbids_mutation_form_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(head.clone()))]),
            ))),
            RuntimeErrorKind::UserMainMissing => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::UserMainMissing".to_string(),
                user_main_missing_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes()]),
            ))),
            RuntimeErrorKind::EvalVerificationFailed { err } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EvalVerificationFailed".to_string(),
                eval_verification_failed_names(),
                Arc::new(vec![floor_message, floor_location, single_cause_fault(err.to_string(), self.span())]),
            ))),
            RuntimeErrorKind::ChannelDisconnected { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ChannelDisconnected".to_string(),
                channel_disconnected_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::ReteCeiling(ceiling) => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReteCeiling".to_string(),
                rete_ceiling_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), rete_ceiling_kind_value(ceiling)]),
            ))),
            RuntimeErrorKind::NoEncodingCtx { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NoEncodingCtx".to_string(),
                no_encoding_ctx_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::NoSourceLoader { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NoSourceLoader".to_string(),
                no_source_loader_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::NoMacroRegistry { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NoMacroRegistry".to_string(),
                no_macro_registry_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::MacroExpansionFailed { op, cause } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::MacroExpansionFailed".to_string(),
                macro_expansion_failed_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    single_cause_fault(cause.to_string(), self.span()),
                    Value::String(Arc::new(op.clone())),
                ]),
            ))),
            RuntimeErrorKind::PatternMatchFailed { value_type } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::PatternMatchFailed".to_string(),
                pattern_match_failed_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new((*value_type).to_string()))]),
            ))),
            RuntimeErrorKind::EffectfulInStep { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EffectfulInStep".to_string(),
                effectful_in_step_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::NoStepRule { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::NoStepRule".to_string(),
                no_step_rule_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::AssertionFailed { actual, expected, .. } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::AssertionFailed".to_string(),
                assertion_failed_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
                    Value::Option(Arc::new(actual.clone().map(|s| Value::String(Arc::new(s))))),
                    Value::Option(Arc::new(expected.clone().map(|s| Value::String(Arc::new(s))))),
                ]),
            ))),
            RuntimeErrorKind::SandboxScopeLeak { offending_name, outer_define_span } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::SandboxScopeLeak".to_string(),
                sandbox_scope_leak_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
                    Value::String(Arc::new(offending_name.clone())),
                    crate::runtime::value_from_span(outer_define_span.clone()),
                ]),
            ))),
            RuntimeErrorKind::ServiceNotRunning { op } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ServiceNotRunning".to_string(),
                service_not_running_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes(), Value::String(Arc::new(op.clone()))]),
            ))),
            RuntimeErrorKind::EdnCoerceMismatch { op, expected, got, path } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::EdnCoerceMismatch".to_string(),
                edn_coerce_mismatch_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
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
                    empty_causes(),
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
                        empty_causes(),
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
                    empty_causes(),
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
                Arc::new(vec![floor_message, floor_location, empty_causes()]),
            ))),
            RuntimeErrorKind::WriteStopped => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::WriteStopped".to_string(),
                write_stopped_names(),
                Arc::new(vec![floor_message, floor_location, empty_causes()]),
            ))),
            RuntimeErrorKind::ReteDefnAxisViolation { name, axis, head } => Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::runtime::ReteDefnAxisViolation".to_string(),
                rete_defn_axis_violation_names(),
                Arc::new(vec![
                    floor_message,
                    floor_location,
                    empty_causes(),
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
                    empty_causes(),
                    Value::String(Arc::new(name.clone())),
                    Value::String(Arc::new(head.clone())),
                ]),
            ))),
        }
    }
}
