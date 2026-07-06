//! Arc 278 — the :wat::telemetry' records exist in core and splice the Scope surface (disconfirming probe).
//!
//! RED at HEAD: `wat/telemetry.wat` does not exist, so `:wat::telemetry'::Metric`/`Log` are undefined and
//! the probe fails to type-check. GREEN when the records ship as a core baked source (registered in
//! src/stdlib.rs), `Metric`/`Log` splice `:wat::telemetry'::Scope` (arc-293 splice), and the unified
//! aggregate ctor + accessors mint `Metric/namespace` (spliced) and `Metric/name` (own).
//!
//! ⛔ IGNORE-LEDGER(278-telemetry-records): un-ignore as the FINAL green step of the records strike.

use wat::freeze::startup_beside;

#[test]
fn telemetry_records_exist_and_splice_scope() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        ":wat::telemetry'::Metric/Log must exist in core and splice the Scope surface (Metric/namespace \
         is a spliced accessor, Metric/name an own one); got: {:?}",
        world.err()
    );
}
