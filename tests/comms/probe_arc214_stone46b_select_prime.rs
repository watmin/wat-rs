//! Arc 214 Stone 4.6b — `select'` (FM-2-bis disconfirming probe).
//!
//! `select' : Vector<peer<I,O>> -> ServiceEvent<I,O>` — blocking first-ready
//! multiplex over same-tier peers, returning a ServiceEvent. Intrinsic —
//! projective (I,O flow from the element peer type; docs/DISPATCH.md).
//! Mixed-tier selection needs no bespoke rejection: Vector homogeneity
//! already makes it unrepresentable at check.
//!
//! Stone 259 Lost-locus: select' returns ServiceEvent (was Tuple<i64,O>);
//! probe 1 matches on :Message{idx, msg}; probe 2 checks wrong-return still
//! fails (ServiceEvent<i64,i64> ≠ String).
//!
//! ## Arc 259 S2c-ii-a — apply-loop PURGE
//!
//! Both probes' spawn progs are SWAPPED to self-peer form
//! `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))`.
//! The `Thread'<i64,i64>` peer type is preserved — `Peer'<O,I>=Peer'<i64,i64>`
//! → `Thread'<R,S>=Thread'<I,O>=Thread'<i64,i64>`. The `select'` multiplex
//! is unchanged; only the spawned progs swap.
//!
//! Run: `cargo test --release --test comms probe_arc214_stone46b_select_prime`

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// ─── Probe 1 (LOAD-BEARING, RUNTIME): select' picks the ready peer ────────────

/// Two thread echo peers; send 7 to peer B ONLY (deterministic — only B will
/// ever have data); select' [a b] must return ServiceEvent::Message{idx=1, msg=7}:
/// index 1 (peer B's position) and the echoed value. Both peers closed after.
///
/// Stone 259: select' returns ServiceEvent<I,O> (was Tuple<i64,O>).
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; select' multiplex unchanged.
#[test]
fn probe_1_select_returns_ready_index_and_value() {
    let got = call_beside_value(file!(), ":user::compute")
        .expect("compute must evaluate (select' dispatch exists)");
    // Stone 259: select' returns ServiceEvent<I,O>; happy path is :Message{idx, msg}.
    match &got {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::spawn::ServiceEvent",
                "select' must return ServiceEvent; got type_path {:?}",
                ev.type_path
            );
            assert_eq!(
                ev.variant_name, "Message",
                "ready peer must yield :Message; got {:?}",
                ev.variant_name
            );
            assert_eq!(ev.fields.len(), 2, "Message must have idx + msg; got {:?}", ev.fields);
            assert_eq!(ev.fields[0], Value::i64(1), "ready peer is index 1 (b); got {:?}", ev.fields[0]);
            assert_eq!(ev.fields[1], Value::i64(7), "the echoed value; got {:?}", ev.fields[1]);
        }
        other => panic!("expected ServiceEvent::Message; got {:?}", other),
    }
}

// ─── Probe 2 (CHECK NEGATIVE): select' return type is ServiceEvent<I,O> ───────

/// Declaring the select' result as `:wat::core::String` over i64-peers MUST
/// fail at check — the projective return is `ServiceEvent<i64,i64>`, not String.
/// Stone 259: return type changed from Tuple<i64,O> to ServiceEvent<I,O>;
/// the wrong-annotation rejection still holds.
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; select' return-type rejection unchanged.
#[test]
fn probe_2_select_wrong_return_annotation_rejected() {
    let result = startup_from_file(
        "tests/comms/probe_arc214_stone46b_select_prime_probe2.wat.bad",
    );
    // Grounded via `--check`: this fixture actually raises FOUR check errors (a
    // DefRestrictedCallerNotAllowed and a TypeMismatch on `send` are coincidental fallout
    // of this probe's spawn-prog shape, not what the test targets). The one this test names —
    // "select' return typed as String" — is the ReturnTypeMismatch on `:user::bad`; membership
    // (not exclusivity) is what `assert_startup_error!`'s `check` arm proves, so the other three
    // don't need to be named. `got`'s trailing `:?NNNN` is a fresh unification-variable id
    // (confirmed non-deterministic across repeated `--check` runs: `:?2950`, `:?10`, `:?3098`),
    // so only the stable prefix up to it is asserted, not the whole string.
    wat::assert_startup_error!(result, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":user::bad"
            && expected == ":wat::core::String"
            && got.starts_with("(:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 :?")
    );
}
