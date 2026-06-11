//! Forward-proof probe — Stone 251.5 / Slice 4.2a: `ast-span` (intueri-named).
//!
//! `(:wat::core::ast-span node) -> {:line N :col N}` — a plain map (HashMap<keyword,i64>) of the
//! node's source START location. The one substrate verb that unlocks wat's comment-faithful
//! codemod (wat's rewrite-clj). Rhymes with `ast-kind`/`ast-name` (property-read of a node).
//! `:file` is dropped (the codemod processes one known file; a mixed-value map is un-typeable in
//! wat's ADT model — `{:line :col}` is homogeneous i64).
//!
//! Ground truth (pinned empirically from `(:wat::core::map x)`):
//!   top `(`            -> {:line 1 :col 1}
//!   head `:wat::core::map` -> {:line 1 :col 2}
//!   symbol `x`         -> {:line 1 :col 18}
//! col = 1-indexed char-count from line start, at the token START.
//!
//! RED at HEAD: `:wat::core::ast-span` is UnknownFunction.
//!
//! Run: `cargo test --release --test probe_arc251_ast_span`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval an i64-returning `compute` body.
fn eval_i64(body: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

/// `head = (first (ast->children (read-string "(:wat::core::map x)")))`; read `key` off its span.
fn head_span_field(key: &str) -> Result<i64, String> {
    eval_i64(&format!(
        "(:wat::core::Option/expect -> :wat::core::i64 \
           (:wat::core::HashMap/get \
             (:wat::core::ast-span \
               (:wat::core::Option/expect -> :wat::WatAST \
                 (:wat::core::first (:wat::core::ast->children (:wat::core::read-string \"(:wat::core::map x)\"))) \"head\")) \
             {key}) \
           \"field\")"
    ))
}

#[test]
fn c01_ast_span_head_line() {
    assert_eq!(head_span_field(":line"), Ok(1), "head keyword line should be 1");
}

#[test]
fn c02_ast_span_head_col() {
    assert_eq!(head_span_field(":col"), Ok(2), "head keyword col should be 2 (just after `(`)");
}

#[test]
fn c03_ast_span_symbol_col() {
    // The second child (symbol `x`) starts at col 18.
    let got = eval_i64(
        "(:wat::core::Option/expect -> :wat::core::i64 \
           (:wat::core::HashMap/get \
             (:wat::core::ast-span \
               (:wat::core::Option/expect -> :wat::WatAST \
                 (:wat::core::first (:wat::core::rest (:wat::core::ast->children (:wat::core::read-string \"(:wat::core::map x)\")))) \"x\")) \
             :col) \
           \"field\")",
    );
    assert_eq!(got, Ok(18), "symbol x col should be 18");
}
