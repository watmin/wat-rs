//! Arc 293.W.2a — the `recv'` struct backstop.
//!
//! A bare `Holder::Struct` value must NOT arrive over a comms boundary.
//! The containment rule (293.W.1) prevents a record from *holding* a struct,
//! but a child can still `pprintln` a bare struct to its stdout and a parent
//! `recv'` it — the untyped path the declaration gate cannot reach.
//!
//! Close it at the wire DECODE door: `decode_trusted_wire` (`src/edn_shim.rs`)
//! must refuse a top-level `Value::Aggregate` whose `holder == Holder::Struct`.
//!
//! RED at HEAD: the struct crosses — decode succeeds, parent gets the struct
//! value with no error. The probe asserting an error FAILs at HEAD.
//! GREEN after fix: `decode_trusted_wire` refuses the struct → `recv'` raises
//! a `RuntimeError` → `eval_in_frozen` returns `Err` → probe PASSES.
//!
//! Control: a base record still round-trips. The backstop must reject ONLY
//! structs, never records; this guard catches over-rejection.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// RED at HEAD (struct crosses; this probe asserts an error → FAILS).
/// GREEN after fix (decode refused; recv' raises → PASSES).
#[test]
fn struct_rejected_at_wire_decode() {
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-struct)")
        .expect("parse (:w2a::probe-struct)");
    let got = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        got.is_err(),
        "recv' of a bare Holder::Struct MUST fail — a struct is in-locus only (§7); \
         if this assertion fails, the wire backstop is missing and the breach is open. \
         got: {:?}",
        got
    );
    let err_str = format!("{}", got.unwrap_err());
    assert!(
        err_str.to_lowercase().contains("struct"),
        "error message must mention 'struct' (§7 rejection); got: {err_str}"
    );
}

/// Control: a base record still round-trips over the process wire.
/// Must be GREEN at HEAD AND after the backstop is added (records are wire-portable).
#[test]
fn record_still_round_trips_after_backstop() {
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-record)")
        .expect("parse (:w2a::probe-record)");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect(
            "record round-trip MUST succeed — the backstop must NOT reject records, \
             only bare structs (§7 is struct-specific)"
        );
    assert!(
        matches!(got, Value::i64(42)),
        "expected i64(42) (field from received record); got {got:?}"
    );
}

// ── OUTBOUND: the send' wire-wall ─────────────────────────────────

/// OUTBOUND. A parent `send'`ing a bare struct to a PROCESS child must fail —
/// a struct is in-locus only (§7) and cannot be WRITTEN to the wire.
/// RED at HEAD (it serializes → send' returns nil). GREEN after the send' guard.
#[test]
fn struct_rejected_at_wire_SEND() {
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-send-struct)")
        .expect("parse (:w2a::probe-send-struct)");
    let got = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        got.is_err(),
        "send' of a bare Holder::Struct over a PROCESS peer MUST fail — a struct is \
         in-locus only (§7); if this assertion fails, the outbound guard is missing. got: {:?}",
        got
    );
    let err_str = format!("{}", got.unwrap_err());
    assert!(
        err_str.to_lowercase().contains("struct"),
        "error message must mention 'struct' (§7 rejection); got: {err_str}"
    );
}

/// Control: a parent `send'`ing a base record to a PROCESS child still works
/// (records are portable). Must be GREEN at HEAD AND after the guard — the guard
/// rejects ONLY structs, never records. Guards against over-rejection on send.
#[test]
fn record_still_sends_after_backstop() {
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-send-record)")
        .expect("parse (:w2a::probe-send-record)");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect(
            "record send MUST succeed — the outbound guard must NOT reject records, \
             only bare structs (§7 is struct-specific)"
        );
    assert!(
        matches!(got, Value::Unit),
        "expected nil (send' returns unit on success); got {got:?}"
    );
}

/// Control: a struct over a THREAD peer round-trips in-locus (same address space,
/// no serialization, no guard). Must be GREEN — the send' guard is process/socket
/// only; a struct over a thread peer is legitimate. Guards against over-reach into
/// the thread tier (symmetric to the inbound thread recv' having no decode door).
#[test]
fn struct_crosses_thread_peer_in_locus() {
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-send-struct-thread)")
        .expect("parse (:w2a::probe-send-struct-thread)");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect(
            "struct over a THREAD peer MUST round-trip — the guard is process/socket \
             only; a struct in-locus over a thread peer is legitimate (§7)"
        );
    assert!(
        matches!(got, Value::i64(99)),
        "expected i64(99) (struct field after thread round-trip); got {got:?}"
    );
}
