//! Arc 209 Stone C.3 — `defservice` generates the full-gRPC CLIENT FACE.
//!
//! C.2 made `defservice` emit `Op` + `Reply` (inline-field variants) + `serve`. C.3 refines the
//! surface to full-gRPC and ADDS the client face:
//!   - per op, a standalone **Request** + **Response** record (`:wat::core::defrecord`);
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

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn defservice_generates_full_grpc_client_face() {
    // The counter as ONE defservice. arc 291 4b-ii: State is now a defstruct.
    // Wat source lives in the co-located fixture: probe_arc209_c3_defservice_client_face.wat
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected GetResponse.value == 5 driven through the generated client face \
         (start 0 → connect → increment(increment-request 5) → get(get-request)); got {got:?}"
    );
}
