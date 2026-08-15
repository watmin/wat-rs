//! Arc 259 S3 stone 1 — `cpu-count`: the host-parallelism escape-hatch field.
//!
//! "How many CPUs does my host have" is pure system interrogation — the same class
//! as pid/tid. By the escape-hatch doctrine (interrogation flows through the env as
//! `wat.*` fields, never a scattered syscall), the honest home is a new env field:
//!   `cpu-count : i64`  — `std::thread::available_parallelism()`, a host constant,
//!                            inherited down the spawn tree (like `started-at`).
//! It is `Parallel.processor_count` done the wat way, and it sizes the `brackets`
//! pool by default. The 7th Env field — the LAST `wat.*` platform field, just before
//! the `user-data` slot (all platform fields grouped, then the user slot).
//!
//! RED at HEAD: `Env` is a 6-arg record → the 7-arg constructor is an arity error,
//! and the seam env carries no `cpu-count`.
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_cpu_count.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run: `cargo test --release --test program probe_arc259_cpu_count`

use wat::freeze::{call_beside_value, invoke_user_main, startup_beside};
use wat::runtime::Value;

/// The record carries `cpu-count` as a readable i64, the 6th constructor arg
/// (before `user-data`). RED via arity at HEAD: a 7-arg `Env` is an arity error.
#[test]
fn env_record_carries_cpu_count() {
    let got = call_beside_value(file!(), ":probe::compute").expect("eval");
    assert_eq!(
        got,
        Value::i64(8),
        "Env carries cpu-count as the 6th field (before user-data)"
    );
}

/// The SEAM stamps the REAL host parallelism — `std::thread::available_parallelism`.
/// The escape hatch reads true: a program's `cpu-count` equals what the kernel
/// reports. The fixture's :user::main asserts env cpu-count == live cpu-count verb.
/// RED at HEAD: the field does not exist → accessor fails → main errors.
#[test]
fn seam_stamps_real_cpu_count() {
    let expected = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    let world = startup_beside(file!()).expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "the seam must stamp the real available_parallelism ({expected}) into cpu-count"
    );
}
