//! RED probe — arc 298.1: `Option` and `Result` serialize as the uniform
//! `#wat.core.<Type>/<Variant>` tagged form.
//!
//! RED at HEAD (before the strike lands):
//! - `Value::Option(None)` → `nil` (transparent special-case, not tagged)
//! - `Value::Option(Some("x"))` → `"x"` (inner unwrapped, not tagged)
//! - `Value::Result(Ok(42))` → `#wat-edn.result/ok 42` (codec-internal tag, lowercase)
//! - `Value::Result(Err("e"))` → `#wat-edn.result/err "e"` (codec-internal tag, lowercase)
//!
//! GREEN after 298.1: both discriminated types use `#wat.core.<Type>/<Variant>`.
//! - None → `#wat.core.Option/None nil`
//! - Some(v) → `#wat.core.Option/Some v`
//! - Ok(v) → `#wat.core.Result/Ok v`
//! - Err(e) → `#wat.core.Result/Err e`
//!
//! Round-trip: `edn_to_value(parse(write(v))) == v` for all four.

use std::sync::Arc;
use wat::edn_shim::{edn_to_value, value_to_edn};
use wat::runtime::Value;

fn write_value(v: &Value) -> String {
    wat_edn::write(&value_to_edn(v))
}

fn round_trip(v: &Value) -> Value {
    let s = write_value(v);
    let owned = wat_edn::parse_owned(&s).expect("parse EDN");
    edn_to_value(&owned, None).expect("decode Value")
}

// ── Write form: Option ────────────────────────────────────────────────────────

#[test]
fn option_none_serializes_as_tagged_none() {
    let v = Value::Option(Arc::new(None));
    let s = write_value(&v);
    assert_eq!(
        s, "#wat.core.Option/None nil",
        "Option(None) must serialize as #wat.core.Option/None nil; got: {s}"
    );
}

#[test]
fn option_some_serializes_as_tagged_some() {
    let v = Value::Option(Arc::new(Some(Value::String(Arc::new("x".into())))));
    let s = write_value(&v);
    assert_eq!(
        s, "#wat.core.Option/Some \"x\"",
        "Option(Some(\"x\")) must serialize as #wat.core.Option/Some \"x\"; got: {s}"
    );
}

// ── Write form: Result ────────────────────────────────────────────────────────

#[test]
fn result_ok_serializes_as_tagged_ok() {
    let v = Value::Result(Arc::new(Ok(Value::i64(42))));
    let s = write_value(&v);
    assert_eq!(
        s, "#wat.core.Result/Ok 42",
        "Result(Ok(42)) must serialize as #wat.core.Result/Ok 42; got: {s}"
    );
}

#[test]
fn result_err_serializes_as_tagged_err() {
    let v = Value::Result(Arc::new(Err(Value::String(Arc::new("e".into())))));
    let s = write_value(&v);
    assert_eq!(
        s, "#wat.core.Result/Err \"e\"",
        "Result(Err(\"e\")) must serialize as #wat.core.Result/Err \"e\"; got: {s}"
    );
}

// ── Round-trips (edn::write → edn::read == id) ───────────────────────────────

#[test]
fn option_none_round_trips() {
    let v = Value::Option(Arc::new(None));
    assert_eq!(
        round_trip(&v), v,
        "Option(None) must round-trip: edn::write → edn::read → same Value"
    );
}

#[test]
fn option_some_round_trips() {
    let v = Value::Option(Arc::new(Some(Value::i64(7))));
    assert_eq!(
        round_trip(&v), v,
        "Option(Some(7)) must round-trip: edn::write → edn::read → same Value"
    );
}

#[test]
fn result_ok_round_trips() {
    let v = Value::Result(Arc::new(Ok(Value::i64(99))));
    assert_eq!(
        round_trip(&v), v,
        "Result(Ok(99)) must round-trip: edn::write → edn::read → same Value"
    );
}

#[test]
fn result_err_round_trips() {
    let v = Value::Result(Arc::new(Err(Value::String(Arc::new("boom".into())))));
    assert_eq!(
        round_trip(&v), v,
        "Result(Err(\"boom\")) must round-trip: edn::write → edn::read → same Value"
    );
}
