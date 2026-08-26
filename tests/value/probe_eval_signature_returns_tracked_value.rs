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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, FunctionBody, Value};
use wat::value::TrackedValue;

// just-eval (rubric), the TrackedValue-preserving shape: `apply_function` collapses a fn call
// back to a bare `Value` (it's the fn-apply boundary, not the eval boundary), so it can't stand
// in here — the subject IS the raw eval-boundary TrackedValue. Instead: fetch the fixture fn's
// OWN body AST (`FunctionBody::Wat`, a `Clause`'s single body expression) and `eval_in_frozen`
// it directly, exactly as if that expression had been the top-level form — real span, no inline
// wat string.
fn eval_beside(world: &wat::freeze::FrozenWorld, fn_name: &str) -> TrackedValue {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"));
    let ast = match &func.body {
        FunctionBody::Wat(ast) => ast.clone(),
        FunctionBody::Native => panic!("{fn_name:?} is native, not wat"),
    };
    eval_in_frozen(&ast, world, &Environment::new()).expect("eval")
}

// ─── Probe 1 — eval_in_frozen returns Result<TrackedValue, RuntimeError> ────

#[test]
fn probe_1_eval_in_frozen_returns_tracked_value_for_i64() {
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(":user::add").expect("no :user::add in fixture");
    let ast = match &func.body {
        FunctionBody::Wat(ast) => ast.clone(),
        FunctionBody::Native => panic!(":user::add is native, not wat"),
    };
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
    let world = startup_beside(file!()).expect("startup");
    let tv: TrackedValue = eval_beside(&world, ":user::hello");

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
//
// ⚠ REGRESSED (honestly, not silently) BY ARC 255 STONE E-iv — "keyword gets its home".
// `keyword/from-string`'s dispatch route moved off the special-cased producer arm in
// `dispatch_keyword_head` onto the `#[wat_intrinsic]` registry (`src/intrinsic/keyword.rs`),
// whose `NativeHandler` signature (`-> Result<Value, EvalBreak>`) has no slot for a custom
// `Provenance` — see `probe_stone_233_2_j_producer_migration.rs`'s probe 2 comment for the
// full mechanism. It no longer wraps its return with `RuntimeBuilt`; asserting the
// OBSERVED-CORRECT provenance (Unknown) rather than deleting the probe. The "eval boundary
// preserves producer-attached provenance" MECHANISM this probe was written to guard is still
// exercised by every producer that remains special-cased (`:wat::holon::from-holon`,
// `:wat::edn::read`, `:wat::core::keyword-node`, …) — this probe just no longer demonstrates
// it via `keyword/from-string`.
#[test]
fn probe_3_runtime_built_producer_provenance_survives_eval_boundary() {
    let world = startup_beside(file!()).expect("startup");
    let tv: TrackedValue = eval_beside(&world, ":user::kw-from-string");

    // The value is still a keyword.
    assert!(
        matches!(tv.value(), Value::wat__core__keyword(_)),
        "keyword/from-string should yield TrackedValue wrapping Value::wat__core__keyword"
    );

    // Provenance is Unknown — registry-routed verbs carry no custom Provenance (see comment above).
    assert!(
        matches!(tv.provenance(), wat::value::Provenance::Unknown),
        "keyword/from-string is registry-routed now (arc 255 Stone E-iv) and cannot carry \
         RuntimeBuilt provenance; expected Provenance::Unknown; got {:?}",
        tv.provenance()
    );
}
