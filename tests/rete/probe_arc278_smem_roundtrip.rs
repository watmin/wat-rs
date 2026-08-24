//! Arc 278 stone S-mem.gate — the functional proof that the baked `:wat::query::mem-store'`
//! (`wat/query/mem.wat`, a real `:wat::service::defservice :satisfies :wat::query::Store`
//! satisfier on the services-as-surfaces operation model) round-trips put -> scan ->
//! keyset-paginate -> scan-index against the REAL backend. A dialed `mem-store'` peer IS the
//! Store, intrinsically (arc 293 Path B) — no wrapper struct.
//!
//! This gate proves the REAL mem-store' actor (spawned via `start` + `connect'`) actually stores
//! and serves data: a 5-row table on one pk, keyset-paginated 2/2/1 across three scans, plus a
//! scan-index over a projected GSI.
//!
//! Run: `cargo nextest run --release -E 'test(smem_roundtrip)'`

use wat::freeze::call_beside;

#[test]
fn five_rows() {
    call_beside(file!(), ":user::five-rows").expect_passed("five-rows helper must yield five StoredRow values");
}

#[test]
fn start_connect() {
    call_beside(file!(), ":user::start-connect").expect_passed("layer 0: mem-store' start+connect");
}

#[test]
fn ensure_schema() {
    call_beside(file!(), ":user::ensure-schema").expect_passed("layer 1: ensure-schema on a live mem-store'");
}

#[test]
fn put_five() {
    call_beside(file!(), ":user::put-five").expect_passed("layer 2: put five rows");
}

#[test]
fn scan_page1() {
    call_beside(file!(), ":user::scan-page1").expect_passed("layer 3: one scan page (2 rows, cursor Some(b))");
}

#[test]
fn scan_page2() {
    call_beside(file!(), ":user::scan-page2").expect_passed("layer 3b: second scan page (2 rows, cursor Some(d))");
}

#[test]
fn scan_page3() {
    call_beside(file!(), ":user::scan-page3").expect_passed("layer 3c: third scan page (1 row, cursor None)");
}

#[test]
fn scan_index() {
    call_beside(file!(), ":user::scan-index").expect_passed("layer 4: scan-index over GSI by-v");
}

#[test]
fn smem_roundtrip() {
    // Arc 278 the vacuous-gate wall — was `call_beside(..).is_ok()`, which certified only
    // that the fixture froze and ran; every assert-eq inside it was decoration.
    call_beside(file!(), ":user::smem_roundtrip").expect_passed(
        "smem_roundtrip deftest must pass (real mem-store' put/scan/scan-index round-trip)",
    );
}
