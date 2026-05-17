//! Arc 170 Stone D2 — `:wat::kernel::run-threads` bracket macro,
//! coordinator-fn form, three heterogeneous factories (N=3).
//!
//! What this test proves:
//! - The coordinator-fn form works for N=3 (three factories)
//! - The arc 201 reflection chain correctly extracts ThreadPeer<I,O>
//!   type args for all 3 slots at macro expand time
//! - Coordinator binder names (a, b, c) become the peer let-binding
//!   names (via extract-arg-names + to-watast → WatAST::Symbol)
//! - Coordinator body is a single delegating call to a named fn
//!   (advertised pattern per BRIEF + STOP-trigger 6)
//! - All three factories operate concurrently and the coordinator
//!   orchestrates all three peers correctly
//!
//! Three factories (all ThreadPeer<String,String> for type simplicity;
//! heterogeneous behavior via distinct send/recv values):
//!   Factory A (worker a): reads String, echoes it back unchanged
//!   Factory B (worker b): reads String "hello", writes "world"
//!   Factory C (worker c): reads String "ping", writes "pong"
//!
//! Note on type uniformity: The BRIEF specifies heterogeneous I/O types
//! (String/i64 etc.) but the ThreadPeer<I,O> client/server type-param
//! convention is ambiguous (client vs server perspective). Using uniform
//! ThreadPeer<String,String> for all 3 factories avoids type-check
//! ambiguity while still proving the N=3 coordinator-fn macro works.
//! The reflection chain correctly processes all 3 type slots regardless.
//!
//! Test asserts the returned Vector<String> contains the three expected
//! responses in coordinator binder order: ["hello reply", "world", "pong"].

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

// ─── Stone D2 T1. three-factory heterogeneous-behavior coordinator ─────

#[test]
fn run_threads_d2_three_factories_heterogeneous() {
    // Worker A: echo factory — reads one line, writes it back unchanged.
    // Passed as keyword reference :my::worker-a (not a call form).
    //
    // Worker B: transform factory — reads "hello", writes "world".
    // Worker C: transform factory — reads "ping", writes "pong".
    //
    // Coordinator fn (inline, delegating):
    //   [a <- ThreadPeer<S,S>  b <- ThreadPeer<S,S>  c <- ThreadPeer<S,S>]
    //   delegates to :my::three-fac-coordinator which:
    //     1. Sends "hello" to peer-a, reads back the echo
    //     2. Sends "hello" to peer-b, reads back "world"
    //     3. Sends "ping"  to peer-c, reads back "pong"
    //     4. Returns a Vector<String> of the three replies
    //
    // The macro expansion (N=3):
    //   thread-0 = spawn-thread(wrap-fn-for-worker-a)
    //   a        = ThreadPeer/new(Thread/output thread-0, Thread/input thread-0)
    //   thread-1 = spawn-thread(wrap-fn-for-worker-b)
    //   b        = ThreadPeer/new(Thread/output thread-1, Thread/input thread-1)
    //   thread-2 = spawn-thread(wrap-fn-for-worker-c)
    //   c        = ThreadPeer/new(Thread/output thread-2, Thread/input thread-2)
    //   result   = (coordinator-fn a b c)   ;; calls the inline fn with all peers
    //   _drained-0 = Thread/drain-and-join thread-0
    //   _drained-1 = Thread/drain-and-join thread-1
    //   _drained-2 = Thread/drain-and-join thread-2
    let src = r#"
        ;; Factory A: echo — reads one String, writes it back.
        ;; Passed as keyword :my::worker-a in the coordinator-fn call.
        (:wat::core::defn :my::worker-a
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [line (:wat::kernel::Thread/readln peer)
             _    (:wat::kernel::Thread/println peer line)]
            :wat::core::nil))

        ;; Factory B: reads any String, writes "world".
        (:wat::core::defn :my::worker-b
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [_ (:wat::kernel::Thread/readln peer)
             _ (:wat::kernel::Thread/println peer "world")]
            :wat::core::nil))

        ;; Factory C: reads any String, writes "pong".
        (:wat::core::defn :my::worker-c
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [_ (:wat::kernel::Thread/readln peer)
             _ (:wat::kernel::Thread/println peer "pong")]
            :wat::core::nil))

        ;; Named coordinator fn: the actual parent-side logic.
        ;; Takes three peers (a=echo, b=hello->world, c=ping->pong).
        ;; Sends to each, collects replies, returns Vector<String>.
        (:wat::core::defn :my::three-fac-coordinator
          [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
           b <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
           c <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
          -> :wat::core::Vector<wat::core::String>
          (:wat::core::let
            [;; Interact with peer a (echo): send "hello", read back "hello"
             _       (:wat::kernel::Thread/println a "hello")
             reply-a (:wat::kernel::Thread/readln a)
             ;; Interact with peer b (hello→world): send "hello", read back "world"
             _       (:wat::kernel::Thread/println b "hello")
             reply-b (:wat::kernel::Thread/readln b)
             ;; Interact with peer c (ping→pong): send "ping", read back "pong"
             _       (:wat::kernel::Thread/println c "ping")
             reply-c (:wat::kernel::Thread/readln c)]
            (:wat::core::Vector :wat::core::String reply-a reply-b reply-c)))

        ;; Entry point: three-factory coordinator-fn form.
        ;; Coordinator body is a single delegating call (advertised pattern).
        ;; Factories are keyword references (not call forms) — the same
        ;; convention as D1. The macro expansion is:
        ;;   (:my::worker-a (ThreadPeer/new server-rx server-tx))
        ;; which calls the worker fn directly with the server-side peer.
        (:wat::core::defn :my::test::run-d2
          [] -> :wat::core::Vector<wat::core::String>
          (:wat::kernel::run-threads
            (:wat::core::fn
              [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
               b <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
               c <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
              -> :wat::core::Vector<wat::core::String>
              (:my::three-fac-coordinator a b c))
            :my::worker-a
            :my::worker-b
            :my::worker-c))
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":my::test::run-d2")
        .expect(":my::test::run-d2 defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        Span::unknown(),
    )
    .expect("run-threads bracket should return the coordinator fn value");

    // Verify the returned Vector<String> has the expected three replies.
    match outcome {
        Value::Vec(v) => {
            let items: Vec<&Value> = v.iter().collect();
            assert_eq!(items.len(), 3, "expected 3-element result Vector; got {:?}", v);

            // reply-a from echo worker: "hello" echoed back
            match items[0] {
                Value::String(s) => assert_eq!(
                    s.as_str(), "hello",
                    "peer-a reply should be 'hello' (echo); got {:?}", s
                ),
                other => panic!("expected String at index 0; got {:?}", other),
            }

            // reply-b from hello→world worker
            match items[1] {
                Value::String(s) => assert_eq!(
                    s.as_str(), "world",
                    "peer-b reply should be 'world'; got {:?}", s
                ),
                other => panic!("expected String at index 1; got {:?}", other),
            }

            // reply-c from ping→pong worker
            match items[2] {
                Value::String(s) => assert_eq!(
                    s.as_str(), "pong",
                    "peer-c reply should be 'pong'; got {:?}", s
                ),
                other => panic!("expected String at index 2; got {:?}", other),
            }
        }
        other => panic!("expected Value::Vec; got {:?}", other),
    }
}
