//! #49 — compiled `where`: lower at rule-compile; native fire uses the circuit.
//! Oracle `eval-test` stays; a cool-Temp rule must still derive one Hit.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn lower_accepts_a_comparison() {
    let v = call_beside_value(file!(), ":user::cmp-lower-ok").expect("lower");
    assert!(matches!(v, Value::Unit), "lower returns nil, got {v:?}");
}

#[test]
fn eval_test_still_the_oracle() {
    let v = call_beside_value(file!(), ":user::cmp-eval").expect("eval-test");
    assert_eq!(v, Value::bool(true));
}

#[test]
fn compiled_where_fires_the_cool_rule() {
    let v = call_beside_value(file!(), ":user::fire-cool").expect("fire");
    assert_eq!(v, Value::i64(1), "Temp 10 is cool, Temp 30 is not — one Hit");
}
