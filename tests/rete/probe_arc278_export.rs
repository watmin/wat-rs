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

// ── strike-calluser-arity (arc 278, class D3) ───────────────────────────────
//
// `exec_program_on` never compared `args.len()` to `program.params.len()`, and its `else if
// i < inner.len()` branch gave a surplus argument a MEANING: it landed in the slot whose number
// happened to equal the argument's POSITION. One missing check with several faces —
//
//   * a surplus that collides with a declared parameter slot **overwrites it**, and a live fence
//     answers 0 hits where it answered 1: a silent wrong answer from wire input;
//   * a surplus past the frame is **silently dropped**, and the same fence answers 2;
//   * a surplus into a callee that declares NO parameters at all **fabricates** a binding for a
//     slot the program never named a parameter;
//   * a MISSING argument surfaces as `UnboundSymbol { "slot 1" }` — a diagnostic naming a
//     compiler-internal slot index, with a span on the CALLER's wat line and no arity and no
//     callee in it.
//
// Every probe below replaces the fixture fence's root with a synthetic `[:user prog args…]` call
// whose body is `(slot N) < 20`, so it runs on the real import → seed → fire → query path and the
// hit count is the observation. `:user::import-and-hits` is that mouth.
//
// ⛔ The refusal is asserted STRUCTURALLY — `RuntimeErrorKind::ArityMismatch` with BOTH counts
// checked — never merely "an error happened". Two of these arms already errored before the fix,
// so a probe asserting only that an error occurred would pass in both states and prove nothing.

/// `Temp{10}` and `Temp{30}` are the fixture's facts, and every synthetic fence compares against
/// this bound. `10 < 20` holds and `30 < 20` does not, which is what makes a hit count able to
/// name WHICH value reached the slot.
const FENCE_BOUND: i64 = 20;

/// Export → hit count, through the real `import` + `seed` + `fire-rules` + `query`. The mouth the
/// arity arms observe through: `import-one` stops at the Session and would see none of this.
fn import_and_hits(
    world: &wat::freeze::FrozenWorld,
    exp: Value,
) -> Result<Value, wat::runtime::RuntimeError> {
    let f = world
        .symbols()
        .get(":user::import-and-hits")
        .expect("import-and-hits")
        .clone();
    apply_function(f, vec![exp], world.symbols(), wat::rust_caller_span!())
}

/// The `RETE_OPS` index of an op, derived from the vocabulary rather than hard-coded — a pinned
/// literal would silently address a different op the day a row is inserted above it.
fn rete_op_index(name: &str) -> i64 {
    rete_ops_names()
        .iter()
        .position(|n| *n == name)
        .unwrap_or_else(|| panic!("RETE_OPS declares no {name}")) as i64
}

/// A packed `[:user [:prog …] args…]` whose callee body is `(slot read_slot) < 20`.
///
/// `frame_len` and `params` are the two dials the arms turn: together they decide whether a
/// surplus argument collides with a declared slot, falls past the end of the frame, or writes
/// into a program that declared no parameters at all. `names` is packed empty on purpose, so an
/// unbound slot renders as `slot N` — that raw rendering is arm 3's whole finding.
fn synthetic_user_fence(frame_len: i64, params: Vec<i64>, read_slot: i64, args: Vec<Value>) -> Value {
    let lt = rete_op_index(":wat::rete::core::i64::<");
    let body = vec_of(vec![
        kw(":call"),
        Value::i64(lt),
        vec_of(vec![kw(":slot"), Value::i64(read_slot)]),
        vec_of(vec![kw(":lit"), Value::i64(FENCE_BOUND)]),
    ]);
    let prog = vec_of(vec![
        kw(":prog"),
        Value::i64(frame_len),
        vec_of(params.into_iter().map(Value::i64).collect()),
        vec_of(vec![]), // names — empty, so an unbound slot renders as `slot N`
        vec_of(vec![]), // reads — the synthetic fence reads no token bindings
        body,
    ]);
    let mut xs = vec![kw(":user"), prog];
    xs.extend(args);
    vec_of(xs)
}

/// `[:lit n]` — an argument expression evaluated in the CALLER's frame.
fn lit(n: i64) -> Value {
    vec_of(vec![kw(":lit"), Value::i64(n)])
}

/// Assert a refusal is an `ArityMismatch` naming BOTH counts. Row 5's trap: two of these arms
/// already produced *an* error before the fix, so only the KIND and the COUNTS separate the fixed
/// runtime from the broken one.
fn expect_arity_mismatch(
    r: Result<Value, wat::runtime::RuntimeError>,
    expected: usize,
    got: usize,
    arm: &str,
) {
    match r {
        Ok(v) => panic!(
            "{arm}: the call was ACCEPTED and the fence answered {v:?}. \
             `exec_program_on` compared no arity: a {got}-argument call ran against a \
             {expected}-parameter program."
        ),
        Err(e) => match e.kind() {
            wat::RuntimeErrorKind::ArityMismatch {
                op,
                expected: exp,
                got: g,
            } => {
                assert_eq!(*exp, expected, "{arm}: wrong `expected` count in {e:?}");
                assert_eq!(*g, got, "{arm}: wrong `got` count in {e:?}");
                // Exact, not `contains`: `CALL_USER_OP` (`expr_ir/eval.rs:381`) is a fixed
                // constant, so a loose check would pass on any op whose name merely embeds it.
                assert_eq!(
                    &**op, ":wat::rete::call-user",
                    "{arm}: the refusal must name the call form"
                );
            }
            other => panic!(
                "{arm}: refused, but NOT as an arity mismatch — got {other:?}. \
                 An error alone is not the contract: this arm errored before the check existed \
                 too, and only an ArityMismatch carrying expected={expected} got={got} \
                 distinguishes the two runtimes."
            ),
        },
    }
}

/// CONTROL — the mouth works and the fixture is live. Nothing is tampered.
#[test]
fn untampered_export_answers_one_hit() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let v = import_and_hits(&world, exp).expect("untampered import must fire");
    assert_eq!(
        v,
        Value::i64(1),
        "the fixture fence is `(?c < 20)` over Temp 10 and Temp 30"
    );
}

/// ⚠ ANTI-VACUITY CONTROL — green BEFORE and AFTER, and it is what makes the other arms mean
/// something.
///
/// A synthetic `:user` call with **matching** arity: one parameter at slot 0, one argument. The
/// callee answers `10 < 20` = true for every fact, so the fence admits BOTH Temps and the count
/// is 2, not the fixture's 1. That number proves three things at once — the synthetic fence
/// really replaced the real one, it really executed, and the new length check does **not** refuse
/// a well-formed call. Without this arm, every green below is consistent with "the check refuses
/// everything".
#[test]
fn a_well_formed_user_call_still_runs() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(1, vec![0], 0, vec![lit(10)])
    });
    let v = import_and_hits(&world, tampered).expect("a matched-arity call must run");
    assert_eq!(
        v,
        Value::i64(2),
        "the synthetic fence is constantly `10 < 20`, so both Temps pass"
    );
}

/// ⚠ ARM 1 — A SURPLUS ARGUMENT COLLIDES WITH A DECLARED PARAMETER SLOT.
///
/// The worst face, and the reason this is not a tidiness fix: **a silent wrong answer through the
/// public surface.** One parameter at slot **1**, two arguments. Pre-fix, `i=0` wrote `10` into
/// slot 1 as declared; then `i=1` found no parameter, fell into the `else if`, and wrote the
/// surplus `30` into `inner[1]` — the SAME slot — by argument position. `30 < 20` is false, the
/// fence rejected every fact, and the import was ACCEPTED and answered **0** where the fixture
/// answers 1. (Had the surplus merely been ignored it would have answered 2; no reading of the
/// input makes 0 correct.)
#[test]
fn arity_refuses_a_surplus_that_collides_with_a_declared_slot() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(2, vec![1], 1, vec![lit(10), lit(30)])
    });
    expect_arity_mismatch(
        import_and_hits(&world, tampered),
        1,
        2,
        "arm 1 (surplus collides with slot 1; pre-fix ACCEPTED, 0 hits — a silent wrong answer)",
    );
}

/// ⚠ ARM 2 — A SURPLUS ARGUMENT PAST THE END OF THE FRAME IS SILENTLY DROPPED.
///
/// The same missing check answering the opposite way. One parameter at slot **0** and
/// `frame_len` 1, so the frame is one wide: `i=1` failed the `i < inner.len()` guard and the
/// argument vanished. Pre-fix the import was ACCEPTED and the fence — left as the constant
/// `10 < 20` — answered **2**. A dropped argument is not a smaller error than a misplaced one:
/// the caller's second operand had no effect anyone could observe.
#[test]
fn arity_refuses_a_surplus_that_falls_past_the_frame() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(1, vec![0], 0, vec![lit(10), lit(30)])
    });
    expect_arity_mismatch(
        import_and_hits(&world, tampered),
        1,
        2,
        "arm 2 (surplus past a 1-wide frame; pre-fix ACCEPTED, 2 hits — the argument was dropped)",
    );
}

/// ⚠ ARM 3 — A SURPLUS INTO A CALLEE THAT DECLARES NO PARAMETERS AT ALL.
///
/// Distinct from arms 1 and 2 in mechanism, not just in numbers: with `params` EMPTY, every
/// `params.get(i)` was `None`, so the deleted branch was the ONLY thing that ran and it
/// **fabricated** bindings — `inner[0] = 10` — for a slot the program never declared a parameter.
/// Pre-fix the import was ACCEPTED and the fence read that fabricated slot as `10 < 20`,
/// answering **2**. This is the branch at its purest: a zero-parameter program executing against
/// arguments.
#[test]
fn arity_refuses_arguments_to_a_zero_parameter_callee() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(1, vec![], 0, vec![lit(10), lit(30)])
    });
    expect_arity_mismatch(
        import_and_hits(&world, tampered),
        0,
        2,
        "arm 3 (two arguments into a 0-param callee; pre-fix ACCEPTED, 2 hits — slot 0 fabricated)",
    );
}

/// ⚠ ARM 4 — A MISSING ARGUMENT, DIAGNOSED AS AN INTERNAL SLOT INDEX.
///
/// ⛔ **THIS ARM ALREADY ERRORED BEFORE THE FIX**, which is exactly why it may not assert that an
/// error occurred. One parameter at slot 1, ZERO arguments: pre-fix nothing wrote slot 1, the
/// body read it, and the refusal was
/// `#wat.runtime/UnboundSymbol {:message "unbound symbol: slot 1"}` — a compiler-internal slot
/// number, on the CALLER's wat span, naming neither the arity nor the callee. A probe saying
/// "this errors" is green in both runtimes and proves nothing; only `ArityMismatch { expected: 1,
/// got: 0 }` separates them.
///
/// This arm also reaches `exec`'s `args.is_empty()` short-circuit, the one path DESIGN's ⚠
/// section flags — see its sibling below, which reaches the evaluating path instead.
#[test]
fn arity_refuses_a_call_with_no_arguments_at_all() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(2, vec![1], 1, vec![])
    });
    expect_arity_mismatch(
        import_and_hits(&world, tampered),
        1,
        0,
        "arm 4 (zero args to a 1-param callee; pre-fix `UnboundSymbol: slot 1`, not an arity error)",
    );
}

/// ⚠ ARM 5 — TOO FEW ARGUMENTS, BUT NOT ZERO: the check on the EVALUATING path.
///
/// Arm 4 enters `exec`'s `Expr::CallUser` arm through its `args.is_empty()` short-circuit, which
/// hands `&[]` straight to `exec_program_on`. This arm has ONE argument for TWO parameters, so it
/// takes the other branch — the one that evaluates each operand into a `Vec` first. Without it,
/// arm 4 alone cannot tell "the arity check runs" from "the arity check runs only on the empty
/// path". Pre-fix this was the same `UnboundSymbol: slot 1` as arm 4, arriving by a different
/// route.
#[test]
fn arity_refuses_too_few_arguments_on_the_evaluating_path() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::cool-export").expect("export");
    let tampered = tamper_first_prog_root(exp, |_root| {
        synthetic_user_fence(2, vec![0, 1], 1, vec![lit(10)])
    });
    expect_arity_mismatch(
        import_and_hits(&world, tampered),
        2,
        1,
        "arm 5 (one arg to a 2-param callee, evaluating path; pre-fix `UnboundSymbol: slot 1`)",
    );
}
