//! E5 — scope = "owned_move" consumed-after-use semantics.
//!
//! A one-shot handle. The first invocation consumes the payload;
//! subsequent attempts error with "owned-move handle already consumed".
//! Models prepared-statement bindings, one-time tokens, capabilities.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
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

#[test]
fn ticket_redeems_once_successfully() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Ticket)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((t :rust::test::Ticket) (:rust::test::Ticket::new 777)))
            (:rust::test::Ticket::redeem t)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(777)), "got {:?}", result);
}

#[test]
fn ticket_second_redemption_errors() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Ticket)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((t :rust::test::Ticket) (:rust::test::Ticket::new 42))
             ((first :wat::core::i64) (:rust::test::Ticket::redeem t)))
            (:rust::test::Ticket::redeem t)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let err = invoke_user_main(&world, Vec::new()).unwrap_err();
    // The second redeem attempts to consume the already-drained cell;
    // OwnedMoveCell::take returns MalformedForm.
    assert!(format!("{:?}", err).contains("already consumed"),
            "expected 'already consumed'; got {:?}", err);
}
