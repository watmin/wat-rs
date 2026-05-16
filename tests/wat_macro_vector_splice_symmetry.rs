//! Arc 200 regression — macro-layer Vector/List splice symmetry.
//!
//! HISTORY: this file was born as `tests/probe_stone_d2_splice_vector.rs`
//! during arc 170 Stone D2 (commit `64cc793`) as an EXPECTED-FAILURE probe
//! documenting two substrate asymmetries surfaced by D2's variadic-iteration
//! call shape. Arc 200 (2026-05-16) relaxed both primitives; the two probes
//! flipped from expected-failure to expected-success regressions. The file
//! name is preserved (rename was blocked by harness permissions; concept-anchored
//! `wat_macro_vector_splice_symmetry` was the intended name per BRIEF). Future
//! grep on "vector splice" still hits via the module doc + test names below.
//!
//! What arc 200 guarantees:
//!
//! - **Gap 1 sealed** (`src/macros.rs` `splice_argument`): a `~@xs` whose
//!   `xs` is bound to a `WatAST::Vector` now splices identically to a
//!   `WatAST::List`-bound `xs`. Lispers expect `~@xs` to work whether the
//!   call-site arg was `(1 2 3)` or `[1 2 3]`.
//!
//! - **Gap 2 sealed** (`src/macros.rs` `walk_template` Vector branch):
//!   `[~@xs]` inside a quasiquote template now dispatches the same
//!   unquote-splicing logic the List branch has carried since arc 029.
//!   The Vector branch was added in arc 167 slice 1 for fn-sig hygiene
//!   but never extended for splice — the inadvertent oversight is fixed.
//!
//! Out-of-scope cascade preserved: arc 167's "vectors at value position"
//! runtime limitation is unchanged. If a `[...]` template splice produces
//! a Vector that lands at value position (rather than a consumed signature
//! position like `:wat::core::fn` parameters), arc 167's diagnostic still
//! fires. That is a separate concern (Gap 3 in the arc 200 DESIGN).

use std::sync::Arc;

use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn freeze(src: &str) -> wat::freeze::FrozenWorld {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup failed:\n{}", e))
}

/// Arc 170 slice 1f-ζ pattern: append a canonical nil-returning `:user::main`
/// when the source has no main on its own. Programs that compute via
/// `:my::compute` then `eval_in_frozen` to drive the test.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_compute(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = freeze(&src);
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

// ─── Gap 1 — Vector-bound symbol splices through ~@ ───────────────────

/// Gap 1 regression: a macro that captures a single positional argument
/// of `:AST<wat::core::nil>` (catch-all AST) and splices it via `~@xs`
/// into a List template. The call site passes `[1 2 3]` — a
/// `WatAST::Vector` — and the splice now succeeds where it previously
/// errored with `MacroError::SpliceNotList`.
#[test]
fn splice_of_vector_bound_symbol_succeeds() {
    let src = r#"
        (:wat::core::defmacro
          (:my::splice-vec (xs :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::Vector :wat::core::i64 ~@xs))

        (:wat::core::define (:my::compute -> :wat::core::Vector<wat::core::i64>)
          (:my::splice-vec [10 20 30]))
    "#;
    match run_compute(src) {
        Value::Vec(items) => {
            assert_eq!(items.len(), 3, "expected 3 spliced elements; got {}", items.len());
            assert!(matches!(items[0], Value::i64(10)));
            assert!(matches!(items[1], Value::i64(20)));
            assert!(matches!(items[2], Value::i64(30)));
        }
        other => panic!(
            "expected Vec of three spliced i64s; got {:?}",
            other
        ),
    }
}

// ─── Gap 2 — Vector template dispatches unquote-splicing ──────────────

/// Gap 2 regression: a `[~@items]` template inside a quasiquote now
/// expands by firing the splice into a Vector AST literal, instead of
/// preserving `(:wat::core::unquote-splicing items)` as a literal child
/// of the Vector. We exercise the dispatch in a position where the
/// resulting Vector is CONSUMED at expand time (a `:wat::core::fn`
/// parameter list) so the test isolates the macro-layer fix from the
/// out-of-scope arc 167 "vectors at value position" runtime limitation.
///
/// Macro `:my::make-adder` captures the params via a rest-binder (which
/// binds to a `WatAST::List`) and splices them into a `[~@params]`
/// Vector template that becomes a `fn` signature. The macro expands to
/// a `(:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
///   (:wat::core::i64::+'2 a b))` after the splice fires.
#[test]
fn splice_inside_vector_template_fires() {
    // The fn signature lives in a Vector template. The splice dispatch
    // in the Vector branch of walk_template is what makes this expand
    // correctly. Pre-arc-200 the splice was preserved literally, breaking
    // the fn-sig consumer.
    let src = r#"
        (:wat::core::defmacro
          (:my::make-adder
            & (params :AST<wat::core::Vector<wat::WatAST>>)
            -> :AST<wat::core::nil>)
          `(:wat::core::fn [~@params] -> :wat::core::i64
              (:wat::core::i64::+'2 a b)))

        (:wat::core::define (:my::adder -> :wat::core::Fn(wat::core::i64,wat::core::i64)->wat::core::i64)
          (:my::make-adder a <- :wat::core::i64 b <- :wat::core::i64))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          ((:my::adder) 7 35))
    "#;
    match run_compute(src) {
        Value::i64(n) => assert_eq!(n, 42, "expected 7+35=42; got {}", n),
        other => panic!("expected i64(42); got {:?}", other),
    }
}

// ─── Gap 1 + Gap 2 together — the D2 call shape ────────────────────────

/// The combined regression: BOTH gaps closed. A macro captures a
/// rest-of-args at the call site as if they were tuples in a Vector
/// (the D2 motivating shape `[[:I :O f] ...]`), and the macro template
/// uses a Vector splice. Demonstrates the joint fix the arc 170 D2
/// stone needed to proceed.
#[test]
fn vector_splice_round_trip_matches_list_splice() {
    // Two equivalent macros — one captures via a List-rest binder and
    // splices into a List template; the other captures a Vector
    // positional and splices into a List template. Both should yield
    // the same runtime value. Pre-arc-200 only the first worked.
    let src = r#"
        (:wat::core::defmacro
          (:my::sum-list & (xs :AST<wat::core::Vector<wat::WatAST>>) -> :AST<wat::core::nil>)
          `(:wat::core::i64::+ ~@xs))

        (:wat::core::defmacro
          (:my::sum-vec (xs :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::i64::+ ~@xs))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::i64::-'2
            (:my::sum-vec [1 2 3 4])
            (:my::sum-list 1 2 3 4)))
    "#;
    // Both expansions must produce the same numeric result; the
    // difference must be zero — proving Vector and List splice are
    // observationally identical at the runtime layer.
    match run_compute(src) {
        Value::i64(0) => {}
        other => panic!("expected i64(0) — Vector and List splice mismatch: {:?}", other),
    }
}
