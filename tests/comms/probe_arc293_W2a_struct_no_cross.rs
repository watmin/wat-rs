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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
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

/// OUTBOUND. A parent `send'`ing a bare struct to a PROCESS child must fail.
///
/// Arc 293.W.2c supersedes the runtime guard: the typed struct→process send is
/// now rejected at CHECK time (`infer_send_prime` portability gate). The world
/// fails to load before it can run — `startup_from_file` returns `Err`.
///
/// The test loads `probe_arc293_W2c_compile_time_send.wat` (the 2c fixture that
/// contains the struct→process send in isolation) and asserts the check error.
/// The outbound probe was removed from this file's .wat to prevent the check
/// error from contaminating the inbound / record / thread-control tests above.
#[test]
#[allow(non_snake_case)]
fn struct_rejected_at_wire_SEND() {
    let result = startup_from_file("tests/comms/probe_arc293_W2c_compile_time_send.wat");
    assert!(
        result.is_err(),
        "send' of a bare struct to a Process' peer MUST fail at CHECK (arc 293.W.2c — \
         a struct is in-locus only, §7; infer_send_prime portability gate must reject \
         this world). If this assertion fails, the compile-time gate is missing. got Ok"
    );
    let err_str = format!("{}", result.unwrap_err());
    let lower = err_str.to_lowercase();
    assert!(
        lower.contains("portable") || lower.contains("struct") || lower.contains("wire"),
        "check error must mention portability, struct, or wire (§7); got: {err_str}"
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
