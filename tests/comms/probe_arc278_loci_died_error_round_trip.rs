//! Arc 278 — the LociDiedError RED gate (DESIGN-loci-died-error.md).
//!
//! A peer dies abnormally (raises a `Fault`). `recv'` hands the owner
//! `RecvOutcome::Lost[cause]`. The ratified UX (four-questions Q1) is that
//! `cause` is a LOCI-AGNOSTIC `:wat::kernel::LociDiedError` — a matchable
//! death report whose `Panic` variant carries the structured reason — NOT
//! the flat `:wat::kernel::Failure` it is today.
//!
//! The fixture matches `cause` as a `LociDiedError` and reads the panic
//! message. RED today (the enum is unregistered; `Lost`'s cause is `Failure`
//! — a check-time error). GREEN once the stone lands: `cause` is a
//! `LociDiedError`, `Panic` matches, and the death report round-trips as a
//! registered record (EDN all the way down).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn recv_lost_cause_is_a_matchable_loci_died_error() {
    let v = call_beside_value(file!(), ":my::died-cause-panic-message")
        .expect("died-cause-panic-message should run");
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(panic message) from the LociDiedError::Panic arm, got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    let msg = match &inner {
        Value::String(s) => s.clone(),
        other => panic!("LociDiedError::Panic message should be a String; got {:?}", other),
    };
    // Arc 278 the string-wrap annihilation — `LociDiedError::Panic.message` is now the
    // HUMAN message (the raised error's own `:message` field), NOT the Fault's EDN. The
    // structured Fault rides on the Panic's `failure` (→ `Failure/error`), proven by the
    // sibling gate `probe_arc278_failure_carries_structured_error`. Here the message is the
    // plain string, read verbatim — no EDN re-parse (the string-wrap is dead).
    assert_eq!(
        msg.as_str(),
        "loci-died-panic-data",
        "the LociDiedError::Panic message must carry the raised error's human message verbatim"
    );
}
