//! Arc 214 Stone 4.6a-ii — process-tier peer verb round-trip (wat surface).
//!
//! Mirrors probe-1 from the nursery (probe_arc214_stone46aii_peer_verbs.rs)
//! but uses `:process` instead of `:thread`, driven end-to-end through the
//! WAT type-checker and runtime via `startup_from_source` + `eval_in_frozen`.
//!
//! Verifies:
//! 1. `spawn-program' :process` infers to `Process'<i64,i64>`.
//! 2. `send' peer 42` encodes 42 → EDN, sends across the fork boundary.
//! 3. `recv' peer` receives the echoed EDN string, decodes → Value::i64(42).
//! 4. `close' peer` closes channels + waits for child; returns 0 (exit code).
//!
//! # Containment
//!
//! The test forks via `spawn-program' :process`. Run under setsid + timeout
//! to prevent fd/lock inheritance from the multi-threaded cargo test binary.
//! Marked `#[ignore]` — run via:
//!   setsid timeout 180 cargo test --release --test kernel peer_verb_round_trip_process -- --ignored --test-threads=1
//! or the `integration-run.sh` harness.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Process-tier send'/recv' round-trip via the WAT surface; peer reaped by RAII Drop.
///
/// Arc 259 S2c-ii-b: spawn-program' migrated to 2-arg `(:wat::spawn::process)` host form.
/// The server reads one i64 and writes n+1 (the proven arc112 echo+1 shape).
/// Parent sends 41, server returns 42.
///
/// `#[ignore]` — process-tier probe; run under setsid + timeout with --test-threads=1.
#[test]
#[ignore = "process-tier probe: run via setsid timeout 180 cargo test --release --test kernel peer_verb_round_trip_process -- --ignored --test-threads=1"]
fn process_peer_verb_round_trip() {
    let result = call_beside_value(file!(), ":user::compute")
        .expect("call_beside_value must succeed: process-tier peer verb round-trip");

    match result {
        Value::i64(n) => assert_eq!(
            n,
            42,
            "process-tier forms-server echo+1 must return 42 for input 41; got {}",
            n
        ),
        other => panic!("expected i64(42); got {:?}", other),
    }
}
