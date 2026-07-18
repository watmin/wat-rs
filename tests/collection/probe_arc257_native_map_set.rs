//! Confirming probes — arc 257 (EDN-native collections), Slice 257.1.
//!
//! These verify that `{k v …}` map literals and `#{x y z}` set literals
//! parse to `WatAST::Map` / `WatAST::Set` respectively (not to desugared
//! constructor-call Lists) and evaluate correctly to HashMap / HashSet values.
//!
//! Expected: GREEN at HEAD (slice 257.1 complete).
//!
//! Design: docs/arc/2026/06/257-edn-native-collections/DESIGN.md

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:t::probeN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside` — no inline wat driver.

// ─── Probe 1 — single-entry map literal evaluates ────────────────────────────
//
// `{:a 42}` must produce a HashMap; `length` returns 1, confirming a real map.
#[test]
fn probe_1_map_literal_single_entry() {
    match call_beside(file!(), ":t::probe1-map-single").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "expected length 1, got {}", n),
        other => panic!("Probe 1: expected i64 1; got {:?}", other),
    }
}

// ─── Probe 2 — multi-entry map literal evaluates ─────────────────────────────
//
// `{:x 10 :y 20}` must produce a HashMap with 2 entries; `length` returns 2.
#[test]
fn probe_2_map_literal_multi_entry() {
    match call_beside(file!(), ":t::probe2-map-multi").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "expected length 2, got {}", n),
        other => panic!("Probe 2: expected i64 2; got {:?}", other),
    }
}

// ─── Probe 3 — set literal evaluates and membership check works ───────────────
//
// `#{1 2 3}` must produce a HashSet; `contains?` must find a member.
#[test]
fn probe_3_set_literal_contains() {
    match call_beside(file!(), ":t::probe3-set-contains").expect("eval") {
        Value::bool(b) => assert!(b, "expected contains? to return true"),
        other => panic!("Probe 3: expected bool true; got {:?}", other),
    }
}
