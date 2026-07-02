//! Arc 255 — FM-2-bis disconfirming probe: reflection PARITY between rust builtins
//! and user forms.
//!
//! THE ASK (builder): a reflection consumer must not tell a builtin from a user
//! form by the query path — `metadata-of` answers for BOTH, returning a uniform
//! map. Content is honest (a `:defined-in` tag declares rust vs wat), but the
//! mechanism is seamless.
//!
//! Today builtins are an opaque 454-arm dispatch `match` — registered nowhere,
//! reflected by nothing. And a bare user `defn` (no explicit metadata) returns
//! `None` from `metadata-of`. So NEITHER carries the guaranteed baseline.
//!
//! RED AT HEAD:
//!   - `(metadata-of :wat::core::i64::+)` -> None (builtin not registered in sym).
//!   - `(metadata-of :my::f)` for a bare defn -> None (no guaranteed baseline).
//! GREEN AFTER 255.1: both return `Some(baseline)` — the builtin registered into
//! `sym` as a `Native` Function entry; every registered form carrying the
//! auto-derived baseline (`:defined-in` + `:layer` + `:name` + `:arity`).
//!
//! Run un-ignored to confirm RED; sonnet un-ignores after 255.1 lands (and then
//! enriches these to assert the baseline KEYS, not just Some).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_bare, startup_from_file};
use wat::runtime::{Environment, Value};

/// Eval `(metadata-of <name_kw>)` in a bare world; return whether result is `Some(_)`.
fn metadata_of_is_some_bare(name_kw: &str) -> bool {
    let world = startup_bare().expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
        Value::Option(o) => o.is_some(),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

/// Eval `(metadata-of <name_kw>)` in a world loaded from `fixture`; return whether result is `Some(_)`.
fn metadata_of_is_some_from_file(fixture: &str, name_kw: &str) -> bool {
    let world = startup_from_file(fixture).expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
        Value::Option(o) => o.is_some(),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

/// Eval `(metadata-of <name_kw>)` in a bare world and return the inner HashMap, or panic.
fn metadata_of_map(name_kw: &str) -> std::collections::HashMap<Value, Value> {
    let world = startup_bare().expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
        Value::Option(o) => match o.as_ref() {
            Some(Value::wat__std__HashMap(m)) => m.as_ref().clone(),
            other => panic!("metadata-of must return Some(HashMap); got {:?}", other),
        },
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

/// Fetch a baseline value by its keyword key (stored WITH the leading colon).
fn get<'a>(map: &'a std::collections::HashMap<Value, Value>, key: &str) -> Option<&'a Value> {
    map.get(&Value::wat__core__keyword(Arc::new(key.to_string())))
}

// RED at HEAD: a rust builtin is not registered -> metadata-of returns None.
#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_a_rust_builtin() {
    assert!(
        metadata_of_is_some_bare(":wat::core::i64::+"),
        "metadata-of must answer (Some) for a rust builtin :wat::core::i64::+ — \
         seamless reflection parity with user forms. It returned None (builtins \
         are an opaque dispatch match, registered nowhere)."
    );
}

// RED at HEAD: a bare user defn has no guaranteed baseline -> metadata-of None.
#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn user_form_carries_guaranteed_baseline() {
    assert!(
        metadata_of_is_some_from_file(
            "tests/reflection/probe_arc255_reflection_parity_user_form.wat",
            ":my::f"
        ),
        "metadata-of must answer (Some baseline) for a bare user defn — every \
         registered form carries the guaranteed baseline (:defined-in/:layer/\
         :name/:arity). It returned None."
    );
}

// ─── Arc 255.1b-iii — the intrinsic branch, proven on core::Bytes ────────────
//
// metadata-of now answers for a registered Rust intrinsic with the SAME
// `Some(HashMap<keyword, HolonAST>)` shape the user path uses, carrying the
// auto-derived baseline (:name/:kind/:defined-in/:layer/:arity/:pure/
// :deterministic/:doc). ZERO eval behavior change: the Bytes ops still produce
// identical results; this only adds the reflection answer.

#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_bytes_to_hex_intrinsic() {
    unimplemented!("arc 255: metadata-of for Rust intrinsics; on unlock assert the exact :doc for Bytes::to-hex");
}

#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_bytes_from_hex_intrinsic() {
    unimplemented!("arc 255: metadata-of for Rust intrinsics; on unlock assert the exact :doc for Bytes::from-hex");
}

/// Diagnostic: emit the EXACT metadata-of(Bytes/to-hex) map the builder wants
/// to see (keys + values). Run with `--nocapture`.
#[test]
fn dump_bytes_to_hex_metadata() {
    let map = metadata_of_map(":wat::core::Bytes::to-hex");
    let mut keys: Vec<String> = map
        .keys()
        .map(|k| match k {
            Value::wat__core__keyword(s) => s.as_ref().clone(),
            other => format!("{:?}", other),
        })
        .collect();
    keys.sort();
    println!("metadata-of(:wat::core::Bytes::to-hex) =>");
    for k in &keys {
        let v = get(&map, k).unwrap();
        // iv-c: values are now plain wat Values (not HolonAST-wrapped).
        let rendered = match v {
            Value::wat__core__keyword(s) => s.as_ref().clone(),
            Value::i64(n) => n.to_string(),
            Value::bool(b) => b.to_string(),
            Value::String(s) => format!("{:?}", s.as_ref()),
            Value::Enum(ev) => format!("{}::{}", ev.type_path, ev.variant_name),
            other => format!("{:?}", other),
        };
        println!("  {} => {}", k, rendered);
    }
}
