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
//!
//! GREEN AFTER 255.1: both return `Some(baseline)` — the builtin registered into
//! `sym` as a `Native` Function entry; every registered form carrying the
//! auto-derived baseline (`:defined-in` + `:layer` + `:name` + `:arity`).
//!
//! Run un-ignored to confirm RED; sonnet un-ignores after 255.1 lands (and then
//! enriches these to assert the baseline KEYS, not just Some).

use std::sync::Arc;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): the metadata-of calls live in co-located fixtures, driven
// via `call_beside_value` / fetch-and-`apply_function`; the Rust side inspects the
// returned typed Value (the same inspection the format!-string driver did).

/// metadata-of(:wat::core::i64::+) via the co-located fixture — is it `Some(_)`?
fn builtin_metadata_is_some() -> bool {
    match call_beside_value(file!(), ":user::builtin-metadata").expect("metadata-of eval") {
        Value::Option(o) => o.is_some(),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

/// metadata-of(:my::f) via the user_form fixture — its `:user::compute` matches
/// Some/None and returns the bool the parity claim asserts.
fn user_form_metadata_is_some() -> bool {
    let world = startup_from_file("tests/reflection/probe_arc255_reflection_parity_user_form.wat")
        .expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("compute") {
        Value::bool(b) => b,
        other => panic!("user_form compute must return bool; got {:?}", other),
    }
}

/// The full metadata-of(:wat::core::Bytes::to-hex) map via the co-located fixture.
fn metadata_of_map(_name_kw: &str) -> std::collections::HashMap<Value, Value> {
    match call_beside_value(file!(), ":user::to-hex-metadata").expect("metadata-of eval") {
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
        builtin_metadata_is_some(),
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
        user_form_metadata_is_some(),
        "metadata-of must answer (Some baseline) for a bare user defn — every \
         registered form carries the guaranteed baseline (:defined-in/:layer/\
         :name/:arity). It returned None."
    );
}

// ─── Arc 255.1b-iii — the intrinsic branch, proven on core::Bytes ────────────
//
// ⊘ 2026-08-16 — TWO UNWRITTEN TESTS WERE DELETED FROM HERE:
//
//   metadata_of_answers_for_bytes_to_hex_intrinsic
//   metadata_of_answers_for_bytes_from_hex_intrinsic
//
// Both bodies were `unimplemented!()`. Their `#[ignore]` said "arc-255 metadata-of
// reflection not yet built" — wrong twice over. They were not "not yet built", they were
// not yet WRITTEN; and the capability they waited on HAS ANSWERED SINCE `7b99d123`
// (2026-06-21). Verified live this session:
//
//   (:wat::runtime::metadata-of :wat::core::Bytes::to-hex)
//   ⇒ Some [{:name :wat.core.Bytes/to-hex :arity 1 :kind Intrinsic :defined-in Rust
//            :layer Substrate :purity Pure :determinism Deterministic
//            :doc "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`. …" …}]
//
// which is exactly what they were meant to assert. The capability shipped two months
// before the tests that were to prove it, and the tests were never written.
//
// ⚠ Do NOT resurrect them from git. `metadata-of` currently has TWO tables (the intrinsic
// registry vs `sym.binding_metadata`) and 255's DESIGN rules "the registry IS `sym`" —
// writing a :doc assertion now would pin the June path as correct and pre-empt the
// entry-shape decision the arc reserves as DAY ONE. See:
//
//   docs/arc/2026/06/255-builtin-registry/NOTE-two-unwritten-bytes-metadata-tests-were-deleted.md
//   docs/arc/2026/06/255-builtin-registry/NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md
//
// The two tests ABOVE (metadata_of_answers_for_a_rust_builtin, user_form_carries_
// guaranteed_baseline) are REAL — they assert — and remain `#[ignore]`d on 255.

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
