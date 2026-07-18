//! Arc 113 closure — `:wat::kernel::raise!` round-trips data
//! through the panic boundary.
//!
//! **Arc 296 re-gate update:** `raise!` now requires `:wat::core::Error`
//! (previously `:wat::holon::HolonAST`), and the round-trip is now proven
//! STRUCTURAL, not stringly. The fixture raises
//! `(:wat::core::Fault/of "arc113-raise-data")`; `Failure/message` carries the
//! Fault's EDN (`#wat.core/Fault {:message …}`); `(:wat::edn::read …)` lifts it
//! back to a `:wat::core::Fault` RECORD via `reconstruct_record`;
//! `Fault/message` reads the field off the reconstructed record. This test
//! asserts that field equals "arc113-raise-data" EXACTLY — the error survived
//! the panic boundary as a record, not a stringified blob.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn raise_data_round_trips_through_failure_message() {
    // Inner program raises a Fault/of "arc113-raise-data".
    // The outer program runs it via run-thread, pulls the
    // Failure off the RunResult, and returns Failure/message
    // (the EDN string). This proves data flows through the
    // panic boundary as structured content (not lost).
    //
    // Arc 296 re-gate: raise! now takes :wat::core::Error; the
    // HolonAST-round-trip via edn::read is replaced with direct
    // Failure/message recovery (String). The EDN string contains
    // the Fault's serialized form with "arc113-raise-data".
    let v = call_beside(file!(), ":my::compute").expect("compute should run");
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(String), got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    // The recovered value is the message field read off the RECONSTRUCTED Fault
    // record: edn::read lifted `#wat.core/Fault {…}` back to a record via
    // reconstruct_record, and Fault/message read the field off it. So it must
    // equal the raised message EXACTLY — proving the error survived the panic
    // boundary as structured data (a record), not a stringified blob.
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
