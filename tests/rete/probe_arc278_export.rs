//! `#wat.rete/Export` — compiled program from source; import fires native.

use std::sync::Arc;

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::{apply_function, Value};
use wat::AggregateValue;

#[test]
fn source_session_derives_one_hit() {
    let v = call_beside_value(file!(), ":user::source-hits").expect("source fire");
    assert_eq!(v, Value::i64(1), "Temp 10 is cool, Temp 30 is not");
}

#[test]
fn spec_refuses_imported_export() {
    let panicked = std::panic::catch_unwind(|| {
        call_beside_value(file!(), ":user::spec-on-import")
    });
    match panicked {
        Err(_) => {}
        Ok(Ok(v)) => panic!(
            "oracle must refuse an Export, not return {v:?} (silent empty is the lie)"
        ),
        Ok(Err(e)) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("oracle cannot consume") || msg.contains("Export"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span line; wall name is the contract
                "refuse must name the wall, got {msg}"
            );
        }
    }
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
fn import_refuses_abi_mismatch() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            fields[1] = Value::String(Arc::new("v1:deadbeefdeadbeef".into()));
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.clone(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    };
    let import = world
        .symbols()
        .get(":user::import-one")
        .expect("import-one")
        .clone();
    let err = apply_function(
        import,
        vec![tampered],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect_err("tampered ABI must refuse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("ABI mismatch"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; ABI mismatch is the contract
        "import must name ABI mismatch, got {msg}"
    );
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
