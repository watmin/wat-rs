//! Arc 296 typed-causes probe — S1, S2.
//!
//! Verifies that:
//!
//! - **S1** `MacroExpansionFailed` EDN carries a nested `#wat.kernel/…`
//!   `:cause`, NOT a `:reason` String.
//! - **S2** `process_died_error_runtime_value` builder produces a structured
//!   payload (tagged EDN) rather than an `e.to_string()` prose string.
//!
//! **S6** (`StdlibError::ParseFailed` `:cause`) is tested as a crate-internal
//! `#[test]` in `src/stdlib.rs` (the type is `pub(crate)` and not constructible
//! from integration tests).

use std::sync::Arc;
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat_edn::OwnedValue;

fn make_span() -> Span {
    Span::new(Arc::new("probe.wat".to_string()), 1, 1)
}

// ─── S1 — MacroExpansionFailed carries a nested :cause, not a :reason string ──

#[test]
fn s1_macro_expansion_failed_carries_typed_cause_not_reason_string() {
    // Build a RuntimeError that wraps a MacroExpansionFailed whose cause is
    // a typed MacroError (ArityMismatch).
    let inner_macro_err = MacroError {
        span: make_span(),
        kind: MacroErrorKind::ArityMismatch {
            name: ":user::my-macro".into(),
            expected: 1,
            got: 2,
        },
    };

    let runtime_err = RuntimeError {
        span: make_span(),
        kind: RuntimeErrorKind::MacroExpansionFailed {
            op: ":wat::core::macroexpand-1".into(),
            cause: Box::new(inner_macro_err),
        },
    };

    let edn = runtime_err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== S1 MacroExpansionFailed edn: {}", s);

    // Must be a tagged form.
    assert!(s.starts_with('#'), "must be tagged EDN; got: {}", s);

    // Must be in the wat.runtime namespace.
    assert!(
        s.contains("wat.runtime"),
        "must use wat.runtime namespace; got: {}",
        s
    );

    // Must carry :cause, NOT :reason (the old prose field).
    assert!(
        s.contains(":cause"),
        "MacroExpansionFailed EDN must carry :cause (typed nested error); got: {}",
        s
    );
    assert!(
        !s.contains(":reason"),
        "MacroExpansionFailed EDN must NOT carry old :reason String; got: {}",
        s
    );

    // The :cause must be a nested tagged MacroError (contains a #wat.macro/ tag).
    assert!(
        s.contains("#wat.macro/"),
        "MacroExpansionFailed :cause must be a nested #wat.macro/... tagged EDN; got: {}",
        s
    );

    // The inner ArityMismatch variant must be identifiable.
    assert!(
        s.contains("ArityMismatch"),
        ":cause must surface the inner ArityMismatch variant; got: {}",
        s
    );

    // Must be valid EDN.
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── S1 — ExpansionDepthExceeded cause (fixpoint site) ───────────────────────

#[test]
fn s1_macro_expansion_failed_fixpoint_site_carries_depth_exceeded_cause() {
    // The fixpoint site synthesises an ExpansionDepthExceeded MacroError.
    let cause = MacroError {
        span: make_span(),
        kind: MacroErrorKind::ExpansionDepthExceeded { limit: 512 },
    };

    let runtime_err = RuntimeError {
        span: make_span(),
        kind: RuntimeErrorKind::MacroExpansionFailed {
            op: ":wat::core::macroexpand".into(),
            cause: Box::new(cause),
        },
    };

    let edn = runtime_err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== S1 fixpoint MacroExpansionFailed edn: {}", s);

    assert!(s.contains(":cause"), "must carry :cause; got: {}", s);
    assert!(
        !s.contains(":reason"),
        "must NOT carry :reason; got: {}",
        s
    );
    assert!(
        s.contains("ExpansionDepthExceeded"),
        ":cause must surface ExpansionDepthExceeded variant; got: {}",
        s
    );
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── S2 — process_died_error_runtime_value builder uses structured EDN ────────
//
// process_died_error_runtime_value is pub(crate); we verify the builder path
// via the underlying RuntimeError-as-WatError wire form. The two bypass sites
// now call `process_died_error_runtime_value(&e)` which internally calls
// `to_wire_edn(e)`. We confirm that `to_wire_edn` for a RuntimeError produces
// tagged EDN, not a bare prose string.

#[test]
fn s2_runtime_error_wire_edn_is_structured_not_prose() {
    let runtime_err = RuntimeError {
        span: make_span(),
        kind: RuntimeErrorKind::UnboundSymbol("some-var".into()),
    };

    // to_wire_edn is what process_died_error_runtime_value calls.
    let wire_edn = wat::to_edn::to_wire_edn(&runtime_err);

    eprintln!("=== S2 wire_edn: {}", wire_edn);

    // The wire form must be a tagged envelope (parsed EDN starts with #).
    assert!(
        wire_edn.starts_with('#'),
        "process_died_error_runtime_value wire payload must be tagged EDN; got: {}",
        wire_edn
    );

    // Must carry the floor fields (:message / :location / :causes).
    assert!(
        wire_edn.contains(":message"),
        "wire payload must carry :message floor field; got: {}",
        wire_edn
    );

    // The parsed form must be Tagged, never a bare String.
    let parsed = wat_edn::parse_owned(&wire_edn).expect("wire payload must be valid EDN");
    assert!(
        matches!(&parsed, OwnedValue::Tagged(..)),
        "wire payload must be a Tagged OwnedValue (structured), not a bare String; got: {:?}",
        parsed
    );
}
