//! Arc 214 Slice 1 smoke probe — verify foundation primitives compile + a
//! sample HolonRepresentable impl roundtrips + error types behave honestly.

use wat::comms::{HolonRepresentable, WireError};

// Sample impl — verifies the shape is usable. ToyType wraps an i64;
// roundtrips via HolonAST::I64 (I64 is a leaf variant — no nested structure).
struct ToyType(i64);

impl HolonRepresentable for ToyType {
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::I64(self.0)
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError> {
        match ast {
            holon::HolonAST::I64(n) => Ok(ToyType(*n)),
            other => Err(WireError::new(format!(
                "ToyType::from_holon_ast: expected I64 variant, got {:?}",
                other
            ))),
        }
    }
}

#[test]
fn probe_slice1_holon_representable_compiles() {
    let t = ToyType(42);
    let ast = t.to_holon_ast();
    let t2 = ToyType::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(t.0, t2.0);
}

#[test]
fn probe_slice1_send_error_carries_unsent_value() {
    // SendError holds the unsent value for caller recovery (crossbeam pattern).
    let s = wat::comms::SendError(42i64);
    assert_eq!(s.0, 42);
}

#[test]
fn probe_slice1_recv_error_is_unit_struct() {
    // RecvError is a unit struct (no payload; senders dropped or shutdown fired).
    let _r = wat::comms::RecvError;
}

#[test]
fn probe_slice1_try_recv_error_variants_are_distinct() {
    // TryRecvError variants MUST be distinguishable — drives retry-vs-bail-out
    // logic at every try_recv site (Empty → may become ready; Disconnected →
    // never will). Conflation is a deadlock vector.
    assert_ne!(
        wat::comms::TryRecvError::Empty,
        wat::comms::TryRecvError::Disconnected
    );
}

#[test]
fn probe_slice1_close_error_carries_diagnostic_text() {
    // CloseError field is private; constructed via new(impl Into<String>);
    // text retrieved via message() accessor.
    let c = wat::comms::CloseError::new("close-test");
    assert_eq!(c.message(), "close-test");
}

#[test]
fn probe_slice1_wire_error_carries_diagnostic_text() {
    // WireError field is private; same new()/message() pattern as CloseError.
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
    }

    // Disconnected recv (the fired receiver's senders all dropped).
    let err: SelectOutcome<i64> = SelectOutcome::Recv {
        index: ReceiverIndex(1),
        result: Err(RecvError),
    };
    match err {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, ReceiverIndex(1));
            assert_eq!(result, Err(RecvError));
        }
        SelectOutcome::Shutdown => panic!("expected Recv"),
    }

    // Substrate-shutdown cascade fired before any data receiver.
    let shutdown: SelectOutcome<i64> = SelectOutcome::Shutdown;
    assert!(matches!(shutdown, SelectOutcome::Shutdown));
}
