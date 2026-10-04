//! Arc 298.3 — `#[derive(ToEdn)]` on `RuntimeErrorKind` is byte-identical to
//! the deleted `runtime_error_to_edn` hand-written serializer.
//!
//! ## What this proves
//!
//! For every `RuntimeErrorKind` variant (31 probe functions):
//! - The derive generates the same `#wat.runtime/<Name>` tag.
//! - Field keys are snake→kebab in declaration order.
//! - `:span` is appended LAST by `splice_span` (from the outer `RuntimeError.span`).
//! - `Box<T>: ToEdn` delegates through the Box.
//! - `Option<T>` is tagged (`#wat.core/Option.Some`/`None`) per arc 296 H-2.
//! - Secondary `Span` fields emit under their auto-kebab key (e.g. `outer-define-span`).
//! - `MacroExpansionFailed.cause` uses `error_edn_of_boxed` (floor form).
//! - `EdnCoerceMismatch.path` is split into a Vec via `edn_path_segments`.
//! - `Vec<ClauseAttempt>` serializes via `impl ToEdn for ClauseAttempt`.
//!
//! ## How the golden strings were derived
//!
//! Each golden was captured by running the test with a temporary `eprintln!`
//! and pasting the byte-identical stderr output. NEVER hand-guessed.
//!
//! PROBATIO FLEXA MENTITVR — every probe uses `assert_eq!` on exact bytes.

// rune:lint(no-inlined-wat) — this file constructs `RuntimeError`/`ClauseAttempt` Rust structs
// DIRECTLY (no startup/eval pipeline at all) and serializes them via `ToEdn`; the
// `PostconditionFailed` probe's `ensure_expr_snapshot` is an opaque snapshot-text `String` FIELD
// ("(> result 0)") that happens to look like a wat form but is never parsed or evaluated by
// wat's reader — Rust-level data, not wat-under-test (mirrors probe_arc237_stone4_rich_errors.rs).

use std::sync::Arc;
use wat::hash::{HashError, HashErrorKind};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{
    ClauseAttempt, ClauseFailureReason, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot,
};
use wat::span::Span;
use wat::edn::contract::ToEdn;

fn s() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn make(kind: RuntimeErrorKind) -> RuntimeError {
    RuntimeError::new(s(), kind)
}

fn write(err: &RuntimeError) -> String {
    wat_edn::write(&err.to_edn())
}

// ─── 1. UnboundSymbol ────────────────────────────────────────────────────────

#[test]
fn probe_unbound_symbol() {
    let err = make(RuntimeErrorKind::UnboundSymbol("my-var".into()));
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__unbound_symbol.edn");
}

// ─── 2. UnknownFunction ──────────────────────────────────────────────────────

#[test]
fn probe_unknown_function() {
    let err = make(RuntimeErrorKind::UnknownFunction(":user::greet".into()));
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__unknown_function.edn");
}

// ─── 3. NotCallable ──────────────────────────────────────────────────────────

#[test]
fn probe_not_callable() {
    let snap = ValueSnapshot::of(&Value::String(Arc::new("hello".to_string())));
    let err = make(RuntimeErrorKind::NotCallable { got: Box::new(snap) });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__not_callable.edn");
}

// ─── 4. TypeMismatch ─────────────────────────────────────────────────────────

#[test]
fn probe_type_mismatch() {
    let snap = ValueSnapshot::of(&Value::i64(42));
    let err = make(RuntimeErrorKind::TypeMismatch {
        op: ":wat::core::+".into(),
        expected: "i64",
        got: Box::new(snap),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__type_mismatch.edn");
}

// ─── 5. ArityMismatch ────────────────────────────────────────────────────────

#[test]
fn probe_arity_mismatch() {
    let err = make(RuntimeErrorKind::ArityMismatch {
        op: ":my::func".into(),
        expected: 2,
        got: 3,
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__arity_mismatch.edn");
}

// ─── 6. BadCondition ─────────────────────────────────────────────────────────

#[test]
fn probe_bad_condition() {
    let snap = ValueSnapshot::of(&Value::i64(0));
    let err = make(RuntimeErrorKind::BadCondition { got: Box::new(snap) });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__bad_condition.edn");
}

// ─── 7. MalformedForm ────────────────────────────────────────────────────────

#[test]
fn probe_malformed_form() {
    let err = make(RuntimeErrorKind::MalformedForm {
        head: ":wat::core::fn".into(),
        reason: "param list must be a vector".into(),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__malformed_form.edn");
}

// ─── 9. DivisionByZero ───────────────────────────────────────────────────────

#[test]
fn probe_division_by_zero() {
    let err = make(RuntimeErrorKind::DivisionByZero);
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__division_by_zero.edn");
}

// ─── 10. DuplicateDefine ─────────────────────────────────────────────────────

#[test]
fn probe_duplicate_define() {
    let err = make(RuntimeErrorKind::DuplicateDefine(":user::counter".into()));
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__duplicate_define.edn");
}

// ─── 11. ReservedPrefix ──────────────────────────────────────────────────────

#[test]
fn probe_reserved_prefix() {
    let err = make(RuntimeErrorKind::ReservedPrefix(":wat::my-thing".into()));
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__reserved_prefix.edn");
}

// ─── 12. DeclarationInExpressionPosition ─────────────────────────────────────

#[test]
fn probe_declaration_in_expression_position() {
    let err = make(RuntimeErrorKind::DeclarationInExpressionPosition(":wat::core::define".into()));
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__declaration_in_expression_position.edn");
}

// ─── 13. EvalForbidsMutationForm ─────────────────────────────────────────────

#[test]
fn probe_eval_forbids_mutation_form() {
    let err = make(RuntimeErrorKind::EvalForbidsMutationForm { head: ":wat::core::define".into() });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__eval_forbids_mutation_form.edn");
}

// ─── 14. UserMainMissing ─────────────────────────────────────────────────────

#[test]
fn probe_user_main_missing() {
    let err = make(RuntimeErrorKind::UserMainMissing);
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__user_main_missing.edn");
}

// ─── 15. EvalVerificationFailed ──────────────────────────────────────────────

#[test]
fn probe_eval_verification_failed() {
    let err = make(RuntimeErrorKind::EvalVerificationFailed {
        err: HashError::new(s(), HashErrorKind::UnsupportedAlgorithm { algo: "SHA1".into() }),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__eval_verification_failed.edn");
}

// ─── 20. MacroExpansionFailed ────────────────────────────────────────────────

#[test]
fn probe_macro_expansion_failed() {
    let inner_span = Span::new(Arc::new("macro.wat".to_string()), 5, 2);
    let cause = Box::new(MacroError {
        span: inner_span,
        kind: MacroErrorKind::DuplicateMacro("my-macro".into()),
    });
    let err = make(RuntimeErrorKind::MacroExpansionFailed {
        op: ":user::expand".into(),
        cause,
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__macro_expansion_failed.edn");
}

// ─── 21. PatternMatchFailed ──────────────────────────────────────────────────

#[test]
fn probe_pattern_match_failed() {
    let err = make(RuntimeErrorKind::PatternMatchFailed { value_type: "i64" });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__pattern_match_failed.edn");
}

// ─── 22. EffectfulInStep ─────────────────────────────────────────────────────

#[test]
fn probe_effectful_in_step() {
    let err = make(RuntimeErrorKind::EffectfulInStep { op: ":wat::kernel::println".into() });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__effectful_in_step.edn");
}

// ─── 23. NoStepRule ──────────────────────────────────────────────────────────

#[test]
fn probe_no_step_rule() {
    let err = make(RuntimeErrorKind::NoStepRule { op: ":user::custom-op".into() });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__no_step_rule.edn");
}

// ─── 26. ServiceNotRunning ───────────────────────────────────────────────────

#[test]
fn probe_service_not_running() {
    let err = make(RuntimeErrorKind::ServiceNotRunning { op: ":wat::kernel::println".into() });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__service_not_running.edn");
}

// ─── 27a. EdnCoerceMismatch (non-empty path) ─────────────────────────────────
//
// `path` field uses `edn_path_segments` via: splits on '.' → `["x" "y"]`

#[test]
fn probe_edn_coerce_mismatch() {
    let err = make(RuntimeErrorKind::EdnCoerceMismatch {
        op: ":wat::kernel::readln".into(),
        expected: Box::new(":user::Point".into()),
        got: Box::new("Map".into()),
        path: "x.y".into(),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__edn_coerce_mismatch.edn");
}

// ─── 27b. EdnCoerceMismatch (empty path) ─────────────────────────────────────

#[test]
fn probe_edn_coerce_mismatch_empty_path() {
    let err = make(RuntimeErrorKind::EdnCoerceMismatch {
        op: ":wat::kernel::readln".into(),
        expected: Box::new(":user::Point".into()),
        got: Box::new("Integer".into()),
        path: "".into(),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__edn_coerce_mismatch_empty_path.edn");
}

// ─── 28. UnknownField ────────────────────────────────────────────────────────

#[test]
fn probe_unknown_field() {
    let err = make(RuntimeErrorKind::UnknownField {
        record_class: "user::Point".into(),
        field: "z".into(),
        available: vec!["x".into(), "y".into()],
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__unknown_field.edn");
}

// ─── 29. NoMatchingClause ────────────────────────────────────────────────────

#[test]
fn probe_no_matching_clause() {
    let err = make(RuntimeErrorKind::NoMatchingClause {
        name: ":user::process".into(),
        called_arity: 1,
        called_args: vec![ValueSnapshot::of(&Value::i64(42))],
        attempted_clauses: Box::new(vec![
            ClauseAttempt {
                clause_index: 0,
                declared_arity: 2,
                declared_arg_types: vec!["i64".into(), "i64".into()],
                failure_reason: ClauseFailureReason::ArityMismatch { expected: 2, got: 1 },
            },
            ClauseAttempt {
                clause_index: 1,
                declared_arity: 1,
                declared_arg_types: vec!["String".into()],
                failure_reason: ClauseFailureReason::ArgTypeMismatch {
                    position: 0,
                    expected: "String".into(),
                    got: "i64".into(),
                },
            },
        ]),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__no_matching_clause.edn");
}

// ─── 30. PostconditionFailed ─────────────────────────────────────────────────
//
// Multi-span: outer `span` (from `err.span`) appended as `:span` LAST;
// secondary `ensure_span: Span` emitted as `:ensure-span` in declaration order.
//
// Arc 298.3 wire change: old form used `:body-span` for the outer span;
// new form uses `:span`.

#[test]
fn probe_postcondition_failed() {
    let ensure_span = Span::new(Arc::new("defs.wat".to_string()), 20, 8);
    let err = make(RuntimeErrorKind::PostconditionFailed {
        defclause_name: ":user::positive".into(),
        clause_index: 0,
        ensure_expr_snapshot: "(> result 0)".into(),
        returned_value: Box::new(ValueSnapshot::of(&Value::i64(-5))),
        ensure_span: Box::new(ensure_span),
    });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__postcondition_failed.edn");
}

// ─── 31. MacroAbort ──────────────────────────────────────────────────────────

#[test]
fn probe_macro_abort() {
    let err = make(RuntimeErrorKind::MacroAbort { message: "bad template".into() });
    wat::assert_edn_matches_file!(write(&err), "probe_arc298_3_runtime_derive_identical__macro_abort.edn");
}

// ─── 32. ReteDefnRecursive ───────────────────────────────────────────────────

#[test]
fn probe_rete_defn_recursive() {
    let err = make(RuntimeErrorKind::ReteDefnRecursive {
        name: ":probe::countdown".into(),
        head: ":probe::countdown".into(),
    });
    wat::assert_edn_matches_file!(
        write(&err),
        "probe_arc298_3_runtime_derive_identical__rete_defn_recursive.edn"
    );
}

