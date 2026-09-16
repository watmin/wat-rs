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

/// ⚠ DISCONFIRMING PROBE for the vigilia's Class A1 — written BEFORE the strike, expected RED.
///
/// `circumspicere` found that `import_export`'s header counts THREE walls (range refusal at the
/// read, slot bounds as a post-pass, three compat gates) and that **none of them is a GRAPH
/// wall**: nothing proves a child id names a node, that a Negation/Exists/Accumulate `aid` names
/// an Alpha, or that `child > parent` — while `kernel/node.rs:193` and `kernel/arm.rs:592` both
/// state the passes REQUIRE ascending id order as the topological order.
///
/// This truncates the node list, leaving every surviving parent's downstream child id dangling,
/// and imports. A network the engine cannot legally walk must be REFUSED at the door.
#[test]
fn import_refuses_a_node_graph_with_dangling_child_edges() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");

    let all = seq_values(export_field(&exp, "nodes"));
    assert!(
        all.len() >= 4,
        "fixture must have enough nodes for a truncation to dangle an edge; got {}",
        all.len()
    );
    let kept: Vec<Value> = all.iter().take(all.len() / 2).cloned().collect();
    let n_kept = kept.len();
    let tampered = poke_named(exp, "nodes", Value::Vec(Arc::new(kept)));

    let out = import_one(&world, tampered);
    match out {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("malformed") || msg.contains("Malformed"), // rune:lint(loose-assert) — the refusal's KIND is the contract; its wording is not pinned yet
                "import refused, but not as a malformed-form refusal: {msg}"
            );
        }
        Ok(v) => panic!(
            "IMPORT ACCEPTED A BROKEN GRAPH: kept {n_kept} of {} nodes, so every surviving \
             parent's downstream child id names nothing, and import returned {v:?}. \
             There is no graph wall.",
            all.len()
        ),
    }
}

/// Read a named field out of an `Export` aggregate — the read half of [`poke_named`].
fn export_field<'a>(exp: &'a Value, field: &str) -> &'a Value {
    match exp {
        Value::Aggregate(a) => {
            let i = a.names.iter().position(|n| n == field).expect(field);
            &a.fields[i]
        }
        other => panic!("expected Export, got {other:?}"),
    }
}

// ── strike-import-depth (arc 278, class A6) ─────────────────────────────────
//
// `import_export` had no depth criterion: what it accepted was whatever the importing
// THREAD's remaining stack allowed. The same 20,000-deep Export was ACCEPTED on a 256 MiB
// thread and killed a 2 MiB one with `fatal runtime error: stack overflow, aborting` — an
// abort, not a panic, so no `catch_unwind` and no wat error. These probes therefore never
// go near the stack: each sits just past the DECLARED bound, where pre-fix the import
// accepted the tower without complaint. That acceptance is the RED.
//
// `MAX_IMPORT_DEPTH` is 300 (see the constant in `src/rete/export.rs`); the numbers below
// are derived from it and are stated in each probe.

/// The declared wall, mirrored here because the constant is private to `src/rete/export.rs`.
const BOUND: usize = 300;

fn kw(name: &str) -> Value {
    Value::wat__core__keyword(Arc::new(name.to_string()))
}

fn vec_of(items: Vec<Value>) -> Value {
    Value::Vec(Arc::new(items))
}

/// Every packed side table an `Export` carries a `[:prog …]` or a driver inside.
const SIDE_TABLES: [&str; 5] = ["progs", "conds", "drivers", "folds", "rhs"];

/// Find the first packed `[:prog frame params names reads root]` anywhere inside `v` and
/// replace its `root` with `f(root)`. The packed forms are `Value::Vec`; the table that holds
/// them may be either sequence flavour, so both are walked.
fn poke_first_prog_root(v: &mut Value, f: &mut dyn FnMut(Value) -> Value) -> bool {
    let mut xs: Vec<Value> = match v {
        Value::Vec(items) => items.as_ref().clone(),
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => return false,
    };
    if matches!(xs.first(), Some(Value::wat__core__keyword(k)) if k.as_str() == ":prog")
        && xs.len() >= 6
    {
        let root = xs[5].clone();
        xs[5] = f(root);
        *v = vec_of(xs);
        return true;
    }
    for x in &mut xs {
        if poke_first_prog_root(x, f) {
            *v = vec_of(xs);
            return true;
        }
    }
    false
}

/// Rewrite the first `[:prog …]` root found in any side table, and return the tampered Export.
/// Panics if the fixture packs no program at all — that would silently make these probes vacuous.
fn tamper_first_prog_root(exp: Value, mut f: impl FnMut(Value) -> Value) -> Value {
    for name in SIDE_TABLES {
        let mut field = export_field(&exp, name).clone();
        if poke_first_prog_root(&mut field, &mut f) {
            return poke_named(exp, name, field);
        }
    }
    panic!("cool-export packs no [:prog …] — these depth probes would be vacuous");
}

/// ⚠ DISCONFIRMING PROBE — the plain `unpack_expr` descent.
///
/// A tower of `:and` nodes, `BOUND + 8` deep, poked into a packed program's root. Pre-fix this
/// is ACCEPTED (the wall does not exist); post-fix it must be refused as `malformed` naming the
/// bound. The depth is chosen to clear the bound by 8 and to stay two orders of magnitude below
/// the 3,000–5,000 window where the stack guard aborts — a probe near THAT would be a flake
/// generator and could not be caught anyway.
#[test]
fn import_refuses_an_and_tower_past_the_depth_bound() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let layers = BOUND + 8;
    let tampered = tamper_first_prog_root(exp, |_root| {
        let mut inner = vec_of(vec![kw(":lit"), Value::i64(1)]);
        for _ in 0..layers {
            inner = vec_of(vec![kw(":and"), inner]);
        }
        inner
    });
    match import_one(&world, tampered) {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("MAX_IMPORT_DEPTH"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; naming the bound is the contract
                "the refusal must name the depth bound, got {msg}"
            );
        }
        Ok(v) => panic!(
            "IMPORT ACCEPTED A {layers}-DEEP :and TOWER (bound {BOUND}) and returned {v:?}. \
             The import door has no depth criterion: what it accepts is a property of the \
             importing thread's stack, not of the format."
        ),
    }
}

/// ⚠ DISCONFIRMING PROBE — the `:user` ↔ `:prog` CYCLE, which an expr-only counter walks past.
///
/// `unpack_expr`'s `:user` arm calls `unpack_prog`, whose root calls `unpack_expr` again, so a
/// tower of `:user` nodes alternates between the two functions. `layers` is deliberately chosen
/// so that:
///
/// * counting only `unpack_expr` frames gives `layers + 1` = 159, which is UNDER the bound —
///   a budget threaded through `unpack_expr` alone ACCEPTS this tower and this probe stays red;
/// * counting every frame gives `2 * layers + 2` = 318, which is OVER the bound — only the
///   shared budget refuses it.
///
/// That is the whole reason this arm exists separately from the `:and` one: the `:and` tower
/// never alternates, so it cannot see the difference.
#[test]
fn import_refuses_a_user_prog_cycle_tower_past_the_depth_bound() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let layers = BOUND / 2 + 8; // 158
    let tampered = tamper_first_prog_root(exp, |_root| {
        // Innermost is a literal, and every synthetic inner program declares frame_len 0 —
        // so nothing here can be refused by the SLOT wall instead of the depth wall.
        let mut inner = vec_of(vec![kw(":lit"), Value::i64(1)]);
        for _ in 0..layers {
            let prog = vec_of(vec![
                kw(":prog"),
                Value::i64(0),
                vec_of(vec![]),
                vec_of(vec![]),
                vec_of(vec![]),
                inner,
            ]);
            inner = vec_of(vec![kw(":user"), prog]);
        }
        inner
    });
    match import_one(&world, tampered) {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("MAX_IMPORT_DEPTH"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; naming the bound is the contract
                "the refusal must name the depth bound, got {msg}"
            );
        }
        Ok(v) => panic!(
            "IMPORT ACCEPTED A {layers}-LAYER :user/:prog TOWER and returned {v:?}. \
             Only {} expr frames but {} frames in total: a budget counted on unpack_expr \
             alone is walked past by the cycle.",
            layers + 1,
            2 * layers + 2
        ),
    }
}

/// ⚠ DISCONFIRMING PROBE — `unpack_pat`'s OWN recursion, reached through `:match`.
///
/// `unpack_expr`'s `:match` arm calls `unpack_pat`, which recurses on `Pat::Variant`. A budget
/// that stops at the expression tree never enters this arm. Everything here is synthetic and
/// slot-free (`:wild` binds nothing), so the slot wall cannot answer in the depth wall's place.
#[test]
fn import_refuses_a_pattern_tower_past_the_depth_bound() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let layers = BOUND + 8;
    let tampered = tamper_first_prog_root(exp, |_root| {
        let mut pat = vec_of(vec![kw(":wild")]);
        for _ in 0..layers {
            pat = vec_of(vec![kw(":pvar"), Value::String(Arc::new("V".into())), pat]);
        }
        vec_of(vec![
            kw(":match"),
            vec_of(vec![kw(":lit"), Value::i64(1)]),
            vec_of(vec![vec_of(vec![pat, vec_of(vec![kw(":lit"), Value::i64(1)])])]),
        ])
    });
    match import_one(&world, tampered) {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("MAX_IMPORT_DEPTH"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; naming the bound is the contract
                "the refusal must name the depth bound, got {msg}"
            );
        }
        Ok(v) => panic!(
            "IMPORT ACCEPTED A {layers}-DEEP :pvar PATTERN TOWER and returned {v:?}. \
             unpack_pat recurses on Pat::Variant with no budget of its own."
        ),
    }
}

/// ⚠ DISCONFIRMING PROBE — `unpack_driver`'s OWN recursion.
///
/// Not named in the strike brief, and its doc comment states the defect as a feature: *"the
/// composite arms recurse, so a driver tree of any depth round-trips WITHOUT A DEPTH
/// PARAMETER — the wire's nesting IS the recursion."* That is a second unbounded tower at the
/// same door, independent of `unpack_expr`, and it reaches `unpack_prog` through `:where`.
#[test]
fn import_refuses_a_driver_tower_past_the_depth_bound() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let layers = BOUND + 8;
    let drivers = seq_values(export_field(&exp, "drivers"));
    assert!(
        !drivers.is_empty(),
        "cool-export must pack at least one driver or this probe is vacuous"
    );
    let mut pairs = drivers;
    let first = seq_values(&pairs[0]);
    assert!(first.len() >= 2, "driver table entry is [id driver]");
    let mut wrapped = first[1].clone();
    for _ in 0..layers {
        wrapped = vec_of(vec![kw(":not"), wrapped]);
    }
    pairs[0] = vec_of(vec![first[0].clone(), wrapped]);
    let tampered = poke_named(exp, "drivers", vec_of(pairs));
    match import_one(&world, tampered) {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("MAX_IMPORT_DEPTH"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; naming the bound is the contract
                "the refusal must name the depth bound, got {msg}"
            );
        }
        Ok(v) => panic!(
            "IMPORT ACCEPTED A {layers}-DEEP :not DRIVER TOWER and returned {v:?}. \
             unpack_driver recurses on the wire's nesting with no budget."
        ),
    }
}
