//! Arc 113 closure — `:wat::kernel::raise!` round-trips data
//! through the panic boundary.
//!
//! The architectural insight: Failure's `message: String` IS the
//! data field. Rust serializes to text because that's the
//! universal rendering, but the conceptual content is EDN.
//! `raise!` renders its HolonAST argument via `:wat::edn::write`
//! and uses the result as `message`; receivers reconstruct the
//! original HolonAST via `(:wat::edn::read message)`.
//!
//! No new field on Failure. No new field on AssertionPayload. The
//! string IS the data, just rendered.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn raise_data_round_trips_through_failure_message() {
    // Inner program raises a HolonAST literal `(panic-data 42)`.
    // The outer program runs it via run-sandboxed-ast, pulls the
    // Failure off the RunResult, reads Failure/message back as
    // EDN, and asserts the recovered HolonAST shape.
    //
    // Pre-arc-113-closure: no `raise!`; the only way to ship
    // structured data through a panic was to hand-render it as a
    // String. Post-closure: the verb does the render; recovery
    // is `:wat::edn::read`.
    //
    // Arc 170 slice 1f-ζ: outer uses :my::compute; inner uses canonical nil main.
    // Arc 170 slice 4c-α-ii: migrated from `:wat::kernel::run-sandboxed-ast`
    // to `:wat::test::run-thread`. Body calls `raise!` (panics with EDN
    // payload); outer reads only `RunResult/failure`. None of FM 7-ter's
    // three rules fire — no stdio-slot reads, no stdio verbs in body, no
    // runtime config mutation. Thread is the correct (cheaper) destination.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let v = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute should run")
        .value_owned();
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(HolonAST), got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    // The recovered value is a HolonAST representing the form
    // (panic-data 42). The exact internal shape depends on
    // wat-edn's holon-tag round-trip; what matters is that the
    // recovered Value carries a HolonAST (not e.g. a plain
    // String). This proves data flows through the panic
    // boundary as data, not stringified-and-lost.
    assert!(
        matches!(inner, Value::holon__HolonAST(_)),
        "recovered Value should be a HolonAST; got {:?}",
        inner
    );
}
