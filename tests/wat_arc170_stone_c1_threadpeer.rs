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

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, Value};

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

// ─── Stone C1 T1. type mint — both ThreadPeer<i64,String> and the
//      mirror ThreadPeer<String,i64> type-check ────────────────────────

#[test]
fn stone_c1_thread_peer_type_mint_both_orientations_type_check() {
    // Declare two helper fns, one per orientation. Each takes a
    // ThreadPeer parameter and returns nil. We never CALL them — the
    // mint test is purely that the parametric type resolves at freeze
    // time. Bodies use `:wat::core::nil` to satisfy the return.
    let src = r#"
        (:wat::core::defn :my::server-side
          [_peer <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
          -> :wat::core::nil
          :wat::core::nil)

        (:wat::core::defn :my::client-side
          [_peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
          -> :wat::core::nil
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
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
    // Empty parent world (no defines needed) just so we have a
    // SymbolTable / TypeEnv to drive eval. Pre-build the peer pair via
    // the substrate-internal Rust helper, bind them into the
    // environment, then hand-roll the readln + println call ASTs.
    //
    // Peer A is ThreadPeer<String, i64> — it WRITES i64 (its O = i64).
    // Peer B is ThreadPeer<i64, String> — it READS i64 (its I = i64).
    let world = freeze_ok("");
    let (peer_a, peer_b) =
        wat::typed_channel::make_thread_peer_pair_for_test();

    // peer A writes 42i64.
    let env_w = Environment::new()
        .child()
        .bind("peer_a", peer_a)
        .build();
    let write_call = wat::parse_one!("(:wat::kernel::Thread/println peer_a 42)")
        .expect("println AST parses");
    let write_outcome = eval(&write_call, &env_w, world.symbols())
        .expect("Thread/println should return Ok(nil)");
    assert!(
        matches!(write_outcome, Value::Unit),
        "Thread/println must return Unit (== nil); got {:?}",
        write_outcome
    );

    // peer B reads — value must come back as i64(42).
    let env_r = Environment::new()
        .child()
        .bind("peer_b", peer_b)
        .build();
    let read_call = wat::parse_one!("(:wat::kernel::Thread/readln peer_b)")
        .expect("readln AST parses");
    let read_outcome = eval(&read_call, &env_r, world.symbols())
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
    let world = freeze_ok("");
    let (peer_a, peer_b) =
        wat::typed_channel::make_thread_peer_pair_for_test();

    // ── Direction 1: peer A writes i64 7 → peer B reads i64 7.
    let env_aw = Environment::new()
        .child()
        .bind("peer_a", peer_a.clone())
        .build();
    let write_i64 = wat::parse_one!("(:wat::kernel::Thread/println peer_a 7)")
        .expect("println AST parses");
    let w1 = eval(&write_i64, &env_aw, world.symbols())
        .expect("Thread/println i64 should succeed");
    assert!(matches!(w1, Value::Unit), "Unit expected; got {:?}", w1);

    let env_br = Environment::new()
        .child()
        .bind("peer_b", peer_b.clone())
        .build();
    let read_i64 = wat::parse_one!("(:wat::kernel::Thread/readln peer_b)")
        .expect("readln AST parses");
    let r1 = eval(&read_i64, &env_br, world.symbols())
        .expect("Thread/readln should surface the i64");
    match r1 {
        Value::i64(n) => assert_eq!(n, 7, "peer B's I = i64; got {}", n),
        other => panic!("peer B must read i64 (its I); got {:?}", other),
    }

    // ── Direction 2: peer B writes String "pong" → peer A reads String "pong".
    let env_bw = Environment::new()
        .child()
        .bind("peer_b", peer_b)
        .build();
    let write_str = wat::parse_one!(r#"(:wat::kernel::Thread/println peer_b "pong")"#)
        .expect("println string AST parses");
    let w2 = eval(&write_str, &env_bw, world.symbols())
        .expect("Thread/println String should succeed");
    assert!(matches!(w2, Value::Unit), "Unit expected; got {:?}", w2);

    let env_ar = Environment::new()
        .child()
        .bind("peer_a", peer_a)
        .build();
    let read_str = wat::parse_one!("(:wat::kernel::Thread/readln peer_a)")
        .expect("readln AST parses");
    let r2 = eval(&read_str, &env_ar, world.symbols())
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
