//! Arc 209 Stone C.3 — `defservice` generates the full-gRPC CLIENT FACE.
//!
//! C.2 made `defservice` emit `Op` + `Reply` (inline-field variants) + `serve`. C.3 refines the
//! surface to full-gRPC and ADDS the client face:
//!   - per op, a standalone **Request** + **Response** record (`:wat::Record::def`);
//!   - `Op`/`Reply` WRAP them (one field per variant: `req` / `resp`) — not inline fields;
//!   - `serve` unwraps the request, runs the body, wraps the Response in `Reply`;
//!   - **request constructors** `<fqdn>/<op>-request`, type-safe **methods** `<fqdn>/<op>`
//!     (explicit connected peer `c`), a **start fn** `<fqdn>/start` returning a `<fqdn>::Handle`.
//!
//! THE GATE: defservice the counter, then drive ENTIRELY through the generated client face on a
//! thread — `start 0` mints the listener + spawns serve; `connect'` the Handle's addr; call the
//! generated `increment`/`get` methods with generated request constructors; assert the Get
//! response's `value` is 5 (Increment 5 set state 0→5; Get read it back). Dropping the Handle at
//! scope-exit → RAII drain → `:Shutdown` → serve exits → join completes (deadlock-free).
//!
//! RED at HEAD: C.2's macro emits inline-field variants and NO client face — `<Op>Request`,
//! `<fqdn>/start`, `<fqdn>/increment`, `<fqdn>::Handle`, etc. are unresolved; the world fails to
//! build. Deterministically GREEN once C.3 ships the full-gRPC generation.
//!
//! The composition this rests on (a defmacro emitting `Record::def` calls that re-expand, a
//! `defenum` wrapping the emitted records) is proven independently by
//! `tests/probe_diagnostic_c3_macro_emits_record_def.rs`.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c3_defservice_client_face

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as ONE defservice — RPC `:ops` (the body now constructs the Response record inside
// Outcome::Reply). C.3 must generate the wrapped-record enums + serve + the full client face.
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::service::Outcome::Reply s' (:my::counter::IncrementResponse s'))))])

;; Drive ENTIRELY through the generated client face: start → connect → method calls via request
;; constructors. `h` stays bound for the whole let, so the service lives until compute returns;
;; scope-exit drops `h` → :Shutdown → join completes.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start (:wat::spawn::thread) 0)
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::counter/increment c (:my::counter/increment-request 5))
     r  (:my::counter/get c (:my::counter/get-request))]
    (:my::counter::GetResponse/value r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_generates_full_grpc_client_face() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C.3: defservice generates the full-gRPC client face)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected GetResponse.value == 5 driven through the generated client face \
         (start 0 → connect → increment(increment-request 5) → get(get-request)); got {got:?}"
    );
}
