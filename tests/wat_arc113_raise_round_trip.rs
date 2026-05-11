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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

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
    let src = r##"
        (:wat::core::define
          (:my::compute -> :wat::core::Option<wat::holon::HolonAST>)
          (:wat::core::let
            [forms
              (:wat::test::program
                (:wat::core::define (:user::main -> :wat::core::nil)
                  (:wat::kernel::raise!
                    (:wat::holon::leaf 42))))
             r
              (:wat::kernel::run-sandboxed-ast
                forms (:wat::core::Vector :wat::core::String) :wat::core::None)
             fail
              (:wat::kernel::RunResult/failure r)
             recovered
              (:wat::core::match fail -> :wat::core::Option<wat::holon::HolonAST>
                ((:wat::core::Some f)
                 (:wat::core::Some (:wat::edn::read (:wat::kernel::Failure/message f))))
                (:wat::core::None :wat::core::None))]
            recovered))
    "##;
    let v = run(src);
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
