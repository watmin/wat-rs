//! Arc 259 S2d — the surface cut: `spawn-thread'` / `spawn-process'` / `close'` are
//! INTERNAL-only (the converged model: the ONLY user concurrency entry points are
//! `spawn-program` and `brackets`; `spawn-thread'`/`spawn-process'` are called only
//! by the `spawn-program'` defclause; `close'` is replaced by RAII Drop).
//!
//! Enforced by the `:restricted-to` caller-prefix whitelist (arc 198): these verbs
//! are restricted to `:wat::kernel::` callers, so a `:user::` call is a CHECK error
//! (the `walk_for_restricted_call` walker fires), not a runtime surprise.
//!
//! RED at HEAD: no restriction yet → a `:user::` fn calling `spawn-thread'` /
//! `close'` type-checks fine (startup succeeds). Post-S2d it is rejected at check.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2d_internal_only`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// A `:user::` fn calling the internal `spawn-thread'` must be a CHECK error
/// (restricted-to `:wat::kernel::`). RED at HEAD where it is allowed.
#[test]
fn user_calling_spawn_thread_prime_is_a_check_error() {
    // An otherwise-valid 2-arg spawn-thread' call from :user:: — the ONLY thing
    // wrong is the caller (a user, not the kernel defclause).
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::kernel::spawn-thread' \
                     (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil nil) \
                     (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv))) \
                   nil)) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "a :user:: caller of the internal spawn-thread' must be a check error (restricted-to :wat::kernel::)"
    );
}

/// A `:user::` fn calling the internal `close'` must be a CHECK error — teardown is
/// RAII, the user never holds the rope. RED at HEAD where close' is user-callable.
#[test]
fn user_calling_close_prime_is_a_check_error() {
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
                 (:wat::core::let \
                   [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                             (:wat::kernel::send' self (:wat::kernel::recv' self)))) \
                    _ (:wat::kernel::close' peer)] \
                   0)) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "a :user:: caller of the internal close' must be a check error (teardown is RAII)"
    );
}
