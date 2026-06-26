//! `nth` bug fix: `nth` is typed `Vector<T>`-only (core.wat) but should be the generic get-or-raise
//! positional accessor across all indexed sequences — at minimum Vector AND PersistentVector.
//! RED at HEAD (nth rejects a PersistentVector arg at type-check); GREEN when nth is made generic.
//!
//! Design: `nth` = get-or-raise (bare element, raise on OOB); `get`/`first`/`second`/`third` = safe (Option).
//! Run: cargo test --release -p wat --test probe_nth_persistent_vector

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run(src: &str, call: &str) -> Result<Value, String> {
    let full = format!("{src}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&full, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|t| t.value_owned())
}

/// THE disconfirm — nth on a PersistentVector returns the element (bare). RED at HEAD: nth's `Vector<T>`
/// param rejects a PersistentVector at type-check.
#[test]
fn nth_on_persistent_vector_returns_element() {
    let src = "\
(:wat::core::defn :test::pv-nth [] -> :wat::core::i64\n\
  (:wat::core::nth\n\
    (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) 7)\n\
    0))";
    let r = run(src, "(:test::pv-nth)");
    assert!(matches!(r, Ok(Value::i64(7))), "nth on a PersistentVector must return the element 7; got {r:?}");
}

/// Regression guard — nth on a std Vector still returns the element (bare), unchanged.
#[test]
fn nth_on_vector_still_returns_element() {
    let src = "\
(:wat::core::defn :test::vec-nth [] -> :wat::core::i64\n\
  (:wat::core::nth (:wat::core::Vector :wat::core::i64 10 20 30) 1))";
    let r = run(src, "(:test::vec-nth)");
    assert!(matches!(r, Ok(Value::i64(20))), "nth on a Vector must still return 20; got {r:?}");
}
