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
//!
//! Positive tests use the co-located fixture: vector_splice_symmetry.wat
//! (slurped via startup_beside(file!())). Named compute fns: :my::compute-splice,
//! :my::compute-hygienic, :my::compute-round-trip.
//!
//! Negative test uses: tests/macros/vector_splice_symmetry.wat.bad
//! (loaded via startup_from_file; must fail with "hygiene-scope divergence").

use wat::freeze::{call_beside, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside — no inline wat driver expression.

// ─── Gap 1 — Vector-bound symbol splices through ~@ ───────────────────

/// Gap 1 regression: a macro that captures a single positional argument
/// of `:AST<wat::core::nil>` (catch-all AST) and splices it via `~@xs`
/// into a List template. The call site passes `[1 2 3]` — a
/// `WatAST::Vector` — and the splice now succeeds where it previously
/// errored with `MacroError::SpliceNotList`.
#[test]
fn splice_of_vector_bound_symbol_succeeds() {
    match call_beside(file!(), ":my::compute-splice").expect("compute should run") {
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
/// THE HYGIENE REFUSAL WITNESS (re-diagnosed at the 245 close; supersedes the
/// "arc 200 gap 2 / runtime param-binding gap" framing below-in-history).
///
/// The OLD diagnosis ("the fn evaluator's param-binding pass treats the
/// spliced Vector differently") was WRONG. The real mechanism is the 249.5
/// hygiene annihilation working correctly: the body's `a`/`b` are LITERAL
/// TEMPLATE SYMBOLS — walk_template stamps them with the macro's fresh
/// ScopeId — while the binders arrive via `~@params` from the CALLER and
/// keep caller scope-sets. `(name, scope-set)` identities differ, so the
/// body's `a{macro-scope}` does not see the binder's bare `a`. That is
/// TEXTBOOK ANAPHORIC CAPTURE, and hygiene refuses it BY DESIGN. This test
/// passed before 249.5b only because name-only resolution allowed the
/// accidental capture.
///
/// Arc 291: the refusal is now caught at COMPILE time. The `HygieneScopeDivergence`
/// gate (`check.rs` / `src/scope/resolution.rs`) sees the body's `a{scope 433}`
/// reference miss its binder while a same-name `a{}` binder exists under a
/// different scope — and refuses it with a typed check error, earlier and
/// stronger than the old runtime `UnboundSymbol`. (Stale `:AST<…>` rest-param
/// syntax migrated to `:wat::core::Vector<wat::WatAST>` / `-> :wat::WatAST`.)
///
/// PERMANENT witness — never "fix" this; fixing it would mean breaking
/// hygiene. The hygienic way to write this macro is the sibling test below.
#[test]
#[should_panic(expected = "hygiene-scope divergence")]
fn anaphoric_splice_capture_refused_by_hygiene() {
    startup_from_file("tests/macros/vector_splice_symmetry.wat.bad")
        .unwrap_or_else(|e| panic!("startup failed:\n{}", e));
}

/// THE HYGIENIC ADDER — the correct way to write the macro above: the body's
/// references are COMPUTED FROM THE SAME SPLICED MATERIAL as the binders, so
/// the `(name, scope-set)` identities MATCH. NOT YET EXPRESSIBLE — this test
/// is the RED CONTRACT for the splice stone, with the diagnosis chain
/// grounded at the 245 close (2026-06-06):
///   layer 1: the anaphoric form fails by DESIGN (hygiene; sibling test above);
///   layer 2: computed-unquote `(get params 0)` fails — substitute_bindings
///     splices the rest-list as raw AST, so `(a <- :i64 ...)` EVALUATES as a
///     call (unbound symbol: a at expand time);
///   layer 3 (this body): the program-body path binds `params` as a form-value
///     and the let computes the names — but the output template's `~@params`
///     does NOT flatten an env-bound list value into the argspec vector
///     (check: "triple is incomplete; ran out of items").
/// The stone: program-body quasiquote `~@` must splice env-bound list
/// form-values element-wise (the 249.3a splice built for computed-unquote
/// results, extended to the program-body env path). LANDED: the splice stone
/// extended `walk_quasiquote`'s Vector arm (runtime.rs) to mirror the List
/// arm's `~@` depth-1 splice loop — `Value::Vec` of `Value::wat__WatAST`
/// elements are now flattened element-wise into the argspec Vector.
#[test]
fn hygienic_splice_adder_binds_via_spliced_names() {
    match call_beside(file!(), ":my::compute-hygienic").expect("compute should run") {
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
    // Both expansions must produce the same numeric result; the
    // difference must be zero — proving Vector and List splice are
    // observationally identical at the runtime layer.
    match call_beside(file!(), ":my::compute-round-trip").expect("compute should run") {
        Value::i64(0) => {}
        other => panic!("expected i64(0) — Vector and List splice mismatch: {:?}", other),
    }
}
