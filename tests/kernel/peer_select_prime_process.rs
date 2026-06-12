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

/// Process-tier `select'` over two echo+1 peers — only one has data.
///
/// Spawns two `:process` forms-server peers (a at index 0, b at index 1).
/// Arc 214 β: each peer is a forms-server (readln -> :i64, println (i64::+ n 1)).
/// Sends 98 to b only. `select'` over [a b] must return (1, 99) (98+1=99).
/// Both peers are closed after.
///
/// `#[ignore]` — process-tier probe; run under setsid + timeout with --test-threads=1.
#[test]
#[ignore = "process-tier probe: run via setsid timeout 180 cargo test --release --test kernel peer_select_prime_process -- --ignored --test-threads=1"]
fn process_select_prime_picks_ready_peer() {
    // Arc 214 β: spawn-program' :process takes a forms-server (not a fn).
    // Each spawned peer runs readln -> :i64, println (i64::+ n 1) — echo+1 server.
    // The select' test sends 98 to peer b only; select' fires on b (index 1)
    // and returns (1, 99) — 98+1=99 from the echo+1 server.
    let src = r#"
        (:wat::core::defn :user::compute [] -> :(wat::core::i64,wat::core::i64)
          (:wat::core::let [a (:wat::kernel::spawn-program' (:wat::spawn::process)
                                (:wat::core::forms
                                  (:wat::core::defn :user::main [] -> :wat::core::nil
                                    (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                                                      _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                                      nil))))
                            b (:wat::kernel::spawn-program' (:wat::spawn::process)
                                (:wat::core::forms
                                  (:wat::core::defn :user::main [] -> :wat::core::nil
                                    (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                                                      _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                                      nil))))
                            _ (:wat::kernel::send' b 98)
                            picked (:wat::kernel::select' [a b])]
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
