//! EVIDENCE for EXPECTATIONS-process-signal-p2-mint.md row 4: "a faced call compiles and
//! runs" — the child must ACTUALLY observe the signal, not merely have `signal` exist and do
//! nothing. See the co-located .wat fixture for the full mechanism/rationale, including the
//! SUPERSEDED-BY note (P3 rebuilds this properly as a wat `deftest`).
//!
//! Invocation: cargo nextest run --release -p wat --test process signal_user1_delivers_child_observes_flag
use wat::freeze::call_beside_value;
use wat::Value;

#[test]
fn signal_user1_delivers_child_observes_flag() {
    let result = call_beside_value(file!(), ":user::compute")
        .expect(":user::compute must run to completion and reply with the child's observation");
    let observed = match result {
        Value::String(s) => (*s).clone(),
        other => panic!("expected a String reply, got: {other:?}"),
    };
    assert_eq!(
        observed, "OBSERVED-TRUE",
        "the child must observe (sigusr1?) as true after :wat::kernel::signal delivers User1"
    );
}
