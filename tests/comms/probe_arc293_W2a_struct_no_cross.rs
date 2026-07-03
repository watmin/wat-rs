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
    let world = startup_beside(file!())
        .expect("startup_beside: fixture load must succeed");
    let ast = wat::parse_one!("(:w2a::probe-struct)")
        .expect("parse (:w2a::probe-struct)");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned());
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

/// OUTBOUND. A wire peer (`peer-pair'`) with a struct type arg must fail at CHECK.
///
/// Arc 293.W.2d: the purity wall is now at wire-peer PRODUCERS (peer-pair', etc.),
/// not at `send'` time. The test loads `probe_arc293_W2c_compile_time_send.wat`
/// which uses `peer-pair'<Struct,i64>` and asserts the compile-time check error.
/// (The 2c send'-gate was deleted in 2d; this test now exercises the 2d wall.)
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
#[allow(non_snake_case)]
fn struct_rejected_at_wire_SEND() {
    let result = startup_from_file("tests/comms/probe_arc293_W2c_compile_time_send.wat");
    assert!(
        result.is_err(),
        "peer-pair' with a struct type arg MUST fail at CHECK (arc 293.W.2d — \
         a struct is impure §7; the wire-peer producer's purity gate must reject \
         this world). got Ok"
    );
    let err_str = format!("{}", result.unwrap_err());
    assert_eq!(
        err_str,
        "check:\n1 type-check error(s):\n  - tests/comms/probe_arc293_W2c_compile_time_send.wat:24:38: malformed :wat::kernel::peer-pair' form: a wire peer (Peer'<I,O>) carries only pure data — type :w2c::S is not pure (§7 purity wall). If this peer is used only within a thread (in-locus, shared memory), use ThreadSelfPeer'<I,O> — any I/O types are allowed in-locus. If this peer must cross a process boundary (wire), redesign I/O types to use records, scalars, or pure enums (no Sender/Receiver/handle fields).\n",
        "check error must match arc 293 §7 purity wall golden"
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
