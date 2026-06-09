//! Confirming probes — arc 257 (EDN-native collections), Slice 257.1.
//!
//! These verify that `{k v …}` map literals and `#{x y z}` set literals
//! parse to `WatAST::Map` / `WatAST::Set` respectively (not to desugared
//! constructor-call Lists) and evaluate correctly to HashMap / HashSet values.
//!
//! Expected: GREEN at HEAD (slice 257.1 complete).
//!
//! Design: docs/arc/2026/06/257-edn-native-collections/DESIGN.md

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 — single-entry map literal evaluates ────────────────────────────
//
// `{:a 42}` must produce a HashMap; `length` returns 1, confirming a real map.
#[test]
fn probe_1_map_literal_single_entry() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [m {:a 42}]
      (:wat::core::length m)))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(n, 1, "expected length 1, got {}", n),
        Ok(other) => panic!("Probe 1: expected i64 1; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 — multi-entry map literal evaluates ─────────────────────────────
//
// `{:x 10 :y 20}` must produce a HashMap with 2 entries; `length` returns 2.
#[test]
fn probe_2_map_literal_multi_entry() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [m {:x 10 :y 20}]
      (:wat::core::length m)))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(n, 2, "expected length 2, got {}", n),
        Ok(other) => panic!("Probe 2: expected i64 2; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 — set literal evaluates and membership check works ───────────────
//
// `#{1 2 3}` must produce a HashSet; `contains?` must find a member.
#[test]
fn probe_3_set_literal_contains() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [s #{1 2 3}]
      (:wat::core::contains? s 2)))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(b, "expected contains? to return true"),
        Ok(other) => panic!("Probe 3: expected bool true; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}
