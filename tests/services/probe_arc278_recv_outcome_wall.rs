//! Arc 278 — the recv'-outcome wall RED GATE (acceptance; reshaped from
//! probe_arc278_crash_split_measure). Asserts, all four paths (panic/rterr × thread/process):
//!   ADMIN  (Handle/handle) MATCHES `RecvOutcome::Lost cause` as a VALUE (never a raise) and
//!          reports that `(Failure/message cause)` carries the crash sentinel — the owner gets the reason.
//!   CLIENT (connected peer) MATCHES `RecvOutcome::Lost` (NEVER `::Closed` — the mute we killed)
//!          and reports its reason-free 500 does NOT carry the sentinel.
//! At HEAD (pre-reshape) recv' raised → no RecvOutcome to match → RED. GREEN once the reshape lands.
//!
//! EXACT DATA-EQUALITY (no `.contains`, no string-eq): each entrypoint returns a STRUCTURED
//! outcome record (`:probe::Outcome`), the .rs renders it to EDN and asserts data-equality against a
//! co-located `.edn` golden (captured via `UPDATE_EDN=1`, never hand-authored). The discriminant
//! (which RecvOutcome variant matched) and the deterministic sentinel-presence boolean are the data
//! asserted — the per-run-variable Failure location never enters the golden (it is checked in-wat,
//! its boolean RESULT asserted exactly). "wat stdio is edn — it's always data; assert the structure
//! exactly" (builder, 2026-07-22). arc 278 R55 `REVOLVTIONE, NVLLA LARVA`.
//!
//! Run: cargo nextest run --release -E 'test(recv_outcome_wall)'
//! Capture goldens: UPDATE_EDN=1 cargo nextest run --release -E 'test(recv_outcome_wall)'

use wat::freeze::call_beside_value;

/// Render the entrypoint's returned `Value` (a `:probe::Outcome` record — EDN data) and assert it is
/// DATA-equal to the co-located golden. `call_beside_value` MUST return `Ok` — the fixture matched the
/// `RecvOutcome` as a VALUE (a raise would unwind past the reader, the mask the wall kills).
fn assert_outcome(fn_name: &str, golden: &str) {
    let v = call_beside_value(file!(), fn_name).unwrap_or_else(|e| {
        panic!(
            "{fn_name}: recv' must MATCH RecvOutcome as a VALUE (never a raise, which would unwind \
             past the reader); got Err: {e:?}"
        )
    });
    let edn = ::wat_edn::write(&wat::edn::render::value_to_edn(&v));
    wat::assert_edn_matches_file!(edn, golden, fn_name);
}

// ── PANIC crash (assertion-failed!) ──────────────────────────────────────────────
#[test]
fn recv_outcome_wall_panic_thread_admin_carries() {
    assert_outcome(":user::boom-admin-thread", "recv_outcome_wall__panic_thread_admin.edn");
}
#[test]
fn recv_outcome_wall_panic_process_admin_carries() {
    assert_outcome(":user::boom-admin-process", "recv_outcome_wall__panic_process_admin.edn");
}
#[test]
fn recv_outcome_wall_panic_thread_client_reason_free() {
    assert_outcome(":user::boom-client-thread", "recv_outcome_wall__panic_thread_client.edn");
}
#[test]
fn recv_outcome_wall_panic_process_client_reason_free() {
    assert_outcome(":user::boom-client-process", "recv_outcome_wall__panic_process_client.edn");
}

// ── RUNTIME-ERROR crash (div-by-zero quot) ───────────────────────────────────────
#[test]
fn recv_outcome_wall_rterr_thread_admin_carries() {
    assert_outcome(":user::boomrt-admin-thread", "recv_outcome_wall__rterr_thread_admin.edn");
}
#[test]
fn recv_outcome_wall_rterr_process_admin_carries() {
    assert_outcome(":user::boomrt-admin-process", "recv_outcome_wall__rterr_process_admin.edn");
}
#[test]
fn recv_outcome_wall_rterr_thread_client_reason_free() {
    assert_outcome(":user::boomrt-client-thread", "recv_outcome_wall__rterr_thread_client.edn");
}
#[test]
fn recv_outcome_wall_rterr_process_client_reason_free() {
    assert_outcome(":user::boomrt-client-process", "recv_outcome_wall__rterr_process_client.edn");
}
