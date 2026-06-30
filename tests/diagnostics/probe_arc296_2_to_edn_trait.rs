//! Arc 296 slice 2 probe — `ToEdn` trait unifies all error serializers.
//!
//! Asserts that every error type can call `.to_edn()` and produces the
//! same result as the existing free function (behavior-preserving).
//!
//! RED before 296.2: `wat::to_edn::ToEdn` does not exist; `.to_edn()`
//! method is not callable — FAILS to compile.
//!
//! GREEN after 296.2: all impls present; behavior-preserving check passes.

use std::sync::Arc;
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::freeze::StartupError;
use wat::span::Span;
use wat::to_edn::ToEdn;

// ─── Probe 1 — RuntimeError.to_edn() matches runtime_error_to_edn() ─────────

#[test]
fn probe_1_runtime_error_to_edn_behavior_preserving() {
    let span = Span::new(Arc::new("test.wat".to_string()), 3, 5);
    let err = RuntimeError {
        span: span.clone(),
        kind: RuntimeErrorKind::UnboundSymbol("foo".into()),
    };

    // Trait path and free-function path must produce identical output.
    let via_trait = err.to_edn();
    let via_fn = wat::runtime_error_edn::runtime_error_to_edn(&err);

    let trait_str = wat_edn::write(&via_trait);
    let fn_str = wat_edn::write(&via_fn);
    assert_eq!(
        trait_str, fn_str,
        "RuntimeError.to_edn() must equal runtime_error_to_edn(); trait={} fn={}",
        trait_str, fn_str
    );
}

// ─── Probe 2 — Span.to_edn() produces structured {:file :line :col} map ──────

#[test]
fn probe_2_span_to_edn_is_structured_map() {
    let span = Span::new(Arc::new("src/lib.wat".to_string()), 10, 3);

    let edn = span.to_edn();
    let s = wat_edn::write(&edn);

    // Must be a map, not a bare string.
    assert!(
        matches!(&edn, wat_edn::OwnedValue::Map(_)),
        "Span.to_edn() must produce a Map; got {:?}",
        edn
    );
    // Must contain the expected fields.
    assert!(s.contains("\"src/lib.wat\""), "span edn must contain file; got: {}", s);
    assert!(s.contains(":line"), "span edn must contain :line; got: {}", s);
    assert!(s.contains(":col"), "span edn must contain :col; got: {}", s);
}

// ─── Probe 3 — MacroError.to_edn() matches macro_error_to_edn() ─────────────

#[test]
fn probe_3_macro_error_to_edn_behavior_preserving() {
    let span = Span::new(Arc::new("test.wat".to_string()), 5, 1);
    let err = MacroError {
        span: span.clone(),
        kind: MacroErrorKind::DuplicateMacro("my-macro".into()),
    };

    let via_trait = err.to_edn();
    let via_fn = wat::macros::error_edn::macro_error_to_edn(&err);

    let trait_str = wat_edn::write(&via_trait);
    let fn_str = wat_edn::write(&via_fn);
    assert_eq!(
        trait_str, fn_str,
        "MacroError.to_edn() must equal macro_error_to_edn(); trait={} fn={}",
        trait_str, fn_str
    );
}

// ─── Probe 4 — StartupError.to_edn() matches startup_error_to_edn() ─────────

#[test]
fn probe_4_startup_error_to_edn_behavior_preserving() {
    let span = Span::new(Arc::new("test.wat".to_string()), 1, 1);
    let macro_err = MacroError {
        span: span.clone(),
        kind: MacroErrorKind::MalformedDefmacro {
            reason: "test reason".into(),
        },
    };
    let startup_err = StartupError::Macro(macro_err);

    let via_trait = startup_err.to_edn();
    let via_fn = wat::macros::error_edn::startup_error_to_edn(&startup_err);

    let trait_str = wat_edn::write(&via_trait);
    let fn_str = wat_edn::write(&via_fn);
    assert_eq!(
        trait_str, fn_str,
        "StartupError.to_edn() must equal startup_error_to_edn(); trait={} fn={}",
        trait_str, fn_str
    );
}
