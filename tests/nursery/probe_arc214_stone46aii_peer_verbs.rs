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
//!   - Probe 1: a RUNTIME round-trip — `(:user::compute)` is EVALUATED; at HEAD
//!     the verbs have no eval dispatch, so eval fails → RED. Post-stone → 42.
//!   - Probes 2/3: check-side NEGATIVES — recv' used as the wrong type and
//!     send' fed the wrong payload MUST fail at check once the projective
//!     inference exists. At HEAD the fresh vars let them pass → RED.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone46aii_peer_verbs`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 (LOAD-BEARING, RUNTIME): thread-tier full lifecycle ─────────────

/// spawn :thread echo → send' 42 → recv' → close' → return the echoed value.
/// At HEAD: check passes (fresh vars), but the verbs have NO eval dispatch —
/// eval of (:user::compute) errors → RED. Post-stone: 42, and the spawned
/// thread is joined by close' (no leak, no hang).
#[test]
fn probe_1_thread_round_trip_via_verbs() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [peer (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                   (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input))
                            _ (:wat::kernel::send' peer 42)
                            got (:wat::kernel::recv' peer)
                            _ (:wat::kernel::close' peer)]
            got))
    "#;
    assert_eq!(run_i64(src), 42, "thread peer echo round-trip via the prime verbs");
}

// ─── Probe 2 (CHECK NEGATIVE): recv' return type projects O ──────────────────

/// A defn declaring `-> :wat::core::String` whose body recv's from an
/// i64-peer MUST fail at check — recv' : peer<I,O> -> O projects i64.
/// RED at HEAD (fresh var unifies with String).
#[test]
fn probe_2_recv_projects_o_wrong_use_rejected() {
    let src = r#"
        (:wat::core::defn :user::bad-recv [] -> :wat::core::String
          (:wat::core::let [peer (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                   (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input))]
            (:wat::kernel::recv' peer)))
    "#;
    let _err = startup_err(src);
}

// ─── Probe 3 (CHECK NEGATIVE): send' payload must match I ────────────────────

/// Sending a String into an i64-peer MUST fail at check — send' : peer<I,O>, I.
/// RED at HEAD (fresh var accepts anything).
#[test]
fn probe_3_send_checks_i_wrong_payload_rejected() {
    let src = r#"
        (:wat::core::defn :user::bad-send [] -> :wat::core::nil
          (:wat::core::let [peer (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                   (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input))
                            _ (:wat::kernel::send' peer "not-an-i64")]
            nil))
    "#;
    let _err = startup_err(src);
}
