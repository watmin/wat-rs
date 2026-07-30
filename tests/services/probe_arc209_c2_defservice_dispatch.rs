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
//!
//! Wire = `Peer'<Reply, Op>` (server-side peer; mirrors c0b1b's `Peer'<reply, request>` order).
//!
//! THE GATE: defservice a counter, hand-drive the generated `serve` on a thread (C.3 adds the
//! start fn + client wrappers; here the probe drives `serve` directly). connect' → send'
//! (Op::Increment wrapping IncrementRequest{n=5}) → recv' = Reply::Increment{resp=IncrementResponse{5}}
//! → send' (Op::Get wrapping GetRequest{}) → recv' = Reply::Get{resp=GetResponse{5}} →
//! owner drops the handle → :Shutdown → join completes.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn defservice_generates_dispatch_loop_round_trips_on_thread() {
    // arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; serve takes ::State (struct).
    // Wat source lives in the co-located fixture: probe_arc209_c2_defservice_dispatch.wat
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected the Get reply's value 5 (Increment 5 set state 0→5; Get read it back) \
         round-tripped through the spawned counter service's generated serve-loop \
         (wrapped-record C.3 shape: Op::Increment wraps IncrementRequest, \
          Reply::Get wraps GetResponse); got {got:?}"
    );
}
