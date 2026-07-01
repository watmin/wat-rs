//! Arc 296 probe — macro/startup diagnostic chain serializes as fully structured EDN.
//!
//! Asserts that `startup_error_to_edn` and `macro_error_to_edn` mint
//! STRUCTURED `OwnedValue` trees, NOT prose strings.
//!
//! Pre-arc-296:
//!   - `wat::macros::error_edn` module does not exist;
//!   - `startup_error_to_edn` / `macro_error_to_edn` do not exist;
//!   - Probes 1-3 FAIL to compile.
//!
//! Post-arc-296: all 3 PASS.
//!
//! Permanent regression guard. Per arc 296 thesis: the leaf cause of a
//! `StartupError::Macro` must be machine-navigable EDN, not a prose string
//! that embeds the error message.

use std::sync::Arc;
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::freeze::StartupError;
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat_edn::OwnedValue;

// ─── Probe 1 — startup_error_to_edn produces Tagged OwnedValue ──────────────

#[test]
fn probe_1_startup_error_to_edn_is_tagged() {
    // Build the error chain:
    //   StartupError::Macro(
    //     MacroError { ProgramBodyEvalFailed {
    //       cause: MacroError { MacroEvalRuntimeFailed {
    //         cause: RuntimeError::UnboundSymbol("str") }}}})
    let runtime_span = Span::new(Arc::new("test.wat".to_string()), 3, 5);
    let runtime_err = RuntimeError {
        span: runtime_span.clone(),
        kind: RuntimeErrorKind::UnboundSymbol("str".into()),
    };

    let inner_macro_err = MacroError {
        span: runtime_span.clone(),
        kind: MacroErrorKind::MacroEvalRuntimeFailed {
            cause: Box::new(runtime_err),
        },
    };

    let outer_macro_err = MacroError {
        span: Span::new(Arc::new("test.wat".to_string()), 1, 1),
        kind: MacroErrorKind::ProgramBodyEvalFailed {
            macro_name: "my-macro".to_string(),
            cause: Box::new(inner_macro_err),
        },
    };

    let startup_err = StartupError::Macro(outer_macro_err);

    // Pre-arc-296: this function does not exist — FAILS to compile.
    let edn = wat::macros::error_edn::startup_error_to_edn(&startup_err);

    // The top-level result must be a Tagged value (not a String).
    // `startup_error_to_edn(StartupError::Macro(e))` delegates to
    // `macro_error_to_edn(e)` which returns the specific variant tag.
    assert!(
        matches!(&edn, OwnedValue::Tagged(..)),
        "startup_error_to_edn must produce a Tagged OwnedValue; got {:?}",
        edn
    );

    // Serialize and verify the structured chain is present.
    let serialized = wat_edn::write(&edn);

    // Must contain the leaf cause: the UnboundSymbol "str".
    assert!(
        serialized.contains("UnboundSymbol"),
        "startup_error_to_edn for Macro(..) must transitively surface 'UnboundSymbol'; got: {}",
        serialized
    );
    assert!(
        serialized.contains("\"str\""),
        "startup_error_to_edn must surface the symbol name 'str'; got: {}",
        serialized
    );

    // Must NOT be a bare String (the old prose-collapse behavior).
    assert!(
        !matches!(&edn, OwnedValue::String(_)),
        "startup_error_to_edn must NOT return a bare String; got {:?}",
        edn
    );

    // Round-trip.
    let _parsed = wat_edn::parse_owned(&serialized).expect("round-trip must parse");
}

// ─── Probe 2 — macro_error_to_edn is navigable to the leaf cause ────────────

#[test]
fn probe_2_macro_error_to_edn_leaf_cause_is_not_string() {
    let runtime_span = Span::new(Arc::new("src/my-lib.wat".to_string()), 18, 3);
    let runtime_err = RuntimeError {
        span: runtime_span.clone(),
        kind: RuntimeErrorKind::UnboundSymbol("str".into()),
    };

    let inner = MacroError {
        span: runtime_span.clone(),
        kind: MacroErrorKind::MacroEvalRuntimeFailed {
            cause: Box::new(runtime_err),
        },
    };

    let outer = MacroError {
        span: Span::new(Arc::new("src/my-lib.wat".to_string()), 10, 1),
        kind: MacroErrorKind::ProgramBodyEvalFailed {
            macro_name: "expand-call".to_string(),
            cause: Box::new(inner),
        },
    };

    // Arc 298.3: now calls the derive-generated ToEdn impl.
    let edn = outer.to_edn();
    let serialized = wat_edn::write(&edn);

    eprintln!("=== probe_2 serialized: {}", serialized);

    // The serialized form must NOT be a bare string containing "unbound symbol".
    assert!(
        !matches!(&edn, OwnedValue::String(_)),
        "macro_error_to_edn must NOT be a bare String; got {:?}",
        edn
    );

    // The serialized form MUST carry "UnboundSymbol" as a tag.
    assert!(
        serialized.contains("UnboundSymbol"),
        "serialized macro error must carry 'UnboundSymbol' tag; got: {}",
        serialized
    );

    // The serialized form MUST carry the symbol name.
    assert!(
        serialized.contains("str"),
        "serialized macro error must carry ':name \"str\"'; got: {}",
        serialized
    );
}

// ─── Probe 3 — MacroEvalRuntimeFailed variant carries the cause ──────────────

#[test]
fn probe_3_macro_eval_runtime_failed_variant_exists() {
    let span = Span::new(Arc::new("test.wat".to_string()), 5, 3);
    let runtime_err = RuntimeError {
        span: span.clone(),
        kind: RuntimeErrorKind::UnboundSymbol("my-symbol".into()),
    };

    let err = MacroError {
        span: span.clone(),
        kind: MacroErrorKind::MacroEvalRuntimeFailed {
            cause: Box::new(runtime_err),
        },
    };

    // Verify the cause is recoverable.
    assert!(
        matches!(&err.kind, MacroErrorKind::MacroEvalRuntimeFailed { cause }
            if matches!(&cause.kind, RuntimeErrorKind::UnboundSymbol(name) if name == "my-symbol")),
        "MacroEvalRuntimeFailed must carry the RuntimeError cause; got {:?}",
        err.kind
    );
}
