//! a-fired-deadline-hands-back-a-live-peer — redials are visible; a dead
//! peer answers Lost, never DeadlineFired.
//!
//! cargo nextest run --release -E 'test(fired_deadline)'

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn as_string(v: Value, who: &str) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("{who} expected String, got {other:?}"),
    }
}

#[test]
fn fired_deadline_redials_are_visible_and_the_handle_serves() {
    let s = as_string(
        call_beside_value(file!(), ":user::redials-and-own-tag")
            .unwrap_or_else(|e| panic!("redials-and-own-tag raised: {e:?}")),
        "redials-and-own-tag",
    );
    assert_eq!(
        s.as_str(),
        "before=0;first=DeadlineFired;after=1;second=Answered:22",
        "a fired deadline re-establishes the handle (redials 0→1) and the next call on the SAME binding gets its own tag"
    );
}

#[test]
fn fired_deadline_dead_peer_is_lost_not_deadline_fired() {
    let s = as_string(
        call_beside_value(file!(), ":user::dead-is-lost")
            .unwrap_or_else(|e| panic!("dead-is-lost raised: {e:?}")),
        "dead-is-lost",
    );
    assert!(
        s == "Lost" || s == "Closed",
        "a dead peer must not be reported as a repair; got {s:?} (DeadlineFired would be the lie)"
    );
    assert_ne!(
        s.as_str(),
        "DeadlineFired",
        "DeadlineFired now promises a live handle"
    );
}
