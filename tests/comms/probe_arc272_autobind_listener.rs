//! Arc 272 step 2b — `listener'(process)` autobinds, returning `Bound<S,R>` whose address is the
//! minted capability (no chosen name). Unifies the process tier's signature with the thread tier:
//! `(listener' host :S :R) -> Bound<S,R>`.
//!
//! At HEAD (pre-2b) the process tier of `listener'` was the 2-arg named form `(listener' (process)
//! addr)` returning a bare Listener' — a 3-arg `(listener' (process) :i64 :i64)` did not type-check.
//! GREEN once 2b ships the autobind 3-arg arm (runtime + checker) + the binary-safe SocketAddress.
//!
//! THIS PROBE proves the wat-level WIRING in-process (no fork needed — the Rust unit test
//! `comms::process::autobind_tests` already proved the socket round-trips): the 3-arg autobind form
//! type-checks, runs (mints a kernel-unique address), returns a `Bound` with `/listener` + `/address`
//! accessors, and `connect'` dials the minted address capability. No fixed name anywhere — collision
//! and squatting are structurally absent.
//!
//! Run: cargo test --release -p wat --test probe_arc272_autobind_listener

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn autobind_listener_process_returns_bound_and_connects() {
    let world = startup_beside(file!())
        .expect("startup should succeed (listener'(process) 3-arg autobind → Bound<S,R>)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: autobind listener'(process) → Bound, connect' dials the minted address; got {got:?}"
    );
}
