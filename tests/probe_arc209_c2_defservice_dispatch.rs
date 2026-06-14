//! Arc 209 Stone C.2 — `defservice` generates the dispatch loop (`serve`), RPC model.
//!
//! C.1 made `defservice` emit the request enum `<fqdn>::Op`. C.2 makes it ALSO emit the
//! response enum `<fqdn>::Reply` AND `<fqdn>::serve` — the `poll'`/`ServiceEvent` dispatch
//! loop that owns the live `:state` (state-as-self = the mutex), decodes each `Op`, runs the
//! INLINE handler `(s, in) -> (:Tuple new-state reply)`, `send'`s the `Reply`, and TCO-recurs.
//!
//! THE RPC MODEL (builder, 2026-06-14): an op is `(InRecord, OutRecord)`. Inline-and-minted:
//!   - input fields  → `Op::<Op>`    variant (the request record)
//!   - output fields → `Reply::<Op>` variant (the response record)
//! Wire = `Peer'<Reply, Op>` (mirrors c0b1b's `Peer'<reply, request>` order). Names locked by
//! an intueri cast: Op / Reply / serve / :state / :ops, variant-IS-the-record (no ::In/::Out).
//!
//! THE GATE: defservice a counter, hand-drive the generated `serve` on a thread (C.3 will add
//! the start fn + client wrappers; here the probe drives `serve` directly with a literal initial
//! state). connect' → send' (Increment 5) → recv' = Reply::Increment{value 5} → send' Get →
//! recv' = Reply::Get{value 5} → owner drops the handle → :Shutdown → join completes.
//!
//! RED at HEAD: C.1's macro emits only `Op`; `<fqdn>::Reply` and `<fqdn>::serve` do not exist,
//! and the op bodies reference `:my::counter::Reply::*` (undefined) — the world fails to build.
//! Deterministically GREEN once C.2 ships the two-enum + serve generation.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as ONE defservice — RPC model, inline In/Out, locked names. C.2 must generate
// `Op` (C.1, unchanged), `Reply`, and `serve`.
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::core::Tuple s (:my::counter::Reply::Get s)))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::core::Tuple s' (:my::counter::Reply::Increment s'))))])

;; Hand-drive the GENERATED serve (C.3 will wrap start + clients). Mirrors c0b1b's thread-tier
;; driver: parent mints the listener, spawns serve with the captured listener + empty clients +
;; literal initial state 0, connects a client, round-trips two ops, reads the typed Reply.
(:wat::core::defn :user::reply-value [r <- :my::counter::Reply] -> :wat::core::i64
  (:wat::core::match r -> :wat::core::i64
    ((:my::counter::Reply::Get value) value)
    ((:my::counter::Reply::Increment value) value)))

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :my::counter::Op :my::counter::Reply)
     l    (:wat::core::first pair)
     addr (:wat::core::second pair)
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>] -> :wat::core::nil
              (:my::counter::serve self l
                (:wat::core::Vector :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>)
                0)))
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c (:my::counter::Op::Increment 5))
     r1   (:wat::kernel::recv' c)
     _    (:wat::kernel::send' c :my::counter::Op::Get)
     r2   (:wat::kernel::recv' c)]
    ;; Increment 5 → state 0→5, reply 5; Get → reply 5. Assert the Get reply is 5.
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
        "expected the Get reply value 5 (Increment 5 set state 0→5; Get read it back) round-tripped \
         through the spawned counter service's generated serve-loop; got {got:?}"
    );
}
