//! FM 2-bis probe for Stone 241.7 — mint `:wat::runtime::metadata-of` reflection verb.
//!
//! Reads SymbolTable.binding_metadata that Stone 241.6 stored. Returns
//! Option<HashMap<Keyword, HolonAST>> per arc 216.7 + 218.2 FQDN tagged-literal
//! encoding (`#wat.core/Some {...}` / `#wat.core/None nil`).
//!
//! Pre-stone: contracts FAIL — the verb doesn't exist; calls error.
//! Post-stone: N/N PASS; reflection round-trips metadata stored by 241.6.
//!
//! Run: `cargo test --release --test probe_arc241_stone7_metadata_of_reflection`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn try_compute(src: &str) -> Result<Value, String> {
    let full = with_nil_main(src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn is_some(v: &Value) -> bool {
    matches!(v, Value::Option(opt) if opt.as_ref().is_some())
}

fn is_none(v: &Value) -> bool {
    matches!(v, Value::Option(opt) if opt.as_ref().is_none())
}

// ─── Contracts 1–3: presence path (Some) ─────────────────────────────────────

#[test]
fn contract_01_def_with_metadata_returns_some() {
    // def with single-entry metadata; metadata-of returns Some(map).
    let src = r#"
        (:wat::core::def :my::x
          {:doc "the x value"}
          42)
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::runtime::metadata-of :my::x))
    "#;
    let result = try_compute(src).expect("def-with-metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "def-with-metadata metadata-of returns Some; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defn_with_metadata_returns_some() {
    // defn with metadata; the fn-peel substrate flows metadata to the BINDING level;
    // metadata-of on the binding name returns Some(map).
    let src = r#"
        (:wat::core::defn :my::f
          {:doc "doubles x"}
          [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+ x x))
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::runtime::metadata-of :my::f))
    "#;
    let result = try_compute(src).expect("defn-with-metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "defn-with-metadata metadata-of returns Some via fn-peel round-trip; got: {:?}",
        result
    );
}

#[test]
fn contract_03_multi_entry_metadata_returns_some() {
    // Multi-entry metadata.
    let src = r#"
        (:wat::core::def :my::y
          {:doc "documented"
           :deprecated true}
          100)
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::runtime::metadata-of :my::y))
    "#;
    let result = try_compute(src).expect("multi-entry metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "multi-entry metadata round-trips via Some; got: {:?}",
        result
    );
}

// ─── Contracts 4–5: absence path (None) ──────────────────────────────────────

#[test]
fn contract_04_def_without_metadata_returns_none() {
    // def with NO metadata; metadata-of returns None.
    let src = r#"
        (:wat::core::def :my::no-meta 42)
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::runtime::metadata-of :my::no-meta))
    "#;
    let result = try_compute(src).expect("def-without-metadata metadata-of must not error");
    assert!(
        is_none(&result),
        "def-without-metadata metadata-of returns None; got: {:?}",
        result
    );
}

#[test]
fn contract_05_unknown_binding_returns_none() {
    // Unknown name → None (not an error).
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::runtime::metadata-of :my::nonexistent))
    "#;
    let result = try_compute(src).expect("unknown binding metadata-of must not error");
    assert!(
        is_none(&result),
        "unknown binding metadata-of returns None; got: {:?}",
        result
    );
}
