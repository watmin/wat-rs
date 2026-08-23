//! Arc 293.W.2d / arc 278 — the PEER RELATION, and the wall that is an OMISSION.
//!
//! 293.W.2d minted two peer heads on the shared-memory-or-not line — `ThreadSelfPeer'<S,R>`
//! (in-locus, ANY I/O) and `Peer'<S,R>` (wire-safe, PURE I/O only) — but never stated the
//! RELATION between them. The checker enumerated the pair by hand at ~7 sites instead
//! (`check.rs` 9835 / 10159 / 11091 / `poll'`-`select'`'s self / the 10176 error string): one
//! derivation implemented N times.
//!
//! `Peer'` is strictly stricter, so it satisfies every constraint a `ThreadSelfPeer'` position
//! imposes. `wat/spawn.wat` now states that once, as a derive edge, and these two tests pin
//! BOTH directions — because only one of them may ever hold.
//!
//! ★ WHY THE NEGATIVE IS THE LOAD-BEARING ONE. The safe edge is written down and a hundred
//! things would break if it vanished. The FORBIDDEN edge is *not written down* — and an
//! un-written rule is invisible: nothing fails the day someone adds it "for symmetry." The
//! `.wat.bad` gate is what converts that absence into something enforceable. Same discipline
//! that keeps `:wat::core::Value` a fixed point rather than an `any` (arc 278 R7 — the top type
//! is honest BECAUSE the second, looser rule was never added).
//!
//! Grounded: this cannot weaken the wire wall. `is_pure_type` (`check.rs` ~12979) refuses all
//! four peer heads BY NAME in an exhaustive match — "they are resources — they are not pure"
//! (builder, 2026-08-03) — and a subtype edge does not touch a head-keyed match. Only ADDRESSES
//! cross a boundary (293.W); a peer is dialled, never shipped.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

/// SAFE DIRECTION — a `Peer'` satisfies a `ThreadSelfPeer'` parameter.
///
/// Before the derive edge this file was a located `TypeMismatch` naming both heads; the
/// measurement is on the disk in
/// `docs/arc/2026/06/293-struct-record-symmetry/NOTE-peer-and-threadselfpeer-are-one-relation-never-stated.md`.
#[test]
fn peer_satisfies_a_thread_self_peer_parameter() {
    startup_from_file("tests/services/probe_arc293w_peer_derives_threadselfpeer.wat")
        .expect("a wire-safe Peer' must satisfy an in-locus ThreadSelfPeer' parameter");
}

/// ⛔ FORBIDDEN DIRECTION — a `ThreadSelfPeer'` must NEVER satisfy a `Peer'` parameter.
///
/// This is arc 293.W's mobility wall. If this test ever goes green, an in-locus peer holding
/// live crossbeam handles can be laundered into a wire-safe position — do NOT "fix" the fixture,
/// find the change that widened the edge and revert it.
#[test]
fn thread_self_peer_is_refused_where_a_peer_is_expected() {
    let err = startup_from_file("tests/services/probe_arc293w_peer_derives_threadselfpeer.wat.bad")
        .expect_err(
            "MOBILITY WALL: an in-locus ThreadSelfPeer' must NOT satisfy a wire-safe Peer' \
             parameter — if this now passes, the reverse derive edge was added and 293.W is gone",
        );
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // Pin the exact arm: the refusal must be the TYPE relation, not an incidental error
    // elsewhere in the fixture (a fixture that fails for the wrong reason is a green that
    // proves nothing — arc 278 R59).
    // STONE-defservice-emits-the-binder (arc 109) — same call site, re-rendered: the
    // checker stopped minting `Head<a,b>` (a spelling the reader now refuses) and emits
    // the surviving `(Head :- [args])` form instead.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, callee, .. }
            if callee == ":probe::takes-peer"
            && expected.contains("(:wat::kernel::Peer :- [")
            && got.contains("(:wat::kernel::ThreadSelfPeer :- ["));
}
