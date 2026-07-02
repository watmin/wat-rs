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

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

/// make-channel's receiver must be comms-backed (the flip's fingerprint).
#[test]
fn probe_1_make_channel_receiver_is_comms_backed() {
    let world = startup_bare()
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:wat::kernel::make-channel :wat::core::i64)")
        .expect("parse make-channel");
    let env = Environment::new();
    let pair = eval_in_frozen(&ast, &world, &env)
        .expect("make-channel must evaluate")
        .value_owned();
    let Value::Tuple(xs) = pair else {
        panic!("make-channel returns a (Sender, Receiver) tuple; got {:?}", pair);
    };
    let Value::wat__kernel__Receiver(inner) = &xs[1] else {
        panic!("tuple slot 1 must be the Receiver; got {:?}", xs[1]);
    };
    let dbg = format!("{:?}", inner);
    assert_eq!(
        dbg,
        "Comms(Receiver::Channel(Receiver { .. }))",
        "make-channel's Receiver must be comms::thread-backed post-flip"
    );
}

/// Same fingerprint on the Sender side.
#[test]
fn probe_2_make_channel_sender_is_comms_backed() {
    let world = startup_bare()
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:wat::kernel::make-channel :wat::core::i64)")
        .expect("parse make-channel");
    let env = Environment::new();
    let pair = eval_in_frozen(&ast, &world, &env)
        .expect("make-channel must evaluate")
        .value_owned();
    let Value::Tuple(xs) = pair else {
        panic!("make-channel returns a (Sender, Receiver) tuple; got {:?}", pair);
    };
    let Value::wat__kernel__Sender(inner) = &xs[0] else {
        panic!("tuple slot 0 must be the Sender; got {:?}", xs[0]);
    };
    let dbg = format!("{:?}", inner);
    assert_eq!(
        dbg,
        "Comms { sender: Sender { inner: Sender { .. } }, closed: false }",
        "make-channel's Sender must be comms::thread-backed post-flip"
    );
}
