//! Arc 232 Stone 232.3 — protocol-method dispatch (the keystone).
//!
//! 232.1 registered the forms; 232.2 made `:P` a usable bound. 232.3 makes protocol methods
//! CALLABLE: `(:P/method receiver args…)` type-checks via the method's declared sig and dispatches
//! at RUNTIME on the receiver's CONCRETE type via the extend registry. This is the shape the whole
//! arc exists for — a fn typed over `:P` calling a method, dispatching on whatever extender is
//! passed (exactly the host-agnostic `start`).
//!
//! THE PROOF: one protocol `:t::Greeter` with `greet`; TWO extenders (`:t::Robot` → "beep",
//! `:t::Dog` → "woof"); a fn `greet-it [g <- :t::Greeter]` that calls `(:t::Greeter/greet g 3)`.
//! `(greet-it (:t::Robot))` → "beep" and `(greet-it (:t::Dog))` → "woof" proves BOTH that the
//! `:P`-bound forwarding type-checks AND that dispatch selects the impl by the receiver's concrete
//! type (not the static `:Greeter` bound).
//!
//! RED at HEAD (232.2 shipped): `:t::Greeter/greet` is an unresolved call head (no dispatch yet).
//! GREEN once 232.3 wires check-time inference + runtime dispatch.
//!
//! Run: cargo test --release -p wat --test probe_arc232_3_protocol_dispatch

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter loudness <- :wat::core::i64] -> :wat::core::String))
(:wat::Record::def :t::Robot [])
(:wat::Record::def :t::Dog [])
(:wat::core::extend-type :t::Robot :t::Greeter (greet [self loudness] "beep"))
(:wat::core::extend-type :t::Dog   :t::Greeter (greet [self loudness] "woof"))

;; A fn typed over the protocol bound — calls the method, dispatching on the concrete receiver.
(:wat::core::defn :user::greet-it [g <- :t::Greeter] -> :wat::core::String
  (:t::Greeter/greet g 3))

(:wat::core::defn :user::go-robot [] -> :wat::core::String (:user::greet-it (:t::Robot)))
(:wat::core::defn :user::go-dog   [] -> :wat::core::String (:user::greet-it (:t::Dog)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn run(call: &str) -> String {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (232.3: protocol-method dispatch)");
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{call} raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn protocol_method_dispatches_on_concrete_receiver_through_the_bound() {
    assert_eq!(
        run("(:user::go-robot)"), "beep",
        "a :t::Robot passed through a :t::Greeter-typed param must dispatch greet to the Robot impl"
    );
    assert_eq!(
        run("(:user::go-dog)"), "woof",
        "a :t::Dog through the same :t::Greeter param must dispatch to the Dog impl — dispatch is on \
         the CONCRETE receiver type, not the static :Greeter bound"
    );
}
