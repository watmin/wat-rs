//! Arc 255 spec-complete witnesses — variadic, @yields, @category (Part A/B/C).
//!
//! RED at HEAD (before this strike):
//!   - `:wat::intrinsic::variadic-args-measurement` does not exist yet.
//!   - `@yields` directive is not parsed.
//!   - `@category` is not required.
//!
//! GREEN after this strike:
//!   - variadic-args-measurement returns the count of args.
//!   - yields-witness applies f(42).
//!   - @category bites on unknown variant (compile_error!).
//!   - @yields cross-check bites on wrong type (yields_type_matches_fn_arg_param test).
//!   - render-doc shows Category: and Yields: lines.

use wat::freeze::{eval_in_frozen, startup_bare, startup_from_file};
use wat::parse_one_with_file;
use wat::runtime::{Environment, Value};

/// Eval an intrinsic expression in a bare world (no user source needed).
fn eval_expr(expr: &str) -> Value {
    let world = startup_bare().expect("startup");
    let ast = parse_one_with_file(expr, "<probe>").expect("parse");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .expect("eval")
        .value_owned()
}

/// Run the `:user::compute` defn from a fixture file and return the i64 result.
fn run_program_i64_from_file(fixture: &str) -> i64 {
    let world = startup_from_file(fixture).expect("startup");
    let ast = parse_one_with_file("(:user::compute)", "<probe>").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}


// ─── Part A: variadic-args-measurement ───────────────────────────────────────

/// The variadic witness with 3 args returns 3.
#[test]
fn variadic_args_measurement_three_args() {
    let result = eval_expr("(:wat::intrinsic::variadic-args-measurement 1 2 3)");
    assert_eq!(
        result,
        Value::i64(3),
        "variadic-args-measurement with 3 args must return 3; got {:?}",
        result
    );
}

/// The variadic witness with 0 args returns 0.
#[test]
fn variadic_args_measurement_zero_args() {
    let result = eval_expr("(:wat::intrinsic::variadic-args-measurement)");
    assert_eq!(
        result,
        Value::i64(0),
        "variadic-args-measurement with 0 args must return 0; got {:?}",
        result
    );
}

/// The variadic witness with 1 arg returns 1.
#[test]
fn variadic_args_measurement_one_arg() {
    let result = eval_expr("(:wat::intrinsic::variadic-args-measurement :x)");
    assert_eq!(
        result,
        Value::i64(1),
        "variadic-args-measurement with 1 arg must return 1; got {:?}",
        result
    );
}

// ─── Part B: yields-witness ───────────────────────────────────────────────────

/// The yields-witness applies f(42), returning f's result.
#[test]
fn yields_witness_applies_fn_to_42() {
    // f = fn [x <- :i64] -> :i64 (+ x 1) -> f(42) = 43
    let n = run_program_i64_from_file(
        "tests/reflection/probe_arc255_spec_complete_yields_witness.wat",
    );
    assert_eq!(
        n, 43,
        "yields-witness(fn [x] (+ x 1)) must return 43; got {}",
        n
    );
}

/// render-doc output for yields-witness includes a Yields: line.
#[test]
fn render_doc_shows_yields_line() {
    let result = eval_expr("(:wat::core::render-doc :wat::intrinsic::yields-witness)");
    let s = match result {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("render-doc must return String; got {:?}", other),
    };
    assert!(
        s.contains("Yields:"),
        "render-doc for yields-witness must include 'Yields:' line; got:\n{}",
        s
    );
    assert!(
        s.contains(":wat::core::i64"),
        "render-doc for yields-witness must include the yields type ':wat::core::i64'; got:\n{}",
        s
    );
}

// ─── Part C: @category ────────────────────────────────────────────────────────

/// render-doc output for bytes::to-hex includes a Category: Encoding line.
#[test]
fn render_doc_shows_category_encoding() {
    let result = eval_expr("(:wat::core::render-doc :wat::core::Bytes::to-hex)");
    let s = match result {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("render-doc must return String; got {:?}", other),
    };
    assert!(
        s.contains("Category: Encoding"),
        "render-doc for Bytes::to-hex must include 'Category: Encoding'; got:\n{}",
        s
    );
}

/// render-doc output for variadic-args-measurement includes a Category: Reflection line.
#[test]
fn render_doc_shows_category_reflection() {
    let result =
        eval_expr("(:wat::core::render-doc :wat::intrinsic::variadic-args-measurement)");
    let s = match result {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("render-doc must return String; got {:?}", other),
    };
    assert!(
        s.contains("Category: Reflection"),
        "render-doc for variadic-args-measurement must include 'Category: Reflection'; got:\n{}",
        s
    );
}

/// metadata-of returns :category for a registered intrinsic.
#[test]
fn metadata_of_returns_category() {
    let result = eval_expr("(:wat::runtime::metadata-of :wat::core::Bytes::to-hex)");
    // metadata-of returns Option<HashMap<keyword, Value>>; we just check Some.
    match result {
        Value::Option(o) => assert!(o.is_some(), "metadata-of must return Some for a registered intrinsic"),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}
