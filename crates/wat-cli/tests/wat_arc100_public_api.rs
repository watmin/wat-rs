//! Arc 100 — `wat_cli::run` + `wat_cli::Battery` public-API smoke.
//!
//! These tests don't invoke `wat_cli::run` itself (it parses argv and
//! exits, which doesn't compose cleanly with the test runner's argv).
//! They verify that:
//!
//! 1. The `Battery` type alias is reachable from downstream consumers.
//! 2. The `(register, wat_sources)` pair signatures every workspace
//!    `#[wat_dispatch]` extension exposes coerce cleanly into a
//!    `&[Battery]` slice — the shape a custom CLI binary's `main()`
//!    would build before passing to `run`.
//! 3. An empty slice (no extra batteries; substrate-only) is also
//!    valid.
//!
//! In other words: the documented "build your own CLI" snippet
//! type-checks against the actual extension crates in the workspace.

use wat_cli::Battery;

#[test]
fn battery_slice_with_workspace_extensions_type_checks() {
    let _batteries: &[Battery] = &[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register, wat_sqlite::wat_sources),
        (wat_lru::register, wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (
            wat_telemetry_sqlite::register,
            wat_telemetry_sqlite::wat_sources,
        ),
    ];
    // The slice exists and has the expected length. Calling `run`
    // here would parse argv from the test harness and exit; we
    // stop short of that on purpose.
    assert_eq!(_batteries.len(), 5);
}

#[test]
fn battery_slice_with_subset_type_checks() {
    // A "minimal interrogation CLI" would only need telemetry +
    // telemetry-sqlite, not lru. Verify the subset is also valid.
    let _batteries: &[Battery] = &[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (
            wat_telemetry_sqlite::register,
            wat_telemetry_sqlite::wat_sources,
        ),
    ];
    assert_eq!(_batteries.len(), 2);
}

#[test]
fn empty_battery_slice_is_valid() {
    // Substrate-only CLI: no extension batteries. Useful for
    // sandboxed wat scripts that only need :wat::core::* surfaces.
    let _batteries: &[Battery] = &[];
    assert_eq!(_batteries.len(), 0);
}
