//! Arc 299.2 — the TIME entropic measurement. Rust brackets the entropy window
//! [lo,hi] around wat's `now`; wat measures the value ∈ [lo,hi] ∧ > epoch 0.
use std::time::{SystemTime, UNIX_EPOCH};
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn epoch_nanos() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64
}
fn eval_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> i64 {
    let ast = wat::parse_one!(expr).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

#[test]
fn now_is_within_the_bracketed_window() {
    let world = startup_beside(file!()).expect("startup");
    let lo = epoch_nanos();
    let now_ns = eval_i64(&world, "(:wat::time::epoch-nanos (:wat::time::now))"); // wat's now — the entropy
    let hi = epoch_nanos();
    let call = format!("(:probe::measure (:wat::time::at-nanos {now_ns}) {lo} {hi})");
    let ast = wat::parse_one!(&call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::bool(true) => {}
        other => panic!("now={now_ns} not in [{lo},{hi}]: {other:?}"),
    }
}
