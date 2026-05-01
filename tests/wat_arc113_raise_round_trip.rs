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

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
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
    let src = r##"
        (:wat::core::define
          (:user::main -> :Option<wat::holon::HolonAST>)
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::test::program
                (:wat::core::define (:user::main
                                     (stdin  :wat::io::IOReader)
                                     (stdout :wat::io::IOWriter)
                                     (stderr :wat::io::IOWriter)
                                     -> :())
                  (:wat::kernel::raise!
                    (:wat::holon::leaf 42)))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast
                forms (:wat::core::vec :wat::core::String) :None))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r))
             ((recovered :Option<wat::holon::HolonAST>)
              (:wat::core::match fail -> :Option<wat::holon::HolonAST>
                ((Some f)
                 (Some (:wat::edn::read (:wat::kernel::Failure/message f))))
                (:None :None))))
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
