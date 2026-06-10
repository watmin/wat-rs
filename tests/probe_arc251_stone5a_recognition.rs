//! FM 2-bis probe — arc 251 Stone 251.5a-v: node recognition + construction.
//!
//! The last bridge piece before `fix-source`. Four primitives let a wat transform
//! recognize a node's kind, read a Symbol/Keyword node's text verbatim, and construct
//! the inverted node:
//!   `ast-kind`     : :wat::WatAST -> String  (discriminant: "list"/"symbol"/"keyword"/…)
//!   `ast-name`     : :wat::WatAST -> String  (stored token text, verbatim)
//!   `symbol-node`  : String -> :wat::WatAST  (a bare Symbol node)
//!   `keyword-node` : String -> :wat::WatAST  (a Keyword node; arg must start with ':')
//!
//! Round-trip identities (the honesty contract): a Symbol/Keyword read by `ast-name`
//! and reconstructed by the matching constructor is the same node. The kind CHANGE
//! (keyword head -> symbol) is explicit in WHICH constructor the transform calls.
//!
//! C01: ast-name reads a bare Symbol verbatim ("<-").
//! C02: ast-kind discriminates a Keyword node ("keyword").
//! C03: symbol-node + ast-name compose (construct "wat.core/map", read it back).
//! C04: keyword-node + ast-name compose (construct ":-", read it back).
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_recognition`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_bool(body: &str) -> Result<bool, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::bool {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

#[test]
fn contract_01_ast_name_reads_symbol_verbatim() {
    // read-string "<-" -> List[Symbol "<-"]; first -> the Symbol; ast-name -> "<-".
    assert_eq!(
        eval_bool(
            r#"(:wat::core::=
                 (:wat::core::ast-name
                   (:wat::core::Option/expect -> :wat::WatAST
                     (:wat::core::first
                       (:wat::core::ast->children (:wat::core::read-string "<-")))
                     "sym"))
                 "<-")"#
        ),
        Ok(true),
        "ast-name reads a bare Symbol node's text verbatim"
    );
}

#[test]
fn contract_02_ast_kind_discriminates_keyword() {
    // read-string ":-" -> List[Keyword ":-"]; first -> the Keyword; ast-kind -> "keyword".
    assert_eq!(
        eval_bool(
            r#"(:wat::core::=
                 (:wat::core::ast-kind
                   (:wat::core::Option/expect -> :wat::WatAST
                     (:wat::core::first
                       (:wat::core::ast->children (:wat::core::read-string ":-")))
                     "kw"))
                 "keyword")"#
        ),
        Ok(true),
        "ast-kind discriminates a Keyword node"
    );
}

#[test]
fn contract_03_symbol_node_roundtrips() {
    // symbol-node "wat.core/map" -> a Symbol node; ast-name reads it back verbatim.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::=
                 (:wat::core::ast-name (:wat::core::symbol-node "wat.core/map"))
                 "wat.core/map")"#
        ),
        Ok(true),
        "symbol-node constructs a Symbol whose ast-name is the input string"
    );
}

#[test]
fn contract_04_keyword_node_roundtrips() {
    // keyword-node ":-" -> a Keyword node; ast-name reads it back verbatim (sigil kept).
    assert_eq!(
        eval_bool(
            r#"(:wat::core::=
                 (:wat::core::ast-name (:wat::core::keyword-node ":-"))
                 ":-")"#
        ),
        Ok(true),
        "keyword-node constructs a Keyword whose ast-name is the (':'-prefixed) input"
    );
}
