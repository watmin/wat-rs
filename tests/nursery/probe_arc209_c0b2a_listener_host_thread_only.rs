//! Arc 209 C0b.2a — `listener'` host must be load-bearing (thread-tier only, for now).
//!
//! THE HONESTY GAP: `infer_listener_prime` (check.rs:9864) infers the host arg "for error
//! coverage; type NOT constrained", and `eval_listener_prime` ignores it and always builds a
//! crossbeam rendezvous. So `(listener' (process) :S :R)` type-checks and SILENTLY degrades to a
//! thread rendezvous — a surface claiming a parity it does not have (the process connection tier
//! is C0b.2b/2c, unbuilt).
//!
//! THE FIX (C0b.2a): constrain `listener'`'s host to `(:wat::spawn::thread)`. A `(process)` (or any
//! non-thread) host is a clean CHECK ERROR naming the gap — not a silent thread. When C0b.2c builds
//! the process tier, the host widens to dispatch for real.
//!
//! RED at HEAD: `(listener' (process) …)` type-checks (startup succeeds) — the silent degrade.
//! GREEN once C0b.2a makes it a check error. The thread form `(listener' (thread) …)` stays valid
//! (the c0b1/c0b1b probes are the positive gate).
//!
//! Run: cargo test --release -p wat --test nursery probe_arc209_c0b2a_listener_host_thread_only -- --test-threads=1

use std::sync::Arc;
use wat::load::InMemoryLoader;

const PROCESS_LISTENER: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)]
    nil))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn listener_with_process_host_is_a_check_error_not_a_silent_thread() {
    let result = wat::freeze::startup_from_source(PROCESS_LISTENER, None, Arc::new(InMemoryLoader::new()));
    let err = match result {
        Ok(_) => panic!(
            "expected (listener' (process) …) to be a CHECK ERROR — the process connection tier is \
             unbuilt (C0b.2b/2c); accepting it silently degrades to a thread rendezvous (the gap)."
        ),
        Err(e) => format!("{e:?}"),
    };
    // Confirm it's the host-constraint error (names the tier gap), not an incidental failure.
    assert!(
        err.contains("thread") || err.contains("process") || err.contains("host"),
        "the check error should name the thread-only host constraint / process-tier gap. got: {err}"
    );
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
