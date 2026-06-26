//! FM 2-bis probe — arc 251 Stone 251.5a-iv: `with-children`, the kind-preserving REBUILD.
//!
//! `(:wat::core::with-children <template> <children>)` rebuilds an AST node of the
//! SAME KIND as `template`, carrying `children` (a `Vector<:wat::WatAST>`, as
//! `ast->children` produces) as its new children. It is the inverse of `ast->children`
//! GIVEN the node you decomposed — and because `ast->children` is lossy on kind
//! (List/Vector/Set all collapse to a flat Vec), the template is what restores the kind.
//!
//! This is the tendon that makes a recursive `fix-source` written IN WAT *faithful*:
//! the walk is `(with-children node (map fix (ast->children node)))`, and a Vector
//! stays a Vector, a List stays a List. A List-only rebuild would silently turn a
//! binder `[x :- T]` into a call `(x :- T)` — the corruption this contract forbids.
//!
//! C01: KIND PRESERVATION — a Vector node, decomposed and rebuilt, is NOT a List.
//!      (A List-only rebuild fails here; this is the whole reason for the design.)
//! C02: a List node, decomposed and rebuilt, IS still a List.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_with_children`

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
fn contract_01_kind_preserved_vector_stays_non_list() {
    // read-string "[a b]" → List wrapping ONE top-level form: a Vector. first → the
    // Vector node. with-children(vec, children-of-vec) must rebuild a VECTOR — so it
    // is NOT a List. A List-only rebuild would make List? true and fail this contract.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::not
                 (:wat::core::List?
                   (:wat::core::with-children
                     (:wat::core::first
                       (:wat::core::ast->children
                         (:wat::core::read-string "[a b]")))
                     (:wat::core::ast->children
                       (:wat::core::first
                         (:wat::core::ast->children
                           (:wat::core::read-string "[a b]")))))))"#
        ),
        Ok(true),
        "a Vector node, decomposed and rebuilt via with-children, stays a Vector (not a List)"
    );
}

#[test]
fn contract_02_list_stays_list() {
    // read-string "(a b)" → List wrapping ONE form: a List. first → the List node.
    // with-children(list, children-of-list) rebuilds a List → List? is true.
    assert_eq!(
        eval_bool(
            r#"(:wat::core::List?
                 (:wat::core::with-children
                   (:wat::core::first
                     (:wat::core::ast->children
                       (:wat::core::read-string "(a b)")))
                   (:wat::core::ast->children
                     (:wat::core::first
                       (:wat::core::ast->children
                         (:wat::core::read-string "(a b)"))))))"#
        ),
        Ok(true),
        "a List node, decomposed and rebuilt via with-children, stays a List"
    );
}
