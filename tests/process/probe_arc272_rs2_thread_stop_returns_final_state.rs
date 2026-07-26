//! Arc 272 record-state rs-2 — a service's return value IS its final state: `(<svc>/stop c)` terminates
//! the service and yields its last state. THREAD tier here; the SAME mechanism serves process/remote.
//!
//! The lifecycle counterpart to `start`: `(<svc>/start locus state0) -> Handle` launches with an initial
//! state; `(<svc>/stop c) -> St` stops it and returns the final state (gen_server `terminate`-with-state).
//! Resumability falls out: this `final` is a valid `state0` for the next `start`.
//!
//! Mechanism (the build, not asserted here): the `:Stop` terminal op (gen_server `{stop, State}`).
//! defservice auto-generates a `stop` op + serve's `Outcome::Stop` arm (reply the final state to the
//! client, then EXIT the loop instead of recurring). `(<svc>/stop c)` sends the stop request over the
//! CLIENT connection and `recv'`s the final state AS THE REPLY — CONSTANT SHAPE across thread/process/
//! remote (it rides connect'/send'/recv', identical for every locus). No new substrate, no lineage
//! reshape. A crashed service makes the call RAISE (the existing recv' crash-surfacing) — sibling probe.
//!
//! RED at HEAD: defservice generates no `<fqdn>/stop` op (UnresolvedReference). GREEN once rs-2 ships the
//! `Outcome::Stop` variant + serve's stop arm + the generated stop op/method. `#[ignore]` until then.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_thread_stop_returns_final_state -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn thread_stop_returns_the_services_final_state() {
    // World loaded from co-located probe_arc272_rs2_thread_stop_returns_final_state.wat via call_beside_value.
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: (:my::counter/stop h) stopped the thread service and returned its final state \
         (increment 5 set state 0→5); got {got:?}"
    );
}
