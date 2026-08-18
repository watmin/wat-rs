//! fn-headed :then produces Hit; exists over that derived Hit must fire.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn fn_headed_then_produces_type_another_rule_consumes() {
    let v = call_beside_value(file!(), ":user::source-counts").expect("source");
    let (hits, seen) = match v {
        Value::wat__core__PersistentVector(pv) => {
            let h = match pv.get(0) {
                Some(Value::i64(n)) => *n,
                other => panic!("Hit count: {other:?}"),
            };
            let s = match pv.get(1) {
                Some(Value::i64(n)) => *n,
                other => panic!("Seen count: {other:?}"),
            };
            (h, s)
        }
        other => panic!("expected [hits seen], got {other:?}"),
    };
    assert_eq!(hits, 1, "fn-headed :then must insert a Hit");
    assert_eq!(
        seen, 1,
        "exists over fn-produced Hit must see it (produced type is T, not the fn)"
    );
}
