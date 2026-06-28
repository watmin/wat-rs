//! `nth` bug fix: `nth` is typed `Vector<T>`-only (core.wat) but should be the generic get-or-raise
//! positional accessor across all indexed sequences — at minimum Vector AND PersistentVector.
//! RED at HEAD (nth rejects a PersistentVector arg at type-check); GREEN when nth is made generic.
//!
//! Design: `nth` = get-or-raise (bare element, raise on OOB); `get`/`first`/`second`/`third` = safe (Option).
//! Run: cargo test --release -p wat --test probe_nth_persistent_vector
//!
//! Wat source lives in the co-located fixture: probe_nth_persistent_vector.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(call: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|t| t.value_owned())
}

/// THE disconfirm — nth on a PersistentVector returns the element (bare). RED at HEAD: nth's `Vector<T>`
/// param rejects a PersistentVector at type-check.
#[test]
fn nth_on_persistent_vector_returns_element() {
    let r = run("(:test::pv-nth)");
    assert!(matches!(r, Ok(Value::i64(7))), "nth on a PersistentVector must return the element 7; got {r:?}");
}

/// Regression guard — nth on a std Vector still returns the element (bare), unchanged.
#[test]
fn nth_on_vector_still_returns_element() {
    let r = run("(:test::vec-nth)");
    assert!(matches!(r, Ok(Value::i64(20))), "nth on a Vector must still return 20; got {r:?}");
}
