//! Arc 278 stone S2 gate — the DIFFERENTIAL: the sqlite `:wat::query::Store` satisfier
//! (`SqliteStore`, `wat/query/sqlite_store.wat`) held bit-for-bit against the S-mem MemStore
//! oracle (`wat/query/mem.wat`). One `run-ops` fn drives the same op sequence (ensure-schema with
//! a GSI -> put 5 rows -> keyset-paginate scan 2/2/1 -> scan-index) through the `Store` surface on
//! BOTH backends; the gate asserts the returned Pages are equal, bit-for-bit — this is R21
//! `EXPLORATA CAEDE NON VINCIMVR` turning `PROBATVM`.
//!
//! Run: `cargo nextest run --release -E 'test(sqlite_store_differential)'`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn sqlite_store_differential() {
    let world = startup_beside(file!()).expect(
        "startup should succeed (:wat::query::MemStore and :wat::sqlite'::SqliteStore must load \
         from the baked stdlib)",
    );
    let ast = wat::parse_one!("(:user::sqlite_store_differential)").expect("parse test-fn call");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_ok(),
        "sqlite_store_differential deftest' must pass (MemStore and SqliteStore must return \
         IDENTICAL Pages for the same op sequence); got Err: {result:?}"
    );
}
