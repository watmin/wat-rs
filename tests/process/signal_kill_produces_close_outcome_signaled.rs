//! EVIDENCE for EXPECTATIONS-process-signal-p2-mint.md row 5: "Kill reaches the owner-side
//! observable" — the gate no handler can fake (SIGKILL is uncatchable). See the co-located
//! .wat fixture's header for the full mechanism and, importantly, why this reads the outcome
//! via `Process::wait()` in Rust rather than via wat's `close'`: `close'` is restricted to
//! `:wat::kernel::` callers AND (measured here) any non-stdlib source defining anything under
//! that prefix hits `ReservedPrefix` — there is currently no wat-level door into `close'` for a
//! test fixture at all. This file reads the SAME underlying mechanism `close'` itself calls,
//! one layer below the restricted verb, against the EXACT peer `:wat::kernel::signal` (the new
//! P2 code) just delivered `Kill` through.
//!
//! Invocation: cargo nextest run --release -p wat --test process signal_kill_produces_close_outcome_signaled
use wat::freeze::call_beside_value;
use wat::kernel::spawn::{ProcessPeerCell, ProcessSelectable, PROCESS_PEER_TYPE_PATH};
use wat::rust_deps::marshal::downcast_ref_opaque;
use wat::Value;

#[test]
fn signal_kill_produces_close_outcome_signaled() {
    let result = call_beside_value(file!(), ":user::compute")
        .expect(":user::compute must run to completion and return the signalled Process peer");

    let inner = match &result {
        Value::RustOpaque(inner) if inner.type_path == PROCESS_PEER_TYPE_PATH => inner,
        other => panic!("expected a Process<I,O> RustOpaque peer, got: {other:?}"),
    };
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        inner,
        PROCESS_PEER_TYPE_PATH,
        "signal_kill_produces_close_outcome_signaled",
        wat::rust_caller_span!(),
    )
    .expect("downcast the returned peer to ProcessPeerCell");

    let selectable = cell
        .with_mut(
            "signal_kill_produces_close_outcome_signaled",
            wat::rust_caller_span!(),
            |opt_bundle| opt_bundle.take(),
        )
        .expect("thread-owned cell access")
        .expect("peer must not already be closed");

    let status = match selectable {
        ProcessSelectable::Spawned(bundle) => bundle.peer.wait().expect("wait succeeds"),
        ProcessSelectable::Timer(_) => panic!("expected a Spawned process peer, got a Timer"),
    };

    assert_eq!(
        status,
        wat::process::ExitStatus::Signaled(libc::SIGKILL),
        "expected the child to be terminated BY SIGKILL after :wat::kernel::signal Kill (the \
         mechanism close' itself would report as CloseOutcome::Signaled); got {status:?}"
    );
}
