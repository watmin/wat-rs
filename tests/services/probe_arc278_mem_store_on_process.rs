//! THE MISSING GUARD (arc 278) — a RESERVED-namespace first-party service on a PROCESS locus.
//!
//! `:wat::query::mem-store'` (a `:satisfies :wat::query::Store` service, baked into the binary)
//! round-trips put -> scan on a FORKED `(process)`, through the SAME client face it uses on a
//! `(thread)`. The only delta from `tests/rete/probe_arc278_smem_roundtrip.wat` (thread, GREEN)
//! is `(:wat::spawn::process)`.
//!
//! WHY THIS TEST HAS TO EXIST: every prior process-locus service test uses a USER namespace
//! (`:my::`/`:probe::`), which structurally cannot trip the reserved-prefix gate. So a
//! reserved-`:wat::` service crossing a fork was never exercised — and when the arc-294 kwargs
//! flip made every aggregate emit a companion `defmacro`, the forked child's re-declaration of
//! the baked `:wat::query::Store` messages started tripping the reserved-prefix gate
//! (`#wat.macro/ReservedPrefix`), breaking process loci for ALL first-party stdlib services
//! (`mem-store'`/`sqlite-store'`/`journal'`) — silently, because no test covered the case.
//!
//! ROOT: the reserved-prefix gate is checked BEFORE the Arc-054 idempotent-redeclaration no-op
//! in both registries (`macros/registry.rs`, `types.rs`). The child re-bakes the full stdlib, so
//! its re-declaration of those forms is byte-identical and should be a no-op — but the gate fires
//! first. FIX: reorder idempotent-before-gate.
//!
//! RED at HEAD: `#wat.macro/ReservedPrefix` StartupError from the forked child.
//! GREEN after the reorder: the child's benign re-declaration is a no-op; put/scan round-trips.
//!
//! This test FORKS (`spawn-program' (process)`).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn mem_store_reserved_ns_service_round_trips_on_a_process_locus() {
    let world = startup_beside(file!())
        .expect("startup should succeed (mem-store' baked; the child re-bakes the same stdlib)");
    // Look up + apply the co-located `:user::compute` directly (no inline wat string — keeps this
    // off the no_inlined_wat lint; the actual round-trip logic lives in the sibling .wat).
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("mem-store' on a process locus raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(2)),
        "expected scan to read back 2 rows put through a RESERVED-ns (:wat::query::) service on a \
         FORKED PROCESS — same client face as the thread tier. A #wat.macro/ReservedPrefix here means \
         the reserved-prefix gate is still blocking the child's benign re-declaration of baked stdlib \
         forms (the idempotent-before-gate reorder is missing or incomplete). got {got:?}"
    );
}
