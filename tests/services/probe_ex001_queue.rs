//! Excursus 001 stone 3 — SQS in userland. The queue lives at
//! `wat-scripts/queue/sqs.wat` (not stdlib). This harness drives `:user::compute`,
//! which runs the full lifecycle against mem-store and sqlite-store and returns
//! one summary iff they agree.
//!
//! `assert_eq!` of the whole summary — no `.contains(` (no_loose_string_assert).

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

const AGREED_SUMMARY: &str = "bound=x;r1=a,b;r2=c;r3=;redel=b";

#[test]
fn queue_lifecycle_mem_and_sqlite_agree() {
    let world = startup_from_file("wat-scripts/queue/sqs.wat")
        .expect("startup should succeed (queue + mem-store' + sqlite-store' baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("queue lifecycle raised: {e:?}"),
    };
    assert_eq!(
        stored, AGREED_SUMMARY,
        "send 3 / receive 2 / second receive is the third / ack a+c / unacked b reappears \
         after the window, and a message at exactly now is received. Both backends must \
         agree. A DIFFERENTIAL-MISMATCH prefix means they diverged. got: {stored}"
    );
}
