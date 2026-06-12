//! Arc 259 — the program-env's first IDENTITY fields (the escape hatch grows).
//!
//! `:wat::program::Env` is wat's structured **escape hatch for system
//! interrogation**: kernel-stamped, user-unforgeable `wat.*` fields a pure wat
//! program reads as ordinary data instead of reaching for raw syscalls. It carried
//! only the two timing fields; this stone adds the first IDENTITY fields — the
//! companions to timing:
//!   `wat.process-id`   : i64 — `std::process::id()`
//!   `wat.os-thread-id` : i64 — the OS thread (Linux `gettid`)
//! Both kernel-stamped at the post-bootstrap / pre-`:user::main` seam; read via
//! the record accessors. (`peer-kind`, the typed `PeerKind` enum, is the next
//! stone.)
//!
//! RED at HEAD: `Env` is a 2-arg record → the 4-arg constructor is an arity error,
//! and the seam-installed env carries no `wat.process-id`.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_env_identity`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// The record carries `wat.process-id` + `wat.os-thread-id` as readable i64 fields
/// (RED via arity at HEAD: a 4-arg `Env` constructor is an arity error).
#[test]
fn env_record_carries_process_and_thread_id() {
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
                 (:wat::program::Env/wat.process-id \
                   (:wat::program::Env (:wat::time::now) (:wat::time::now) 12345 67890 :wat::program::PeerKind::process (:wat::program::EmptyEnv)))) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup/check should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned();
    assert_eq!(
        got,
        Value::i64(12345),
        "Env carries wat.process-id as its 3rd field (os-thread-id is the 4th)"
    );
}

/// The SEAM stamps the REAL process-id: `invoke_user_main` installs an env whose
/// `wat.process-id` the running `:user::main` can read and assert equals the
/// Rust-side `std::process::id()`. RED at HEAD because the field does not exist.
#[test]
fn seam_installs_env_with_process_id() {
    let expected_pid = std::process::id() as i64;
    let src = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil \
           (:wat::core::do \
             (:wat::test::assert-eq<:wat::core::i64> \
               (:wat::program::Env/wat.process-id (:wat::program::env)) \
               {expected_pid}) \
             (:wat::program::Env/wat.os-thread-id (:wat::program::env)) \
             nil))"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let result = invoke_user_main(&world, vec![]);
    assert!(
        result.is_ok(),
        "seam must stamp the real process-id ({expected_pid}); assert-eq failed: {:?}",
        result.err()
    );
}
