//! VIGILIA experiri probe — `retract` removes EVERY equal fact where `insert` stages ONE.
//!
//! Run: cargo nextest run --release -p wat --test rete probe_vig_retract_multiplicity

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn nums() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::multiplicity").expect("fixture must run");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    items
        .into_iter()
        .map(|v| match v {
            Value::i64(x) => *x,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect()
}

#[test]
fn report_multiplicity() {
    let n = nums();
    panic!(
        "PROBE REPORT (deliberate): facts_after_two_identical_inserts={} \
         facts_after_one_retract={} seen_rows_after_retract_and_refire={} \
         seen_rows_before_retract={}",
        n[0], n[1], n[2], n[3]
    );
}
