//! Integration tests for arc 167 — flat-shape fn signature.
//!
//! `:wat::core::fn` consumes the canonical 5-element flat shape:
//!
//!   (:wat::core::fn  ARGS-VECTOR  ->  :RET-TYPE  BODY)
//!
//! `ARGS-VECTOR` is a `WatAST::Vector` whose body is flat triples
//! `name <- :T name <- :T ...` (empty vector → zero-arity fn). The
//! `<-` arrow reads "consumes" (input direction); the sibling `->`
//! reads "produces" (output direction). Arrows-as-duals.
//!
//! Slice 4 hard-retired the legacy nested-sig parser arm; legacy
//! syntax `((p :T) ... -> :R)` post-retirement produces a generic
//! `MalformedForm` parser error rather than a dedicated walker
//! diagnostic.
//!
//! ## Test cases
//!
//!   1. `fn_with_flat_shape_compiles_and_runs` — basic positive path
//!   2. `defn_with_flat_shape_compiles_and_runs` — defn macro forwards
//!   3. `recursive_defn_with_flat_shape` — fact(5)=120 via flat-shape defn
//!   4. `zero_arg_fn_with_empty_vector` — empty `[]` args
//!   7. `fn_body_type_mismatch_surfaces` — declared-vs-actual ret mismatch
//!   8. `malformed_args_vector_clear_error` — clear error on missing `<- :T`
//!   9. `reflection_on_flat_defn_resolves` — `lookup-define` round-trip
//!
//! Wat source: tests/function/fn_signature.wat (positive, shared via startup_beside)
//! and tests/function/fn_signature_*.wat (negative fixtures).

use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `compute_fn` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(compute_fn: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(compute_fn)
        .unwrap_or_else(|| panic!("no {compute_fn} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should run")
}

fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        // Stone B: `Display` and `Debug` both emit EDN now, so the old
        // `format!("{}\n---\n{:?}", e, e)` concatenation just glued the
        // same EDN face to itself with a "---" separator in between —
        // not valid EDN as a whole. Debug alone is the golden.
        Err(e) => format!("{:?}", e),
    }
}

// ─── Test 1 — fn_with_flat_shape_compiles_and_runs ───────────────────────────

/// Inline `(:wat::core::fn [x <- :i64 y <- :i64] -> :i64 ...)` applied
/// at runtime. Exercises the new 5-element fn-form shape end-to-end:
/// parser → eval_fn → parse_fn_signature → apply_function.
#[test]
fn fn_with_flat_shape_compiles_and_runs() {
    let v = run(":my::compute_t1");
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from (fn ... 2 3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — defn_with_flat_shape_compiles_and_runs ─────────────────────────

/// `:wat::core::defn` with the new flat shape: name + args-vector +
/// `->` + ret-type + body. The defn macro splices the trailing 4
/// pieces directly into `(:wat::core::fn ,@rest)`.
#[test]
fn defn_with_flat_shape_compiles_and_runs() {
    let v = run(":my::compute_t2");
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from add(2,3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — recursive_defn_with_flat_shape ─────────────────────────────────

/// Recursive `defn` with the flat shape — verifies arc 166's recursive
/// name-binding contract survives the shape change.
#[test]
fn recursive_defn_with_flat_shape() {
    let v = run(":my::compute_t3");
    match v {
        Value::i64(n) => assert_eq!(n, 120, "expected 120 from fact(5); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 4 — zero_arg_fn_with_empty_vector ──────────────────────────────────

/// Zero-arity fn — empty args-vector `[]` followed by `-> :Ret body`.
#[test]
fn zero_arg_fn_with_empty_vector() {
    let v = run(":my::compute_t4");
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 from zero-arg fn; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 7 — fn_body_type_mismatch_surfaces ─────────────────────────────────

/// Flat-shape fn whose body's type doesn't match the declared `-> :T`.
#[test]
fn fn_body_type_mismatch_surfaces() {
    let err = startup_err("tests/function/fn_signature_body_mismatch.wat");
    wat::assert_edn_matches_file!(err, "fn_signature__fn_body_type_mismatch_surfaces.edn", "fns7: body-type-mismatch golden");
}

// ─── Test 8 — malformed_args_vector_clear_error ──────────────────────────────

/// Args-vector with a missing `<- :T` triple.
#[test]
fn malformed_args_vector_clear_error() {
    let err = startup_err("tests/function/fn_signature_malformed_args.wat");
    wat::assert_edn_matches_file!(err, "fn_signature__malformed_args_vector_clear_error.edn", "fns8: malformed args-vector golden");
}

// ─── Test 9 — reflection_on_flat_defn_resolves ───────────────────────────────

/// After a flat-shape defn registers `:user::add_t2` in the SymbolTable,
/// `(:wat::runtime::lookup-define :user::add_t2)` returns Some(...).
#[test]
fn reflection_on_flat_defn_resolves() {
    let v = run(":my::compute_t9");
    match v {
        Value::i64(n) => assert_eq!(n, 1, "expected lookup-define to return Some (1); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}
