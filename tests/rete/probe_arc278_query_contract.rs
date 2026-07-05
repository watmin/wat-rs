//! Arc 278 stone S0 — acceptance gate for `wat/query.wat` (the `:wat::query` backend-agnostic
//! storage CONTRACT: Store/ReadStore surfaces, the Error recovery-axis enum + Fault record, and
//! the plain records every satisfier speaks). DESIGN-store-contract.md; BRIEF-STONE-S0-query-
//! contract.md.
//!
//! The co-located fixture (`probe_arc278_query_contract.wat`) constructs a `StoredRow` +
//! `ScanRequest` + `Page`, defines a tiny in-file `MemStore` struct, `extend-type`s it to
//! `:wat::query::ReadStore`, and dispatches `scan` through it — proving the surfaces + records +
//! dispatch are real, not just declared.
//!
//! Run: `cargo nextest run --release -E 'test(query_contract)'`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn query_contract() {
    let world = startup_beside(file!())
        .expect("startup should succeed (:wat::query:: contract must load from the baked stdlib)");
    let ast = wat::parse_one!("(:user::query_contract)").expect("parse test-fn call");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_ok(),
        "query_contract deftest' must pass (surfaces + records + dispatch); got Err: {result:?}"
    );
}
