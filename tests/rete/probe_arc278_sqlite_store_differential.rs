//! Arc 278 stone S4 gate — the DIFFERENTIAL: the sqlite `:wat::query::Store` satisfier
//! (`sqlite-store'`, `wat/query/sqlite-store.wat`) held bit-for-bit against the S-mem mem-store'
//! oracle (`wat/query/mem.wat`). Both are `:satisfies :wat::query::Store` services on the
//! operation model — a dialed peer IS the Store, intrinsically (arc 293 Path B); no wrapper
//! struct. One `run-ops` fn drives the same op sequence (ensure-schema with a GSI -> put 5 rows ->
//! keyset-paginate scan 2/2/1 -> scan-index) through the `Store` surface on BOTH backends; the
//! gate asserts the returned Pages are equal, bit-for-bit — this is R21
//! `EXPLORATA CAEDE NON VINCIMVR` turning `PROBATVM`.
//!
//! Run: `cargo nextest run --release -E 'test(sqlite_store_differential)'`

use wat::freeze::call_beside;

#[test]
fn run_ops_on_mem_store() {
    call_beside(file!(), ":user::run-ops-on-mem-store").expect_passed(
        "run-ops against mem-store' alone must match the S-mem gate shape (page1=2, page3 cursor None, ipage=2)",
    );
}

#[test]
fn sqlite_store_differential() {
    // Arc 278 the vacuous-gate wall — was `call_beside(..).is_ok()`, which certified only
    // that the fixture froze and ran; every assert-eq inside it was decoration.
    call_beside(file!(), ":user::sqlite_store_differential").expect_passed(
        "sqlite_store_differential deftest must pass (mem-store' and sqlite-store' must return \
         IDENTICAL Pages for the same op sequence)",
    );
}
