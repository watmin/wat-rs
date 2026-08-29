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
//!   - @yields cross-check bites on wrong type (yields_type_matches_fn_arg_param test) — arc
//!     255 Stone P5-b later DELETED this test: `@yields` no longer carries a type to drift
//!     (`@yields <argname> <desc>`, subject-only), and the mandate it also enforced (a
//!     value-carrying fn-shaped `@arg` must carry a matching subject) moved to an
//!     expand-time `compile_error!` in `wat_intrinsic.rs`.
//!   - render-doc shows Category: and Yields: lines (Yields: is now an N-line SECTION,
//!     one line per subject, P5-b).

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): the intrinsic probes live in co-located fixtures — the
// bare-world calls in `probe_arc255_spec_complete.wat` (driven via `call_beside_value`)
// and the higher-order yields-witness in `probe_arc255_spec_complete_yields_witness.wat`.
// The Rust side inspects the returned typed Value.

/// Invoke a zero-arg fn in the co-located `.wat` and return its i64 result.
fn call_i64(fn_name: &str) -> i64 {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// Invoke a zero-arg fn in the co-located `.wat` and return its String result.
fn call_string(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("render-doc must return String; got {:?}", other),
    }
}

/// Run the `:user::compute` defn from a fixture file and return the i64 result.
fn run_program_i64_from_file(fixture: &str) -> i64 {
    let world = startup_from_file(fixture).expect("startup");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}


// ─── Part A: variadic-args-measurement ───────────────────────────────────────

/// The variadic witness with 3 args returns 3.
#[test]
fn variadic_args_measurement_three_args() {
    let result = call_i64(":user::variadic-three");
    assert_eq!(
        result, 3,
        "variadic-args-measurement with 3 args must return 3; got {:?}",
        result
    );
}

/// The variadic witness with 0 args returns 0.
#[test]
fn variadic_args_measurement_zero_args() {
    let result = call_i64(":user::variadic-zero");
    assert_eq!(
        result, 0,
        "variadic-args-measurement with 0 args must return 0; got {:?}",
        result
    );
}

/// The variadic witness with 1 arg returns 1.
#[test]
fn variadic_args_measurement_one_arg() {
    let result = call_i64(":user::variadic-one");
    assert_eq!(
        result, 1,
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
    let s = call_string(":user::render-yields");
    assert_eq!(
        s,
        ":wat::intrinsic::yields-witness\n\nA minimal higher-order-function witness for `@yields` (arc 255 spec-complete).\n\nApplies `f` to the constant value `42` and returns `f(42)`. The yielded\nvalue is `:wat::core::i64`; `@yields` documents the type handed to `f`.\n\nSyntax: (yields-witness <f>)\n\nCategory: ControlFlow\n\nPurity: Pure\n\nDeterminism: Deterministic\n\nYields:\n  f :wat::core::i64\n\nExamples:\n  (:wat::intrinsic::yields-witness (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1)))  #=> 43\n",
        "render-doc for yields-witness must match golden"
    );
}

// ─── Part C: @category ────────────────────────────────────────────────────────

/// render-doc output for bytes::to-hex includes a Category: Transform line.
#[test]
fn render_doc_shows_category_encoding() {
    let s = call_string(":user::render-to-hex");
    assert_eq!(
        s,
        ":wat::core::Bytes::to-hex\n\nEncode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body.\n\nSyntax: (to-hex <bs>)\n\nCategory: Transform\n\nPurity: Pure\n\nDeterminism: Deterministic\n\nExamples:\n  (:wat::core::Bytes::to-hex (:wat::core::Vector :u8 (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16)))  #=> \"ff0010\"\n\nSee also:\n  :wat::core::Bytes::from-hex\n",
        "render-doc for Bytes::to-hex must match golden"
    );
}

/// render-doc output for variadic-args-measurement includes a Category: Reflection line.
#[test]
fn render_doc_shows_category_reflection() {
    let s = call_string(":user::render-variadic");
    assert_eq!(
        s,
        ":wat::intrinsic::variadic-args-measurement\n\nCount the number of arguments passed — a variadic intrinsic witness.\n\nAccepts zero or more arguments (any type); evaluates none of them.\nReturns the argument count as `:wat::core::i64`. Pure and deterministic.\n\nSyntax: (variadic-args-measurement <xs>)\n\nCategory: Reflection\n\nPurity: Pure\n\nDeterminism: Deterministic\n\nExamples:\n  (:wat::intrinsic::variadic-args-measurement 1 2 3)  #=> 3\n  (:wat::intrinsic::variadic-args-measurement)  #=> 0\n",
        "render-doc for variadic-args-measurement must match golden"
    );
}

/// metadata-of returns :category for a registered intrinsic.
#[test]
fn metadata_of_returns_category() {
    let result = call_beside_value(file!(), ":user::to-hex-metadata").expect("eval");
    // metadata-of returns Option<HashMap<keyword, Value>>; we just check Some.
    match result {
        Value::Option(o) => assert!(o.is_some(), "metadata-of must return Some for a registered intrinsic"),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}
