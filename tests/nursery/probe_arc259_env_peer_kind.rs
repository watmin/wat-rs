//! Arc 259 — `wat.peer-kind`: the program-env's first TYPED-ENUM identity field.
//!
//! peer-kind answers "what KIND of peer am I" as a nominal `:wat::program::PeerKind`
//! enum, not a raw keyword:
//!   `:thread`  — shares the parent's address space (a thread peer)
//!   `:process` — owns its own address space (the root `:user::main`, or a forked
//!                `:process` peer)
//! The root main owns its address space → **`:process`** (builder, 2026-06-11).
//! Kernel-stamped at the post-bootstrap / pre-main seam; read via the accessor.
//!
//! RED at HEAD: `Env` is a 4-arg record → the 5-arg constructor is an arity error,
//! and the seam env carries no `wat.peer-kind`.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_env_peer_kind`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// The record carries `wat.peer-kind` as a `PeerKind` value (RED via arity at HEAD:
/// a 5-arg `Env` constructor is an arity error). `conforms?` proves the field holds
/// a genuine `:wat::program::PeerKind` (the proven nominal-membership idiom).
#[test]
fn env_record_carries_peer_kind() {
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::bool \
                 (:wat::core::conforms? \
                   (:wat::program::Env/wat.peer-kind \
                     (:wat::program::Env (:wat::time::now) (:wat::time::now) 0 0 \
                       :wat::program::PeerKind::thread)) \
                   :wat::program::PeerKind)) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup/check should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned();
    assert_eq!(
        got,
        Value::bool(true),
        "Env carries wat.peer-kind (5th field) holding a :wat::program::PeerKind"
    );
}

/// The SEAM stamps `:process` for the root `:user::main` (it owns its address
/// space). RED at HEAD because the field does not exist → the accessor fails →
/// main errors → invoke returns Err. The `conforms?` of the stamped value to the
/// `:process` variant proves the value, not just the field's presence.
#[test]
fn seam_stamps_process_for_root_main() {
    let pid_kind_eq = "(:wat::core::conforms? \
                         (:wat::program::Env/wat.peer-kind (:wat::program::env)) \
                         :wat::program::PeerKind)";
    let src = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil \
           (:wat::core::do {pid_kind_eq} nil))"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "the seam must stamp wat.peer-kind before :user::main; main's read failed"
    );
}
