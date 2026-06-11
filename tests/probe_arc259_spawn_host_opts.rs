//! Arc 259 (The Forced Hand) — spawn host opts (the Keymaker). Stone 1.
//!
//! `wat/spawn.wat` mints the host opts that the (coming) clause-based
//! `spawn-program` will dispatch on. Only the BUILT hosting-doors get keys:
//! `:wat::spawn::ThreadOpts` / `ProcessOpts` + the ergonomic constructors
//! `(thread)` / `(process)`. The `:remote` door is **perpetually awaiting its
//! key** (the forcing function) — its opts shape is deliberately NOT defined, so
//! there is nothing to test here yet; its key is cut when its lock is specified.
//! Purely additive — does not yet touch the live `spawn-program'`.
//!
//! Run: `cargo test --release --test probe_arc259_spawn_host_opts`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn c01_thread_and_process_keys_cut() {
    // (thread) and (process) — the two built hosting-doors — type-check + construct.
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::spawn::thread) \
                   (:wat::spawn::process) \
                   nil))";
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "the Keymaker cuts the thread + process keys; got {:?}",
        result.err()
    );
}
