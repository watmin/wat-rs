//! FM 2-bis probe for arc 233 Stone 233.3 (Errors-as-EDN extension).
//!
//! Asserts that the new `edn::error` module mints:
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
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use wat::value::{Provenance, TrackedValue};
use wat::span::Span;
use wat::edn::contract::ToEdn;

// ─── Probe 1 — NotCallable serializes to #wat.kernel/NotCallable ────────────

#[test]
fn probe_1_not_callable_serializes_to_tagged_edn() {
    let span = Span::new(Arc::new("test.wat".to_string()), 3, 7);
    let snap = ValueSnapshot::of(&Value::String(Arc::new("not-fn".to_string())));
    let err = RuntimeError::new(span.clone(), RuntimeErrorKind::NotCallable {
        got: Box::new(snap)
    });

    // Arc 298.3: now calls the derive-generated ToEdn impl.
    let edn = err.to_edn();

    // Round-trip via wat-edn writer + parser.
    let serialized = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(serialized.clone(), "probe_stone_233_3_runtime_error_edn__not_callable.edn", "Stone 233.3: NotCallable RuntimeError must serialize to exact tagged EDN");
    let parsed = wat_edn::parse_owned(&serialized).expect("parse round-trip");
    assert!(
        matches!(&parsed, wat_edn::OwnedValue::Tagged(tag, _) if tag.name() == "NotCallable"),
        "parsed EDN must be Tagged with exact 'NotCallable' tag name; got {:?}",
        parsed
    );
}

// ─── Probe 2 — TypeMismatch carries op + expected + got + span keys ─────────

#[test]
fn probe_2_type_mismatch_carries_all_struct_fields() {
    let span = Span::new(Arc::new("test.wat".to_string()), 5, 12);
    let snap = ValueSnapshot::of(&Value::i64(42));
    let err = RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
        op: ":wat::core::+".into(),
        expected: "wat::core::i64",
        got: Box::new(snap)
    });

    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);

    // All 4 struct fields (op, expected, got, span) must surface in exact EDN.
    wat::assert_edn_matches_file!(serialized, "probe_stone_233_3_runtime_error_edn__type_mismatch.edn", "TypeMismatch serialization must include all fields: op, expected, got, span");
}

// ─── Probe 3 — AssertionFailed Option<String> fields render Nil/String ──────

#[test]
fn probe_3_assertion_failed_with_optional_fields() {
    let span = Span::new(Arc::new("test.wat".to_string()), 1, 1);
    let err = RuntimeError::new(span.clone(), RuntimeErrorKind::AssertionFailed {
        message: "assertion fired".into(),
        actual: Some("42".into()),
        expected: None, // tests the Nil branch
    });

    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);

    wat::assert_edn_matches_file!(serialized, "probe_stone_233_3_runtime_error_edn__assertion_failed.edn", "AssertionFailed must surface message, actual (Some), and expected (None/nil)");
}

// ─── Probe 4 — Tuple variant (ParamShadowsBuiltin: String + Span) ───────────

#[test]
fn probe_4_tuple_variant_serializes() {
    let span = Span::new(Arc::new("test.wat".to_string()), 9, 4);
    let err = RuntimeError::new(span, RuntimeErrorKind::ParamShadowsBuiltin("my-fn".into()));

    let edn = err.to_edn();
    let serialized = wat_edn::write(&edn);

    wat::assert_edn_matches_file!(serialized, "probe_stone_233_3_runtime_error_edn__param_shadows_builtin.edn", "ParamShadowsBuiltin must serialize to exact tagged EDN with :name and :span");
}

// ─── Probe 5 — Provenance variants render with per-variant tags ─────────────

#[test]
fn probe_5_provenance_variants_render_with_tags() {
    let span = Span::new(Arc::new("test.wat".to_string()), 4, 8);

    // SymbolBound — Stone 233.2.e populates this on let-bound symbol lookup
    let prov = Provenance::SymbolBound {
        binding_span: span.clone(),
        head_span: Span::new(span.file.clone(), 5, 12),
    };

    let edn = wat::edn::error::provenance_to_edn(&prov);
    let serialized = wat_edn::write(&edn);

    wat::assert_edn_matches_file!(serialized, "probe_stone_233_3_runtime_error_edn__provenance_symbol_bound.edn", "Provenance::SymbolBound must surface exact binding-span + head-span in EDN");

    // Also test RuntimeBuilt — populated by 5 producers
    let prov_rb = Provenance::RuntimeBuilt {
        producer: ":wat::core::keyword/from-string",
        call_span: span.clone(),
    };
    let edn_rb = wat::edn::error::provenance_to_edn(&prov_rb);
    let serialized_rb = wat_edn::write(&edn_rb);

    wat::assert_edn_matches_file!(serialized_rb, "probe_stone_233_3_runtime_error_edn__provenance_runtime_built.edn", "Provenance::RuntimeBuilt must surface exact producer + call-span in EDN");
}

// ─── helper — silence unused import warnings ────────────────────────────────

#[allow(dead_code)]
fn _silence_unused_imports(_: TrackedValue) {}
