//! Arc 293.W.2d — Peer-type purity: compile-time wire-wall via `Peer'<I,O>` well-formedness.
//!
//! ## RED at HEAD / GREEN after 2d
//!
//! After 2d, `Peer'<I,O>` requires `I,O` to be `:Pure` (wire-safe) by construction —
//! the producers (`:wat::program::self-peer`, `connect'`, `accept'`, `socket-pair'`) reject impure type
//! args at CHECK. `ThreadSelfPeer'<I,O>` (any I/O, in-locus) is the escape hatch.
//!
//! ## RED state (HEAD before 2d)
//!
//! Creating a `Peer'` with an impure type arg (struct) via `:wat::program::self-peer` COMPILES today:
//! the 2c gate only covers `Process'`; bare `Peer'` producers have no purity check yet.
//! The probe's `Err` assertion FAILS at HEAD because the world loads without error.
//!
//! ## GREEN state (after 2d)
//!
//! The `:wat::program::self-peer` producer's purity check fires on the struct type arg → the world fails
//! to load with a type error naming the impure type. The probe's `Err` assertion passes.
//!
//! ## Fixtures
//!
//! - `probe_arc293_W2d_peer_purity.wat` (co-located, `startup_beside`): the failing case —
//!   `:wat::program::self-peer` with a struct type arg. World FAILS to load after 2d.
//! - `probe_arc293_W2d_positive.wat` (sibling): positive cases — `ThreadSelfPeer'` carrying
//!   impure I/O type-checks (in-locus); `:wat::program::self-peer` with pure types still type-checks.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};

// ─── Main probe (compile-time rejection) ──────────────────────────────────────

/// `:wat::program::self-peer` with a struct type arg produces `Peer'<struct,i64>` — MUST fail at CHECK.
///
/// RED at HEAD: `:wat::program::self-peer :S :i64` creates `Peer'<S,i64>` without a purity check
/// on the type args → `startup_beside` returns `Ok`. The `Err` assertion FAILS.
///
/// GREEN after 2d: the `:wat::program::self-peer` producer's `is_pure_type` check fires on S
/// (a struct, `Nature::Struct` → impure) → `startup_beside` returns `Err`.
#[test]
fn impure_type_arg_on_wire_peer_is_check_error() {
    let result = startup_beside(file!());
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::program::self-peer"
            && reason == "a comm carries only pure data — type :w2d::S is not \
                pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. \
                Redesign I/O as records, scalars, or pure enums \
                (no Sender, Receiver, or handle fields)."
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

/// 255.30 — the pure self-peer still loads. The impure thread peer is the
/// struct row, and it is refused.
#[test]
fn pure_wire_peer_type_checks_and_a_struct_on_a_thread_peer_is_refused() {
    let pure = startup_from_file("tests/comms/probe_arc293_W2d_positive.wat");
    assert!(pure.is_ok(), "a pure self-peer must still load: {:?}", pure.err());
    let impure = startup_from_file("tests/kernel/probe_arc255_30_struct_on_thread_peer.wat.bad");
    wat::assert_startup_error!(impure, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "a comm carries only pure data — type :p30::S is not \
                pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. \
                Redesign I/O as records, scalars, or pure enums \
                (no Sender, Receiver, or handle fields)."
    );
}
