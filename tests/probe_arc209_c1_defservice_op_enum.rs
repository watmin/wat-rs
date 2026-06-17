//! Arc 209 Stone C.1 — `defservice` emits the op enum (the skeleton's first generated form).
//!
//! Stone C mints `:wat::service::defservice` (a PURE-WAT defmacro in `wat/service.wat`) that
//! generates a complete service — per-op Request/Response records, op/reply enums, the dispatch
//! loop, and the client face — from a flat `:state` + `:ops` surface. **C.1 is the first
//! sub-stone: the macro skeleton + the OP ENUM only.** C.2 adds the dispatch loop; C.3 the
//! client face + start fn.
//!
//! THE SURFACE (settled 2026-06-13, four-questions → option A): each op is a self-contained
//! List `(OpName [s <- :State ...client-args] -> [out-fields] body)` — bodies INLINE, the whole
//! service in one form. The macro walks `:ops` WatAST-native (`ast->children` per op-List).
//!
//! C.3 SHAPE (single format — no dual-format detection): per op, the macro emits a standalone
//! Request record (`<fqdn>::<Op>Request`) and `Op::<Op>` WRAPS it (one field: `req`). No inline
//! variant fields. The C.1 probe validates only the Op enum + the underlying IncrementRequest
//! record (C.3 emits both before the enum).
//!
//! THE PROOF: construct `(:my::counter/increment-request 5)` (generated ctor), wrap in
//! `(:my::counter::Op::Increment req)`, match — the bare `(:my::counter::Op::Get _r) 0` arm
//! compiles (proving Get variant exists wrapping a GetRequest), the Increment arm extracts
//! `n` via the generated accessor `(:my::counter::IncrementRequest/n req)` and returns 5.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as ONE defservice (C.3 surface — wrapped-record shape). C.1 reads the op surface
// and emits the Request/Response records + the op enum; probe-op exercises ONLY the generated
// enum + the IncrementRequest record.
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

;; Exercise the GENERATED op enum (wrapped-record C.3 shape):
;;   1. Build an IncrementRequest via the generated constructor.
;;   2. Wrap it in the Op::Increment variant.
;;   3. Match: Get arm returns 0 (proves Get variant exists + wraps GetRequest);
;;      Increment arm extracts n via IncrementRequest/n accessor → 5.
(:wat::core::defn :user::probe-op [] -> :wat::core::i64
  (:wat::core::let [req (:my::counter/increment-request 5)
                    op  (:my::counter::Op::Increment req)]
    (:wat::core::match op -> :wat::core::i64
      ((:my::counter::Op::Get _r) 0)
      ((:my::counter::Op::Increment req) (:my::counter::IncrementRequest/n req))
      ((:my::counter::Op::Stop _r) 0))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_emits_op_enum_with_wrapped_request_records() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Stone C.1: defservice emits Op enum + Request records)");
    let ast = wat::parse_one!("(:user::probe-op)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("probe-op raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: defservice :my::counter must emit `:my::counter::Op` with a `:Get` variant \
         wrapping `GetRequest` and an `:Increment` variant wrapping `IncrementRequest`; \
         constructing `(increment-request 5)` + `(Op::Increment req)` + matching via \
         `IncrementRequest/n` accessor extracts n=5; got {got:?}"
    );
}
