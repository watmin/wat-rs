//! A1 — `JoinLeftIndex` owns keys and buckets. A `:where` then two fact
//! conditions must not drop the second-round derived row.
//!
//! Adjacent fixture: `probe_arc278_left_idx_latch.wat`.
//! Run: cargo nextest run --release -p wat --test rete probe_arc278_left_idx_latch

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
    assert_eq!(c[0], 2, "guarded chain OutW must be 2 after the latch cure; got {}", c[0]);
}
