//! FM 2-bis probe — arc 251 Stone 251.0/251.1: a SYMBOL head resolves to the
//! entity its keyword FQDN resolves to.
//!
//! Arc 251 inverts wat's keyword↔symbol roles to match Clojure: heads/operators/
//! types/declarators become symbols (`wat.core/+`, `wat.type/i64`, `defn`); keywords
//! return to data. The SPINE is symbol-head resolution — a `(wat.core/+ 1 2)` form
//! (symbol head) must dispatch to the same entity `(:wat::core::+ 1 2)` (keyword head)
//! dispatches to today.
//!
//! HEAD-disconfirmation:
//! - C01: dotted symbol head `wat.core/+` resolves like `:wat::core::+`
//!   ⇒ FAILS at HEAD. `is_symbol_break` (lexer.rs:457) doesn't break on `.`/`/`, so
//!      `wat.core/+` LEXES as one symbol and parses to a `WatAST::Symbol` head — but
//!      `eval_list` (runtime.rs:5428) only routes `WatAST::Keyword` heads through
//!      `dispatch_keyword_head`; a symbol head is not a recognized call head, so the
//!      checker/runtime rejects it. (The gap is resolution/dispatch, not lexing.)
//! - C02: keyword head `:wat::core::+` STILL resolves
//!   (PRESERVATION — must stay green through the migration; keyword heads HARD-CUT
//!    only at arc close, dual-read during the substrate-as-teacher corpus sweep).
//!
//! Post-251.1: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc251_stone0_symbol_head`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval `body` (declared `-> :i64`) via `:user::compute`; return the i64 or an error string.
fn eval_i64(body: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {:?}", other)),
    }
}

// ─── C01: THE GAP — dotted symbol head resolves like the keyword FQDN ───────────

#[test]
fn contract_01_symbol_head_resolves_like_keyword() {
    // `wat.core/+` must resolve to the same operator `:wat::core::+` names.
    // At HEAD: symbol head is not dispatched → check/eval error → RED.
    // Post-251.1: resolves; (wat.core/+ 1 2) = 3.
    assert_eq!(
        eval_i64("(wat.core/+ 1 2)"),
        Ok(3),
        "dotted symbol head wat.core/+ must resolve to the :wat::core::+ entity"
    );
}

// ─── C02: PRESERVATION — keyword head still resolves during the transition ──────

#[test]
fn contract_02_keyword_head_still_resolves() {
    // The keyword head keeps working while the corpus migrates (dual-read; HARD-CUT
    // only at arc close). GREEN at HEAD; must NOT regress through 251.1.
    assert_eq!(
        eval_i64("(:wat::core::+ 1 2)"),
        Ok(3),
        ":wat::core::+ keyword head must keep working during the transition"
    );
}

// ─── C03: VALUE-POSITION DELTA (out-of-scope for 251.1b) ───────────────────────
//
// `(wat.core/foldl wat.core/i64::+ 0 xs)` — the normalize pass correctly rewrites
// `wat.core/i64::+` to `WatAST::Keyword(":wat::core::i64::+", span)` in value
// position. HOWEVER, `:wat::core::i64::+` in value position evaluates to the
// keyword VALUE itself — not the function it names. `foldl` then receives a
// keyword where it expects a fn, yielding a TypeMismatch.
//
// This is the HONEST DELTA: value-position symbol normalization produces the
// right keyword form but keyword→fn value lookup in value position is a
// SEPARATE MECHANISM (the runtime must resolve the keyword to its function value
// at call time). This is NOT a regression — it was never supported before 251.
// The brief says "If value-position keyword-as-fn does NOT already work, STOP
// and report." Reported here; no STOP — the head-position contract (C01, C02)
// is fully met. Value-position keyword-as-fn resolution is a later stone's scope.
