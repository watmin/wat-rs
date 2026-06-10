//! FM 2-bis probe — arc 251 Stone 251.5a-iii: the AST↔walkable bridge.
//!
//! `(:wat::core::ast->children <ast>)` decomposes a `:wat::WatAST` node into a
//! `Vector<:wat::WatAST>` of its children — the SAME walkable shape `:wat::core::forms`
//! produces — so the existing `first`/`rest`/`map` collection vocab applies. This is
//! the tendon between the read/write spine and the fixer's will: without it, a
//! recursive transform written IN WAT can't walk a form (first/map reject `:wat::WatAST`).
//!
//! C01: `first` on `ast->children` of a read form yields the child form (walk works).
//! C02: RECURSION — `ast->children` of an `ast->children` result still walks (decompose
//!      a form, then decompose its child). This is what makes the role-inversion
//!      transform expressible in wat.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_ast_bridge`

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
fn contract_01_ast_children_is_walkable() {
    // read-string "((:a 1) (:b 2))" → List wrapping ONE top-level form (a list of two
    // lists). ast->children → [that form]; first → the form, a List. The walk works.
    // `first` returns Option<:wat::WatAST> (safe head) — unwrap with Option/expect,
    // exactly as the macro engine does (core.wat:211).
    assert_eq!(
        eval_bool(
            r#"(:wat::core::List?
                 (:wat::core::Option/expect -> :wat::WatAST
                   (:wat::core::first
                     (:wat::core::ast->children
                       (:wat::core::read-string "((:a 1) (:b 2))")))
                   "empty"))"#
        ),
        Ok(true),
        "ast->children yields a Vector the first/map vocab walks"
    );
}

#[test]
fn contract_02_recursion_works() {
    // Decompose the program-list → the top form; decompose THAT → its first child
    // `(:a 1)`, still a List. Recursion through ast->children is the transform's spine.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::List?
                 (:wat::core::Option/expect -> :wat::WatAST
                   (:wat::core::first
                     (:wat::core::ast->children
                       (:wat::core::Option/expect -> :wat::WatAST
                         (:wat::core::first
                           (:wat::core::ast->children
                             (:wat::core::read-string "((:a 1) (:b 2))")))
                         "L1")))
                   "L2"))"#
        ),
        Ok(true),
        "ast->children of an ast->children result still walks — recursion is expressible in wat"
    );
}
