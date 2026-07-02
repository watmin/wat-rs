//! Arc 293.W.2d — Peer-type purity: compile-time wire-wall via `Peer'<I,O>` well-formedness.
//!
//! ## RED at HEAD / GREEN after 2d
//!
//! After 2d, `Peer'<I,O>` requires `I,O` to be `:Pure` (wire-safe) by construction —
//! the producers (`peer-pair'`, `connect'`, `accept'`, `socket-pair'`) reject impure type
//! args at CHECK. `ThreadSelfPeer'<I,O>` (any I/O, in-locus) is the escape hatch.
//!
//! ## RED state (HEAD before 2d)
//!
//! Creating a `Peer'` with an impure type arg (struct) via `peer-pair'` COMPILES today:
//! the 2c gate only covers `Process'`; bare `Peer'` producers have no purity check yet.
//! The probe's `Err` assertion FAILS at HEAD because the world loads without error.
//!
//! ## GREEN state (after 2d)
//!
//! The `peer-pair'` producer's purity check fires on the struct type arg → the world fails
//! to load with a type error naming the impure type. The probe's `Err` assertion passes.
//!
//! ## Fixtures
//!
//! - `probe_arc293_W2d_peer_purity.wat` (co-located, `startup_beside`): the failing case —
//!   `peer-pair'` with a struct type arg. World FAILS to load after 2d.
//! - `probe_arc293_W2d_positive.wat` (sibling): positive cases — `ThreadSelfPeer'` carrying
//!   impure I/O type-checks (in-locus); thread `make-channel` of impure type type-checks.

use wat::freeze::{startup_beside, startup_from_file};

// ─── Main probe (compile-time rejection) ──────────────────────────────────────

/// `peer-pair'` with a struct type arg produces `Peer'<struct,i64>` — MUST fail at CHECK.
///
/// RED at HEAD: `peer-pair' :S :i64` creates `Peer'<S,i64>` without a purity check
/// on the type args → `startup_beside` returns `Ok`. The `Err` assertion FAILS.
///
/// GREEN after 2d: the `peer-pair'` producer's `is_pure_type` check fires on S
/// (a struct, `Holder::Struct` → impure) → `startup_beside` returns `Err`.
#[test]
fn impure_type_arg_on_wire_peer_is_check_error() {
    let result = startup_beside(file!());
    assert!(
        result.is_err(),
        "peer-pair' with a struct type arg MUST fail at CHECK (arc 293.W.2d — \
         a wire Peer'<I,O> carries only pure data; the producer must reject impure type \
         args). If this assertion fails, the Peer'<I,O> well-formedness gate is missing \
         from the producer."
    );
    let err_str = format!("{}", result.unwrap_err());
    let lower = err_str.to_lowercase();
    assert!(
        lower.contains("pure") || lower.contains("portable") || lower.contains("struct") || lower.contains("wire"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "check error must mention purity, portable, struct, or wire (§7 rejection by \
         Peer'<I,O> well-formedness gate); got: {err_str}"
    );
}

// ─── Positive cases (must NOT be rejected) ────────────────────────────────────

/// Positive: `ThreadSelfPeer'` with impure I/O type-checks (in-locus, any I/O).
///
/// `ThreadSelfPeer'<Sender<i64>, i64>` is in-locus (crossbeam, same address space).
/// The purity constraint does NOT apply to `ThreadSelfPeer'`. The world must load.
///
/// Also asserts: a thread-tier `make-channel` of an impure payload type-checks
/// (thread channel exemption — thread-tier channels are in-process).
#[test]
fn thread_self_peer_and_make_channel_impure_type_checks() {
    let result = startup_from_file("tests/comms/probe_arc293_W2d_positive.wat");
    assert!(
        result.is_ok(),
        "ThreadSelfPeer'<Sender<i64>,i64> and make-channel of impure type MUST type-check \
         (in-locus; no purity constraint — arc 293.W.2d positive cases). \
         Error: {:?}",
        result.err()
    );
}
