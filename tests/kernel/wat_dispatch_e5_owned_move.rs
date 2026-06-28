//! E5 — scope = "owned_move" consumed-after-use semantics.
//!
//! A one-shot handle. The first invocation consumes the payload;
//! subsequent attempts error with "owned-move handle already consumed".
//! Models prepared-statement bindings, one-time tokens, capabilities.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::{eval_in_frozen, startup_beside};
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

fn run_fn(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let call = format!("({fn_name})");
    let ast = wat::parse_one!(&call).expect("parse compute call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

#[test]
fn ticket_redeems_once_successfully() {
    install();
    let val = run_fn(":my::compute-redeem");
    assert!(matches!(val, Value::i64(777)), "got {:?}", val);
}

#[test]
fn ticket_second_redemption_errors() {
    install();
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:my::compute-double-redeem)").expect("parse compute call");
    let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
    // The second redeem attempts to consume the already-drained cell;
    // OwnedMoveCell::take returns MalformedForm.
    assert!(format!("{:?}", err).contains("already consumed"),
            "expected 'already consumed'; got {:?}", err);
}
