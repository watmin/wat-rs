//! E5 — scope = "owned_move" consumed-after-use semantics.
//!
//! A one-shot handle. The first invocation consumes the payload;
//! subsequent attempts error with "owned-move handle already consumed".
//! Models prepared-statement bindings, one-time tokens, capabilities.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use wat_macros::wat_dispatch;

/// A ticket that can be redeemed exactly once.
pub struct Ticket {
    value: i64,
}

#[wat_dispatch(path = ":rust::test::Ticket", scope = "owned_move")]
impl Ticket {
    pub fn new(value: i64) -> Self {
        Ticket { value }
    }

    /// Consumes `self`. Returns the inner value.
    pub fn redeem(self) -> i64 {
        self.value
    }
}

fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_Ticket::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

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
fn ticket_redeems_once_successfully() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Ticket)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [t (:rust::test::Ticket::new 777)]
            (:rust::test::Ticket::redeem t)))
    "#;
    assert!(matches!(run(src), Value::i64(777)), "got {:?}", run(src));
}

#[test]
fn ticket_second_redemption_errors() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Ticket)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [t (:rust::test::Ticket::new 42)
             first (:rust::test::Ticket::redeem t)]
            (:rust::test::Ticket::redeem t)))
    "#;
    let src_with_nil = with_nil_main(src);
    let world = startup_from_source(&src_with_nil, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    let err = eval_in_frozen(&ast, &world, &env).unwrap_err();
    // The second redeem attempts to consume the already-drained cell;
    // OwnedMoveCell::take returns MalformedForm.
    assert!(format!("{:?}", err).contains("already consumed"),
            "expected 'already consumed'; got {:?}", err);
}
