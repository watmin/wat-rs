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
//! Run: `cargo test --release --test nursery probe_arc214_stone46aii_peer_verbs`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 2 (CHECK NEGATIVE): recv' return type projects O ──────────────────

/// A defn declaring `-> :wat::core::String` whose body recv's from an
/// i64-peer MUST fail at check — recv' : peer<I,O> -> O projects i64.
/// RED at HEAD (fresh var unifies with String).
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; recv'/send' type assertions unchanged.
#[test]
fn probe_2_recv_projects_o_wrong_use_rejected() {
    let src = r#"
        (:wat::core::defn :user::bad-recv [] -> :wat::core::String
          (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                                   (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                     (:wat::kernel::send' self (:wat::kernel::recv' self))))]
            (:wat::kernel::recv' peer)))
    "#;
    let _err = startup_err(src);
}

// ─── Probe 3 (CHECK NEGATIVE): send' payload must match I ────────────────────

/// Sending a String into an i64-peer MUST fail at check — send' : peer<I,O>, I.
/// RED at HEAD (fresh var accepts anything).
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; recv'/send' type assertions unchanged.
#[test]
fn probe_3_send_checks_i_wrong_payload_rejected() {
    let src = r#"
        (:wat::core::defn :user::bad-send [] -> :wat::core::nil
          (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                                   (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                     (:wat::kernel::send' self (:wat::kernel::recv' self))))
                            _ (:wat::kernel::send' peer "not-an-i64")]
            nil))
    "#;
    let _err = startup_err(src);
}
