//! FM 2-bis probe for arc 233 Stone 233.2.h (TrackedValue mint).
//!
//! Asserts the TrackedValue type's contracts:
//! - Construction via `new(value, provenance)`
//! - Accessors `value()` (borrow), `provenance()` (borrow), `value_owned(self)`
//! - `impl From<Value> for TrackedValue` (provenance = Unknown)
//! - Clone, Debug derived
//! - NOT derived: Eq/PartialEq/Hash (callers compare via .value()/.provenance() explicitly)
//!
//! Pre-stone state: FAILS (TrackedValue doesn't exist).
//! Post-stone state: PASSES (the 6 contracts hold).
//!
//! Stays as permanent regression guard against future TrackedValue shape drift.
//!
//! Per Stone 233.2.g sub-DESIGN: TrackedValue is the substrate's eval-boundary
//! type. It lives parallel to Value::Tracked variant in this stone (Stone 233.2.i+
//! flip eval signature; Stone 233.2.k retires Value::Tracked entirely).

use std::sync::Arc;
use wat::runtime::Value;
use wat::value::{Provenance, TrackedValue};
use wat::span::Span;

// ─── Probe 1 — Construction + value() borrow accessor ───────────────────────

#[test]
fn probe_1_new_and_value_borrow_accessor() {
    let tv = TrackedValue::new(Value::i64(42), Provenance::Unknown);
    assert!(
        matches!(tv.value(), Value::i64(42)),
        "value() must borrow the inner Value::i64(42)"
    );
}

// ─── Probe 2 — provenance() borrow accessor ─────────────────────────────────

#[test]
fn probe_2_provenance_borrow_accessor() {
    let prov = Provenance::RuntimeBuilt {
        producer: ":wat::test::probe",
        call_span: wat::rust_caller_span!(),
    };
    let tv = TrackedValue::new(Value::i64(7), prov);

    assert!(
        matches!(
            tv.provenance(),
            Provenance::RuntimeBuilt { producer: ":wat::test::probe", .. }
        ),
        "provenance() must borrow the RuntimeBuilt producer; got {:?}",
        tv.provenance()
    );
}

// ─── Probe 3 — value_owned consumes self ────────────────────────────────────

#[test]
fn probe_3_value_owned_consumes_self() {
    let tv = TrackedValue::new(Value::i64(99), Provenance::Unknown);
    let owned: Value = tv.value_owned();
    assert!(
        matches!(owned, Value::i64(99)),
        "value_owned() must consume self and yield bare Value::i64(99)"
    );
}

// ─── Probe 4 — From<Value> wraps with Unknown provenance ────────────────────

#[test]
fn probe_4_from_value_yields_unknown_provenance() {
    let v = Value::String(Arc::new("hello".to_string()));
    let tv: TrackedValue = v.into();
    assert!(
        matches!(tv.provenance(), Provenance::Unknown),
        "From<Value> must wrap with Provenance::Unknown; got {:?}",
        tv.provenance()
    );
    assert!(
        matches!(tv.value(), Value::String(s) if s.as_str() == "hello"),
        "From<Value> must preserve the inner value"
    );
}

// ─── Probe 5 — Clone produces equal value + provenance ──────────────────────

#[test]
fn probe_5_clone_preserves_value_and_provenance() {
    let prov = Provenance::RuntimeBuilt {
        producer: ":wat::test::clone",
        call_span: wat::rust_caller_span!(),
    };
    let tv = TrackedValue::new(Value::i64(13), prov);
    let cloned = tv.clone();

    assert!(
        matches!(cloned.value(), Value::i64(13)),
        "Clone must preserve inner value"
    );
    assert!(
        matches!(
            cloned.provenance(),
            Provenance::RuntimeBuilt { producer: ":wat::test::clone", .. }
        ),
        "Clone must preserve provenance"
    );
}

// ─── Probe 6 — Debug renders both value and provenance ──────────────────────

#[test]
fn probe_6_debug_includes_value_and_provenance() {
    let tv = TrackedValue::new(Value::i64(42), Provenance::Unknown);
    let debug_str = format!("{:?}", tv);
    assert_eq!(
        debug_str,
        "TrackedValue { value: i64(42), provenance: Unknown }",
        "Debug output must include value and provenance"
    );
}
