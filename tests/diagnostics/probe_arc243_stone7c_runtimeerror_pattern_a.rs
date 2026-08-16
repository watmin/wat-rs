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
    let _span: &Span = err.span();
    assert!(matches!(err.kind(), RuntimeErrorKind::DivisionByZero));
}

/// Contract 2: RuntimeErrorKind variants do NOT carry per-variant span fields.
/// `UnboundSymbol` was `UnboundSymbol(String, Span)` pre-stone; post-stone the
/// span lives on the outer struct and the kind holds only the domain data.
#[test]
fn runtimeerrorkind_variants_have_no_span_field() {
    let kind = RuntimeErrorKind::UnboundSymbol("x".to_string());
    let err = RuntimeError::new(wat::rust_caller_span!(), kind);
    let _span: &Span = err.span();
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
        let _span: &Span = err.span(); // single-path access for EVERY kind
    }
}

/// Contract 4 (SUPERSEDED 2026-08-16, arc 298 Stone 298.2 — rewritten, not recaptured):
/// this used to assert that the freeze pair's Display *elides* its outer span entirely
/// (no source location — the constructor's only option was `wat::rust_caller_span!()`,
/// treated as "unknown"). Arc 298.2 (`DESIGN-298.2-annihilate-span-unknown.md`)
/// deliberately annihilated `Span::unknown()` / `is_unknown()`: *"every span is real, so
/// the elide/skip logic that existed to hide `<runtime>:0:0` noise retires too"*
/// (`src/to_edn.rs:214-216`). The freeze pair still constructs with the honest
/// Rust-caller fallback span (no wat source location is available at this call site),
/// but Display now surfaces it rather than hiding it — that is the whole point of 298.2's
/// "no sentinel, no elision" cure. Rewritten to pin the NEW honest-location contract
/// rather than the sentinel-elision contract 298.2 killed. (Note, out of this brief's
/// `src/` blast radius: `src/value/signal.rs`'s doc comments at the `RuntimeError` struct
/// and `UserMainMissing` variant still say "honestly elided by Display" — that prose is
/// now stale and should be corrected in a future `src/` strike, not here.)
#[test]
fn runtimeerror_freeze_pair_carries_real_rust_caller_location() {
    let err = RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::UserMainMissing);
    let rendered = err.to_string();
    // rune:lint(loose-assert) — the message is a fixed string but sits inside an EDN blob
    // whose :location embeds rust_caller_span!()'s absolute host filesystem path to THIS
    // test file (varies by host/checkout, and a golden captured with it would be
    // non-portable — unlike every other .edn golden in this dir, which pins a relative
    // wat-source or src/*.rs path). Targeted presence of the message is the real contract.
    assert!(
        rendered.contains(":user::main not defined — a wat program needs an entry point"),
        "298.2: message text must be preserved verbatim; got: {rendered}"
    );
    // rune:lint(loose-assert) — same as above: rendered embeds the absolute host
    // filesystem path via rust_caller_span!(); `file!()` alone (this file's own relative
    // path, always a substring of that absolute path) is host-portable. The presence of a
    // REAL, non-elided span naming this file is the real contract.
    assert!(
        rendered.contains(file!()),
        "298.2: the freeze pair's span is now a REAL rust-caller location (never elided); \
         got: {rendered}"
    );
    // rune:lint(loose-assert) — same as above: targeted absence of the <runtime> sentinel
    // (which 298.2 deleted entirely) is the real contract; the full string is host-specific.
    assert!(
        !rendered.contains("<runtime>"),
        "298.2 deleted the <runtime> sentinel entirely — it must never appear; got: {rendered}"
    );
}
