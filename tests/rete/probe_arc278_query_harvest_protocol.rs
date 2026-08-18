//! fire → query → insert → query without fire (stale) → fire → query grew.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn query_is_last_fire_harvest_insert_does_not_refresh() {
    let v = call_beside_value(file!(), ":user::protocol").expect("protocol");
    let (n1, n2, n3) = match v {
        Value::wat__core__PersistentVector(pv) => {
            let at = |i: usize| match pv.get(i) {
                Some(Value::i64(n)) => *n,
                other => panic!("slot {i} missing: {other:?}"),
            };
            (at(0), at(1), at(2))
        }
        other => panic!("expected [n1 n2 n3], got {other:?}"),
    };
    assert_eq!(n1, 1, "first fire parks one Hit");
    assert_eq!(n2, 1, "insert after fire must not refresh query-memory");
    assert_eq!(n3, 2, "second fire harvests both cool Temps");
}
