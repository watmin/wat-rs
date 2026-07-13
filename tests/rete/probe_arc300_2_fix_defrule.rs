//! Arc 300.2 — the fix conversion as PURE rete defrules.
//!
//! rete is always pure in wat: the rules DEDUCE classification facts; the deductions are
//! QUERIED OUT and ACTIONED (transformed + written) by the drive, OUTSIDE rete. No :then
//! ever transforms a value — it inserts a fact carrying only ?var bindings (offset/len/name).
//!
//! This test proves the three rules DEDUCE the right classification facts:
//!   - a head-keyword Node (kind=keyword, "::", ¬post-arrow, ¬type-shaped) → HeadConv
//!   - an arrow Node (kind=symbol, "<-"/"->")                              → ArrowConv
//!   - a post-arrow / type-shaped keyword Node                            → TypeConv
//! and that the facts carry the node's offset/len/name unchanged (the drive does the transform).
//!
//! The byte-identical golden reproduction is verified by running the DRIVE
//! (wat-scripts/fixes/to-faithful-clojure-rete.wat) against `:wat::fix::fix-text` — see the
//! arc 300.2 report / commit message.
//!
//! Run: cargo test --release -p wat --test rete probe_arc300_2

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn ev(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    eval_in_frozen(
        &wat::parse_one!(expr).expect("parse"),
        world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
    .value_owned()
}

/// Build the fire lifecycle for a single asserted Node, then a query tail expression.
fn fire_one(node_ctor: &str, query_tail: &str) -> String {
    format!(
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session {node_ctor})
              fired   (:wat::rete::fire-rules session)]
             {query_tail})"#
    )
}

// ── head-keyword→conv ────────────────────────────────────────────────────────

#[test]
fn head_keyword_deduces_headconv() {
    let world = startup_beside(file!()).expect("world freezes with fact model + rules");
    // :wat::core::defrecord — head keyword, not post-arrow, not type-shaped.
    let node = r#"(:fix::Node :kind "keyword" :name ":wat::core::defrecord" :offset 1 :len 21 :post-arrow false)"#;
    let count = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::HeadConv))"));
    assert_eq!(count, Value::i64(1), "one HeadConv deduced; got {count:?}");
    // The fact carries the RAW name (pure — no transform in :then).
    let name = ev(&world, &fire_one(node,
        "(:fix::HeadConv/name (:wat::core::first (:wat::rete::query fired :fix::HeadConv)))"));
    assert_eq!(name, Value::String(Arc::new(":wat::core::defrecord".to_string())),
        "HeadConv.name is the raw keyword string (pure); got {name:?}");
    // offset/len passed through.
    let off = ev(&world, &fire_one(node,
        "(:fix::HeadConv/offset (:wat::core::first (:wat::rete::query fired :fix::HeadConv)))"));
    assert_eq!(off, Value::i64(1), "HeadConv.offset passed through; got {off:?}");
}

#[test]
fn post_arrow_keyword_is_not_headconv() {
    let world = startup_beside(file!()).expect("world freezes");
    // post-arrow=true → excluded from head-keyword→conv (the ¬post-arrow guard).
    let node = r#"(:fix::Node :kind "keyword" :name ":wat::core::String" :offset 10 :len 18 :post-arrow true)"#;
    let count = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::HeadConv))"));
    assert_eq!(count, Value::i64(0), "post-arrow keyword is not a HeadConv; got {count:?}");
}

// ── arrow→conv ────────────────────────────────────────────────────────────────

#[test]
fn left_arrow_deduces_arrowconv() {
    let world = startup_beside(file!()).expect("world freezes");
    let node = r#"(:fix::Node :kind "symbol" :name "<-" :offset 0 :len 2 :post-arrow false)"#;
    let count = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::ArrowConv))"));
    assert_eq!(count, Value::i64(1), "'<-' deduces one ArrowConv; got {count:?}");
    let off = ev(&world, &fire_one(node,
        "(:fix::ArrowConv/offset (:wat::core::first (:wat::rete::query fired :fix::ArrowConv)))"));
    assert_eq!(off, Value::i64(0), "ArrowConv.offset passed through; got {off:?}");
}

#[test]
fn right_arrow_deduces_arrowconv() {
    let world = startup_beside(file!()).expect("world freezes");
    let node = r#"(:fix::Node :kind "symbol" :name "->" :offset 5 :len 2 :post-arrow false)"#;
    let count = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::ArrowConv))"));
    assert_eq!(count, Value::i64(1), "'->' also deduces an ArrowConv; got {count:?}");
}

#[test]
fn non_arrow_symbol_deduces_nothing() {
    let world = startup_beside(file!()).expect("world freezes");
    let node = r#"(:fix::Node :kind "symbol" :name "path" :offset 10 :len 4 :post-arrow false)"#;
    let arrows = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::ArrowConv))"));
    let heads = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::HeadConv))"));
    assert_eq!(arrows, Value::i64(0), "non-arrow symbol → no ArrowConv; got {arrows:?}");
    assert_eq!(heads, Value::i64(0), "non-arrow symbol → no HeadConv; got {heads:?}");
}

// ── type-keyword→conv ─────────────────────────────────────────────────────────

#[test]
fn post_arrow_keyword_deduces_typeconv() {
    let world = startup_beside(file!()).expect("world freezes");
    // :wat::core::String immediately after "<-" → post-arrow=true → TypeConv (not type-shaped, but post-arrow).
    let node = r#"(:fix::Node :kind "keyword" :name ":wat::core::String" :offset 10 :len 18 :post-arrow true)"#;
    let count = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::TypeConv))"));
    assert_eq!(count, Value::i64(1), "post-arrow keyword deduces a TypeConv; got {count:?}");
    let name = ev(&world, &fire_one(node,
        "(:fix::TypeConv/name (:wat::core::first (:wat::rete::query fired :fix::TypeConv)))"));
    assert_eq!(name, Value::String(Arc::new(":wat::core::String".to_string())),
        "TypeConv.name is the raw keyword string (pure); got {name:?}");
}

#[test]
fn type_shaped_keyword_deduces_typeconv_even_when_not_post_arrow() {
    let world = startup_beside(file!()).expect("world freezes");
    // A structurally-type-shaped keyword (Vector<...>) is a TypeConv even at head position.
    let node = r#"(:fix::Node :kind "keyword" :name ":wat::core::Vector<wat::core::i64>" :offset 0 :len 30 :post-arrow false)"#;
    let types = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::TypeConv))"));
    let heads = ev(&world, &fire_one(node, "(:wat::core::length (:wat::rete::query fired :fix::HeadConv))"));
    assert_eq!(types, Value::i64(1), "type-shaped keyword deduces a TypeConv; got {types:?}");
    // ...and is EXCLUDED from HeadConv (the ¬type-shaped guard) — no double edit.
    assert_eq!(heads, Value::i64(0), "type-shaped keyword excluded from HeadConv; got {heads:?}");
}
