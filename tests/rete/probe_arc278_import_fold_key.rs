//! DISCONFIRMING PROBE — vigilia Class A2: a wire-supplied accumulate fold key reaches a host
//! `panic!` whose licence is a compile-time proof the import door never runs.
//!
//! `acc.rs`'s `acc_var_i64` says it "Panics on an unbound var or a non-i64 value (a
//! compile-time-impossible shape)", carrying `rune:struere(invariant-coupling) — AccFold compile
//! proved i64`. That proof is `build_rete_arm`'s. `unpack_fold` (`export.rs`, the `:sum` arm)
//! takes the key straight off the wire as an arbitrary `Value` and `import_export` interns it,
//! and the graph wall added for Class A1 validates node EDGES, not side tables — deliberately.

use std::sync::Arc;
use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::{apply_function, Value};
use wat::value::AggregateValue;

/// Untampered control: the fixture is live and the two paths agree.
#[test]
fn fold_key_fixture_native_and_imported_agree() {
    let world = startup_beside(file!()).expect("freeze");
    let native = call_beside_value(file!(), ":user::fold-native-fire").expect("native");
    let exp = call_beside_value(file!(), ":user::fold-export").expect("export");
    let imported = call_fold_import(&world, exp).expect("untampered import must fire");
    assert_eq!(native, imported, "native and imported fire must agree");
    assert_eq!(native, Value::i64(1), "one SumF derived for group 1");
}

/// ⚠ EXPECTED RED before Class A2 lands.
///
/// Replaces the `:sum` fold's key with a keyword no condition binds, then imports and fires.
/// A wire value must not be able to panic the host: the engine may refuse it, but the refusal
/// has to arrive as a value the caller can match, the way `driver_of` already does for the same
/// class of missing-id (`fire/mod.rs`).
#[test]
fn import_refuses_a_fold_key_no_condition_binds() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::fold-export").expect("export");

    let folds = seq_values(field_of(&exp, "folds"));
    assert!(!folds.is_empty(), "fixture must export at least one fold");
    let (tampered_folds, n_rewritten) = rewrite_sum_keys(&folds, unbound_key());
    assert_eq!(
        n_rewritten, 1,
        "fixture must carry exactly one :sum fold to tamper; got {n_rewritten}"
    );
    let tampered = poke(exp, "folds", Value::Vec(Arc::new(tampered_folds)));

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_fold_import(&world, tampered)
    }));
    match outcome {
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic payload>".into());
            panic!(
                "A WIRE VALUE PANICKED THE HOST. Importing an Export whose :sum fold key is a \
                 keyword no condition binds, then firing, unwound the process instead of \
                 refusing: {msg}"
            );
        }
        Ok(Err(_)) => { /* refused as a value — the contract */ }
        Ok(Ok(v)) => panic!(
            "import+fire ACCEPTED a fold key no condition binds and returned {v:?} — the fold \
             read something it should not have been able to name"
        ),
    }
}

fn call_fold_import(
    world: &wat::freeze::FrozenWorld,
    exp: Value,
) -> Result<Value, wat::runtime::RuntimeError> {
    call_import(world, ":user::fold-import-and-fire", exp)
}

fn call_import(
    world: &wat::freeze::FrozenWorld,
    entry: &str,
    exp: Value,
) -> Result<Value, wat::runtime::RuntimeError> {
    let f = world.symbols().get(entry).expect("import entry point").clone();
    apply_function(f, vec![exp], world.symbols(), wat::rust_caller_span!())
}

/// Rewrite every `[:sum <key>]` entry's key to a keyword nothing binds. Returns the new folds
/// seq and how many were rewritten, so the probe can refuse to run on a fixture that drifted.
fn rewrite_sum_keys(folds: &[Value], key: Value) -> (Vec<Value>, usize) {
    let mut n = 0;
    let out = folds
        .iter()
        .map(|pair| {
            let items = seq_values(pair);
            if items.len() != 2 {
                return pair.clone();
            }
            let inner = seq_values(&items[1]);
            let is_sum = matches!(inner.first(), Some(Value::wat__core__keyword(k)) if k.as_str() == ":sum");
            if !is_sum || inner.len() < 2 {
                return pair.clone();
            }
            n += 1;
            let mut fixed = inner.clone();
            fixed[1] = key.clone();
            Value::Vec(Arc::new(vec![items[0].clone(), Value::Vec(Arc::new(fixed))]))
        })
        .collect();
    (out, n)
}

fn field_of<'a>(exp: &'a Value, field: &str) -> &'a Value {
    match exp {
        Value::Aggregate(a) => {
            let i = a.names.iter().position(|n| n == field).expect(field);
            &a.fields[i]
        }
        other => panic!("expected Export, got {other:?}"),
    }
}

fn poke(exp: Value, field: &str, v: Value) -> Value {
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

// ── THE UNPACKED HALF — the same gap through `Bindings::get` ────────────────────────────────
//
// The two tests above prove `acc_var_i64`'s PACKED arm and nothing else: their fixture's `:from`
// is an all-i64 record, so `skip_span` holds and elements carry no binding span. `:ifk::Tagged`
// has a String field, which denies it a packed row, and its `?tag` is neither token-bound nor
// named by the fold, so the pass takes the grouped gather into `accumulate_value`. Both of
// `acc_var_i64`'s `Bindings::get` arms are reachable from there — one per tampered key.

/// The key nothing binds — reaches the `None` arm of `Bindings::get` (and, on the packed path,
/// the slot-keys arm).
fn unbound_key() -> Value {
    Value::wat__core__keyword(Arc::new("?no-condition-binds-this".into()))
}

/// `?tag` IS bound by the `:from` cond — to a String. Reaches the `Some(non-i64)` arm.
/// Vars intern as `Value::String`, which is what an untampered fold key is, so this key is
/// indistinguishable from a legitimate one until the value behind it is read.
fn string_bound_key() -> Value {
    Value::String(Arc::new("?tag".into()))
}

/// Untampered control for the unpacked path: two tag groups (a=10, b=20), the `where` fence
/// admits only the one summing to 10, so the count sees the sum.
#[test]
fn unpacked_fold_fixture_native_and_imported_agree() {
    let world = startup_beside(file!()).expect("freeze");
    let native = call_beside_value(file!(), ":user::tag-native-fire").expect("native");
    let exp = call_beside_value(file!(), ":user::tag-export").expect("export");
    let imported =
        call_import(&world, ":user::tag-import-and-fire", exp).expect("untampered import must fire");
    assert_eq!(native, imported, "native and imported fire must agree");
    assert_eq!(native, Value::i64(1), "one TagSum: group a sums to 10, group b to 20");
}

#[test]
fn import_refuses_an_unpacked_fold_key_no_condition_binds() {
    assert_tag_fold_key_refused(unbound_key());
}

#[test]
fn import_refuses_an_unpacked_fold_key_bound_to_a_string() {
    assert_tag_fold_key_refused(string_bound_key());
}

/// Tamper the tag rule's `:sum` key, import, fire — and require a VALUE back, never an unwind
/// and never an answer.
fn assert_tag_fold_key_refused(key: Value) {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::tag-export").expect("export");

    let folds = seq_values(field_of(&exp, "folds"));
    let (tampered_folds, n_rewritten) = rewrite_sum_keys(&folds, key.clone());
    assert_eq!(
        n_rewritten, 1,
        "tag fixture must carry exactly one :sum fold to tamper; got {n_rewritten}"
    );
    let tampered = poke(exp, "folds", Value::Vec(Arc::new(tampered_folds)));

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_import(&world, ":user::tag-import-and-fire", tampered)
    }));
    match outcome {
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic payload>".into());
            panic!(
                "A WIRE VALUE PANICKED THE HOST. Importing an Export whose :sum fold key was \
                 rewritten to {key:?}, then firing, unwound the process instead of refusing: {msg}"
            );
        }
        Ok(Err(_)) => { /* refused as a value — the contract */ }
        Ok(Ok(v)) => panic!(
            "import+fire ACCEPTED the fold key {key:?} and returned {v:?} — the fold read \
             something it should not have been able to name"
        ),
    }
}

// ── THE SLOT HALF — `fold_bucket`'s unpacked path ───────────────────────────────────────────
//
// `slot_i64`'s non-i64 arm. Reaching it needs `group_keys` EMPTY so the pass takes `fold_bucket`
// rather than `accumulate_value`, and a tampered fold key normally makes `group_keys` non-empty
// by evicting the real operand into it — see the fixture's own note for the join shape that
// defeats that.

/// Untampered control for the slot path.
#[test]
fn slot_fold_fixture_native_and_imported_agree() {
    let world = startup_beside(file!()).expect("freeze");
    let native = call_beside_value(file!(), ":user::slot-native-fire").expect("native");
    let exp = call_beside_value(file!(), ":user::slot-export").expect("export");
    let imported = call_import(&world, ":user::slot-import-and-fire", exp)
        .expect("untampered import must fire");
    assert_eq!(native, imported, "native and imported fire must agree");
    assert_eq!(native, Value::i64(1), "one SlotSum, and only because the bucket sums to 7");
}

#[test]
fn import_refuses_a_slot_fold_key_bound_to_a_string() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::slot-export").expect("export");

    let folds = seq_values(field_of(&exp, "folds"));
    let (tampered_folds, n_rewritten) = rewrite_sum_keys(&folds, string_bound_key());
    assert_eq!(
        n_rewritten, 1,
        "slot fixture must carry exactly one :sum fold to tamper; got {n_rewritten}"
    );
    let tampered = poke(exp, "folds", Value::Vec(Arc::new(tampered_folds)));

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_import(&world, ":user::slot-import-and-fire", tampered)
    }));
    match outcome {
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic payload>".into());
            panic!(
                "A WIRE VALUE PANICKED THE HOST. Importing an Export whose :sum fold key names a \
                 String-bound var on the SLOT path, then firing, unwound the process instead of \
                 refusing: {msg}"
            );
        }
        Ok(Err(_)) => { /* refused as a value — the contract */ }
        Ok(Ok(v)) => panic!(
            "import+fire ACCEPTED a String-bound slot fold key and returned {v:?} — the fold read \
             something it should not have been able to name"
        ),
    }
}

/// ⚠ DISCONFIRMING PROBE for the vigilia's Class A2b — the SILENT ZERO.
///
/// `c449cd24d` converted nine `panic!` arms to refusals, and this path is the one that never
/// panicked: `fold_bucket`'s `Sum` arm answers `operand_slot`'s `None` with
/// `Ok(Some(Value::i64(0)))` (`acc.rs:321-323`). But that `None` carries **two facts** —
/// `bucket.first()?` (an EMPTY bucket, where sum's identity genuinely is 0) and `.position(…)`
/// (the var names nothing, which is the same defect the other eight arms now refuse). The
/// empty-bucket identity is being reused to answer "the var isn't there".
///
/// Its own siblings disagree about the same `None`: `Min`/`Max`/`Mean` return `Ok(None)` and drop.
///
/// A silent wrong answer is worse than the panic it replaced, so this must refuse.
#[test]
fn import_refuses_a_slot_fold_key_no_condition_binds() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::slot-export").expect("export");

    let folds = seq_values(field_of(&exp, "folds"));
    let (tampered_folds, n_rewritten) = rewrite_sum_keys(&folds, unbound_key());
    assert_eq!(
        n_rewritten, 1,
        "slot fixture must carry exactly one :sum fold to tamper; got {n_rewritten}"
    );
    let tampered = poke(exp, "folds", Value::Vec(Arc::new(tampered_folds)));

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_import(&world, ":user::slot-import-and-fire", tampered)
    }));
    match outcome {
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic payload>".into());
            panic!("A WIRE VALUE PANICKED THE HOST on the slot path: {msg}");
        }
        Ok(Err(_)) => { /* refused as a value — the contract */ }
        Ok(Ok(v)) => panic!(
            "SILENT WRONG ANSWER: import+fire ACCEPTED a :sum fold key no condition binds and \
             returned {v:?} instead of refusing. `operand_slot` answered `None` because the var \
             names nothing, and `fold_bucket`'s Sum arm read that as the EMPTY-BUCKET identity \
             and summed to 0. Min/Max/Mean answer the same `None` by dropping — one `Option`, \
             two facts, two different wrong answers."
        ),
    }
}

/// ⚠ DISCONFIRMING PROBE — the OTHER consumer of the same conflated `None`.
///
/// `fold_bucket`'s `Min`/`Max`/`Mean` arm (`acc.rs:345-347`) answers `operand_slot`'s `None` with
/// `Ok(None)` — it silently DROPS the derived fact instead of refusing. Same one `Option`, same
/// two facts, a different wrong answer from the `Sum` arm above.
///
/// Reaching it needs no second rule and no fixture change: `unpack_fold` (`export.rs`) takes the
/// fold TAG off the wire too, so rewriting the slot fixture's `[:sum ?v]` to `[:min <unbound>]`
/// routes the very same three-var join — the only shape a tampered fold key cannot divert away
/// from `fold_bucket` — down the `Min` arm.
#[test]
fn import_refuses_a_slot_min_fold_key_no_condition_binds() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::slot-export").expect("export");

    let folds = seq_values(field_of(&exp, "folds"));
    let (tampered_folds, n_rewritten) = rewrite_sum_folds_to(&folds, ":min", unbound_key());
    assert_eq!(
        n_rewritten, 1,
        "slot fixture must carry exactly one :sum fold to retag; got {n_rewritten}"
    );
    let tampered = poke(exp, "folds", Value::Vec(Arc::new(tampered_folds)));

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_import(&world, ":user::slot-import-and-fire", tampered)
    }));
    match outcome {
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic payload>".into());
            panic!("A WIRE VALUE PANICKED THE HOST on the slot :min path: {msg}");
        }
        Ok(Err(_)) => { /* refused as a value — the contract */ }
        Ok(Ok(v)) => panic!(
            "SILENTLY DROPPED FACT: import+fire ACCEPTED a :min fold key no condition binds and \
             returned {v:?} instead of refusing. `operand_slot` answered `None` because the var \
             names nothing, and `fold_bucket`'s Min/Max/Mean arm read that as the EMPTY-BUCKET \
             absence and dropped the derived fact. Sum answers the same `None` with i64(0) — one \
             `Option`, two facts, two different wrong answers."
        ),
    }
}

/// Rewrite every `[:sum <key>]` fold entry to `[<tag> <key>]`. The wire carries the fold TAG as
/// well as its key (`unpack_fold`, `export.rs`), so retagging reaches `fold_bucket`'s other arm
/// on the SAME fixture. Returns the new folds seq and how many were rewritten, so a probe can
/// refuse to run on a fixture that drifted.
fn rewrite_sum_folds_to(folds: &[Value], tag: &str, key: Value) -> (Vec<Value>, usize) {
    let mut n = 0;
    let out = folds
        .iter()
        .map(|pair| {
            let items = seq_values(pair);
            if items.len() != 2 {
                return pair.clone();
            }
            let inner = seq_values(&items[1]);
            let is_sum = matches!(inner.first(), Some(Value::wat__core__keyword(k)) if k.as_str() == ":sum");
            if !is_sum || inner.len() < 2 {
                return pair.clone();
            }
            n += 1;
            let mut fixed = inner.clone();
            fixed[0] = Value::wat__core__keyword(Arc::new(tag.into()));
            fixed[1] = key.clone();
            Value::Vec(Arc::new(vec![items[0].clone(), Value::Vec(Arc::new(fixed))]))
        })
        .collect();
    (out, n)
}
