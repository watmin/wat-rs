//! Probe — arc 243 Stone 243.6 — CheckError Pattern A structural verification
//!
//! FM 2-bis disconfirming probe: asserts the POST-stone shape.
//!
//! - PRE-stone state: this probe FAILS TO COMPILE. `CheckError` is currently
//!   a flat 34-variant enum (`pub enum CheckError { Variant { span, ... }, ...
//!   }`) where every variant carries its own `span` field; there is no
//!   `CheckErrorKind` type and `CheckError` has no `kind`/`span` struct fields.
//!   Constructing `CheckError { span, kind }` is a type error, and
//!   `CheckErrorKind` is an unresolved import.
//! - POST-stone state: this probe COMPILES + PASSES. `CheckError` is the
//!   Pattern A shape (`pub struct CheckError { pub span: Span, pub kind:
//!   CheckErrorKind }`); the constructor requires `span` at the outer field;
//!   the variants live on `CheckErrorKind` with no per-variant `span` field
//!   (multi-span variants keep only their SECONDARY spans as domain-named
//!   kind fields per CONFORMARE.md § Multi-span).
//!
//! The disconfirmation is STRUCTURAL not behavioral: pre-stone CheckError
//! requires a `span` on EVERY variant (and `diagnostic()` does N-arm span
//! extraction); post-stone span lives once at the outer struct and every
//! consumer reads `err.span` — one path, not a 34-arm match. The probe
//! demonstrates the structural enforcement Rust's type system now imposes.

use wat::check::{CheckError, CheckErrorKind};
use wat::span::Span;

/// Contract 1: CheckError carries `span: Span` at the outer struct level —
/// every variant inherits the location discipline by construction.
#[test]
fn checkerror_outer_span_field_required() {
    let err = CheckError {
        span: Span::unknown(),
        kind: CheckErrorKind::ArityMismatch {
            callee: "f".to_string(),
            expected: 2,
            got: 3,
        },
    };

    // The struct's span field is universally accessible — no exhaustive match
    // across 34 variants required. This is the load-bearing UX win.
    let _span: &Span = &err.span;

    // The kind enum holds variant-specific data only — no span field per
    // variant.
    assert!(matches!(err.kind, CheckErrorKind::ArityMismatch { .. }));
}

/// Contract 2: CheckErrorKind variants do NOT carry per-variant span fields.
/// This contract documents the type-level shape post-Pattern-A retrofit.
///
/// Pre-stone (current): the variants live directly on `enum CheckError` with
/// each carrying its own `span: Span` field (ArityMismatch, TypeMismatch,
/// UnknownCallee, ... — all 34).
///
/// Post-stone: variants live on `enum CheckErrorKind` with NO per-variant
/// span field; span lives at the outer struct level uniformly.
#[test]
fn checkerrorkind_variants_have_no_span_field() {
    // Construct a variant; verify no `span:` field is required at the kind
    // level (the kind holds only the variant's domain data).
    let kind = CheckErrorKind::UnknownCallee {
        callee: "g".to_string(),
    };

    let err = CheckError {
        span: Span::unknown(),
        kind,
    };

    let _span: &Span = &err.span;
    assert!(matches!(err.kind, CheckErrorKind::UnknownCallee { .. }));
}

/// Contract 3: Span access is universal — no exhaustive match across 34
/// variants required. This contract demonstrates the consumer-side UX win:
/// any CheckError consumer needing span data accesses `err.span` directly.
///
/// Pre-stone (current): `diagnostic()` (src/check.rs:1361) does N-arm span
/// extraction across the variants — the consumer overhead conformare flagged.
///
/// Post-stone: `err.span` — single field access; the N-arm match collapses.
#[test]
fn checkerror_span_access_is_single_path() {
    let variants_under_test: Vec<(CheckError, Span)> = vec![
        (
            CheckError {
                span: Span::unknown(),
                kind: CheckErrorKind::ArityMismatch {
                    callee: "a".into(),
                    expected: 1,
                    got: 0,
                },
            },
            Span::unknown(),
        ),
        (
            CheckError {
                span: Span::unknown(),
                kind: CheckErrorKind::UnknownCallee { callee: "b".into() },
            },
            Span::unknown(),
        ),
    ];

    for (err, expected_span) in &variants_under_test {
        // Universal single-path access — works for EVERY CheckError regardless
        // of which kind variant. The whole point of Pattern A.
        let actual_span: &Span = &err.span;
        assert_eq!(actual_span, expected_span);
    }
}
