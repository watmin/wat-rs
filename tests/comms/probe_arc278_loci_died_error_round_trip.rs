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

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn recv_lost_cause_is_a_matchable_loci_died_error() {
    let v = call_beside(file!(), ":my::died-cause-panic-message")
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
    // The Panic message IS the raised Fault rendered to EDN. Parse it (structure-exact,
    // never a loose `contains`) and assert the Fault's `:message` field carries the raised
    // data verbatim — the Fault's `:location` field embeds a host-absolute path (via
    // `:wat::kernel::here`) we deliberately ignore by keying on the `:message` field only.
    let fault = wat_edn::parse_owned(&msg).expect("Panic message is the raised Fault's EDN");
    let body = match &fault {
        wat_edn::OwnedValue::Tagged(tag, body) => {
            assert_eq!(tag.namespace(), "wat.core", "raised value tag namespace; got {:?}", tag);
            assert_eq!(tag.name(), "Fault", "raised value is a Fault; got tag {:?}", tag);
            body
        }
        other => panic!("expected a #wat.core/Fault tagged value; got {:?}", other),
    };
    let pairs = match &**body {
        wat_edn::OwnedValue::Map(pairs) => pairs,
        other => panic!("the Fault body is a map; got {:?}", other),
    };
    let fault_message = pairs
        .iter()
        .find_map(|(k, v)| match (k, v) {
            (wat_edn::OwnedValue::Keyword(kw), wat_edn::OwnedValue::String(s))
                if kw.name() == "message" =>
            {
                Some(s.to_string())
            }
            _ => None,
        })
        .expect("the Fault carries a :message field");
    assert_eq!(
        fault_message, "loci-died-panic-data",
        "the LociDiedError::Panic message must carry the raised Fault's data verbatim"
    );
}
