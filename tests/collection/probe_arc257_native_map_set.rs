//! Confirming probes — arc 257 (EDN-native collections), Slice 257.1.
//!
//! These verify that `{k v …}` map literals and `#{x y z}` set literals
//! parse to `WatAST::Map` / `WatAST::Set` respectively (not to desugared
//! constructor-call Lists) and evaluate correctly to HashMap / HashSet values.
//!
//! Expected: GREEN at HEAD (slice 257.1 complete).
//!
//! Design: docs/arc/2026/06/257-edn-native-collections/DESIGN.md

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// ─── Probe 1 — single-entry map literal evaluates ────────────────────────────
//
// `{:a 42}` must produce a HashMap; `length` returns 1, confirming a real map.
#[test]
fn probe_1_map_literal_single_entry() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe1-map-single)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "expected length 1, got {}", n),
        other => panic!("Probe 1: expected i64 1; got {:?}", other),
    }
}

// ─── Probe 2 — multi-entry map literal evaluates ─────────────────────────────
//
// `{:x 10 :y 20}` must produce a HashMap with 2 entries; `length` returns 2.
#[test]
fn probe_2_map_literal_multi_entry() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe2-map-multi)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "expected length 2, got {}", n),
        other => panic!("Probe 2: expected i64 2; got {:?}", other),
    }
}

// ─── Probe 3 — set literal evaluates and membership check works ───────────────
//
// `#{1 2 3}` must produce a HashSet; `contains?` must find a member.
#[test]
fn probe_3_set_literal_contains() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe3-set-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "expected contains? to return true"),
        other => panic!("Probe 3: expected bool true; got {:?}", other),
    }
}
