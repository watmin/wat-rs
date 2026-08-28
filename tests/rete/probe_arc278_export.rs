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
fn spec_once_refuses_imported_export() {
    let panicked = std::panic::catch_unwind(|| {
        call_beside_value(file!(), ":user::spec-once-on-import")
    });
    match panicked {
        Err(_) => {}
        Ok(Ok(v)) => panic!(
            "fire-once$oracle must refuse an Export, not return {v:?} (silent empty is the lie)"
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

fn import_one(world: &wat::freeze::FrozenWorld, exp: Value) -> Result<Value, wat::runtime::RuntimeError> {
    let import = world
        .symbols()
        .get(":user::import-one")
        .expect("import-one")
        .clone();
    apply_function(import, vec![exp], world.symbols(), wat::rust_caller_span!())
}

fn poke_named(exp: Value, field: &str, v: Value) -> Value {
    match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let i = a.names.iter().position(|n| n == field).expect(field);
            fields[i] = v;
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    }
}

fn seq_values(v: &Value) -> Vec<Value> {
    match v {
        Value::Vec(xs) => xs.as_ref().clone(),
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        other => panic!("expected seq, got {other:?}"),
    }
}

fn seq_strings(v: &Value) -> Vec<String> {
    seq_values(v)
        .into_iter()
        .map(|x| match x {
            Value::String(s) => (*s).clone(),
            other => panic!("expected string, got {other:?}"),
        })
        .collect()
}

fn strings_value(ss: &[String]) -> Value {
    Value::Vec(Arc::new(
        ss.iter()
            .map(|s| Value::String(Arc::new(s.clone())))
            .collect(),
    ))
}

fn rows_value(rows: &[Vec<String>]) -> Value {
    Value::Vec(Arc::new(rows.iter().map(|r| strings_value(r)).collect()))
}

fn fnv1a(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Mirror of `export.rs::abi_of`. Integration tests cannot see `pub(crate)`.
/// Names come from the RETE_OPS table in source order; a stale parse fails
/// the honest-pack equality assert before Import is asked.
fn abi_of(classes: &[String], fields: &[Vec<String>]) -> String {
    let mut s = String::from("v1");
    for (c, fs) in classes.iter().zip(fields.iter()) {
        s.push(';');
        s.push_str(c);
        s.push('[');
        s.push_str(&fs.join(","));
        s.push(']');
    }
    s.push_str(";ops:");
    for (i, op) in rete_ops_names().iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(op);
    }
    format!("v1:{:016x}", fnv1a(&s))
}

fn rete_ops_names() -> Vec<&'static str> {
    include_str!("../../src/rete/vocabulary.rs")
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("rete_name: \"")?;
            rest.strip_suffix("\",")
                .or_else(|| rest.strip_suffix('"'))
        })
        .collect()
}

fn poke_first_call_op(v: &mut Value, op: i64) -> bool {
    match v {
        Value::Vec(items) => {
            let mut xs = items.as_ref().clone();
            if matches!(xs.first(), Some(Value::wat__core__keyword(k)) if k.as_str() == ":call")
                && xs.len() >= 2
            {
                xs[1] = Value::i64(op);
                *v = Value::Vec(Arc::new(xs));
                return true;
            }
            for x in &mut xs {
                if poke_first_call_op(x, op) {
                    *v = Value::Vec(Arc::new(xs));
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

#[test]
// rune:vocare(vantage-bypass-test) — classes/fields zip refuse is a host Aggregate.fields poke
fn import_refuses_classes_fields_len_mismatch() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let i = a
                .names
                .iter()
                .position(|n| n == "classes")
                .expect("classes");
            let extra = match &fields[i] {
                Value::Vec(xs) => {
                    let mut v = xs.as_ref().clone();
                    v.push(Value::String(Arc::new("bogus::Class".into())));
                    Value::Vec(Arc::new(v))
                }
                other => panic!("expected packed classes Vec, got {other:?}"),
            };
            fields[i] = extra;
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    };
    let err = import_one(&world, tampered).expect_err("classes/fields zip miss must refuse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("classes length"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span; zip wall is the contract
        "import must name classes/fields length mismatch, got {msg}"
    );
}

#[test]
// rune:vocare(vantage-bypass-test) — host TypeEnv field-order refuse is a fields-row poke + abi restamp; wat has no Export setter
fn import_refuses_host_typeenv_field_order() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = match exp {
        Value::Aggregate(a) => {
            let mut rec = a.fields.as_ref().clone();
            let classes_i = a.names.iter().position(|n| n == "classes").expect("classes");
            let fields_i = a.names.iter().position(|n| n == "fields").expect("fields");
            let abi_i = a.names.iter().position(|n| n == "abi").expect("abi");
            let classes = seq_strings(&rec[classes_i]);
            // rune:perspicere(read-once) — one vantage-bypass poke; a test-local alias would be a one-site mumble
            let mut fields: Vec<Vec<String>> = seq_values(&rec[fields_i])
                .iter()
                .map(seq_strings)
                .collect();
            let orig_abi = match &rec[abi_i] {
                Value::String(s) => (**s).clone(),
                other => panic!("expected abi string, got {other:?}"),
            };
            assert_eq!(
                abi_of(&classes, &fields),
                orig_abi,
                "test abi_of must match packed abi (RETE_OPS parse order)"
            );
            let temp_i = classes
                .iter()
                .position(|c| c == "exp::Temp")
                .expect("cool-export packs exp::Temp");
            let row = fields
                .get_mut(temp_i)
                .expect("fields row for exp::Temp");
            let c_i = row
                .iter()
                .position(|f| f == "c")
                .expect("exp::Temp packed field c");
            row[c_i] = "renamed".to_string();
            rec[fields_i] = rows_value(&fields);
            rec[abi_i] = Value::String(Arc::new(abi_of(&classes, &fields)));
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
                a.names.clone(),
                Arc::new(rec),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    };
    let err = import_one(&world, tampered).expect_err("host TypeEnv field-order miss must refuse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("host TypeEnv field-order"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span; host conjunct is the contract
        "import must name host TypeEnv field-order, got {msg}"
    );
}

#[test]
// rune:vocare(vantage-bypass-test) — FORMAT_V refuse is a host Aggregate.fields poke
fn import_refuses_unsupported_version() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = poke_named(exp, "v", Value::i64(2));
    let err = import_one(&world, tampered).expect_err("v=2 must refuse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("unsupported Export version"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span; version wall is the contract
        "import must name unsupported Export version, got {msg}"
    );
}

#[test]
// rune:vocare(vantage-bypass-test) — opcode wrap refuse is a host poke of packed :call
fn import_refuses_op_outside_rete_ops() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let mut poked = false;
            for name in ["conds", "progs", "drivers"] {
                if let Some(i) = a.names.iter().position(|n| n == name) {
                    let mut packed = fields[i].clone();
                    if poke_first_call_op(&mut packed, 65537) {
                        fields[i] = packed;
                        poked = true;
                        break;
                    }
                }
            }
            assert!(
                poked,
                "cool-export must pack a :call so the wrap-into-range refuse is reachable"
            );
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    };
    let err = import_one(&world, tampered).expect_err("op 65537 must refuse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("outside RETE_OPS"), // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span; opcode wall is the contract
        "import must name outside RETE_OPS, got {msg}"
    );
}

/// ★★ `Op::Eval` ACROSS THE WIRE — fix-list F's serialization arms, driven.
///
/// Fix-list F added `Op::Eval` so an inline constraint with a COMPUTED operand could run, and gave
/// it `pack_cond_op` / `unpack` / `check_cond_ops` arms so a compiled program carrying one could
/// still be exported. Those arms were never DRIVEN: every other rule in this fixture uses a
/// `where` fence over plain operands, so no test in the tree had ever serialized an `Op::Eval`.
///
/// Presence is not aliveness. An untested serialization arm is the same class as the defect F
/// itself was — something that reads correct and answers wrong — and this arc has been bitten by
/// exactly that often enough to stop assuming.
///
/// The SOURCE row is the control and is not redundant: a fixture whose rule silently stopped
/// discriminating would make the round-trip agree with itself at the wrong number, which is
/// entry F's own signature one level up.
#[test]
fn an_op_eval_survives_export_and_import() {
    let src = call_beside_value(file!(), ":user::computed-source-hits").expect("computed source");
    assert_eq!(
        src,
        Value::i64(1),
        "(c+5)<20 admits c=10 and rejects c=30 — if this is not 1 the fixture stopped \
         discriminating and the round-trip below proves nothing"
    );
    let round = call_beside_value(file!(), ":user::computed-roundtrip-hits").expect("round-trip");
    assert_eq!(
        round,
        src,
        "export -> import must preserve an `Op::Eval`: the computed operand has to survive \
         pack, unpack, the slot bounds check, and exec"
    );
}
