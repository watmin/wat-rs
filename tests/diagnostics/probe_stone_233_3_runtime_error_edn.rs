//! FM 2-bis probe for arc 233 Stone 233.3 (Errors-as-EDN extension).
//!
//! Asserts that the `edn::error` module mints:
//!   - `runtime_error_to_edn(err: &RuntimeError) -> wat_edn::OwnedValue`
//!   - `value_snapshot_to_edn(snap: &ValueSnapshot) -> wat_edn::OwnedValue`
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
//!
//! Excursus 003 strike G item 4: `provenance_to_edn`/`Provenance` retired
//! wholesale (no production caller, per the audit) — probe 5, which drove
//! that fn directly, is retired with it. Probes 1/2 (RuntimeError) are
//! unaffected — `ValueSnapshot` never carried provenance (strike C already
//! removed that field).

use std::sync::Arc;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
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

// ─── Probes 3 & 4 — RETIRED (excursus 003 strike G item 1) ──────────────────
//
// `RuntimeErrorKind::AssertionFailed` and `RuntimeErrorKind::ParamShadowsBuiltin`
// were both measured dead (zero construction sites via the frozen-pipeline
// census, GD2a) and retired. `:wat::runtime::AssertionFailed` the WAT RECORD
// stays declared — assertion panics still build it directly via
// `assertion_failed_value` (`src/value/runtime_records.rs`), never through
// this `RuntimeErrorKind` variant — so its own EDN shape has no remaining
// probe here; `src/panic_hook.rs`'s own unit tests cover the panic path's
// `#wat.runtime/AssertionFailed` wire tag.

// ─── Probe 5 — RETIRED (excursus 003 strike G item 4) ───────────────────────
//
// `Provenance`/`provenance_to_edn` are gone (measured: no production caller).
// This probe drove `provenance_to_edn` directly on `Provenance::SymbolBound`/
// `RuntimeBuilt`; both the fn and the type it probed are retired together.
// Its 2 goldens (`..__provenance_symbol_bound.edn`, `..__provenance_runtime_built.edn`)
// are removed with it.
