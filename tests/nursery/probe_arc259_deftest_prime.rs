//! Arc 259 S3.5a — `:wat::test::deftest'`: the test macro on the new substrate.
//!
//! A test is a ONE-SHOT computation with an OUTCOME (pass / structured-failure), NOT a
//! streaming self-peer (`spawn-program'` discards its outcome by design — a server streams).
//! `deftest'` rides the existing one-shot outcome-capture (`catch_unwind → SpawnOutcome →
//! Result<(), Vec<ThreadDiedError>>`, already on the clean `comms::thread::pair` substrate),
//! bundled in a primed `run-thread'` exactly as the legacy `deftest`/`run-thread` bundled the
//! old `Thread/join-result`. See DESIGN-STONE-259.S3.5a-deftest-prime.md.
//!
//! This is the `run-thread.wat` two-path, on the new substrate:
//!   - a PASSING `deftest'` → its `RunResult.failure` is `None`,
//!   - a FAILING `deftest'` → its `RunResult.failure` is `Some(_)` (the structured assertion
//!     failure surfaced as a value — not a bare process-killing panic).
//! `compute` scores both: +1 if passing has no failure, +2 if failing has a failure → 3.
//!
//! RED at HEAD: `:wat::test::deftest'` does not exist (unknown macro → startup fails).
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test nursery probe_arc259_deftest_prime -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(body: &str) -> i64 {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (deftest' macro must exist + expand)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// The two-path: a passing `deftest'` yields `RunResult.failure = None`; a failing one yields
/// `Some(_)`. `compute` returns 1 (passing clean) + 2 (failing detected) = 3.
#[test]
fn deftest_prime_two_path() {
    let v = run_compute_i64(
        "(:wat::test::deftest' :user::passing () \
           (:wat::test::assert-eq 4 (:wat::core::+ 2 2))) \
         (:wat::test::deftest' :user::failing () \
           (:wat::test::assert-eq 5 (:wat::core::+ 2 2))) \
         (:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::+ \
             (:wat::core::match (:wat::kernel::RunResult/failure (:user::passing)) \
               -> :wat::core::i64 \
               (:wat::core::None 1) \
               ((:wat::core::Some _f) 0)) \
             (:wat::core::match (:wat::kernel::RunResult/failure (:user::failing)) \
               -> :wat::core::i64 \
               (:wat::core::None 0) \
               ((:wat::core::Some _f) 2))))",
    );
    assert_eq!(
        v, 3,
        "deftest' passing → no failure (+1); deftest' failing → Some(structured failure) (+2)"
    );
}
