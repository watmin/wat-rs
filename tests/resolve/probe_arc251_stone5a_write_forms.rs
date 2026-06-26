//! FM 2-bis probe — arc 251 Stone 251.5a-ii: `write-forms`, the round-trip closed.
//!
//! `(:wat::core::write-forms <forms>)` serializes a forms-value (`wat__WatAST`) to
//! clean EDN text — the write side of the homoiconic round-trip, the inverse of
//! `read-string`. Together they are the wat-to-wat fixer's read→(transform)→write
//! cycle, all in wat's own primitives.
//!
//! C01: the WHOLE cycle, `::`-surface in / clean EDN out / re-read: `read-string`
//!      reads the `::`-spelled source → `write-forms` emits CLEAN EDN (`::`→`.`/`/`)
//!      → `read-string` re-reads THAT back → a List. The bird reads its own ashes
//!      and writes them clean. (The re-reader is `read-string`, not `edn::read`:
//!      a program contains SYMBOLS — `<-`, `->` — which are forms, not values;
//!      `edn::read` makes values and has no symbol value, `read-string` makes forms.)
//!
//! NOTE: write-forms only serializes EDN-EXPRESSIBLE forms. The `<>`/`Fn(…)->`
//! sugar is a single keyword `:wat::core::Vector<…>` whose `<` is not a legal EDN
//! keyword char — it cannot round-trip raw, which is precisely why the fixer's
//! TRANSFORM replaces it with `(wat.type/Vector …)` BEFORE write-forms ever sees it.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_write_forms`

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
fn contract_01_homoiconic_roundtrip_dirty_in_clean_out() {
    // read-string reads the `::`-spelled source → write-forms emits CLEAN EDN
    // (`::`→`.`/`/`) → read-string re-reads THAT back → a List.
    // The fixer's full read→write→read cycle, closed in wat's own primitives.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::List?
                 (:wat::core::read-string
                   (:wat::core::write-forms
                     (:wat::core::read-string
                       "(:wat::core::defn :f [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))"))))"#
        ),
        Ok(true),
        "read-string(::source) → write-forms(clean EDN) → read-string → a List: the round-trip closes in wat"
    );
}
