//! Arc 214 Stone 4.6b — process-tier `select'` integration test.
//!
//! Mirrors `peer_verb_round_trip_process.rs` (round-trip baseline) but
//! verifies `select'` over a Vector of two `:process` echo peers.
//!
//! Stone 259: select' returns ServiceEvent<I,O> (was Tuple<i64,O>).
//!
//! Verifies:
//! 1. Two `:process` echo peers are spawned.
//! 2. `send'` is called on exactly ONE peer (peer B, index 1).
//! 3. `select'` over both returns `ServiceEvent::Message{idx=1, msg=value}`.
//! 4. Both peers are `close'`d (exit 0 each).
//!
//! # Containment
//!
//! The test forks via `spawn-program' :process`. Run under setsid + timeout
//! to prevent fd/lock inheritance from the multi-threaded cargo test binary.
//! Marked `#[ignore]` — run via:
//!   setsid timeout 180 cargo test --release --test kernel peer_select_prime_process -- --ignored --test-threads=1
//! or the `integration-run.sh` harness.

use wat::freeze::{eval_in_frozen, startup_beside};
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
    // and returns ServiceEvent::Message{idx=1, msg=99} — 98+1=99 from the echo+1 server.
    // Stone 259: select' returns ServiceEvent<I,O> (was Tuple<i64,O>).
    let world = startup_beside(file!())
        .expect("startup must succeed: process-tier select' test");

    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    let result = eval_in_frozen(&ast, &world, &env)
        .expect("eval_in_frozen must succeed: process-tier select' picks ready peer");

    // Stone 259: select' returns ServiceEvent<I,O>; happy path is :Message{idx, msg}.
    match result.value_owned() {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::spawn::ServiceEvent",
                "select' must return ServiceEvent; got type_path {:?}",
                ev.type_path
            );
            assert_eq!(
                ev.variant_name, "Message",
                "ready peer must yield :Message; got {:?}",
                ev.variant_name
            );
            assert_eq!(ev.fields.len(), 2, "Message must have idx + msg; got {:?}", ev.fields);
            assert_eq!(
                ev.fields[0],
                Value::i64(1),
                "ready peer is index 1 (b); got {:?}",
                ev.fields[0]
            );
            assert_eq!(
                ev.fields[1],
                Value::i64(99),
                "echoed value must be 99; got {:?}",
                ev.fields[1]
            );
        }
        other => panic!("expected ServiceEvent::Message; got {:?}", other),
    }
}
