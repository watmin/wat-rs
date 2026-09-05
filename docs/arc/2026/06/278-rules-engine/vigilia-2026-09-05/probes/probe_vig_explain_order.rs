//! VIGILIA experiri probe — does `fire-rules-explain` attribute a derived fact to the SAME
//! rule as `fire-rules-explain$oracle`, when two rules produce it?
//!
//! Run: cargo nextest run --release -p wat --test rete probe_vig_explain_order

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn rules() -> Vec<String> {
    let out = call_beside_value(file!(), ":user::attribution").expect("fixture must run");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    items
        .into_iter()
        .map(|v| match v {
            Value::String(s) => s.to_string(),
            other => panic!("expected String; got {other:?}"),
        })
        .collect()
}

#[test]
fn report_attribution() {
    let r = rules();
    panic!(
        "PROBE REPORT (deliberate): Out native={:?} oracle={:?} | CONTROL Solo native={:?} oracle={:?}",
        r[0], r[1], r[2], r[3]
    );
}

#[test]
fn native_and_oracle_attribute_the_same_rule() {
    let r = rules();
    assert_eq!(r[2], r[3], "CONTROL: a single-producer fact must agree ({:?} vs {:?})", r[2], r[3]);
    assert_eq!(
        r[0], r[1],
        "fire-rules-explain attributes the two-producer fact to {:?}; the $oracle says {:?}. \
         `harvest-support` walks HAMT key order, the native walks ascending ids.",
        r[0], r[1]
    );
}
