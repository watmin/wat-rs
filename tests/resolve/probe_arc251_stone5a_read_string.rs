//! FM 2-bis probe — arc 251 Stone 251.5a-i: the homoiconic `read`.
//!
//! `(:wat::core::read-string <source>)` parses wat SOURCE text into forms-as-data
//! (a `WatAST::List` of top-level forms), WITHOUT evaluating — the read side of the
//! wat-to-wat fixer. Distinct from `:wat::edn::read`, which runs the EDN parser and
//! REJECTS the pre-251.5 surface (`::`, `<>`, `Fn(…)->`). `read-string` runs wat's
//! OWN source parser, so it reads the corpus as it stands today.
//!
//! C01: read-string returns a forms-LIST the macro engine can walk (`List?` true).
//! C02: read-string reads the DIRTY surface (`:wat::core::Vector<…>`) that
//!      `edn::read` cannot — proving it is the source parser, not the EDN parser.
//!      This is the property the whole sweep depends on.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_read_string`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval `body` (declared `-> :wat::core::bool`); return the bool.
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
fn contract_01_read_string_returns_walkable_forms() {
    assert_eq!(
        eval_bool(r#"(:wat::core::List? (:wat::core::read-string "(:wat::core::i64::+ 1 2)"))"#),
        Ok(true),
        "read-string must return a forms-List the macro engine can walk (List? recognizes it)"
    );
}

#[test]
fn contract_02_read_string_reads_the_dirty_surface() {
    // A defn whose binder type is the legacy `Vector<…>` keyword — non-EDN, which
    // `edn::read` rejects at the lexer. read-string (source parser) handles it.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::List? (:wat::core::read-string
                 "(:wat::core::defn :f [x <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 0)"))"#
        ),
        Ok(true),
        "read-string must read the dirty pre-251.5 surface (Vector<…>) the EDN reader can't — \
         it is the source parser, the foundation the fixer reads its input through"
    );
}
