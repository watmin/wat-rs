//! Arc 170 Stone D1 — `:wat::kernel::run-threads` bracket macro,
//! single-factory tuple form, single-peer round-trip.
//!
//! What D1 proves: the bracket macro skeleton — spawn → peer pairing
//! → factory invocation → client-fn invocation → drain-and-join — for
//! the N=1 case. The factory writes against the user-facing
//! `ThreadPeer<I, O>` surface; the macro adapter converts spawn-thread's
//! raw `[Receiver<I>, Sender<O>]` channel pair into a server-side peer.
//!
//! What D1 does NOT prove (future stones):
//!   D2 — multi-factory heterogeneous tuples (the macro extends to N
//!        factories with N concrete `ThreadPeer<Iₖ, Oₖ>` instances).
//!   D3 — panic cascade semantics (factory panics propagate through
//!        drain-and-join Result; bracket return becomes
//!        `Result<R, ProcessGroupErr>`).
//!   Stone E — `run-processes` bracket macro (mirrors D family for
//!             forked OS processes).
//!
//! T1 — single-factory round-trip. The wat program defines an echo
//!      factory that takes `ThreadPeer<String, String>`, reads one
//!      line via `Thread/readln`, writes it back via `Thread/println`,
//!      and returns nil. The bracket call shape (D1) takes 4 args:
//!      `(:wat::kernel::run-threads server-rx-type server-tx-type factory client-fn)`
//!      — server-rx-type is the full `:Receiver<I>` keyword and
//!      server-tx-type is the full `:Sender<O>` keyword that the wrap-
//!      fn binders must take. Honest delta: wat tokenizes parametric
//!      types `<...>` atomically — `~` unquote does NOT splice INTO a
//!      `<>` bracket pair at expand time. Same constraint
//!      `:wat::test::run-hermetic-with-io` documented; the bracket
//!      caller spells the full channel-end type, the macro splices
//!      directly at the binder position.
//!      Client-fn signature is `:Fn(ThreadPeer<String,String>) -> :String`;
//!      its body writes "hello" via `Thread/println`, reads the echo via
//!      `Thread/readln`, returns the read String. The bracket's value is
//!      the client-fn's return; the test asserts equality with "hello".
//!
//! Bare-factory vs Tuple-wrapped (D1 picked bare): wat has no AST
//! destructuring at expand time, so extracting the single child of a
//! `(Tuple factory)` AST is impossible at the wat-level defmacro
//! layer. D2 extends to N factories via variadic positional collector
//! (`& (factories ...)`), not via Tuple-AST iteration — the future
//! shape is `(run-threads I O factory-A factory-B ... client-fn)`,
//! consistent with D1's bare-factory form at N=1.

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat::span::Span;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

// ─── Stone D1 T1. single-factory String round-trip ────────────────────

#[test]
fn run_threads_d1_single_factory_round_trips_string() {
    // Echo factory: reads one String from the peer, writes it back,
    // returns nil. Receives `ThreadPeer<String, String>` from the macro
    // adapter (server-side: reads input String, writes output String).
    //
    // Client fn: receives `ThreadPeer<String, String>` constructed by
    // the bracket from the parent end of the channels (parent writes
    // input String, reads output String). Writes "hello"; reads echo;
    // returns the echoed value.
    //
    // The bracket call: takes type-keyword args `:wat::core::String`
    // (I — what the factory reads) + `:wat::core::String` (O — what
    // the factory writes), the `(Tuple factory)` form (D1 single-
    // factory case), and the client-fn. Returns the client-fn's value.
    let src = r#"
        (:wat::core::defn :my::echo-factory
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [line (:wat::kernel::Thread/readln peer)
             _    (:wat::kernel::Thread/println peer line)]
            :wat::core::nil))

        (:wat::core::defn :my::echo-client
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::String
          (:wat::core::let
            [_     (:wat::kernel::Thread/println peer "hello")
             reply (:wat::kernel::Thread/readln peer)]
            reply))

        (:wat::core::defn :my::test::run-d1
          [] -> :wat::core::String
          (:wat::kernel::run-threads
            :rust::crossbeam_channel::Receiver<wat::core::String>
            :rust::crossbeam_channel::Sender<wat::core::String>
            :my::echo-factory
            :my::echo-client))
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":my::test::run-d1")
        .expect(":my::test::run-d1 defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        Span::unknown(),
    )
    .expect("run-threads bracket should return the client-fn value");
    match outcome {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "hello",
            "echo round-trip must return client's 'hello'; got {:?}",
            s
        ),
        other => panic!("expected Value::String(\"hello\"); got {:?}", other),
    }
}
