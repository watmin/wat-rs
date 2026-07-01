//! Arc 113 closure — `:wat::kernel::raise!` round-trips data
//! through the panic boundary.
//!
//! **Arc 296 re-gate update:** `raise!` now requires `:wat::core::Error`
//! (previously `:wat::holon::HolonAST`). The fixture raises
//! `(:wat::core::Fault/of "arc113-raise-data")` and recovers the
//! `Failure/message` EDN string — proving data flows through the panic
//! boundary as serialized structured content.
//!
//! The Fault's EDN (`#wat.core/Fault {:message ...}`) is stored in
//! `Failure/message`; when read back via `(:wat::edn::read ...)` it
//! returns a `Value::Aggregate` (the Fault record), not a `HolonAST`.
//! This test therefore returns the raw message String and asserts it
//! contains "arc113-raise-data", proving the payload traverses the
//! panic boundary intact.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

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
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let v = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute should run")
        .value_owned();
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(String), got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    // The recovered value is the Failure/message String containing the
    // Fault's EDN representation. Prove it contains the raise message.
    let msg = match &inner {
        Value::String(s) => s.clone(),
        other => panic!("recovered value should be a String; got {:?}", other),
    };
    assert!(
        msg.contains("arc113-raise-data"),
        "Failure/message should contain 'arc113-raise-data'; got: {}",
        msg
    );
}
