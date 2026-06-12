//! Arc 259 S2c-ii-a — the apply-loop PURGE (FM-2-bis disconfirming probe).
//!
//! The builder's ruling: "purging the heresy of misconfiguration — the true forms
//! remain." The apply-loop (`[I] -> O`, the platform owns the message loop) is the
//! heresy; the self-peer (`[self <- Peer'<S,R>] -> nil`, the worker owns its own
//! loop + channel) is the true form. S2c-ii-a annihilates the apply-loop branches
//! (`spawn_thread_peer`'s `is_self_peer_model` dispatch + apply-loop arm; the
//! `infer_thread_prog_type` apply-loop projection). After the purge, a thread prog
//! MUST be a self-peer prog; a legacy apply-loop prog is REJECTED at check.
//!
//! ## Why this is RED at HEAD
//!
//! At HEAD the `:thread` dual-mode still accepts an apply-loop prog `[i64]->i64`
//! (the S2a transitional branch), so startup succeeds. Post-purge, the
//! apply-loop projection is gone → the prog is rejected with a clear error
//! ("expected a self-peer prog `[Peer'<S,R>] -> nil`").
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2cii_a`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// An apply-loop prog `[i64]->i64` handed to `spawn-program' :thread` — the heresy.
/// Post-purge it must be REJECTED (only the self-peer form `[Peer'<S,R>]->nil`
/// is a valid thread prog).
#[test]
fn s2cii_a_apply_loop_prog_rejected() {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let [peer (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                   (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input))]
            nil))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!(
            "the apply-loop prog [i64]->i64 must be REJECTED post-purge; got Ok (still accepted)"
        ),
        Err(_) => { /* GREEN: the apply-loop is purged — the true forms remain */ }
    }
}
