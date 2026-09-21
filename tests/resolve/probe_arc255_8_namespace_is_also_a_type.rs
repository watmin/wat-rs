//! 255.8 — `my.Journal/Req` is a nested type, not a member of `Journal`.
//! `Option/expect` stays a member. Both in one file.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn nested_type_name_and_member_call_both_resolve() {
    let world_ok = wat::freeze::startup_beside(file!());
    assert!(
        world_ok.is_ok(),
        "clojure my.Journal/Req and colon :my::Journal::Req must both check: {world_ok:?}"
    );
    let got = call_beside_value(file!(), ":user::member")
        .unwrap_or_else(|e| panic!("Option/expect must still be a member call: {e:?}"));
    assert_eq!(got, Value::i64(7));
}
