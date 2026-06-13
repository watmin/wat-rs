//! Arc 209 Stone C0b.2e-i-a gate — `CommSender`/`CommReceiver` usable as trait
//! objects over BOTH tiers, with an explicit `ReactorClass` discriminant and
//! `Any`-downcast back to the concrete receiver (the i-b `select'` bridge).
//!
//! RED at HEAD: `close(self)` lacks `where Self: Sized`, so neither trait is
//! object-safe and `Box<dyn Comm*<Value>>` will not compile; `ReactorClass`,
//! `reactor_class`, and `as_any` do not exist yet. GREEN after the i-a
//! foundation lands. This proves every mechanism i-b depends on: boxing over
//! both tiers, the named reactor-class discriminant, and concrete recovery.

use wat::comms::{CommReceiver, CommSender, ReactorClass};
use wat::value::Value;

#[test]
fn probe_arc209_c0b2eia_boxed_comm_both_tiers() {
    // ── in-memory (crossbeam) tier ──────────────────────────────────────────
    let (t_tx, t_rx) = wat::comms::thread::pair::<Value>();
    let bt_tx: Box<dyn CommSender<Value>> = Box::new(t_tx);
    let bt_rx: Box<dyn CommReceiver<Value>> = Box::new(t_rx);
    bt_tx.send(Value::i64(7)).expect("boxed crossbeam send");
    assert_eq!(
        bt_rx.recv().expect("boxed crossbeam recv"),
        Value::i64(7),
        "Value round-trips through a boxed crossbeam Sender/Receiver"
    );
    assert!(
        matches!(bt_rx.reactor_class(), ReactorClass::InMemory),
        "thread receiver reports the in-memory reactor class"
    );
    assert!(
        bt_rx
            .as_any()
            .downcast_ref::<wat::comms::thread::Receiver<Value>>()
            .is_some(),
        "boxed crossbeam receiver downcasts to concrete thread::Receiver (i-b select' bridge)"
    );

    // ── fd-backed (process pipe) tier ───────────────────────────────────────
    let (p_tx, p_rx) = wat::comms::process::pair::<Value>().expect("process pair::<Value>()");
    let bp_tx: Box<dyn CommSender<Value>> = Box::new(p_tx);
    let bp_rx: Box<dyn CommReceiver<Value>> = Box::new(p_rx);
    bp_tx.send(Value::i64(42)).expect("boxed fd send");
    assert_eq!(
        bp_rx.recv().expect("boxed fd recv"),
        Value::i64(42),
        "Value round-trips through a boxed process Sender/Receiver (encodes internally)"
    );
    assert!(
        matches!(bp_rx.reactor_class(), ReactorClass::Fd),
        "process receiver reports the fd reactor class"
    );
    assert!(
        bp_rx
            .as_any()
            .downcast_ref::<wat::comms::process::Receiver<Value>>()
            .is_some(),
        "boxed fd receiver downcasts to concrete process::Receiver (i-b select' bridge)"
    );
}
