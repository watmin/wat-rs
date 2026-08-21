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
fn reexport_shape_matches_source_export() {
    let v = call_beside_value(file!(), ":user::reexport-shape").expect("reexport shape");
    let ns = match v {
        Value::wat__core__PersistentVector(pv) => pv
            .iter()
            .map(|x| match x {
                Value::i64(n) => *n,
                other => panic!("expected i64, got {other:?}"),
            })
            .collect::<Vec<_>>(),
        other => panic!("expected shape vector, got {other:?}"),
    };
    assert_eq!(ns.len(), 8, "deps/nodes/conds/rhs × 2");
    assert_eq!(ns[0], ns[1], "deps length e1 vs e2: {ns:?}");
    assert_eq!(ns[2], ns[3], "nodes length e1 vs e2: {ns:?}");
    assert_eq!(ns[4], ns[5], "conds length e1 vs e2: {ns:?}");
    assert_eq!(ns[6], ns[7], "rhs length e1 vs e2: {ns:?}");
    assert!(ns[0] > 0 && ns[2] > 0 && ns[4] > 0, "export must pack live circuits: {ns:?}");
}

#[test]
fn reexport_keeps_deps() {
    let v = call_beside_value(file!(), ":user::reexport-deps-length").expect("reexport deps");
    match v {
        Value::i64(n) => assert!(n > 0, "export(import(e)) must pack arm.rule_deps, not empty rules AST; got {n}"),
        other => panic!("expected i64 deps length, got {other:?}"),
    }
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
// rune:vocare(vantage-bypass-test) — ABI refuse is a host Aggregate.fields poke; wat has no Export field setter
fn import_refuses_abi_mismatch() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let i = a
                .names
                .iter()
                .position(|n| n == "abi")
                .expect("Export named abi field");
            fields[i] = Value::String(Arc::new("v1:deadbeefdeadbeef".into()));
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
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
// rune:vocare(vantage-bypass-test) — empty-deps wall is proven by host Aggregate.fields swap; no wat Export setter
fn empty_deps_import_refuses_fire() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let emptied = match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let i = a
                .names
                .iter()
                .position(|n| n == "deps")
                .expect("Export named deps field");
            fields[i] = call_beside_value(file!(), ":user::empty-pv").expect("empty pv");
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
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
    let session = apply_function(
        import,
        vec![emptied],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("empty-deps Export still imports (deps live on the arm)");
    let seed = world
        .symbols()
        .get(":exp::seed")
        .expect("seed")
        .clone();
    let seeded = apply_function(
        seed,
        vec![session],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("seed");
    let fire = world
        .symbols()
        .get(":wat::rete::fire-rules")
        .expect("fire-rules")
        .clone();
    let err = apply_function(
        fire,
        vec![seeded],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect_err("empty-deps Import with live productions must refuse fire");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("cannot consume an Export without interned stratify schedule"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span line; wall name is the contract
        "empty-deps fire must refuse the Export-without-arm wall, got {msg}"
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

#[test]
fn reexport_edn_is_identical() {
    let v = call_beside_value(file!(), ":user::reexport-edn-identical").expect("reexport edn");
    assert_eq!(
        v,
        Value::bool(true),
        "edn-write(e) must equal edn-write(export(import(e)))"
    );
}

#[test]
fn reexport_import_fires() {
    let v = call_beside_value(file!(), ":user::reexport-import-fires").expect("reexport fire");
    assert_eq!(
        v,
        Value::i64(1),
        "import(export(import(e))) must fire the same Hit"
    );
}
