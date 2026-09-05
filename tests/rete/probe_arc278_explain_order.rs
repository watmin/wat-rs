//! F1 — `harvest-support` walks topological-node-ids. Native and oracle
//! attribute a multi-producer fact to the same rule.
//!
//! Adjacent fixture: `probe_arc278_explain_order.wat` (byte-identical to
//! the vigilia probe that found the defect, minus this file's panic).
//! ≥8 producing rules; single-producer control.
//!
//! Gate B is behavioural and was only probabilistically red at HEAD.
//! Gate A (`no_raw_network_keys_in_oracle`) is the proof.
//!
//! Run: cargo nextest run --release -p wat --test rete probe_arc278_explain_order

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn rules() -> Vec<String> {
    let out = call_beside_value(file!(), ":user::attribution").expect("fixture must run");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 4, "witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::String(s) => s.to_string(),
            other => panic!("expected String; got {other:?}"),
        })
        .collect()
}

#[test]
fn the_control_is_a_single_producer_and_agrees() {
    let r = rules();
    assert_eq!(
        r[2], r[3],
        "CONTROL: a single-producer fact must agree on both engines ({:?} vs {:?})",
        r[2], r[3]
    );
}

#[test]
fn native_and_oracle_attribute_the_same_rule() {
    let r = rules();
    assert_eq!(
        r[2], r[3],
        "CONTROL: a single-producer fact must agree ({:?} vs {:?})",
        r[2], r[3]
    );
    assert_eq!(
        r[0], r[1],
        "fire-rules-explain attributes the two-producer fact to {:?}; the $oracle says {:?}",
        r[0], r[1]
    );
}

fn or_via() -> Vec<String> {
    let out = call_beside_value(file!(), ":user::or-attribution").expect("or fixture must run");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 4, "or-witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::String(s) => s.to_string(),
            other => panic!("expected String; got {other:?}"),
        })
        .collect()
}

#[test]
fn or_control_a_single_arm_agrees() {
    let v = or_via();
    assert_eq!(
        v[2], v[3],
        "CONTROL: A1-only :or must agree on both engines ({:?} vs {:?})",
        v[2], v[3]
    );
}

#[test]
fn or_eight_arms_native_and_oracle_attribute_the_same_token() {
    let v = or_via();
    assert_eq!(
        v[2], v[3],
        "CONTROL: A1-only :or must agree ({:?} vs {:?})",
        v[2], v[3]
    );
    assert_eq!(
        v[0], v[1],
        "an :or with eight arms deriving the same fact: native via[0] type {:?}; oracle {:?}",
        v[0], v[1]
    );
}
