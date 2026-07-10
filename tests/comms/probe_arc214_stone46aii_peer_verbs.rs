//! Arc 214 Stone 4.6a-ii — the four peer verbs (FM-2-bis disconfirming probe).
//!
//! Over the 4.6a-i typed-peer foundation: `send'` / `recv'` / `try-recv'` as
//! projective intrinsics (the element types flow from the peer's
//! `Parametric{Thread'|Process', [I,O]}`) + `close'` dispatching on the peer head.
//!
//! ## Probe shape (vacuity-aware — the 4.6a-i lesson)
//!
//! Unknown verb heads infer fresh vars, so check-side POSITIVES are vacuous.
//! The discriminators here are:
//!   - Probes 2/3: check-side NEGATIVES — recv' used as the wrong type and
//!     send' fed the wrong payload MUST fail at check once the projective
//!     inference exists. At HEAD the fresh vars let them pass → RED.
//!
//! ## Arc 259 S2c-ii-a — apply-loop PURGE
//!
//! p1 (probe_1_thread_round_trip_via_verbs) was a DUPLICATE of
//! probe_arc259_s2a's self-peer round-trip and is RETIRED. All spawn progs
//! are now self-peer `[self <- Peer'<S,R>] -> nil` (the only valid form).
//!
//! Run: `cargo test --release --test comms probe_arc214_stone46aii_peer_verbs`

use wat::freeze::startup_from_file;

fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure for {path}; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 2 (CHECK NEGATIVE): recv' return type projects O ──────────────────

/// A defn declaring `-> :wat::core::String` whose body recv's from an
/// i64-peer MUST fail at check — recv' : peer<I,O> -> O projects i64.
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; recv'/send' type assertions unchanged.
#[test]
fn probe_2_recv_projects_o_wrong_use_rejected() {
    let _err = startup_err(
        "tests/comms/probe_arc214_stone46aii_peer_verbs_probe2.wat.bad",
    );
}

// ─── Probe 3 (CHECK NEGATIVE): send' payload must match I ────────────────────

/// Sending a String into an i64-peer MUST fail at check — send' : peer<I,O>, I.
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; recv'/send' type assertions unchanged.
#[test]
fn probe_3_send_checks_i_wrong_payload_rejected() {
    let _err = startup_err(
        "tests/comms/probe_arc214_stone46aii_peer_verbs_probe3.wat.bad",
    );
}
