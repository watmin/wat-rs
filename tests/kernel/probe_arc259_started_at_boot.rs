//! Arc 259 — the timing correction: `wat.started-at` is the BOOT instant, not the seam.
//!
//! `259.0c` shipped `started-at = peer-started-at = now` — a placeholder LIE that
//! collapses the measurement. The corrected model:
//!   - `wat.started-at`      = captured at the EARLIEST point (wat-cli's boot),
//!     held in a pid-keyed process-global that re-captures across a fork (so a
//!     `:process` peer measures its OWN boot, never the parent's stale value).
//!   - `wat.peer-started-at` = `now` at the seam (this frame's entry).
//! Their gap is the real boot→entry latency — and a program reads it with the
//! Duration readout family (`(seconds (- peer-started-at started-at))`).
//!
//! RED at HEAD: there is no boot clock — `set_process_boot_instant` does not exist,
//! and the seam stamps `now` for started-at (ignoring any earlier capture). This
//! standalone test isolates the compile/behavior gap from the nursery binary.
//!
//! Run: `cargo test --release -p wat --test probe_arc259_started_at_boot`

use std::sync::Arc;
use chrono::{TimeZone, Utc};
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;

/// The seam reads the PRIMED boot clock for started-at — not a fresh `now`.
/// Inject a known-old boot (epoch 1000s) for this process; `:user::main` asserts
/// `wat.started-at` is epoch 1000 (the primed value), proving the seam reads the
/// boot global rather than stamping `now`.
#[test]
fn started_at_is_the_primed_boot_not_the_seam() {
    wat::time::set_process_boot_instant(Utc.timestamp_opt(1000, 0).unwrap());
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::test::assert-eq<:wat::core::i64> \
                     (:wat::time::epoch-seconds \
                       (:wat::program::Env/wat.started-at (:wat::program::env))) \
                     1000) \
                   nil))";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "wat.started-at must be the primed boot (epoch 1000), not the seam's now"
    );
}

/// The two stamps are DISTINCT and ordered: with the boot primed in the past,
/// `peer-started-at` (the seam's now) is strictly after `started-at` — the gap is
/// positive, read out as whole seconds via the Duration readout family. RED at
/// HEAD where both are `now` → gap is 0 → `(> gap 0)` is false.
#[test]
fn peer_started_at_is_after_started_at() {
    wat::time::set_process_boot_instant(Utc.timestamp_opt(1000, 0).unwrap());
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::test::assert-true \
                     (:wat::core::> \
                       (:wat::time::seconds \
                         (:wat::time::- \
                           (:wat::program::Env/wat.peer-started-at (:wat::program::env)) \
                           (:wat::program::Env/wat.started-at (:wat::program::env)))) \
                       0)) \
                   nil))";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "peer-started-at must be strictly after the (past-primed) started-at"
    );
}

/// The boot clock is pid-stable: two reads in the same process return the same
/// instant (the lazy capture happens once; it is not a fresh `now` each call).
#[test]
fn process_boot_instant_is_stable_within_a_process() {
    let a = wat::time::process_boot_instant();
    let b = wat::time::process_boot_instant();
    assert_eq!(a, b, "process_boot_instant is stable within one process (pid-keyed)");
}
