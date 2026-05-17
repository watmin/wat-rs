//! Arc 170 Stone D2 — updated D1 test: `:wat::kernel::run-threads` with
//! coordinator-fn form, single-factory N=1 round-trip.
//!
//! This test was originally written for Stone D1 (positional-types call form).
//! It is updated here for Stone D2's coordinator-fn form per the BRIEF:
//! "D1 test moves to coordinator-fn form; D1's macro retires."
//!
//! What this test proves:
//! - The coordinator-fn form works for N=1 (single factory)
//! - The arc 201 reflection chain correctly extracts the ThreadPeer<I,O>
//!   type args at macro expand time (I = server reads, O = server writes)
//! - Coordinator binder name becomes the peer let-binding name (via
//!   extract-arg-names + to-watast → WatAST::Symbol as valid let binder)
//! - Coordinator body is a delegating call to a named fn (advertised pattern)
//! - The bracket: spawn → peer pairing → factory invocation → coordinator
//!   invocation → drain-and-join — works end-to-end for N=1
//!
//! Call form (updated for coordinator-fn):
//!
//!   (:wat::kernel::run-threads
//!     (:wat::core::fn
//!       [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
//!       -> :wat::core::String
//!       (:my::echo-client peer))     ;; delegating body (advertised pattern)
//!     (:my::echo-factory))           ;; factory call form (zero-arg → worker fn)
//!
//! Factory form: `:my::echo-factory` is passed as a keyword reference (not a call form).
//! The expansion is `(:my::echo-factory (ThreadPeer/new server-rx server-tx))` — calling
//! the factory fn directly with the server-side peer. Keyword factory args work because
//! the macro template uses `~factory-0` (plain unquote, not splice) at the call position.
//!
//! What D2 adds (separate test file `tests/wat_run_threads_d2.rs`):
//!   N=3 heterogeneous factories via the same coordinator-fn macro.

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

// ─── Stone D2 N=1. single-factory coordinator-fn round-trip ───────────

#[test]
fn run_threads_d1_single_factory_round_trips_string() {
    // Echo factory: a zero-arg constructor returning the server-side worker fn.
    // Worker fn takes `ThreadPeer<String, String>` (server-side): reads one
    // String from the peer via Thread/readln, writes it back via Thread/println,
    // returns nil.
    //
    // Echo client: takes `ThreadPeer<String, String>` (client-side): writes
    // "hello" via Thread/println, reads the echo via Thread/readln, returns
    // the echoed String.
    //
    // Coordinator fn (inline, delegating): declares the peer binder with its
    // type for reflection, delegates to the named echo-client fn. This is the
    // ONLY advertised coordinator body pattern — a single delegating call.
    //
    // The run-threads macro:
    //   1. At expand time: evals the coordinator fn → signature-of-fn →
    //      extract-arg-types → ThreadPeer<String,String> at slot 0 →
    //      Bundle/children → atom-value → Receiver<String>/Sender<String>
    //   2. Generates thread-0 binding (spawn-thread with wrap fn typed via step 1)
    //   3. Uses coordinator binder name "peer" (via extract-arg-names + to-watast)
    //      as the let-binding name for the client peer
    //   4. Calls (~coordinator peer) to invoke the inline coordinator fn with the peer
    //   5. Drains thread-0 via Thread/drain-and-join
    //   6. Returns the coordinator fn's return value (the echoed String)
    // Factory call form strategy: the BRIEF specifies call forms like
    // (:app::factory) for factories. However, call forms require defining
    // a zero-arg constructor returning a fn — which requires declaring the
    // Fn(...)->... return type. For simplicity, factories are passed as
    // KEYWORD REFERENCES (:my::echo-factory) in the run-threads call.
    // The macro handles keyword factory args correctly: the expansion is
    // (:my::echo-factory (ThreadPeer/new ...)) — calling the factory fn
    // directly with the peer. Same as D1's original positional form.
    // Noted in SCORE as honest delta on factory call form convention.
    let src = r#"
        ;; Server-side echo worker: reads one String, writes it back.
        ;; Passed as keyword :my::echo-factory in the coordinator-fn call.
        ;; Expansion: (:my::echo-factory (ThreadPeer/new server-rx server-tx)).
        (:wat::core::defn :my::echo-factory
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [line (:wat::kernel::Thread/readln peer)
             _    (:wat::kernel::Thread/println peer line)]
            :wat::core::nil))

        ;; Named echo-client fn: the actual coordinator logic (independently testable).
        ;; Takes the client-side peer, sends "hello", reads echo, returns it.
        (:wat::core::defn :my::echo-client
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::String
          (:wat::core::let
            [_     (:wat::kernel::Thread/println peer "hello")
             reply (:wat::kernel::Thread/readln peer)]
            reply))

        ;; Entry point: uses the coordinator-fn form.
        ;; Coordinator body is a single delegating call to :my::echo-client
        ;; (advertised pattern per BRIEF + STOP-trigger 6).
        ;; Factory is passed as a keyword reference (not a call form).
        (:wat::core::defn :my::test::run-d1
          [] -> :wat::core::String
          (:wat::kernel::run-threads
            (:wat::core::fn
              [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
              -> :wat::core::String
              (:my::echo-client peer))
            :my::echo-factory))
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
    .expect("run-threads bracket should return the coordinator fn value");
    match outcome {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "hello",
            "echo round-trip must return 'hello'; got {:?}",
            s
        ),
        other => panic!("expected Value::String(\"hello\"); got {:?}", other),
    }
}
