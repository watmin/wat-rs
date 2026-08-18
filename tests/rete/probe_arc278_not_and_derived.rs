//! :not of :and whose leaf is derived Bad. Want Bad=1 Ok=1, not Ok=2.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn pair_i64(v: Value) -> (i64, i64) {
    match v {
        Value::wat__core__PersistentVector(pv) => {
            let a = match pv.get(0) {
                Some(Value::i64(n)) => *n,
                other => panic!("Bad count missing: {other:?}"),
            };
            let b = match pv.get(1) {
                Some(Value::i64(n)) => *n,
                other => panic!("Ok count missing: {other:?}"),
            };
            (a, b)
        }
        other => panic!("expected [bad ok], got {other:?}"),
    }
}

#[test]
fn not_and_over_derived_is_stratified() {
    let (bad, ok) = pair_i64(call_beside_value(file!(), ":user::source-counts").expect("source"));
    assert_eq!(bad, 1, "Bad for k=2 only");
    assert_eq!(ok, 1, ":not of :and(derived Bad) must raise — Ok=1, not Ok=2");
}

#[test]
fn not_and_over_derived_spec_matches_native() {
    let nat = call_beside_value(file!(), ":user::source-counts").expect("native");
    let spec = call_beside_value(file!(), ":user::spec-counts").expect("spec");
    assert_eq!(nat, spec, "oracle must match native on :not of :and(derived)");
    let (bad, ok) = pair_i64(spec);
    assert_eq!(bad, 1, "spec Bad");
    assert_eq!(ok, 1, "spec Ok");
}

#[test]
fn not_and_over_derived_imported_export() {
    let src = call_beside_value(file!(), ":user::source-counts").expect("source");
    let imp = call_beside_value(file!(), ":user::import-counts").expect("import");
    assert_eq!(src, imp, "imported :not-and-derived must match source");
    let (bad, ok) = pair_i64(imp);
    assert_eq!(bad, 1, "import Bad");
    assert_eq!(ok, 1, "import Ok");
}
