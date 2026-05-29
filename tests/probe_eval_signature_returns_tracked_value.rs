//! FM 2-bis probe for arc 233 Stone 233.2.i (eval signature flip).
//!
//! Asserts that the public `eval_in_frozen` surface returns
//! `Result<TrackedValue, _>` instead of `Result<Value, _>`.
//!
//! Pre-stone state: FAILS (type-mismatch — eval_in_frozen returns Result<Value, _>).
//! Post-stone state: PASSES (eval + eval_in_frozen return Result<TrackedValue, _>).
//!
//! Stays as permanent regression guard against future eval-boundary drift.
//!
//! Per Stone 233.2.g sub-DESIGN: the eval boundary surfaces TrackedValue;
//! internal callers extract `.value()` / `.value_owned()` to get bare Value.
//! Helpers (require_X, expect_X) take TrackedValue and extract internally.
//!
//! NOTE on transitional state: this stone ships before Stone 233.2.j
//! (producer migration) and 233.2.k (Value::Tracked variant retirement).
//! Pattern-matches on the extracted bare Value remain vulnerable to the
//! Value::Tracked variant until 233.2.k. The CLASS is closed at 233.2.k.
//! This stone establishes the BOUNDARY shape so 233.2.j + 233.2.k can land.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, TrackedValue, Value};

// ─── Probe 1 — eval_in_frozen returns Result<TrackedValue, RuntimeError> ────

#[test]
fn probe_1_eval_in_frozen_returns_tracked_value_for_i64() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:wat::core::+ 2 3)").expect("parse");
    let env = Environment::new();

    // Compile-shape assertion: eval_in_frozen returns Result<TrackedValue, _>.
    // Pre-stone: returns Result<Value, _>; this type annotation FAILS to compile.
    let result: Result<TrackedValue, _> = eval_in_frozen(&ast, &world, &env);

    let tv: TrackedValue = result.expect("(+ 2 3) should succeed");
    assert!(
        matches!(tv.value(), Value::i64(5)),
        "(+ 2 3) should yield TrackedValue wrapping Value::i64(5)"
    );
}

// ─── Probe 2 — TrackedValue API composes with eval_in_frozen result ─────────

#[test]
fn probe_2_eval_result_yields_tracked_value_with_api() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("\"hello\"").expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

    // .value() borrows the inner Value
    assert!(matches!(tv.value(), Value::String(_)));

    // .value_owned() consumes self → bare Value
    let bare: Value = tv.value_owned();
    assert!(
        matches!(bare, Value::String(ref s) if s.as_str() == "hello"),
        "value_owned() should yield bare Value::String(\"hello\")"
    );
}

// ─── Probe 3 — TrackedValue carries provenance from producer-tagged path ────

#[test]
fn probe_3_runtime_built_producer_provenance_survives_eval_boundary() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    // keyword/from-string is a producer (Stone 233.2.b) — wraps return with provenance.
    // Through the eval boundary, the wrapping survives at the TrackedValue layer.
    let ast = wat::parse_one!("(:wat::core::keyword/from-string \"wat::core::nil\")")
        .expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

    // The value is a keyword
    assert!(
        matches!(tv.value(), Value::wat__core__keyword(_)),
        "keyword/from-string should yield TrackedValue wrapping Value::wat__core__keyword"
    );

    // Provenance is RuntimeBuilt with the producer string ":wat::core::keyword/from-string"
    // (This asserts the eval boundary preserves producer-attached provenance.)
    assert!(
        matches!(
            tv.provenance(),
            wat::runtime::Provenance::RuntimeBuilt {
                producer: ":wat::core::keyword/from-string",
                ..
            }
        ),
        "TrackedValue from keyword/from-string should carry RuntimeBuilt provenance; got {:?}",
        tv.provenance()
    );
}
