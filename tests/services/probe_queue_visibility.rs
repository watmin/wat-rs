//! Visibility redelivery — the mechanism the durable-topic design rests on.
//!
//! An internal topic worker that cannot hand a message to a subscriber simply does not ack,
//! and relies on the message becoming visible again. `sqs.wat:62` states that intent but
//! nothing exercised it: the circuit sets `visibility-ns` to 10^12 ns so redelivery never
//! fires. This drives `wat-scripts/scratch-pad/probe-visibility-redelivers.wat`, which
//! checks all three parts on one message.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn call_string(name: &str) -> String {
    // The probe has a relative `load-file!` of ../queue/sqs.wat, so it needs the FsLoader —
    // same reason probe_async_publish::outbox_term_removed_loses_messages does this.
    let rel = "wat-scripts/scratch-pad/probe-visibility-redelivers.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let world = startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("visibility probe should freeze");
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{name} returned non-String: {other:?}"),
        Err(e) => panic!("{name} raised: {e:?}"),
    }
}

fn field<'a>(s: &'a str, key: &str) -> &'a str {
    for part in s.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("missing field {key:?}: {s}");
}

#[test]
fn unacked_message_is_redelivered_after_visibility_expiry() {
    let out = call_string(":user::compute");

    assert_eq!(field(&out, "first"), "got", "the message must arrive at all; got {out}");

    // The half that is easy to forget: while in flight it must be INVISIBLE. A queue that
    // hands the same message to two workers at once is not redelivering, it is losing the
    // guarantee the circuit's dup=0 invariant rests on.
    assert_eq!(
        field(&out, "while-inflight"),
        "none",
        "an in-flight message must not be handed out again; got {out}"
    );

    assert_eq!(
        field(&out, "after-expiry"),
        "got",
        "an unacked message must return after its visibility window; got {out}"
    );
    assert_eq!(
        field(&out, "same"),
        "yes",
        "the redelivered message must be the SAME one; got {out}"
    );
}
