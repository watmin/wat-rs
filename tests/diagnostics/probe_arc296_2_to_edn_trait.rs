//! Arc 296 slice 2 probe — `ToEdn` trait unifies all error serializers.
//!
//! Arc 298.3 update: probes 1 and 3 compared `.to_edn()` output against the
//! now-deleted `runtime_error_to_edn` / `macro_error_to_edn` free functions.
//! Those probes are deleted here as proven duplicates — the 298.3 golden probes
//! (`probe_arc298_3_runtime_derive_identical.rs`,
//! `probe_arc298_3_macro_derive_identical.rs`) cover the same variants with
//! stronger `assert_eq!` byte-identical assertions.
//!
//! Remaining probes:
//!   - Probe 2: `Span.to_edn()` produces structured `{:file :line :col}` map
//!   - Probe 4: `StartupError.to_edn()` matches `startup_error_to_edn()` (kept)
//!
//! RED before 296.2: `wat::edn::contract::ToEdn` does not exist; `.to_edn()`
//! method is not callable — FAILS to compile.
//!
//! GREEN after 296.2 / 298.3: all impls present.

use std::sync::Arc;
use wat::macros::{MacroError, MacroErrorKind};
use wat::freeze::StartupError;
use wat::span::Span;
use wat::edn::contract::ToEdn;

// ─── Probe 2 — Span.to_edn() produces structured {:file :line :col} map ──────

#[test]
fn probe_2_span_to_edn_is_structured_map() {
    let span = Span::new(Arc::new("src/lib.wat".to_string()), 10, 3);

    let edn = span.to_edn();
    let s = wat_edn::write(&edn);

    // Arc 298.3: upgraded from contains-checks to byte-identical assert_eq!
    wat::assert_edn_matches_file!(s, "probe_arc296_2_to_edn_trait__span_structured_map.edn", "Span.to_edn() must produce exact structured map");
    // Stone B (arc 296): Span is now a #[derive(ToEdn)] tagged record, so
    // to_edn() produces a Tagged(#wat.core/Span, Map{...}) rather than a
    // bare Map. The subject this assertion protects is unchanged: the span
    // must render as structured data, not as a string.
    assert!(
        matches!(&edn, wat_edn::OwnedValue::Tagged(_, body) if matches!(**body, wat_edn::OwnedValue::Map(_))),
        "Span.to_edn() must produce a Tagged(..., Map) OwnedValue; got {:?}",
        edn
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
