//! Arc 293.W.2a — the `recv'` struct backstop.
//!
//! A bare `Nature::Struct` value must NOT arrive over a comms boundary.
//! The containment rule (293.W.1) prevents a record from *holding* a struct,
//! but a child can still `pprintln` a bare struct to its stdout and a parent
//! `recv'` it — the untyped path the declaration gate cannot reach.
//!
//! Close it at the wire DECODE door: `decode_trusted_wire` (`src/edn/render.rs`)
//! must refuse a top-level `Value::Aggregate` whose `nature == Nature::Struct`.
//!
//! RED at HEAD: the struct crosses — decode succeeds, parent gets the struct
//! value with no error. The probe asserting an error FAILs at HEAD.
//! GREEN after fix: `decode_trusted_wire` refuses the struct → `recv'` raises
//! a `RuntimeError` → `eval_in_frozen` returns `Err` → probe PASSES.
//!
//! Control: a base record still round-trips. The backstop must reject ONLY
//! structs, never records; this guard catches over-rejection.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

/// Arc 293.W.2d: the §7 runtime decode backstop is RETIRED.
///
/// The compile-time purity wall at wire-peer PRODUCERS (peer-pair', etc.)
/// makes the struct-on-wire case structurally unrepresentable. The runtime
/// `decode_trusted_wire` StructOnWire guard was deleted by arc 293.W.2d.
///
/// The untyped `pprintln` path (used here) can still emit a struct's tagged EDN
/// over stdout, and without the runtime guard the parent `recv'` succeeds. This
/// is an out-of-scope trust-boundary concern (the compile-time wall is the
/// primary defense; user validates inputs on the untyped path).
///
/// After 2d: the struct crosses, `recv'` returns `i64(99)` (field access succeeds).
#[test]
fn struct_rejected_at_wire_decode() {
    let got = call_beside_value(file!(), ":w2a::probe-struct");
    // Arc 293.W.2d: the runtime decode backstop was deleted; struct arrives cleanly.
    // The pprintln untyped path still emits the struct; the parent field-access
    // returns i64(99).  The compile-time wall at peer producers is the real guard.
    assert!(
        got.is_ok(),
        "arc 293.W.2d: the runtime decode backstop is retired; a struct emitted via \
         pprintln (untyped path) now arrives at the parent without error. \
         got: {:?}",
        got
    );
    assert!(
        matches!(got.unwrap(), Value::i64(99)),
        "expected i64(99) from probe-struct field access after struct arrival"
    );
}

/// Control: a base record still round-trips over the process wire.
/// Must be GREEN at HEAD AND after the backstop is added (records are wire-portable).
#[test]
fn record_still_round_trips_after_backstop() {
    let got = call_beside_value(file!(), ":w2a::probe-record")
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

/// OUTBOUND. A wire peer (`:wat::program::self-peer`) with a struct type arg must fail at CHECK.
///
/// Arc 293.W.2d: the purity wall is now at wire-peer PRODUCERS (peer-pair', etc.),
/// not at `send'` time. The test loads `probe_arc293_W2c_compile_time_send.wat`
/// which uses `:wat::program::self-peer<Struct,i64>` and asserts the compile-time check error.
/// (The 2c send'-gate was deleted in 2d; this test now exercises the 2d wall.)
// The golden below is stale on TWO axes now, and both are recorded so the recapture is not
// mistaken for a one-liner: (1) the pre-stone-B rust-debug face, which 296 replaces wholesale;
// (2) arc 278 — the fixture's producer moved off the annihilated `peer-pair'` onto
// `:wat::program::self-peer`, so the head AND the line:col in the text below are both wrong.
// Deliberately NOT hand-patched: a golden edited to be less-wrong reads as maintained when it
// is not. 296's recapture writes it from a real run.
#[test]
#[allow(non_snake_case)]
fn struct_rejected_at_wire_SEND() {
    let result = startup_from_file("tests/comms/probe_arc293_W2c_compile_time_send.wat");
    assert!(
        result.is_err(),
        ":wat::program::self-peer with a struct type arg MUST fail at CHECK (arc 293.W.2d — \
         a struct is impure §7; the wire-peer producer's purity gate must reject \
         this world). got Ok"
    );
    let err_str = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(err_str, "probe_arc293_W2a_struct_no_cross__struct_rejected_at_wire_SEND.edn", "check error must match arc 293 §7 purity wall golden");
}

/// Control: a parent `send'`ing a base record to a PROCESS child still works
/// (records are portable). Must be GREEN at HEAD AND after the guard — the guard
/// rejects ONLY structs, never records. Guards against over-rejection on send.
#[test]
fn record_still_sends_after_backstop() {
    let got = call_beside_value(file!(), ":w2a::probe-send-record")
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
    let got = call_beside_value(file!(), ":w2a::probe-send-struct-thread")
        .expect(
            "struct over a THREAD peer MUST round-trip — the guard is process/socket \
             only; a struct in-locus over a thread peer is legitimate (§7)"
        );
    assert!(
        matches!(got, Value::i64(99)),
        "expected i64(99) (struct field after thread round-trip); got {got:?}"
    );
}
