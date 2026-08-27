//! Arc 293.W.2c — compile-time send' wire-wall.
//!
//! A typed struct sent to a `Process'` peer MUST be rejected by the type-checker
//! (the portability gate in `infer_send_prime`, `src/check.rs`). The world fails
//! to load (`startup_beside` returns `Err`) — the CHECK error fires before
//! execution, and no runtime guard is needed.
//!
//! This is the higher rung above arc 293.W.2a's runtime backstop. A typed
//! struct→process send was previously only caught at runtime; it is now
//! unrepresentable (the checker rejects the world before it can run).
//!
//! ## Gate scope
//!
//! The gate fires for `Process'` peers (serialized stdin/stdout) only.
//! `Thread'` peers are in-locus (same address space, crossbeam channel) and
//! are always exempt. `Peer'` typed values cover both connection peers
//! (from `connect'`, which are wire) and thread self-handles
//! (`[self <- Peer'<I,O>]` closures, which are in-locus); these cannot be
//! distinguished at the `send'` call site so `Peer'` is deferred until the
//! type system introduces a distinct `ConnPeer'` head.
//!
//! ## Fixtures
//!
//! - `probe_arc293_W2c_compile_time_send.wat` (co-located): the failing case —
//!   `send'` of a bare struct to a `Process'` peer. World FAILS to load.
//! - `probe_arc293_W2c_controls.wat` (sibling): the exempted cases — struct
//!   over Thread', record over Process'. Both must type-check (world loads).
//!
//! ## RED at HEAD (before 2c gate)
//!
//! With no portability gate in `infer_send_prime`, the typed struct→process send
//! type-checks without error — `startup_beside` returns `Ok`. The probe asserting
//! `Err` FAILs at HEAD. GREEN after the gate is added to `infer_send_prime`.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};

// ─── Main probe (compile-time rejection) ──────────────────────────────────────

/// A wire peer (`:wat::program::self-peer`) with a struct type arg MUST fail at CHECK.
///
/// Arc 293.W.2d: the purity wall moved from `send'` time (2c, deleted) to wire-peer
/// PRODUCER time. `:wat::program::self-peer<Struct,i64>` is a CHECK error because a struct is
/// impure (Nature::Struct) and the producer's purity gate fires (§7 purity wall).
///
/// GREEN after 2d: `startup_beside` returns `Err` with a check error mentioning
/// "pure", "struct", or "wire".
#[test]
fn struct_send_to_process_peer_is_check_error() {
    let result = startup_beside(file!());
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::program::self-peer"
            && reason == "a wire peer (Peer<I,O>) carries only pure data — type :w2c::S is not \
                pure (§7 purity wall). If this peer is used only within a thread (in-locus, \
                shared memory), use ThreadSelfPeer<I,O> — any I/O types are allowed in-locus. \
                If this peer must cross a process boundary (wire), redesign I/O types to use \
                records, scalars, or pure enums (no Sender/Receiver/handle fields)."
    );
    let err_str = format!("{}", result.unwrap_err());
    let lower = err_str.to_lowercase();
    assert!(
        lower.contains("pure") || lower.contains("portable") || lower.contains("struct") || lower.contains("wire"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "check error must mention pure/portability/struct/wire (§7 rejection by \
         wire-peer producer); got: {err_str}"
    );
}

// ─── Controls (must NOT be rejected) ─────────────────────────────────────────

/// Control: a struct `send'` to a THREAD peer must still type-check.
///
/// Thread peers are in-locus (crossbeam channel, same address space) — the 2c
/// gate must not fire for `Thread'`. The world in `probe_arc293_W2c_controls.wat`
/// must load without error.
#[test]
fn struct_send_to_thread_peer_still_type_checks() {
    let result = startup_from_file("tests/comms/probe_arc293_W2c_controls.wat");
    assert!(
        result.is_ok(),
        "struct send' to a Thread' peer MUST type-check (in-locus, no gate) — the \
         2c portability gate must not fire for Thread'. If this assertion fails, the \
         gate over-reaches into the thread tier. Error: {:?}",
        result.unwrap_err()
    );
}

/// Control: a record `send'` to a `Process'` peer must still type-check.
///
/// Records are wire-portable; the portability gate must pass them through. The
/// same controls world in `probe_arc293_W2c_controls.wat` contains this case.
#[test]
fn record_send_to_process_peer_still_type_checks() {
    // The controls world is already loaded by struct_send_to_thread_peer_still_type_checks
    // above; both controls live in the same fixture.
    let result = startup_from_file("tests/comms/probe_arc293_W2c_controls.wat");
    assert!(
        result.is_ok(),
        "record send' to a Process' peer MUST type-check (records are portable) — \
         the 2c portability gate must pass portable payload types. If this fails, \
         the gate over-rejects portable payloads. Error: {:?}",
        result.unwrap_err()
    );
}
