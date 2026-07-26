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
fn smem_roundtrip() {
    // Arc 278 the vacuous-gate wall — was `call_beside(..).is_ok()`, which certified only
    // that the fixture froze and ran; every assert-eq inside it was decoration.
    call_beside(file!(), ":user::smem_roundtrip").expect_passed(
        "smem_roundtrip deftest must pass (real mem-store' put/scan/scan-index round-trip)",
    );
}
