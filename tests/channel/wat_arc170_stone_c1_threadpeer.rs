//! Arc 170 Stone C1 — `:wat::kernel::ThreadPeer<I, O>` substrate type
//! plus the two peer-relative verbs `:wat::kernel::Thread/readln` and
//! `:wat::kernel::Thread/println`.
//!
//! Per `INTERSTITIAL-REALIZATIONS.md` § 2026-05-16 (Stone C revision):
//! one struct, peer-relative type parameters — the conceptual
//! client/server distinction is encoded by mirror bindings of
//! `<I, O>`. Both peers are instances of the SAME struct.
//!
//! Test 1 — type mint. Wat source declares `ThreadPeer<i64, String>`
//! and the mirror `ThreadPeer<String, i64>` as function parameter
//! types; both must type-check.
//!
//! Test 2 — verb dispatch. A substrate-internal Rust helper
//! (`make_thread_peer_pair`) constructs two cross-wired peers; peer A
//! writes via `Thread/println`; peer B reads via `Thread/readln`; the
//! value round-trips with the correct type.
//!
//! Test 3 — type-param swap. With symmetric peers
//! `ThreadPeer<i64, String>` ↔ `ThreadPeer<String, i64>`, both
//! directions of the cross-wired conversation succeed and surface
//! values of the expected runtime variant — proving the substrate
//! does not collapse I and O.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// ─── Stone C1 T1. type mint — both ThreadPeer<i64,String> and the
//      mirror ThreadPeer<String,i64> type-check ────────────────────────

#[test]
fn stone_c1_thread_peer_type_mint_both_orientations_type_check() {
    // Declare two helper fns, one per orientation. Each takes a
    // ThreadPeer parameter and returns nil. We never CALL them — the
    // mint test is purely that the parametric type resolves at freeze
    // time. Bodies use `:wat::core::nil` to satisfy the return.
    let world = startup_beside(file!()).expect("startup");
    assert!(
        world.symbols().get(":my::server-side").is_some(),
        "server-side fn must be present after freeze"
    );
    assert!(
        world.symbols().get(":my::client-side").is_some(),
        "client-side fn must be present after freeze"
    );
}

// ─── Stone C1 T2. verb dispatch — peer A writes i64; peer B reads i64;
//      value round-trips ──────────────────────────────────────────────

#[test]
fn stone_c1_thread_peer_verb_dispatch_round_trips_i64() {
    // Pre-build the peer pair via the substrate-internal Rust helper, then apply the
    // co-located fixture's println/readln fns with each peer as the argument.
    //
    // Peer A is ThreadPeer<String, i64> — it WRITES i64 (its O = i64).
    // Peer B is ThreadPeer<i64, String> — it READS i64 (its I = i64).
    //
    // just-eval (rubric): peer_a/peer_b are Rust-native handles — the println/readln calls
    // live in the co-located fixture's `:my::write-i64-42` / `:my::read-i64`, driven via
    // `apply_function` with the peer as the argument.
    let world = startup_beside(file!()).expect("startup");
    let (peer_a, peer_b) =
        wat::channel::make_thread_peer_pair_for_test();

    // peer A writes 42i64.
    let write_func = world.symbols().get(":my::write-i64-42").expect(":my::write-i64-42 defined");
    let write_outcome = apply_function(write_func.clone(), vec![peer_a], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/println should return Ok(nil)");
    assert!(
        matches!(write_outcome, Value::Unit),
        "Thread/println must return Unit (== nil); got {:?}",
        write_outcome
    );

    // peer B reads — value must come back as i64(42).
    let read_func = world.symbols().get(":my::read-i64").expect(":my::read-i64 defined");
    let read_outcome = apply_function(read_func.clone(), vec![peer_b], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/readln should surface the i64");
    match read_outcome {
        Value::i64(n) => assert_eq!(n, 42, "round-tripped i64 must be 42; got {}", n),
        other => panic!("expected Value::i64(42); got {:?}", other),
    }
}

// ─── Stone C1 T3. type-param swap — both directions of the
//      cross-wired conversation surface the right runtime variant ────

#[test]
fn stone_c1_thread_peer_type_param_swap_both_directions_round_trip() {
    // Cross-wired peers — peer A: ThreadPeer<String, i64> (reads
    // String, writes i64); peer B: ThreadPeer<i64, String> (reads i64,
    // writes String). Drive both directions and verify each surface
    // value's runtime variant matches the expected I parameter.
    //
    // just-eval (rubric): peer_a/peer_b are Rust-native handles — each println/readln call
    // lives in the co-located fixture (`:my::write-i64-7`, `:my::read-i64`, `:my::write-pong`,
    // `:my::read-string`), driven via `apply_function` with the peer as the argument.
    let world = startup_beside(file!()).expect("startup");
    let (peer_a, peer_b) =
        wat::channel::make_thread_peer_pair_for_test();

    // ── Direction 1: peer A writes i64 7 → peer B reads i64 7.
    let write_i64_func = world.symbols().get(":my::write-i64-7").expect(":my::write-i64-7 defined");
    let w1 = apply_function(write_i64_func.clone(), vec![peer_a.clone()], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/println i64 should succeed");
    assert!(matches!(w1, Value::Unit), "Unit expected; got {:?}", w1);

    let read_i64_func = world.symbols().get(":my::read-i64").expect(":my::read-i64 defined");
    let r1 = apply_function(read_i64_func.clone(), vec![peer_b.clone()], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/readln should surface the i64");
    match r1 {
        Value::i64(n) => assert_eq!(n, 7, "peer B's I = i64; got {}", n),
        other => panic!("peer B must read i64 (its I); got {:?}", other),
    }

    // ── Direction 2: peer B writes String "pong" → peer A reads String "pong".
    let write_pong_func = world.symbols().get(":my::write-pong").expect(":my::write-pong defined");
    let w2 = apply_function(write_pong_func.clone(), vec![peer_b], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/println String should succeed");
    assert!(matches!(w2, Value::Unit), "Unit expected; got {:?}", w2);

    let read_string_func = world.symbols().get(":my::read-string").expect(":my::read-string defined");
    let r2 = apply_function(read_string_func.clone(), vec![peer_a], world.symbols(), wat::rust_caller_span!())
        .expect("Thread/readln should surface the String");
    match r2 {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "pong",
            "peer A's I = String; got {:?}",
            s
        ),
        other => panic!("peer A must read String (its I); got {:?}", other),
    }
}
