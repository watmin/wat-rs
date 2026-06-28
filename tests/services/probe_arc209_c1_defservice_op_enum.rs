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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn defservice_emits_op_enum_with_wrapped_request_records() {
    // arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; accessors read through durable.
    // Wat source lives in the co-located fixture: probe_arc209_c1_defservice_op_enum.wat
    let world = startup_beside(file!())
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
