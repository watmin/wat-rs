//! RED probe — arc 278 Stone A.0: UNIFORM vector-bodied variant encoding.
//!
//! One rule, no special cases:
//!   - record          → `#ns/Type {field-map}` (map body)
//!   - enum variant    → `#ns/Variant [field-vec]` (vector body, ANY arity:
//!     unit = `[]`, N fields = `[v0 v1 …]`)
//!   - `nil`           → the unit value (`Value::Unit`) ONLY — never a variant
//!
//! This RETIRES the arc-298.1 Option/Result direct-body special-case:
//!   None → `[]`, Some(v) → `[v]`, Some(nil) → `[nil]`, Ok(v) → `[v]`, Err(e) → `[e]`.
//!
//! RED at HEAD (before the cutover):
//!   - a user unit variant wrote `#ns/Variant nil` (bare-nil body)
//!   - `None` wrote `#wat.core.Option/None nil`
//!   - `Some(v)` wrote `#wat.core.Option/Some v` (direct body)
//!   - `Some(nil)` wrote `#wat.core.Option/Some nil` (arity-ambiguous with None)
//!
//! GREEN after A.0: every variant is `#tag [field-vec]`; body-shape is a
//! perfect discriminator (map=record, vector=variant, nil=unit-value).

use std::sync::Arc;
use wat::edn_shim::{edn_to_value, value_to_edn, value_to_edn_with};
use wat::runtime::{EnumValue, Value};
use wat::types::{EnumDef, EnumVariant, Purity, TypeDef, TypeEnv};

fn write_value(v: &Value) -> String {
    wat_edn::write(&value_to_edn(v))
}

fn write_value_with(v: &Value, types: &TypeEnv) -> String {
    wat_edn::write(&value_to_edn_with(v, Some(types)))
}

fn round_trip(v: &Value) -> Value {
    let s = write_value(v);
    let owned = wat_edn::parse_owned(&s).expect("parse EDN");
    edn_to_value(&owned, None, None).expect("decode Value")
}

fn round_trip_with(v: &Value, types: &TypeEnv) -> Value {
    let s = write_value_with(v, types);
    let owned = wat_edn::parse_owned(&s).expect("parse EDN");
    edn_to_value(&owned, Some(types), None).expect("decode Value")
}

/// A TypeEnv holding a user enum `:a0::Color` with three UNIT variants.
fn color_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register(TypeDef::Enum(EnumDef {
        name: ":a0::Color".to_string(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Red".to_string()),
            EnumVariant::Unit("Green".to_string()),
            EnumVariant::Unit("Blue".to_string()),
        ],
    }))
    .expect("register :a0::Color");
    env
}

fn color_red() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":a0::Color".to_string(),
        variant_name: "Red".to_string(),
        // Unit variant — no fields, so nothing to name (arc 296 G′).
        names: Arc::new(vec![]),
        fields: vec![],
    }))
}

// ── User enum UNIT variant → `#ns/Variant []` ────────────────────────────────

#[test]
fn user_unit_variant_writes_empty_vector_body() {
    let env = color_env();
    let s = write_value_with(&color_red(), &env);
    wat::assert_edn_matches_file!(s, "probe_arc278_a0_uniform_variant__user_unit.edn", "a user enum unit variant must write a `#ns/Variant []` (empty-vector) body");
}

#[test]
fn user_unit_variant_round_trips() {
    let env = color_env();
    let v = color_red();
    assert_eq!(
        round_trip_with(&v, &env),
        v,
        "a user enum unit variant must round-trip via `[]`"
    );
}

// ── Option: None → `[]`, Some(v) → `[v]`, Some(nil) → `[nil]` ─────────────────

#[test]
fn option_none_writes_empty_vector_body() {
    let v = Value::Option(Arc::new(None));
    wat::assert_edn_matches_file!(write_value(&v), "probe_arc278_a0_uniform_variant__option_none.edn", "None must write `#wat.core.Option/None []`");
}

#[test]
fn option_some_writes_single_element_vector_body() {
    let v = Value::Option(Arc::new(Some(Value::i64(7))));
    wat::assert_edn_matches_file!(write_value(&v), "probe_arc278_a0_uniform_variant__option_some.edn", "Some(v) must write `#wat.core.Option/Some [v]`");
}

#[test]
fn option_some_of_unit_writes_nil_inside_vector() {
    // Some(nil): arity is VISIBLE — `[nil]`, one field holding the unit value —
    // and never collides with None (`[]`).
    let v = Value::Option(Arc::new(Some(Value::Unit)));
    wat::assert_edn_matches_file!(write_value(&v), "probe_arc278_a0_uniform_variant__option_some_unit.edn", "Some(nil) must write `#wat.core.Option/Some [nil]` (arity visible)");
}

#[test]
fn option_variants_round_trip() {
    for v in [
        Value::Option(Arc::new(None)),
        Value::Option(Arc::new(Some(Value::i64(7)))),
        Value::Option(Arc::new(Some(Value::Unit))),
        Value::Option(Arc::new(Some(Value::String(Arc::new("x".into()))))),
    ] {
        assert_eq!(round_trip(&v), v, "Option variant must round-trip via vector body");
    }
}

// ── Result: Ok(v) → `[v]`, Err(e) → `[e]` ────────────────────────────────────

#[test]
fn result_ok_writes_single_element_vector_body() {
    let v = Value::Result(Arc::new(Ok(Value::i64(42))));
    wat::assert_edn_matches_file!(write_value(&v), "probe_arc278_a0_uniform_variant__result_ok.edn", "Ok(v) must write `#wat.core.Result/Ok [v]`");
}

#[test]
fn result_err_writes_single_element_vector_body() {
    let v = Value::Result(Arc::new(Err(Value::String(Arc::new("e".into())))));
    wat::assert_edn_matches_file!(write_value(&v), "probe_arc278_a0_uniform_variant__result_err.edn", "Err(e) must write `#wat.core.Result/Err [e]`");
}

#[test]
fn result_variants_round_trip() {
    for v in [
        Value::Result(Arc::new(Ok(Value::i64(99)))),
        Value::Result(Arc::new(Err(Value::String(Arc::new("boom".into()))))),
    ] {
        assert_eq!(round_trip(&v), v, "Result variant must round-trip via vector body");
    }
}

// ── `nil` is the unit value ONLY (never a variant) ───────────────────────────

#[test]
fn bare_nil_decodes_to_unit_value() {
    let owned = wat_edn::parse_owned("nil").expect("parse nil");
    let decoded = edn_to_value(&owned, None, None).expect("decode nil");
    assert_eq!(
        decoded,
        Value::Unit,
        "a bare `nil` must decode to the unit value, never a variant"
    );
}
