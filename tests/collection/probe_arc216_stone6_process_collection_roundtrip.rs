//! Arc 216 Stone 6 — process-tier HolonRepresentable cascade validation.
//!
//! Proves that `Sender<T>::send` → tagged-EDN over pipe → `Receiver<T>::recv`
//! round-trips correctly for all three collection types added by Stones 216.1/216.2/216.3:
//! `HashSet<T>`, `Vec<T>`, `HashMap<K, V>` — including nested combinations.
//!
//! Pattern mirrors `tests/comms/process.rs` Stone C probes exactly:
//! `pair::<T>()` → `tx.send(...)` → `rx.recv()` → `assert_eq!(got, original)`.
//!
//! Wire chain (same as Stone C):
//! `T → HolonAST → tagged EDN string → newline-framed bytes → libc::write →
//!  io_uring Read → bytes → EDN → HolonAST → T`
//!
//! Tests:
//!   1. `probe_1_hashmap_string_string_round_trip`          — HashMap<String, String>
//!   2. `probe_2_hashset_string_round_trip`                 — HashSet<String>
//!   3. `probe_3_vec_string_round_trip`                     — Vec<String> (order preserved)
//!   4. `probe_4_nested_hashmap_string_vec_string`          — HashMap<String, Vec<String>>
//!   5. `probe_5_nested_vec_hashset_string`                 — Vec<HashSet<String>>
//!   6. `probe_6_triple_nested_hashmap_vec_hashset`         — HashMap<String, Vec<HashSet<String>>>
//!   7. `probe_7_empty_hashmap_round_trips_as_empty`        — empty HashMap preserves length 0
//!   8. `probe_8_fifo_ordering_with_collection_payloads`    — three sends, three recvs, FIFO preserved
//!   9. `probe_9_compile_time_holon_representable_check`    — static proof all collection types satisfy bound

use std::collections::{HashMap, HashSet};

use wat::comms::process::pair;

// ─── Probe 1 — HashMap<String, String> ───────────────────────────────────────

#[test]
fn probe_1_hashmap_string_string_round_trip() {
    // Verifies the full process-tier wire chain for HashMap<String, String>.
    // Two entries; recv must return a map with exactly the same entries.
    let (tx, rx) = pair::<HashMap<String, String>>().expect("pair");

    let mut original = HashMap::new();
    original.insert("alpha".to_string(), "one".to_string());
    original.insert("beta".to_string(), "two".to_string());

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent map");

    assert_eq!(got.len(), original.len(), "received map must have same entry count");
    assert_eq!(got, original, "received map must equal sent map");
}

// ─── Probe 2 — HashSet<String> ───────────────────────────────────────────────

#[test]
fn probe_2_hashset_string_round_trip() {
    // Verifies the full process-tier wire chain for HashSet<String>.
    // Three elements; recv must return a set with exactly the same elements.
    let (tx, rx) = pair::<HashSet<String>>().expect("pair");

    let original: HashSet<String> = ["apple", "banana", "cherry"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent set");

    assert_eq!(got.len(), original.len(), "received set must have same element count");
    assert_eq!(got, original, "received set must equal sent set");
}

// ─── Probe 3 — Vec<String> ───────────────────────────────────────────────────

#[test]
fn probe_3_vec_string_round_trip_order_preserved() {
    // Verifies the full process-tier wire chain for Vec<String>.
    // Three elements; recv must return a vec with the same elements in the
    // same order (positional-Bind encoding preserves sequence).
    let (tx, rx) = pair::<Vec<String>>().expect("pair");

    let original: Vec<String> = vec!["first".to_string(), "second".to_string(), "third".to_string()];

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent vec");

    assert_eq!(got.len(), original.len(), "received vec must have same length");
    assert_eq!(got, original, "received vec must equal sent vec (order preserved)");
}

// ─── Probe 4 — HashMap<String, Vec<String>> ──────────────────────────────────

#[test]
fn probe_4_nested_hashmap_string_vec_string() {
    // Verifies nested cascade: HashMap<String, Vec<String>> round-trips.
    // Stone 216.3 (HashMap) wraps Stone 216.2 (Vec) — both must compose
    // correctly across the wire chain.
    let (tx, rx) = pair::<HashMap<String, Vec<String>>>().expect("pair");

    let mut original: HashMap<String, Vec<String>> = HashMap::new();
    original.insert(
        "colors".to_string(),
        vec!["red".to_string(), "green".to_string(), "blue".to_string()],
    );
    original.insert(
        "shapes".to_string(),
        vec!["circle".to_string(), "square".to_string()],
    );

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent nested map");

    assert_eq!(got.len(), original.len(), "received nested map must have same entry count");
    assert_eq!(got, original, "received nested map must equal sent (Vec order preserved per key)");
}

// ─── Probe 5 — Vec<HashSet<String>> ─────────────────────────────────────────

#[test]
fn probe_5_nested_vec_hashset_string() {
    // Verifies nested cascade: Vec<HashSet<String>> round-trips.
    // Stone 216.2 (Vec) wraps Stone 216.1 (HashSet) — positional-Bind
    // outer encoding + bundle inner encoding both carry through the wire.
    let (tx, rx) = pair::<Vec<HashSet<String>>>().expect("pair");

    let set_a: HashSet<String> = ["x".to_string(), "y".to_string()].into_iter().collect();
    let set_b: HashSet<String> = ["p".to_string(), "q".to_string(), "r".to_string()].into_iter().collect();
    let original: Vec<HashSet<String>> = vec![set_a, set_b];

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent nested vec");

    assert_eq!(got.len(), original.len(), "received nested vec must have same length");
    assert_eq!(got, original, "received nested vec must equal sent (set equality, vec order preserved)");
}

// ─── Probe 6 — HashMap<String, Vec<HashSet<String>>> ────────────────────────

#[test]
fn probe_6_triple_nested_hashmap_vec_hashset() {
    // Verifies triple-nested cascade: HashMap<String, Vec<HashSet<String>>>.
    // Stone 216.3 wrapping Stone 216.2 wrapping Stone 216.1 — three layers
    // of HolonRepresentable composition across the process-tier wire chain.
    let (tx, rx) = pair::<HashMap<String, Vec<HashSet<String>>>>().expect("pair");

    let inner_a: HashSet<String> = ["m".to_string(), "n".to_string()].into_iter().collect();
    let inner_b: HashSet<String> = ["x".to_string()].into_iter().collect();
    let inner_c: HashSet<String> = ["a".to_string(), "b".to_string(), "c".to_string()].into_iter().collect();

    let mut original: HashMap<String, Vec<HashSet<String>>> = HashMap::new();
    original.insert("group1".to_string(), vec![inner_a, inner_b]);
    original.insert("group2".to_string(), vec![inner_c]);

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent triple-nested collection");

    assert_eq!(got.len(), original.len(), "received triple-nested map must have same entry count");
    assert_eq!(got, original, "received triple-nested map must equal sent");
}

// ─── Probe 7 — Empty HashMap ─────────────────────────────────────────────────

#[test]
fn probe_7_empty_hashmap_round_trips_as_empty() {
    // Verifies an empty HashMap<String, String> round-trips correctly.
    // Empty bundle (0 Bind children) must decode back to a HashMap with len 0.
    let (tx, rx) = pair::<HashMap<String, String>>().expect("pair");

    let original: HashMap<String, String> = HashMap::new();

    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the empty map");

    assert_eq!(got.len(), 0, "received empty map must have length 0");
    assert_eq!(got, original, "received empty map must equal sent empty map");
}

// ─── Probe 8 — FIFO ordering with collection payloads ────────────────────────

#[test]
fn probe_8_fifo_ordering_with_collection_payloads() {
    // Verifies that three sends with distinct collection payloads are received
    // in FIFO order. Mirrors probe_slice3c_fifo_ordering_preserved_across_sends
    // from Stone C, but with Vec<String> payloads instead of String.
    let (tx, rx) = pair::<Vec<String>>().expect("pair");

    let first = vec!["a".to_string(), "b".to_string()];
    let second = vec!["c".to_string()];
    let third = vec!["d".to_string(), "e".to_string(), "f".to_string()];

    tx.send(first.clone()).expect("send 1");
    tx.send(second.clone()).expect("send 2");
    tx.send(third.clone()).expect("send 3");

    assert_eq!(rx.recv().expect("recv 1"), first, "first recv must return first payload");
    assert_eq!(rx.recv().expect("recv 2"), second, "second recv must return second payload");
    assert_eq!(rx.recv().expect("recv 3"), third, "third recv must return third payload");
}

// ─── Probe 9 — Compile-time HolonRepresentable check ────────────────────────

#[test]
fn probe_9_compile_time_holon_representable_check() {
    // Verifies at compile time that all collection variants satisfy the
    // HolonRepresentable bound. The function body is a static proof:
    // if this file compiles, the bound is satisfied for each type.
    //
    // The fact that cargo test --release --test probe_arc216_stone6_process_collection_roundtrip
    // BUILDS proves Stones 216.1/216.2/216.3 are correctly wired to the trait.
    // The runtime call proves the monomorphized symbols exist in the binary.
    fn assert_holon_representable<T: wat::comms::HolonRepresentable>() {}

    assert_holon_representable::<HashMap<String, String>>();
    assert_holon_representable::<HashSet<String>>();
    assert_holon_representable::<Vec<String>>();
    assert_holon_representable::<HashMap<String, Vec<String>>>();
    assert_holon_representable::<Vec<HashSet<String>>>();
    assert_holon_representable::<HashMap<String, Vec<HashSet<String>>>>();

    // Proof inscribed: six monomorphizations compile + link. Cascade is real.
}
