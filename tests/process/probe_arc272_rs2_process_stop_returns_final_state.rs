//! Arc 272 record-state rs-2 — a service's return value IS its final state: `(<svc>/stop c)` terminates
//! the service and yields its last state. PROCESS tier here; identical mechanism to the thread tier.
//!
//! Proves `stop` is locus-agnostic: the reply rides `connect'`/`send'`/`recv'` unchanged across
//! thread and process. The final state crosses the socket (process tier) just as it crosses the
//! channel (thread tier) — constant client shape, no lineage reshape.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_process_stop_returns_final_state -- --include-ignored

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn process_stop_returns_the_services_final_state() {
    // World loaded from co-located probe_arc272_rs2_process_stop_returns_final_state.wat via call_beside.
    let got = call_beside(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: (:my::counter/stop h) stopped the process service and returned its final state \
         (increment 5 set state 0→5); got {got:?}"
    );
}
