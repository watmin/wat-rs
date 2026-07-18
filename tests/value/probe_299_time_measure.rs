//! Arc 299.2 — the TIME entropic measurement. Rust brackets the entropy window
//! [lo,hi] around wat's `now`; wat measures the value ∈ [lo,hi] ∧ > epoch 0.
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn epoch_nanos() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64
}

// just-eval (rubric): fetch a fixture fn from the frozen world and `apply_function` it with
// Rust-constructed args — no inline wat driver. `:wat::time::at-nanos` is a native special form
// (not fn-appliable — `apply_function` on it would hit the `unreachable!` native-dispatch guard),
// so the `:wat::time::Instant` arg is built directly via the same `chrono` construction its own
// `eval_time_at_nanos` (src/time.rs) uses internally.
fn call(world: &wat::freeze::FrozenWorld, fn_name: &str, args: Vec<Value>) -> Value {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    apply_function(func, args, world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("eval {fn_name:?}: {e:?}"))
}

#[test]
fn now_is_within_the_bracketed_window() {
    let world = startup_beside(file!()).expect("startup");
    let lo = epoch_nanos();
    // wat's now — the entropy.
    let now_ns = match call(&world, ":probe::now-nanos", vec![]) {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    let hi = epoch_nanos();
    let instant = Value::Instant(Utc.timestamp_nanos(now_ns));
    match call(&world, ":probe::measure", vec![instant, Value::i64(lo), Value::i64(hi)]) {
        Value::bool(true) => {}
        other => panic!("now={now_ns} not in [{lo},{hi}]: {other:?}"),
    }
}
