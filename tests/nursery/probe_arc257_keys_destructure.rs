//! Disconfirming probe — arc 257 (EDN-native collections), Slice 257.0.
//!
//! The arc's load-bearing assumption: a MAP in binder position is a destructure.
//! The EDN-conformant replacement for the old non-EDN `{x y z}` struct-destructure
//! is the Clojure `{:keys [x y z]}` keys-destructure (binds each named field).
//!
//! At HEAD this is RED: `{:keys [magnitude]}` parses as a map LITERAL
//! (`(:wat::core::HashMap :Infer :Infer :keys [magnitude])`), and
//! `parse_let_binding` rejects a HashMap-list binder ("let binder must be a
//! Symbol, a Vector, or a StructPattern"). The arc makes a Map-in-binder-position
//! a destructure (Slice 257.3), at which point these go GREEN.
//!
//! Design: docs/arc/2026/06/257-edn-native-collections/DESIGN.md
//!
//! Expected: RED at HEAD (binder not recognized) → GREEN after 257.3.

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

// ─── Probe 1 — single-field keys-destructure ────────────────────────────────
#[test]
fn probe_1_keys_destructure_single_field() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [{:keys [magnitude]} (:myapp::Voltage 5.0)]
      magnitude))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!((f - 5.0).abs() < 1e-9, "got {}", f),
        Ok(other) => panic!("Probe 1: expected f64 5.0; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED (expected RED at HEAD until 257.3): {}", e),
    }
}

// ─── Probe 2 — multi-field keys-destructure ─────────────────────────────────
#[test]
fn probe_2_keys_destructure_multi_field() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [{:keys [a b c]} (:myapp::Triple 7 "hello" true)]
      b))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(s.as_str(), "hello"),
        Ok(other) => panic!("Probe 2: expected String \"hello\"; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED (expected RED at HEAD until 257.3): {}", e),
    }
}
