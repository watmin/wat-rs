//! :exists and acc :from a type this fire derives (raised by :not of Bad).
//! Native fire and Export import must both see Bad=1 Ok=1 Seen=1 Tally=1.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn four_i64(v: Value) -> (i64, i64, i64, i64) {
    match v {
        Value::wat__core__PersistentVector(pv) => {
            let at = |i: usize| match pv.get(i) {
                Some(Value::i64(n)) => *n,
                other => panic!("count {i} missing: {other:?}"),
            };
            (at(0), at(1), at(2), at(3))
        }
        other => panic!("expected [bad ok seen tally], got {other:?}"),
    }
}

fn assert_closure((bad, ok, seen, tally): (i64, i64, i64, i64), via: &str) {
    assert_eq!(bad, 1, "{via}: Bad for k=2 only");
    assert_eq!(ok, 1, "{via}: Ok for k=1 only");
    assert_eq!(
        seen, 1,
        "{via}: :exists over derived Ok (raised) must see it"
    );
    assert_eq!(
        tally, 1,
        "{via}: acc :from derived Ok (raised) must count 1"
    );
}

#[test]
fn derived_exists_and_acc_native_closure() {
    let v = call_beside_value(file!(), ":user::source-counts").expect("source");
    assert_closure(four_i64(v), "native");
}

#[test]
fn derived_exists_and_acc_spec_matches_native() {
    let nat = call_beside_value(file!(), ":user::source-counts").expect("native");
    let spec = call_beside_value(file!(), ":user::spec-counts").expect("spec");
    assert_eq!(nat, spec, "oracle must match native on derived exists/acc");
    assert_closure(four_i64(spec), "spec");
}

#[test]
fn derived_exists_and_acc_imported_export() {
    let src = call_beside_value(file!(), ":user::source-counts").expect("source");
    let imp = call_beside_value(file!(), ":user::import-counts").expect("import");
    assert_eq!(src, imp, "imported Export must match source closure");
    assert_closure(four_i64(imp), "import");
}
