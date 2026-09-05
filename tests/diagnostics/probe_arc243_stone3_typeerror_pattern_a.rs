//! Probe — arc 243 Stone 243.3 — TypeError Pattern A structural verification
//!
//! FM 2-bis disconfirming probe: asserts the post-stone shape.
//!
//! - PRE-stone state: this probe FAILS to compile. `TypeError` is currently
//!   a flat enum (`pub enum TypeError { Variant { span, ... }, ... }`);
//!   constructing it as a struct with `span: ... , kind: ...` fields is a
//!   type error.
//! - POST-stone state: this probe COMPILES + PASSES. `TypeError` is the
//!   Pattern A shape (`pub struct TypeError { pub span: Span, pub kind:
//!   TypeErrorKind }`); the constructor requires span at the outer field;
//!   CyclicSubtype variant lives in TypeErrorKind without a span field.
//!
//! The disconfirmation is STRUCTURAL not behavioral: pre-stone TypeError can
//! be constructed without supplying span (e.g., the CyclicSubtype variant);
//! post-stone it cannot. The probe demonstrates the structural enforcement
//! Rust's type system now imposes.

use wat::span::Span;
use wat::types::{TypeError, TypeErrorKind};

/// Contract 1: TypeError carries `span: Span` at the outer struct level —
/// every variant inherits the discipline by construction.
#[test]
fn typeerror_outer_span_field_required() {
    let err = TypeError::new(
        wat::rust_caller_span!(),
        TypeErrorKind::CyclicSubtype {
            child: "x".to_string(),
            parent: "y".to_string(),
        },
    );

    // The struct's span is universally accessible — no exhaustive
    // match required across N variants. This is the load-bearing UX win.
    let _span: &Span = err.span();

    // The kind enum holds variant-specific data only — no span field per
    // variant. CyclicSubtype's domain-spanless status is captured via the
    // outer struct's wat::rust_caller_span!() + rune annotation.
    let kind_is_cyclic = matches!(err.kind(), TypeErrorKind::CyclicSubtype { .. });
    assert!(kind_is_cyclic);
}

/// Contract 2: TypeErrorKind variants do NOT carry per-variant span fields.
/// This contract documents the type-level shape post-Pattern-A retrofit.
///
/// Pre-stone (current): the variants live directly on `enum TypeError` with
/// each carrying its own `span: Span` field — and crucially, CyclicSubtype
/// lacks one (the catastrophic-class instance Stone 241.18a surfaced and
/// the conformare arc exists to annihilate).
///
/// Post-stone: variants live on `enum TypeErrorKind` with NO per-variant
/// span fields; span lives at the outer struct level uniformly.
#[test]
fn typeerrorkind_variants_have_no_span_field() {
    // Construct a non-CyclicSubtype variant; verify no `span:` field is
    // required at the kind level.
    let kind = TypeErrorKind::ReservedPrefix {
        name: "wat::reserved".to_string(),
    };

    let err = TypeError::new(wat::rust_caller_span!(), kind);

    let _span: &Span = err.span();
    assert!(matches!(err.kind(), TypeErrorKind::ReservedPrefix { .. }));
}

/// Contract 3: Span access is universal — no exhaustive match across 16
/// variants required. This contract demonstrates the consumer-side UX win:
/// any TypeError consumer needing span data accesses `err.span` directly.
///
/// Pre-stone (current): `src/function/parse.rs:154-172` exhibits a 17-line
/// 16-arm match block for span extraction — the consumer overhead conformare
/// flagged.
///
/// Post-stone: `err.span` — single field access; the 16-arm match collapses.
#[test]
fn typeerror_span_access_is_single_path() {
    let cyclic_span = wat::rust_caller_span!();
    let reserved_span = wat::rust_caller_span!();
    let variants_under_test: Vec<(TypeError, Span)> = vec![
        (
            TypeError::new(
                cyclic_span.clone(),
                TypeErrorKind::CyclicSubtype {
                    child: "a".into(),
                    parent: "b".into(),
                },
            ),
            cyclic_span,
        ),
        (
            TypeError::new(
                reserved_span.clone(),
                TypeErrorKind::ReservedPrefix {
                    name: "wat::x".into(),
                },
            ),
            reserved_span,
        ),
    ];

    for (err, expected_span) in &variants_under_test {
        // Universal single-path access — works for EVERY TypeError regardless
        // of which kind variant. The whole point of Pattern A.
        let actual_span: &Span = err.span();
        assert_eq!(actual_span, expected_span);
    }
}
