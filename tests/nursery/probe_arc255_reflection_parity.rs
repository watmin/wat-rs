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
//!   - `(metadata-of :wat::core::i64::+)` → None (builtin not registered in sym).
//!   - `(metadata-of :my::f)` for a bare defn → None (no guaranteed baseline).
//! GREEN AFTER 255.1: both return `Some(baseline)` — the builtin registered into
//! `sym` as a `Native` Function entry; every registered form carrying the
//! auto-derived baseline (`:defined-in` + `:layer` + `:name` + `:arity`).
//!
//! Run un-ignored to confirm RED; sonnet un-ignores after 255.1 lands (and then
//! enriches these to assert the baseline KEYS, not just Some).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Freeze `src` (+ a nil main) and eval `(metadata-of <name_kw>)`; return whether
/// the result is `Some(_)` (i.e. the form carries reflectable metadata).
fn metadata_of_is_some(src: &str, name_kw: &str) -> bool {
    let full = format!("{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)", src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
        Value::Option(o) => o.is_some(),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

// RED at HEAD: a rust builtin is not registered → metadata-of returns None.
#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_a_rust_builtin() {
    assert!(
        metadata_of_is_some("", ":wat::core::i64::+"),
        "metadata-of must answer (Some) for a rust builtin :wat::core::i64::+ — \
         seamless reflection parity with user forms. It returned None (builtins \
         are an opaque dispatch match, registered nowhere)."
    );
}

// RED at HEAD: a bare user defn has no guaranteed baseline → metadata-of None.
#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn user_form_carries_guaranteed_baseline() {
    let src = "(:wat::core::defn :my::f [x <- :wat::core::i64] -> :wat::core::i64 x)";
    assert!(
        metadata_of_is_some(src, ":my::f"),
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

/// Eval `(metadata-of <name_kw>)` and return the inner `HashMap<Value, Value>`,
/// or panic if the result is not `Some(HashMap)`.
fn metadata_of_map(name_kw: &str) -> std::collections::HashMap<Value, Value> {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup should succeed");
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

/// Read a keyword baseline value's content (without the leading colon).
/// Arc 255.1b-iv-c: values are now plain `Value::wat__core__keyword` (not HolonAST-wrapped).
fn keyword_content(v: &Value) -> String {
    match v {
        Value::wat__core__keyword(s) => {
            // The keyword is stored with the leading colon; strip it to match
            // the pre-iv-c HolonAST::keyword storage (no leading colon).
            s.strip_prefix(':').unwrap_or(s.as_str()).to_string()
        }
        other => panic!("expected plain keyword Value; got {:?}", other),
    }
}

/// Read an i64 baseline value, or panic.
/// Arc 255.1b-iv-c: values are now plain `Value::i64` (not HolonAST-wrapped).
fn i64_val(v: &Value) -> i64 {
    match v {
        Value::i64(n) => *n,
        other => panic!("expected plain i64 Value; got {:?}", other),
    }
}

/// Read a bool baseline value, or panic.
/// Arc 255.1b-iv-c: values are now plain `Value::bool` (not HolonAST-wrapped).
fn bool_val(v: &Value) -> bool {
    match v {
        Value::bool(b) => *b,
        other => panic!("expected plain bool Value; got {:?}", other),
    }
}

/// Read a string baseline value, or panic.
/// Arc 255.1b-iv-c: values are now plain `Value::String` (not HolonAST-wrapped).
fn string_val(v: &Value) -> String {
    match v {
        Value::String(s) => s.as_ref().clone(),
        other => panic!("expected plain String Value; got {:?}", other),
    }
}

/// Read a closed-domain enum's variant name from a `Value::Enum`, or panic.
/// Arc 255.1b-iv-c: :kind/:defined-in/:layer are now `Value::Enum` unit variants.
fn enum_variant(v: &Value) -> String {
    match v {
        Value::Enum(ev) => ev.variant_name.clone(),
        other => panic!("expected Value::Enum; got {:?}", other),
    }
}

#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_bytes_to_hex_intrinsic() {
    let map = metadata_of_map(":wat::core::Bytes::to-hex");

    // iv-c: :kind/:defined-in/:layer are now Value::Enum unit variants.
    assert_eq!(enum_variant(get(&map, ":kind").expect(":kind present")), "Intrinsic");
    assert_eq!(enum_variant(get(&map, ":defined-in").expect(":defined-in present")), "Rust");
    assert_eq!(enum_variant(get(&map, ":layer").expect(":layer present")), "Substrate");
    assert_eq!(i64_val(get(&map, ":arity").expect(":arity present")), 1);
    assert!(bool_val(get(&map, ":pure").expect(":pure present")), "to-hex is pure");
    assert!(
        bool_val(get(&map, ":deterministic").expect(":deterministic present")),
        "to-hex is deterministic"
    );
    // :name is the fqdn as a keyword (HolonAST::keyword strips the leading colon).
    assert_eq!(
        keyword_content(get(&map, ":name").expect(":name present")),
        "wat::core::Bytes::to-hex"
    );
    let doc = string_val(get(&map, ":doc").expect(":doc present (handler has a /// docstring)"));
    assert!(
        doc.contains("lowercase-hex") || doc.contains("lowercase hex"),
        ":doc must surface the to-hex docstring (prose containing 'lowercase-hex' or 'lowercase hex'); got: {:?}",
        doc
    );
}

#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn metadata_of_answers_for_bytes_from_hex_intrinsic() {
    let map = metadata_of_map(":wat::core::Bytes::from-hex");

    // iv-c: :kind/:defined-in/:layer are now Value::Enum unit variants.
    assert_eq!(enum_variant(get(&map, ":kind").expect(":kind present")), "Intrinsic");
    assert_eq!(enum_variant(get(&map, ":defined-in").expect(":defined-in present")), "Rust");
    assert_eq!(enum_variant(get(&map, ":layer").expect(":layer present")), "Substrate");
    assert_eq!(i64_val(get(&map, ":arity").expect(":arity present")), 1);
    assert!(bool_val(get(&map, ":pure").expect(":pure present")), "from-hex is pure");
    assert!(
        bool_val(get(&map, ":deterministic").expect(":deterministic present")),
        "from-hex is deterministic"
    );
    let doc = string_val(get(&map, ":doc").expect(":doc present (handler has a /// docstring)"));
    assert!(
        doc.contains("hex"),
        ":doc must surface the from-hex docstring verbatim; got: {:?}",
        doc
    );
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
