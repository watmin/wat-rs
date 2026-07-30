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
//!
//! and that the facts carry the node's offset/len/name unchanged (the drive does the transform).
//!
//! The byte-identical golden reproduction is verified by running the DRIVE
//! (wat-scripts/fixes/to-faithful-clojure-rete.wat) against `:wat::fix::fix-text` — see the
//! arc 300.2 report / commit message.
//!
//! Run: cargo test --release -p wat --test rete probe_arc300_2

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each (node, query-tail) pair is a fixed, enumerable named entry in the
// co-located fixture — driven via call_beside_value.
fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

// ── head-keyword→conv ────────────────────────────────────────────────────────

#[test]
fn head_keyword_deduces_headconv() {
    // :wat::core::defrecord — head keyword, not post-arrow, not type-shaped.
    let count = call(":user::head-keyword-count");
    assert_eq!(count, Value::i64(1), "one HeadConv deduced; got {count:?}");
    // The fact carries the RAW name (pure — no transform in :then).
    let name = call(":user::head-keyword-name");
    assert_eq!(name, Value::String(Arc::new(":wat::core::defrecord".to_string())),
        "HeadConv.name is the raw keyword string (pure); got {name:?}");
    // offset/len passed through.
    let off = call(":user::head-keyword-offset");
    assert_eq!(off, Value::i64(1), "HeadConv.offset passed through; got {off:?}");
}

#[test]
fn post_arrow_keyword_is_not_headconv() {
    // post-arrow=true → excluded from head-keyword→conv (the ¬post-arrow guard).
    let count = call(":user::post-arrow-keyword-headconv-count");
    assert_eq!(count, Value::i64(0), "post-arrow keyword is not a HeadConv; got {count:?}");
}

// ── arrow→conv ────────────────────────────────────────────────────────────────

#[test]
fn left_arrow_deduces_arrowconv() {
    let count = call(":user::left-arrow-count");
    assert_eq!(count, Value::i64(1), "'<-' deduces one ArrowConv; got {count:?}");
    let off = call(":user::left-arrow-offset");
    assert_eq!(off, Value::i64(0), "ArrowConv.offset passed through; got {off:?}");
}

#[test]
fn right_arrow_deduces_arrowconv() {
    let count = call(":user::right-arrow-count");
    assert_eq!(count, Value::i64(1), "'->' also deduces an ArrowConv; got {count:?}");
}

#[test]
fn non_arrow_symbol_deduces_nothing() {
    let arrows = call(":user::non-arrow-arrows-count");
    let heads = call(":user::non-arrow-heads-count");
    assert_eq!(arrows, Value::i64(0), "non-arrow symbol → no ArrowConv; got {arrows:?}");
    assert_eq!(heads, Value::i64(0), "non-arrow symbol → no HeadConv; got {heads:?}");
}

// ── type-keyword→conv ─────────────────────────────────────────────────────────

#[test]
fn post_arrow_keyword_deduces_typeconv() {
    // :wat::core::String immediately after "<-" → post-arrow=true → TypeConv (not type-shaped, but post-arrow).
    let count = call(":user::post-arrow-typeconv-count");
    assert_eq!(count, Value::i64(1), "post-arrow keyword deduces a TypeConv; got {count:?}");
    let name = call(":user::post-arrow-typeconv-name");
    assert_eq!(name, Value::String(Arc::new(":wat::core::String".to_string())),
        "TypeConv.name is the raw keyword string (pure); got {name:?}");
}

#[test]
fn type_shaped_keyword_deduces_typeconv_even_when_not_post_arrow() {
    // A structurally-type-shaped keyword (Vector<...>) is a TypeConv even at head position.
    let types = call(":user::type-shaped-typeconv-count");
    let heads = call(":user::type-shaped-headconv-count");
    assert_eq!(types, Value::i64(1), "type-shaped keyword deduces a TypeConv; got {types:?}");
    // ...and is EXCLUDED from HeadConv (the ¬type-shaped guard) — no double edit.
    assert_eq!(heads, Value::i64(0), "type-shaped keyword excluded from HeadConv; got {heads:?}");
}
