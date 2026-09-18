//! VIGILIA experiri probe — `join_keys_cache` membership latches the catch-up that is the
//! ONLY bulk builder of `left_idx`, and a second writer sets that latch without building it.
//!
//! Run: cargo nextest run --release -p wat --test rete probe_vig_left_idx_latch

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn counts() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::native-and-oracle")
        .expect("fixture should fire cleanly on both engines");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 8, "witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::i64(x) => *x,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect()
}

#[test]
fn report_the_six_numbers() {
    let c = counts();
    panic!(
        "PROBE REPORT (deliberate): native [OutW={} OutP={} C={} OutN={}] \
         oracle [OutW={} OutP={} C={} OutN={}]",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
    );
}

#[test]
fn the_control_reaches_a_second_round() {
    let c = counts();
    assert_eq!(
        (c[2], c[1]),
        (2, 2),
        "non-vacuity: C must be 2 (one input + one derived one round later) and the \
         NO-GUARD control must see both. Got C={} OutP={}",
        c[2], c[1]
    );
}

#[test]
fn native_agrees_with_the_oracle_on_the_guarded_chain() {
    let c = counts();
    assert_eq!(
        &c[0..4],
        &c[4..8],
        "native and $oracle disagree. native=[OutW={},OutP={},C={},OutN={}] \
         oracle=[OutW={},OutP={},C={},OutN={}]",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
    );
}
