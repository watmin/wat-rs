//! Arc 214 Slice 1 smoke probe — verify foundation primitives compile + a
//! sample EdnRepresentable impl roundtrips + error types behave honestly.
//!
//! Arc 294.h: `HolonRepresentable` (the holographic supertrait) is deleted —
//! it had zero production consumers. `EdnRepresentable` is, and always was,
//! the wire contract; `ToyType` is re-pointed directly at it with a plain-EDN
//! `to_wire` (no HolonAST IR).

use wat::comms::{EdnRepresentable, WireError};

// Sample impl — verifies the shape is usable. ToyType wraps an i64;
// roundtrips via a plain EDN integer literal — the honest minimum the wire
// needs, mirroring how `String` and `Value` are the only production
// `EdnRepresentable` impls.
#[derive(Debug)]
struct ToyType(i64);

impl EdnRepresentable for ToyType {
    fn to_wire(&self) -> String {
        self.0.to_string()
    }
    fn from_wire(s: &str) -> Result<Self, WireError> {
        s.trim()
            .parse::<i64>()
            .map(ToyType)
            .map_err(|e| WireError::new(format!("ToyType from_wire: expected an EDN i64, got {:?} ({e})", s)))
    }
}

#[test]
fn probe_slice1_edn_representable_compiles() {
    let t = ToyType(42);
    let wire = t.to_wire();
    let t2 = ToyType::from_wire(&wire).expect("roundtrip");
    assert_eq!(t.0, t2.0);
}

#[test]
fn probe_slice1_edn_representable_from_wire_is_honest_on_garbage() {
    // Error-honesty property: a malformed wire string produces a WireError
    // naming what was expected, not a panic or a silently wrong value.
    let err = ToyType::from_wire("not-an-i64").expect_err("garbage should fail, not decode");
    assert_eq!(
        err.message(),
        "ToyType from_wire: expected an EDN i64, got \"not-an-i64\" (invalid digit found in string)",
        "error message should name the failing conversion exactly"
    );
}

#[test]
fn probe_slice1_send_error_carries_unsent_value() {
    // SendError holds the unsent value for caller recovery (crossbeam pattern).
    // Arc 278 send-mirrors-recv: SendError is now a four-variant enum (mirroring
    // RecvError) — every arm still carries the unsent value.
    let s = wat::comms::SendError::Disconnected(42i64);
    match s {
        wat::comms::SendError::Disconnected(v) => assert_eq!(v, 42),
        _ => panic!("expected Disconnected"),
    }
}

#[test]
fn probe_slice1_recv_error_is_two_variant_enum() {
    // Arc 214 ε — RecvError is a two-variant enum: the recv arm knows WHICH
    // cause fired. `Disconnected` = all senders dropped / peer closed the
    // write-end (the data EOF arm); `Shutdown` = the substrate shutdown cascade
    // fired (the broadcast / SHUTDOWN_RX arm). The old unit struct collapsed
    // both into one and forced channel/transfer to re-derive the distinction
    // with a SHUTDOWN_RX peek — the peek the select had already computed and
    // thrown away. The enum carries the cause the source already knows.
    let _d = wat::comms::RecvError::Disconnected;
    let _s = wat::comms::RecvError::Shutdown;
}

#[test]
fn probe_slice1_wire_error_carries_diagnostic_text() {
    // WireError field is private; constructed via new(impl Into<String>);
    // text retrieved via message() accessor.
    let w = wat::comms::WireError::new("wire-test");
    assert_eq!(w.message(), "wire-test");
}

#[test]
fn probe_slice1_select_outcome_constructs() {
    use wat::comms::{ReceiverIndex, RecvError, SelectOutcome};

    // Successful recv from a specific receiver index.
    let ok: SelectOutcome<i64> = SelectOutcome::Recv {
        index: ReceiverIndex(0),
        result: Ok(42),
    };
    match ok {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, ReceiverIndex(0));
            assert_eq!(result, Ok(42));
        }
        SelectOutcome::Shutdown => panic!("expected Recv"),
        SelectOutcome::Listener => panic!("expected Recv"),
    }

    // Disconnected recv (the fired receiver's senders all dropped — the data arm).
    let err: SelectOutcome<i64> = SelectOutcome::Recv {
        index: ReceiverIndex(1),
        result: Err(RecvError::Disconnected),
    };
    match err {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, ReceiverIndex(1));
            assert_eq!(result, Err(RecvError::Disconnected));
        }
        SelectOutcome::Shutdown => panic!("expected Recv"),
        SelectOutcome::Listener => panic!("expected Recv"),
    }

    // Substrate-shutdown cascade fired before any data receiver.
    let shutdown: SelectOutcome<i64> = SelectOutcome::Shutdown;
    assert!(matches!(shutdown, SelectOutcome::Shutdown));
}
