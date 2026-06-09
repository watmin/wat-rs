//! Arc 257.2 probe — keys-destructure in binder position.
//!
//! The arc's load-bearing assumption: a MAP in binder position is a destructure.
//! The EDN-conformant replacement for the old non-EDN `{x y z}` struct-destructure
//! is the Clojure `{:keys [x y z]}` keys-destructure (binds each named field).
//!
//! Arc 257.2 wires the parser (all `{…}` → Map) and the 14 binding-context
//! sites to use `classify_map_destructure`. These probes are GREEN after 257.2.
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

// ─── Probe 1 — single-field keys-destructure ────────────────────────────────
// Uses defstruct (TypeDef::Struct) so check-time field lookup works.
// keys-destructure is the EDN-conformant replacement for the old {field} form.
#[test]
fn probe_1_keys_destructure_single_field() {
    let src = r#"
(:wat::core::defstruct :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage/new 5.0)
       {:keys [magnitude]} v]
      magnitude))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!((f - 5.0).abs() < 1e-9, "got {}", f),
        Ok(other) => panic!("Probe 1: expected f64 5.0; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 — multi-field keys-destructure ─────────────────────────────────
#[test]
fn probe_2_keys_destructure_multi_field() {
    let src = r#"
(:wat::core::defstruct :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [t (:myapp::Triple/new 7 "hello" true)
       {:keys [a b c]} t]
      b))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(s.as_str(), "hello"),
        Ok(other) => panic!("Probe 2: expected String \"hello\"; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 — negative: {x y z} in binder position is now a clear error ────
#[test]
fn probe_3_bare_symbol_brace_form_rejected() {
    // `{x y}` parses as a Map with pair (Symbol(x), Symbol(y)) which is
    // NOT a valid destructure (not :keys, not Symbol→Keyword pairs).
    // classify_map_destructure returns None → binder dispatch emits MalformedForm.
    let src = r#"
(:wat::core::defstruct :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage/new 5.0)
       {magnitude something} v]
      magnitude))
"#;
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("Probe 3: expected error for bare-symbol brace-form in binder; got Ok"),
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.to_lowercase().contains("malformed") || msg.to_lowercase().contains("binder") || msg.to_lowercase().contains("keys"),
                "Probe 3: error must explain rejection (migrate to {{:keys [...]}}); got: {}",
                msg
            );
        }
    }
}
