//! FM 2-bis probe — arc 237 Stone 237.4: rich `:NoMatchingClause` +
//! `:PostconditionFailed` diagnostics.
//!
//! Promotes the TEMPORARY error variants from Stones 237.2 + 237.3 to RICH
//! diagnostics per arc 233.3 EDN-shape. Mirrors the construction-and-inspect
//! technique from `tests/probe_stone_233_3_runtime_error_edn.rs` — construct
//! the variant directly in Rust, serialize via `runtime_error_to_edn`, inspect
//! the Tagged EDN value.
//!
//! Refinements (per DESIGN-STONE-237.4):
//!   - NoMatchingClauseRuntime → NoMatchingClause (HARD CUT rename)
//!     + attempted_clauses: Vec<String> → Vec<ClauseAttempt> (structured)
//!   - PostconditionFailedRuntime → PostconditionFailed (HARD CUT rename)
//!     + ensure_expr_snapshot + dual spans (body_span + ensure_span)
//!   - NEW ClauseAttempt struct + ClauseFailureReason enum
//!   - Clean EDN tags: #wat.kernel/NoMatchingClause + #wat.kernel/PostconditionFailed
//!
//! Doctrine (arc 233 — errors as teaching values): the diagnostic surfaces
//! WHY each clause was skipped (arity / type / guard-false), not just THAT
//! none matched.
//!
//! Probe contracts (10):
//!   1.  NoMatchingClause variant constructs with Vec<ClauseAttempt> (rename + structure)
//!   2.  PostconditionFailed variant constructs with ensure_expr_snapshot + dual spans
//!   3.  NoMatchingClause EDN tag contains "NoMatchingClause" (NOT "Runtime")
//!   4.  PostconditionFailed EDN tag contains "PostconditionFailed" (NOT "Runtime")
//!   5.  ClauseAttempt with ArityMismatch reason constructs + serializes
//!   6.  ClauseAttempt with ArgTypeMismatch reason constructs + serializes
//!   7.  ClauseAttempt with GuardFalse reason constructs + serializes
//!   8.  PostconditionFailed carries ensure_expr_snapshot + returned_value through EDN
//!   9.  NoMatchingClause EDN round-trips through wat-edn parser
//!   10. NoMatchingClause attempt-list count preserved through EDN
//!
//! Initial state: file FAILS to compile — ClauseAttempt / ClauseFailureReason
//! don't exist; NoMatchingClause / PostconditionFailed (renamed) don't exist
//! (only the *Runtime variants do).
//!
//! Post-stone 237.4: 10/10 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF.

// rune:lint(no-inlined-wat) — this file constructs `RuntimeError`/`ClauseAttempt` Rust structs
// DIRECTLY (no startup/eval pipeline at all) and serializes them via `ToEdn`; `ensure_expr_snapshot`
// is an opaque snapshot-text `String` FIELD (e.g. probes 2/4/9) that happens to look like a `:fn`
// form but is never parsed or evaluated by wat's reader — Rust-level data, not wat-under-test.

use std::sync::Arc;
use wat::runtime::{ClauseAttempt, ClauseFailureReason, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use wat::span::Span;
use wat::edn::contract::ToEdn;

fn test_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 5, 3)
}

fn arity_attempt() -> ClauseAttempt {
    ClauseAttempt {
        clause_index: 0,
        declared_arity: 2,
        declared_arg_types: vec![":wat::core::i64".into(), ":wat::core::i64".into()],
        failure_reason: ClauseFailureReason::ArityMismatch { expected: 2, got: 1 },
    }
}

fn type_attempt() -> ClauseAttempt {
    ClauseAttempt {
        clause_index: 1,
        declared_arity: 1,
        declared_arg_types: vec![":wat::core::i64".into()],
        failure_reason: ClauseFailureReason::ArgTypeMismatch {
            position: 0,
            expected: ":wat::core::i64".into(),
            got: ":wat::core::String".into(),
        },
    }
}

fn guard_attempt() -> ClauseAttempt {
    ClauseAttempt {
        clause_index: 2,
        declared_arity: 1,
        declared_arg_types: vec![":wat::core::i64".into()],
        failure_reason: ClauseFailureReason::GuardFalse,
    }
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_01_no_matching_clause_constructs_with_structured_attempts() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::NoMatchingClause {
        name: ":my::process".into(),
        called_arity: 1,
        called_args: vec![ValueSnapshot::of(&Value::i64(42))],
        attempted_clauses: Box::new(vec![arity_attempt(), type_attempt(), guard_attempt()])
    });
    match err.kind() {
        RuntimeErrorKind::NoMatchingClause { attempted_clauses, .. } => {
            assert_eq!(attempted_clauses.len(), 3, "three structured attempts");
        }
        _ => panic!("expected NoMatchingClause, got {:?}", err),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_postcondition_failed_constructs_with_ensure_snapshot_and_dual_spans() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::PostconditionFailed {
        defclause_name: ":my::positive".into(),
        clause_index: 0,
        ensure_expr_snapshot: "(:wat::core::fn [result <- :i64] -> :bool (> result 0))".into(),
        returned_value: Box::new(ValueSnapshot::of(&Value::i64(-5))),
        ensure_span: Box::new(test_span()),
    });
    match err.kind() {
        RuntimeErrorKind::PostconditionFailed { ensure_expr_snapshot, .. } => {
            assert_eq!(
                ensure_expr_snapshot,
                "(:wat::core::fn [result <- :i64] -> :bool (> result 0))",
                "ensure snapshot carries the :fn text"
            );
        }
        _ => panic!("expected PostconditionFailed, got {:?}", err),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_no_matching_clause_edn_tag_clean() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::NoMatchingClause {
        name: ":my::process".into(),
        called_arity: 1,
        called_args: vec![ValueSnapshot::of(&Value::i64(42))],
        attempted_clauses: Box::new(vec![arity_attempt()])
    });
    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(serialized, "probe_arc237_stone4_rich_errors__no_matching_clause_tag_clean.edn", "EDN tag must be clean NoMatchingClause (no Runtime suffix)");
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_postcondition_failed_edn_tag_clean() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::PostconditionFailed {
        defclause_name: ":my::positive".into(),
        clause_index: 0,
        ensure_expr_snapshot: "(fn ...)".into(),
        returned_value: Box::new(ValueSnapshot::of(&Value::i64(-5))),
        ensure_span: Box::new(test_span()),
    });
    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(serialized, "probe_arc237_stone4_rich_errors__postcondition_failed_tag_clean.edn", "EDN tag must be clean PostconditionFailed (no Runtime suffix)");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_clause_attempt_arity_mismatch() {
    let attempt = arity_attempt();
    assert!(matches!(
        attempt.failure_reason,
        ClauseFailureReason::ArityMismatch { expected: 2, got: 1 }
    ));
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_clause_attempt_type_mismatch() {
    let attempt = type_attempt();
    match attempt.failure_reason {
        ClauseFailureReason::ArgTypeMismatch { position, expected, got } => {
            assert_eq!(position, 0);
            assert_eq!(expected, ":wat::core::i64");
            assert_eq!(got, ":wat::core::String");
        }
        other => panic!("expected ArgTypeMismatch, got {:?}", other),
    }
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_clause_attempt_guard_false() {
    let attempt = guard_attempt();
    assert!(matches!(
        attempt.failure_reason,
        ClauseFailureReason::GuardFalse
    ));
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_postcondition_edn_carries_ensure_and_returned() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::PostconditionFailed {
        defclause_name: ":my::positive".into(),
        clause_index: 0,
        ensure_expr_snapshot: "ENSURE_MARKER_TEXT".into(),
        returned_value: Box::new(ValueSnapshot::of(&Value::i64(-5))),
        ensure_span: Box::new(test_span()),
    });
    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(serialized, "probe_arc237_stone4_rich_errors__postcondition_ensure_and_returned.edn", "EDN must carry the ensure_expr_snapshot text");
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_no_matching_clause_edn_round_trips() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::NoMatchingClause {
        name: ":my::process".into(),
        called_arity: 1,
        called_args: vec![ValueSnapshot::of(&Value::i64(42))],
        attempted_clauses: Box::new(vec![arity_attempt(), type_attempt()])
    });
    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(serialized.clone(), "probe_arc237_stone4_rich_errors__no_matching_clause_round_trip.edn", "round-tripped EDN must be NoMatchingClause with both clause attempts");
    let parsed = wat_edn::parse_owned(&serialized).expect("EDN round-trip parse");
    assert!(
        matches!(&parsed, wat_edn::OwnedValue::Tagged(tag, _) if tag.name() == "NoMatchingClause"),
        "round-tripped EDN must be Tagged with NoMatchingClause; got {:?}",
        parsed
    );
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_attempt_list_count_preserved_through_edn() {
    let err = RuntimeError::new(test_span(), RuntimeErrorKind::NoMatchingClause {
        name: ":my::process".into(),
        called_arity: 3,
        called_args: vec![
            ValueSnapshot::of(&Value::i64(1)),
            ValueSnapshot::of(&Value::i64(2)),
            ValueSnapshot::of(&Value::i64(3)),
        ],
        attempted_clauses: Box::new(vec![arity_attempt(), type_attempt(), guard_attempt()])
    });
    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);
    // All three failure-reason discriminants and clause indices in one golden.
    wat::assert_edn_matches_file!(serialized.clone(), "probe_arc237_stone4_rich_errors__attempt_list_count.edn", "all three ClauseFailureReason variants present; attempt list embedded in EDN");
    let parsed = wat_edn::parse_owned(&serialized).expect("EDN round-trip parse");
    assert!(
        matches!(&parsed, wat_edn::OwnedValue::Tagged(_, _)),
        "serialized NoMatchingClause is Tagged; attempt list embedded within"
    );
}
