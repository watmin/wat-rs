//! Probe — arc 243 Stone 243.7c — RuntimeError Pattern A structural verification
//!
//! FM 2-bis disconfirming probe: asserts the POST-stone shape.
//!
//! - PRE-stone state: this probe FAILS TO COMPILE. `RuntimeError` is currently
//!   a flat ~30-variant enum (`pub enum RuntimeError { Variant { ..., span },
//!   ... }`) where each variant carries its own `span` (or, for the freeze pair
//!   `UserMainMissing`/`EvalVerificationFailed`, none). There is no
//!   `RuntimeErrorKind` type and `RuntimeError` has no `kind`/`span` struct
//!   fields. Constructing `RuntimeError { span, kind }` is a type error
//!   (E0574 — enum, not struct), and `RuntimeErrorKind` is unresolved (E0433).
//! - POST-stone state: this probe COMPILES + PASSES. `RuntimeError` is the
//!   Pattern A shape (`pub struct RuntimeError { pub span: Span, pub kind:
//!   RuntimeErrorKind }`); the constructor requires `span` at the outer field;
//!   the variants live on `RuntimeErrorKind` with no per-variant `span`
//!   (multi-span variants keep only their SECONDARY spans as domain-named kind
//!   fields per CONFORMARE.md § Multi-span; the freeze pair has no span and
//!   constructs with outer `wat::rust_caller_span!()`, honestly elided).
//!
//! Mirrors `tests/probe_arc243_stone6_checkerror_pattern_a.rs` (the CheckError
//! Pattern A, shipped 243.6a). The signal split (243.7b) already removed the
//! control signals, so `RuntimeError` here is pure-diagnostic.

use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::span::Span;

/// Contract 1: RuntimeError carries `span: Span` at the outer struct level —
/// every variant inherits the location discipline by construction.
#[test]
fn runtimeerror_outer_span_field_required() {
    let err = RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::DivisionByZero);
    // Universal span access — no exhaustive match across ~30 variants.
    let _span: &Span = &err.span();
    assert!(matches!(err.kind(), RuntimeErrorKind::DivisionByZero));
}

/// Contract 2: RuntimeErrorKind variants do NOT carry per-variant span fields.
/// `UnboundSymbol` was `UnboundSymbol(String, Span)` pre-stone; post-stone the
/// span lives on the outer struct and the kind holds only the domain data.
#[test]
fn runtimeerrorkind_variants_have_no_span_field() {
    let kind = RuntimeErrorKind::UnboundSymbol("x".to_string());
    let err = RuntimeError::new(wat::rust_caller_span!(), kind);
    let _span: &Span = &err.span();
    assert!(matches!(err.kind(), RuntimeErrorKind::UnboundSymbol(_)));
}

/// Contract 3: Span access is universal — no exhaustive match across ~30
/// variants. Any RuntimeError consumer needing span data reads `err.span`.
#[test]
fn runtimeerror_span_access_is_single_path() {
    let errs: Vec<RuntimeError> = vec![
        RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::DivisionByZero),
        RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::UnboundSymbol("y".into())),
    ];
    for err in &errs {
        let _span: &Span = &err.span(); // single-path access for EVERY kind
    }
}

/// Contract 4: the freeze pair (no source span) elides an unknown outer span —
/// `UserMainMissing` carries `wat::rust_caller_span!()` and must not emit "<runtime>".
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn runtimeerror_freeze_pair_elides_unknown_span() {
    let err = RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::UserMainMissing);
    let rendered = err.to_string();
    assert_eq!(
        rendered,
        ":user::main not defined — a wat program needs an entry point",
        "unknown span must be elided in Display output"
    );
}
