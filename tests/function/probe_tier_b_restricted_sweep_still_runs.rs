//! ⭐ Fresh control that `check:restricted-call(ALL fns)` still RUNS over stdlib
//! bodies — the demand `probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs` filed
//! against itself when the quoted-mention stone inverted the witness.
//!
//! DESIGN: `docs/excursus/2026/08/001-sns-sqs/no-stdlib-body-names-a-user-declarable-name/DESIGN.md`
//!
//! The inverted witness freezes green because nothing remains for the walker to
//! fire on, and would stay green if `src/check.rs:750` were deleted. This file
//! is the other half of the pair that would not:
//!
//! - Sibling `the_restricted_call_sweep_still_walks_every_wat_body` pins the
//!   source of the ALL-fns walk. Emptying the closure, skipping stdlib, or
//!   deleting the phase removes the snippet.
//! - This file arms the boot census and asserts the `8c` phase recorded ≥1 hit
//!   during a real freeze. Deleting `:750` (the `phase(P_CHECK_RESTRICTED, …)`
//!   wrapper) leaves the report row at 0 hits.
//!
//! Together: *"the sweep ran and found nothing"* (hits ≥ 1, no
//! `DefRestrictedCallerNotAllowed`) vs *"the sweep did not run"* (hits = 0).
//!
//! ⚠ PER-TEST PROCESS ISOLATION IS LOAD-BEARING. `force_mode` is a process-global
//! `OnceLock`. nextest forks a process per test; a shared-process runner that
//! already resolved the census mode cannot be trusted and must not read as a pass.
//! Same contract as `tests/diagnostics/boot_census.rs`.

use wat::freeze::census::{force_mode, rendered_report, Mode};

#[test]
fn the_restricted_call_phase_records_a_hit_on_a_real_freeze() {
    assert!(
        force_mode(Mode::Phases),
        "the census mode was already resolved in this process, so this test did not measure \
         an armed boot. It requires per-test process isolation (nextest forks per test); \
         under a shared-process runner it cannot be trusted and must not be read as a pass."
    );

    // Cache-off so a future elision of check behind a cached snapshot cannot
    // disguise a missing 8c as "the cache answered." Derivation still runs
    // check_program; we want that path.
    std::env::set_var("WAT_BOOT_CACHE", "off");

    let world = wat::freeze::startup_bare().expect("bare stdlib world must freeze");
    let fns = world.symbols.functions_iter().count();
    assert!(
        fns > 1000,
        "only {fns} functions in the bare world — the instrument is broken or the stdlib \
         stopped loading, and a 0-hit 8c row would then be indistinguishable from a missing \
         sweep"
    );

    let report = rendered_report();
    let hits = eight_c_hits(&report).unwrap_or_else(|| {
        panic!("armed census report has no `8c   check:restricted-call(ALL fns)` row:\n{report}")
    });
    assert!(
        hits >= 1,
        "check:restricted-call(ALL fns) recorded 0 hits — the sweep at src/check.rs:750 \
         did not run. That is the world this control exists to make RED. report:\n{report}"
    );
}

fn eight_c_hits(report: &str) -> Option<u64> {
    for line in report.lines() {
        if line.contains("check:restricted-call(ALL fns)") {
            return line.split_whitespace().last()?.parse().ok();
        }
    }
    None
}
