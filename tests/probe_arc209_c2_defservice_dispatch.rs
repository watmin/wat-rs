//! Arc 209 Stone C.2 — `defservice` generates the dispatch loop (`serve`), RPC model.
//!
//! C.1 made `defservice` emit the request enum `<fqdn>::Op` (+ per-op Request records). C.2 makes
//! it ALSO emit the response enum `<fqdn>::Reply` (+ per-op Response records) AND `<fqdn>::serve`
//! — the `poll'`/`ServiceEvent` dispatch loop that owns the live `:state` (state-as-self = the
//! mutex), decodes each `Op` (unwrapping the inner Request record), runs the INLINE handler
//! `(s, in...) -> Outcome::Reply{new-state, ResponseRecord}`, wraps the `Reply::<Op>(resp)`,
//! `send'`s it back, and TCO-recurs.
//!
//! THE RPC MODEL (builder, 2026-06-14): an op is `(RequestRecord, ResponseRecord)`. Emitted as:
//!   - per-op Request + Response records (Record::def)
//!   - `Op::<Op>` variant WRAPS the Request (`req <- <Op>Request`)
//!   - `Reply::<Op>` variant WRAPS the Response (`resp <- <Op>Response`)
//! Wire = `Peer'<Reply, Op>` (server-side peer; mirrors c0b1b's `Peer'<reply, request>` order).
//!
//! THE GATE: defservice a counter, hand-drive the generated `serve` on a thread (C.3 adds the
//! start fn + client wrappers; here the probe drives `serve` directly). connect' → send'
//! (Op::Increment wrapping IncrementRequest{n=5}) → recv' = Reply::Increment{resp=IncrementResponse{5}}
//! → send' (Op::Get wrapping GetRequest{}) → recv' = Reply::Get{resp=GetResponse{5}} →
//! owner drops the handle → :Shutdown → join completes.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as ONE defservice — C.3 wrapped-record shape (single format). C.2 must generate
// Op (C.1), Reply, and serve. Probe hand-drives serve directly.
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::State/count s))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ (:my::counter::State/count s) n)]
       (:wat::service::Outcome::Reply (:my::counter::State s') (:my::counter::IncrementResponse s'))))])

;; Unwrap a Reply enum → extract the `value` field from the inner Response record.
;; Each Reply variant carries `resp <- <Op>Response`; Response carries `value <- :i64`.
(:wat::core::defn :user::reply-value [r <- :my::counter::Reply] -> :wat::core::i64
  (:wat::core::match r -> :wat::core::i64
    ((:my::counter::Reply::Get resp) (:my::counter::GetResponse/value resp))
    ((:my::counter::Reply::Increment resp) (:my::counter::IncrementResponse/value resp))))

;; Hand-drive the GENERATED serve (C.3 will wrap start + clients). Mirrors c0b1b's thread-tier
;; driver: parent mints the listener, spawns serve with the captured listener + empty clients +
;; literal initial state 0, connects a client, round-trips two ops, reads the typed Reply.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :my::counter::Op :my::counter::Reply)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     ;; arc 291 3a-ii-β: serve's `self` is the lineage self-peer (Peer'<LineageUp,Admin>),
     ;; not a client peer. The clients Vector stays the client type (Peer'<Reply,Op>).
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::Peer'<my::counter::LineageUp,my::counter::Admin>] -> :wat::core::nil
              (:my::counter::serve self l
                (:wat::core::Vector :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>)
                (:my::counter::State 0))))
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c (:my::counter::Op::Increment (:my::counter/increment-request 5)))
     r1   (:wat::kernel::recv' c)
     _    (:wat::kernel::send' c (:my::counter::Op::Get (:my::counter/get-request)))
     r2   (:wat::kernel::recv' c)]
    ;; Increment 5 → state 0→5, reply IncrementResponse{5}; Get → reply GetResponse{5}.
    ;; Assert the Get reply's value is 5.
    ;; Scope-exit drops `svc` → RAII drain → :Shutdown → serve exits → join completes.
    (:user::reply-value r2)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_generates_dispatch_loop_round_trips_on_thread() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C.2: defservice generates Reply + serve)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected the Get reply's value 5 (Increment 5 set state 0→5; Get read it back) \
         round-tripped through the spawned counter service's generated serve-loop \
         (wrapped-record C.3 shape: Op::Increment wraps IncrementRequest, \
          Reply::Get wraps GetResponse); got {got:?}"
    );
}
