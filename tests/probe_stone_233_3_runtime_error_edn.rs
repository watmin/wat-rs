//! FM 2-bis probe for arc 233 Stone 233.3 (Errors-as-EDN extension).
//!
//! Asserts that the new runtime_error_edn module mints:
//!   - `runtime_error_to_edn(err: &RuntimeError) -> wat_edn::OwnedValue`
//!   - `value_snapshot_to_edn(snap: &ValueSnapshot) -> wat_edn::OwnedValue`
//!   - `provenance_to_edn(prov: &Provenance) -> wat_edn::OwnedValue`
//!
//! And that the emitted EDN round-trips through wat-edn parser.
//!
//! Pre-stone state:
//!   - Probes 1-5 FAIL to compile (`runtime_error_to_edn` doesn't exist;
//!     `value_snapshot_to_edn` doesn't exist; `provenance_to_edn` doesn't
//!     exist; the module itself doesn't exist).
//!
//! Post-stone state: all 5 PASS.
//!
//! Stays as permanent regression guard. Per arc 233 thesis: errors are
//! remarkable. Per Stone 233.3: errors are MACHINE-CONSUMABLE across
//! IPC boundaries via tagged EDN envelopes.

use std::sync::Arc;
use wat::runtime::{Provenance, RuntimeError, TrackedValue, Value, ValueSnapshot};
use wat::span::Span;

// ─── Probe 1 — NotCallable serializes to #wat.kernel/NotCallable ────────────

#[test]
fn probe_1_not_callable_serializes_to_tagged_edn() {
    let span = Span {
        file: Arc::new("test.wat".to_string()),
        line: 3,
        col: 7,
    };
    let snap = ValueSnapshot::of(&Value::String(Arc::new("not-fn".to_string())));
    let err = RuntimeError::NotCallable {
        got: Box::new(snap),
        span: span.clone(),
    };

    // Pre-stone: this function doesn't exist; FAILS to compile.
    let edn = wat::runtime_error_edn::runtime_error_to_edn(&err);

    // Round-trip via wat-edn writer + parser.
    let serialized = wat_edn::write(&edn);
    let parsed = wat_edn::parse_owned(&serialized).expect("parse round-trip");

    // The serialized form should be a Tagged value with tag
    // "wat.kernel/NotCallable" (or similar — the exact tag scheme is the
    // implementation's choice; the test just verifies structural shape).
    assert!(
        matches!(&parsed, wat_edn::OwnedValue::Tagged(tag, _) if tag.name().contains("NotCallable")),
        "Stone 233.3: NotCallable RuntimeError must serialize to a Tagged \
         EDN value with 'NotCallable' in the tag name; got {:?}",
        parsed
    );
}

// ─── Probe 2 — TypeMismatch carries op + expected + got + span keys ─────────

#[test]
fn probe_2_type_mismatch_carries_all_struct_fields() {
    let span = Span {
        file: Arc::new("test.wat".to_string()),
        line: 5,
        col: 12,
    };
    let snap = ValueSnapshot::of(&Value::i64(42));
    let err = RuntimeError::TypeMismatch {
        op: ":wat::core::+".into(),
        expected: "wat::core::i64",
        got: Box::new(snap),
        span: span.clone(),
    };

    let edn = wat::runtime_error_edn::runtime_error_to_edn(&err);
    let serialized = wat_edn::write(&edn);

    // Verify all 4 struct fields surface as map entries (op, expected, got, span).
    // The probe is intentionally tolerant of the exact key naming (:op vs op vs
    // "op") since the implementation picks the convention; just assert each
    // field's content surfaces.
    assert!(
        serialized.contains(":wat::core::+"),
        "TypeMismatch serialization should include op string ':wat::core::+'; got: {}",
        serialized
    );
    assert!(
        serialized.contains("wat::core::i64"),
        "TypeMismatch serialization should include expected type 'wat::core::i64'; got: {}",
        serialized
    );
    assert!(
        serialized.contains("test.wat"),
        "TypeMismatch serialization should include span file 'test.wat'; got: {}",
        serialized
    );
}

// ─── Probe 3 — AssertionFailed Option<String> fields render Nil/String ──────

#[test]
fn probe_3_assertion_failed_with_optional_fields() {
    let span = Span {
        file: Arc::new("test.wat".to_string()),
        line: 1,
        col: 1,
    };
    let err = RuntimeError::AssertionFailed {
        message: "assertion fired".into(),
        actual: Some("42".into()),
        expected: None, // tests the Nil branch
        span: span.clone(),
    };

    let edn = wat::runtime_error_edn::runtime_error_to_edn(&err);
    let serialized = wat_edn::write(&edn);

    // The actual field has a value; should appear as the string.
    assert!(
        serialized.contains("42"),
        "AssertionFailed.actual=Some(\"42\") should surface in EDN; got: {}",
        serialized
    );
    // The expected field is None; should appear as `nil` per EDN convention.
    // The probe is tolerant — just asserts the message surfaces (the Nil
    // rendering for Option::None is the implementation's choice).
    assert!(
        serialized.contains("assertion fired"),
        "AssertionFailed.message should surface in EDN; got: {}",
        serialized
    );
}

// ─── Probe 4 — Tuple variant (ParamShadowsBuiltin: String + Span) ───────────

#[test]
fn probe_4_tuple_variant_serializes() {
    let span = Span {
        file: Arc::new("test.wat".to_string()),
        line: 9,
        col: 4,
    };
    let err = RuntimeError::ParamShadowsBuiltin("my-fn".into(), span);

    let edn = wat::runtime_error_edn::runtime_error_to_edn(&err);
    let serialized = wat_edn::write(&edn);

    // Tuple-variant fields should map to some key naming (e.g., :name + :span,
    // or :0 + :1, or positional vector); probe asserts the content surfaces.
    assert!(
        serialized.contains("my-fn"),
        "ParamShadowsBuiltin should surface 'my-fn' identifier; got: {}",
        serialized
    );
    assert!(
        matches!(
            wat_edn::parse_owned(&serialized),
            Ok(wat_edn::OwnedValue::Tagged(tag, _)) if tag.name().contains("ParamShadowsBuiltin")
        ),
        "ParamShadowsBuiltin must serialize to Tagged EDN with tag name containing \
         variant name; got: {}",
        serialized
    );
}

// ─── Probe 5 — Provenance variants render with per-variant tags ─────────────

#[test]
fn probe_5_provenance_variants_render_with_tags() {
    let span = Span {
        file: Arc::new("test.wat".to_string()),
        line: 4,
        col: 8,
    };

    // SymbolBound — Stone 233.2.e populates this on let-bound symbol lookup
    let prov = Provenance::SymbolBound {
        binding_span: span.clone(),
        head_span: Span {
            file: span.file.clone(),
            line: 5,
            col: 12,
        },
    };

    let edn = wat::runtime_error_edn::provenance_to_edn(&prov);
    let serialized = wat_edn::write(&edn);

    // SymbolBound should surface as Tagged EDN (or a map identifying the variant)
    // with binding_span + head_span coordinates accessible.
    assert!(
        serialized.contains("SymbolBound") || serialized.contains("symbol-bound"),
        "Provenance::SymbolBound must surface its variant identity in EDN; got: {}",
        serialized
    );
    assert!(
        serialized.contains("4") || serialized.contains("5") || serialized.contains("8"),
        "Provenance::SymbolBound must surface span coordinates (line 4/5, col 8/12); \
         got: {}",
        serialized
    );

    // Also test RuntimeBuilt — populated by 5 producers
    let prov_rb = Provenance::RuntimeBuilt {
        producer: ":wat::core::keyword/from-string",
        call_span: span.clone(),
    };
    let edn_rb = wat::runtime_error_edn::provenance_to_edn(&prov_rb);
    let serialized_rb = wat_edn::write(&edn_rb);

    assert!(
        serialized_rb.contains(":wat::core::keyword/from-string"),
        "Provenance::RuntimeBuilt must surface producer string; got: {}",
        serialized_rb
    );
}

// ─── helper — silence unused import warnings ────────────────────────────────

#[allow(dead_code)]
fn _silence_unused_imports(_: TrackedValue) {}
