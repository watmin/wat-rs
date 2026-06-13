//! Arc 209 C0b.2a — `listener'` host is load-bearing.
//!
//! C0b.2a made the host arg a real dispatch key: a non-thread host was a clean CHECK ERROR naming
//! the tier gap (the process connection tier was unbuilt). C0b.2c BUILT the process tier — so a
//! `(process)` host is now VALID, not an error.
//!
//! C0b.2d SUPERSEDED the mint-and-return form: `(listener' (process) :S :R)` (3-arg, mints a name)
//! is retired in favour of `(listener' (process) addr)` (2-arg, binds a given `SocketAddress'`).
//! Test 1 (below) is updated in C0b.2d to assert the new truth: `(listener' (process) addr)` with
//! `socket-address'` type-checks. The round-trip gate is probe_arc209_c0b2c_process_connection.
//!
//! Run: cargo test --release -p wat --test nursery probe_arc209_c0b2a_listener_host_thread_only -- --test-threads=1

use std::sync::Arc;
use wat::load::InMemoryLoader;

const PROCESS_LISTENER: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [addr     (:wat::kernel::socket-address' "wat.arc209.c0b2a.svc" :wat::core::i64 :wat::core::i64)
     listener (:wat::kernel::listener' (:wat::spawn::process) addr)]
    nil))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

/// C0b.2d updated the process connection tier to bind a named `SocketAddress'` —
/// `(listener' (process) addr)` with `socket-address'` must type-check. The old 3-arg
/// mint form is retired (C0b.2d supersedes C0b.2c). The round-trip gate is probe_arc209_c0b2c.
#[test]
fn listener_with_process_host_now_type_checks() {
    let result = wat::freeze::startup_from_source(PROCESS_LISTENER, None, Arc::new(InMemoryLoader::new()));
    assert!(result.is_ok(),
        "(listener' (process) addr) with socket-address' must type-check after C0b.2d. got: {:?}",
        result.err());
}

/// The thread form stays valid — C0b.2a constrains, it does not break the thread tier.
#[test]
fn listener_with_thread_host_still_type_checks() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)]
    nil))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;
    let result = wat::freeze::startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "(listener' (thread) …) must stay valid — C0b.2a constrains the host, it does not break \
         the thread tier. got: {:?}",
        result.err()
    );
}
