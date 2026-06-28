//! Arc 258.5 — `recv'` infers its type from the constraining consumer (the IO-cluster arrow, narrowed).
//!
//! `recv'` on a process handle returns a FRESH var O (infer_process_prog_type, check.rs:10827 —
//! fresh-by-design: the child's self-peer type is buried in its forms). A consumer that knows the type
//! should let it flow in WITHOUT a `-> :T` ascription. Today `connect'` RIGID pattern-matches its arg
//! against `Address'<S,R>` (infer_connect_prime, check.rs:10485) and errors on a fresh var — so the
//! type can't flow from the consumer, and the ascription is the only source. That ascription is the arrow.
//!
//! CHECK-LEVEL probe (asserts startup type-checks — does NOT eval): isolates the inference
//! from arc-272 6a-i's separate `Address'`-EDN-decode RUNTIME gap. `(connect' (recv' svc))` with NO `-> :T`.
//!
//! GREEN once `connect'` unifies its arg against `Address'<fresh,fresh>` (258.5a) so the fresh O binds.
//!
//! Run: cargo test --release -p wat --test comms probe_arc258_recv_infers_from_consumer

use wat::freeze::startup_beside;

#[test]
fn recv_infers_address_from_connect_consumer() {
    let result = startup_beside(file!());
    assert!(
        result.is_ok(),
        "expected the program to TYPE-CHECK: `(connect' (recv' svc))` with no `-> :T` — connect' should \
         unify its expected Address'<S,R> into recv's fresh result so the type flows from the consumer. \
         Err: {:?}",
        result.err()
    );
}
