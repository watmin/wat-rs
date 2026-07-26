//! Arc 278 — the string-wrap annihilation RED gate (four-questions-ratified Fork B).
//!
//! A peer raises a structured `:wat::core::Error` (a `Fault`). The death crosses
//! back as `RecvOutcome::Lost[LociDiedError::Panic{.., failure}]`, and the
//! `Failure` carries the raised Error in a MANDATORY structured `error` field —
//! NOT `edn::write`'d into a String the consumer must `edn::read` back out.
//!
//! The fixture reads `(:wat::kernel::Failure/error f)` → the `Fault` RECORD →
//! `Fault/message` off it. No string round-trip anywhere. The recovered value is
//! the Fault's message field, read structurally, so it must equal the raised
//! message EXACTLY.
//!
//! RED today: `:wat::kernel::Failure` has no `error` field, so `Failure/error`
//! is an unknown accessor (a runtime `UnknownFunction` — `--check` defers a
//! callee-in-call-position, so this only goes RED at test-run time). GREEN once
//! the stone lands.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn failure_carries_the_raised_error_as_a_structured_record() {
    let v = call_beside_value(file!(), ":my::failure-error-is-structured")
        .expect("failure-error-is-structured should run");
    let inner = match v {
        Value::Option(opt) => match &*opt {
            Some(inner) => inner.clone(),
            None => panic!("expected Some(structured Fault message), got :None"),
        },
        other => panic!("expected Option, got {:?}", other),
    };
    let msg = match &inner {
        Value::String(s) => s.clone(),
        other => panic!("Fault/message read off Failure/error should be a String; got {:?}", other),
    };
    // Read STRUCTURALLY off `Failure/error` (a `:wat::core::Error` record) — the
    // wat side already pulled the `:message` field off the reconstructed Fault,
    // so this is the plain message with no EDN string to re-parse. It must equal
    // the raised message exactly — the Error survived the boundary as a record.
    assert_eq!(
        msg.as_str(),
        "structured-error-data",
        "the raised Fault's message must be readable structurally off Failure/error — no string-wrap"
    );
}
