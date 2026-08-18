//! `#wat.rete/Export` — compiled program from source; import fires native.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn source_session_derives_one_hit() {
    let v = call_beside_value(file!(), ":user::source-hits").expect("source fire");
    assert_eq!(v, Value::i64(1), "Temp 10 is cool, Temp 30 is not");
}

#[test]
fn imported_export_derives_the_same_hit() {
    let v = call_beside_value(file!(), ":user::import-hits").expect("import fire");
    assert_eq!(
        v,
        Value::i64(1),
        "imported Export must fire the same as the source Session"
    );
}

#[test]
fn export_edn_is_smaller_than_session() {
    let v = call_beside_value(file!(), ":user::export-sizes").expect("size");
    let (sl, el) = match v {
        Value::wat__core__PersistentVector(pv) => {
            let a = match pv.get(0) {
                Some(Value::i64(n)) => *n,
                _ => panic!("session size missing: {pv:?}"),
            };
            let b = match pv.get(1) {
                Some(Value::i64(n)) => *n,
                _ => panic!("export size missing: {pv:?}"),
            };
            (a, b)
        }
        other => panic!("expected [session-len export-len], got {other:?}"),
    };
    assert!(
        el < sl,
        "packed Export must be smaller than a Session dump (session={sl} export={el})"
    );
}

#[test]
fn imported_strat_neg_matches_source() {
    let src = call_beside_value(file!(), ":user::strat-source-counts").expect("source strat");
    let imp = call_beside_value(file!(), ":user::strat-import-counts").expect("import strat");
    assert_eq!(
        src, imp,
        "imported negation-over-derived must match source fire (want Bad=1 Ok=1, not Ok=2)"
    );
    match &src {
        Value::wat__core__PersistentVector(pv) => {
            assert_eq!(pv.get(0), Some(&Value::i64(1)), "Bad count");
            assert_eq!(pv.get(1), Some(&Value::i64(1)), "Ok count");
        }
        other => panic!("expected [bad ok] counts, got {other:?}"),
    }
}

#[test]
fn edn_write_read_import_fires() {
    let v = call_beside_value(file!(), ":user::edn-roundtrip-hits").expect("edn roundtrip");
    assert_eq!(
        v,
        Value::i64(1),
        "Export must survive edn write/read and still fire"
    );
}
