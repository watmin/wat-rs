//! Arc 113 closure — `:wat::kernel::raise!` round-trips data
//! through the panic boundary.
//!
//! **Arc 296 re-gate + arc 278 string-wrap annihilation:** `raise!` requires
//! `:wat::core::Error`, and the round-trip is STRUCTURAL WITH NO STRING WRAP.
//! The fixture raises `(:wat::core::Fault/of "arc113-raise-data")`;
//! `(:wat::kernel::Failure/error f)` yields the raised `:wat::core::Fault`
//! RECORD directly (it rode the panic boundary as data — no `edn::write`, no
//! `edn::read`); `Fault/message` reads the field off it. This test asserts that
//! field equals "arc113-raise-data" EXACTLY — the error survived the panic
//! boundary as a record, not a stringified blob.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.
//!
//! Arc 278 IPC de-prime (MAP unit): the driver migrated off the retired
//! non-prime `:wat::test::run-thread` onto the PRIMED peer wire. A thread peer
//! (`spawn-program' (thread)`) runs the raise!; the child crashes before it can
//! send, so `recv'` returns `RecvOutcome::Lost[LociDiedError::Panic{failure}]`,
//! whose `failure` is `Some(Failure)` carrying the raised Fault STRUCTURALLY.
//! Child body unchanged; the assertion below is unchanged.

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn raise_data_round_trips_through_failure_message() {
    // Inner program raises a Fault/of "arc113-raise-data".
    // The outer program runs it on the primed wire (spawn-program' (thread) +
    // recv'), matches the peer's death (Lost → LociDiedError::Panic → its
    // Some(Failure) payload), and returns the message read structurally off
    // Failure/error — the raised Fault RECORD.
    // This proves the error flows through the panic boundary as
    // structured content (a record), not a stringified blob.
    //
    // Arc 278 the string-wrap annihilation: the old edn::write→String→edn::read
    // round-trip is GONE — Failure/error yields the Fault record directly.
    let v = call_beside(file!(), ":my::compute").expect("compute should run");
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(String), got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    // The recovered value is the message field read off the Fault record that
    // rode the Lost cause's Panic.failure across the panic boundary as data:
    // Failure/error yielded the Fault record directly, and Fault/message read the
    // field off it. So it must equal the raised message EXACTLY — proving the
    // error survived the panic boundary as structured data (a record), not a
    // stringified blob.
    let msg = match &inner {
        Value::String(s) => s.clone(),
        other => panic!("reconstructed Fault/message should be a String; got {:?}", other),
    };
    assert_eq!(
        msg.as_str(),
        "arc113-raise-data",
        "the reconstructed Fault's message field must equal the raised message exactly"
    );
}
