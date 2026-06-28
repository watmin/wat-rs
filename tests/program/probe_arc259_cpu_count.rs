//! Arc 259 S3 stone 1 — `wat.cpu-count`: the host-parallelism escape-hatch field.
//!
//! "How many CPUs does my host have" is pure system interrogation — the same class
//! as pid/tid. By the escape-hatch doctrine (interrogation flows through the env as
//! `wat.*` fields, never a scattered syscall), the honest home is a new env field:
//!   `wat.cpu-count : i64`  — `std::thread::available_parallelism()`, a host constant,
//!                            inherited down the spawn tree (like `wat.started-at`).
//! It is `Parallel.processor_count` done the wat way, and it sizes the `brackets`
//! pool by default. The 7th Env field — the LAST `wat.*` platform field, just before
//! the `user.program` slot (all platform fields grouped, then the user slot).
//!
//! RED at HEAD: `Env` is a 6-arg record → the 7-arg constructor is an arity error,
//! and the seam env carries no `wat.cpu-count`.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_cpu_count`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// The record carries `wat.cpu-count` as a readable i64, the 6th constructor arg
/// (before `user.program`). RED via arity at HEAD: a 7-arg `Env` is an arity error.
#[test]
fn env_record_carries_cpu_count() {
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
                 (:wat::program::Env/wat.cpu-count \
                   (:wat::program::Env (:wat::time::now) (:wat::time::now) 0 0 \
                     :wat::program::PeerKind::process 8 (:wat::program::EmptyEnv)))) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup/check should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned();
    assert_eq!(
        got,
        Value::i64(8),
        "Env carries wat.cpu-count as the 6th field (before user.program)"
    );
}

/// The SEAM stamps the REAL host parallelism — `std::thread::available_parallelism`.
/// The escape hatch reads true: a program's `wat.cpu-count` equals what the kernel
/// reports. RED at HEAD: the field does not exist → accessor fails → main errors.
#[test]
fn seam_stamps_real_cpu_count() {
    let expected = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    let src = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil \
           (:wat::core::do \
             (:wat::test::assert-eq<:wat::core::i64> \
               (:wat::program::Env/wat.cpu-count (:wat::program::env)) \
               {expected}) \
             nil))"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "the seam must stamp the real available_parallelism ({expected}) into wat.cpu-count"
    );
}
