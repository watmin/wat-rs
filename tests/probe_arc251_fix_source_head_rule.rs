//! FM 2-bis probe — arc 251 fix-source head-rule: the head role-inversion in the WALK.
//!
//! `fix-source` (loaded from its home `wat/fix.wat`) now converts list-head `::`-keywords
//! into faithful-Clojure symbols, recursively, COMPOSED with the if-annotation strip. This
//! proves the two rules compose in one bottom-up pass and that the head-rule fires at every
//! depth while leaving non-`::` heads (data keywords, threading `->`) alone.
//!
//! C01: a bare call — `(:wat::core::map f xs)` → `(wat.core/map f xs)`.
//! C02: strip + head compose — `(:wat::core::if true -> :wat::core::i64 1 2)` →
//!      `(wat.core/if true 1 2)` (annotation gone AND head inverted).
//! C03: recursion — a nested call's head is inverted too.
//! C04: a data keyword head (`:else` — no `::`) is NOT converted.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_head_rule`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// fix-source loads as blessed stdlib (wat/fix.wat is listed in src/stdlib.rs), so it is
// available at `:wat::fix::*` after startup — no inline definition, no include_str!.

fn eval_string(body: &str) -> Result<String, String> {
    let src = format!(
        "(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST \
            (:wat::core::Option/expect -> :wat::WatAST \
              (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))) \"topform\"))\n\
         (:wat::core::defn :user::compute [] -> :wat::core::String {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

fn embed(payload: &str) -> String {
    payload.replace('\\', "\\\\").replace('"', "\\\"")
}

fn fix(src: &str) -> Result<String, String> {
    eval_string(&format!(
        "(:wat::core::write-forms (:wat::fix::fix-source (:user::topform \"{}\")))",
        embed(src)
    ))
}

#[test]
fn contract_01_bare_call_head_inverted() {
    assert_eq!(fix("(:wat::core::map f xs)"), Ok("(wat.core/map f xs)".into()));
}

#[test]
fn contract_02_strip_and_head_compose() {
    assert_eq!(
        fix("(:wat::core::if true -> :wat::core::i64 1 2)"),
        Ok("(wat.core/if true 1 2)".into()),
        "the annotation is stripped AND the if-head is inverted, in one pass"
    );
}

#[test]
fn contract_03_recurses_into_nested_heads() {
    assert_eq!(
        fix("(:wat::core::do (:wat::core::first xs))"),
        Ok("(wat.core/do (wat.core/first xs))".into()),
        "the nested call's head is inverted too"
    );
}

#[test]
fn contract_04_data_keyword_head_not_converted() {
    // `:else` has no `::` — it is data, not a call head; it must survive untouched.
    assert_eq!(fix("(:else 1)"), Ok("(:else 1)".into()));
}
