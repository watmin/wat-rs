//! Arc 214 Stone 4.6b — process-tier `select'` integration test.
//!
//! Mirrors `peer_verb_round_trip_process.rs` (round-trip baseline) but
//! verifies `select'` over a Vector of two `:process` echo peers.
//!
//! Verifies:
//! 1. Two `:process` echo peers are spawned.
//! 2. `send'` is called on exactly ONE peer (peer B, index 1).
//! 3. `select'` over both returns `(1, value)` — index 1 (peer B fired).
//! 4. Both peers are `close'`d (exit 0 each).
//!
//! # Containment
//!
//! The test forks via `spawn-program' :process`. Run under setsid + timeout
//! to prevent fd/lock inheritance from the multi-threaded cargo test binary.
//! Marked `#[ignore]` — run via:
//!   setsid timeout 180 cargo test --release --test kernel peer_select_prime_process -- --ignored --test-threads=1
//! or the `integration-run.sh` harness.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Process-tier `select'` over two echo peers — only one has data.
///
/// Spawns two `:process` echo peers (a at index 0, b at index 1).
/// Sends 99 to b only. `select'` over [a b] must return (1, 99).
/// Both peers are closed after.
///
/// `#[ignore]` — process-tier probe; run under setsid + timeout with --test-threads=1.
#[test]
#[ignore = "process-tier probe: run via setsid timeout 180 cargo test --release --test kernel peer_select_prime_process -- --ignored --test-threads=1"]
fn process_select_prime_picks_ready_peer() {
    let src = r#"
        (:wat::core::defn :user::mk [] -> :wat::kernel::Process'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :process {}
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
        (:wat::core::defn :user::compute [] -> :(wat::core::i64,wat::core::i64)
          (:wat::core::let [a (:user::mk)
                            b (:user::mk)
                            _ (:wat::kernel::send' b 99)
                            picked (:wat::kernel::select' [a b])
                            _ (:wat::kernel::close' a)
                            _ (:wat::kernel::close' b)]
            picked))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;

    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup_from_source must succeed: process-tier select' test");

    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    let result = eval_in_frozen(&ast, &world, &env)
        .expect("eval_in_frozen must succeed: process-tier select' picks ready peer");

    match result.value_owned() {
        Value::Tuple(xs) => {
            assert_eq!(xs.len(), 2, "select' returns (index, value); got {:?}", xs);
            assert_eq!(
                xs[0],
                Value::i64(1),
                "ready peer is index 1 (b); got {:?}",
                xs[0]
            );
            assert_eq!(
                xs[1],
                Value::i64(99),
                "echoed value must be 99; got {:?}",
                xs[1]
            );
        }
        other => panic!("expected Tuple(index, value); got {:?}", other),
    }
}
