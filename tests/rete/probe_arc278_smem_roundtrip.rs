//! Arc 278 stone S-mem.gate — the functional proof that the baked `:wat::query::MemStore`
//! (`wat/query/mem.wat`, a real `:wat::service::defservice`-backed `Store`/`ReadStore` satisfier)
//! round-trips put -> scan -> keyset-paginate -> scan-index against the REAL backend.
//!
//! S0's `probe_arc278_query_contract.wat` used an in-file stub satisfier returning empty pages —
//! it proved the contract's surfaces/records/dispatch are real. This gate proves the REAL MemStore
//! actor (spawned via `start` + `connect'`) actually stores and serves data: a 5-row table on one
//! pk, keyset-paginated 2/2/1 across three scans, plus a scan-index over a projected GSI.
//!
//! Run: `cargo nextest run --release -E 'test(smem_roundtrip)'`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn smem_roundtrip() {
    let world = startup_beside(file!())
        .expect("startup should succeed (:wat::query::MemStore must load from the baked stdlib)");
    let ast = wat::parse_one!("(:user::smem_roundtrip)").expect("parse test-fn call");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_ok(),
        "smem_roundtrip deftest' must pass (real MemStore put/scan/scan-index round-trip); got Err: {result:?}"
    );
}
