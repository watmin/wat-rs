//! Arc 214 Stone 5.1 — channel substrate flip (FM-2-bis disconfirming probe).
//!
//! `make-channel`'s memory tier re-backs onto `comms::thread` UNDERNEATH the
//! wat `Sender`/`Receiver` values — all ~251 old-verb call sites unchanged
//! (the corpus is the real regression gate; this probe pins the flip itself).
//!
//! The assertion is deliberately a STRING match on the inner enum's Debug
//! render: at HEAD the backing variant is `Crossbeam` (RED); post-flip it is
//! the Comms backing (GREEN) — and because the HARD CUT deletes the Crossbeam
//! variant entirely, a `matches!` probe would stop COMPILING post-flip; the
//! Debug-string form compiles in both worlds.
//!
//! Run: `cargo test --release --test channel probe_arc214_stone51_channel_substrate_flip`

use wat::freeze::call_beside;
use wat::runtime::Value;

/// make-channel's receiver must be comms-backed (the flip's fingerprint).
///
/// just-eval (rubric): make-channel's call lives in the co-located fixture's zero-arg
/// `:user::compute-receiver`, driven via `call_beside` — no inline wat driver.
#[test]
fn probe_1_make_channel_receiver_is_comms_backed() {
    let rx = call_beside(file!(), ":user::compute-receiver").expect("make-channel must evaluate");
    let Value::wat__kernel__Receiver(inner) = &rx else {
        panic!("compute-receiver must return the Receiver; got {:?}", rx);
    };
    let dbg = format!("{:?}", inner);
    assert_eq!(
        dbg,
        "Comms(Receiver::Channel(Receiver { .. }))",
        "make-channel's Receiver must be comms::thread-backed post-flip"
    );
}

/// Same fingerprint on the Sender side.
///
/// just-eval (rubric): make-channel's call lives in the co-located fixture's zero-arg
/// `:user::compute-sender`, driven via `call_beside` — no inline wat driver.
#[test]
fn probe_2_make_channel_sender_is_comms_backed() {
    let tx = call_beside(file!(), ":user::compute-sender").expect("make-channel must evaluate");
    let Value::wat__kernel__Sender(inner) = &tx else {
        panic!("compute-sender must return the Sender; got {:?}", tx);
    };
    let dbg = format!("{:?}", inner);
    assert_eq!(
        dbg,
        "Comms { sender: Sender { inner: Sender { .. } }, closed: false }",
        "make-channel's Sender must be comms::thread-backed post-flip"
    );
}
