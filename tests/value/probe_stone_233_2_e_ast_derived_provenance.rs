//! FM 2-bis probe for arc 233 Stone 233.2.e (AST-derived provenance).
//!
//! Asserts that the two latent Provenance variants — Literal{span} and
//! SymbolBound{binding_span, head_span} — are POPULATED on the sealed
//! substrate (post-j/k/l).
//!
//! Pre-stone state:
//!   - Probe 1 FAILS (i64 literal carries Provenance::Unknown; target Literal{span})
//!   - Probe 2 FAILS (String literal carries Unknown; target Literal{span})
//!   - Probe 3 FAILS (let-bound symbol carries whatever the RHS produced;
//!     target SymbolBound{binding_span, head_span} via env.lookup wrap)
//!   - Probe 4 FAILS (destructure slot carries Unknown; target SymbolBound
//!     via per-name span tracking in Destructure pattern)
//!   - Probe 5 FAILS (error message has no source-coordinate context for
//!     literal-bound values; target: Literal{span} renders into Display)
//!
//! Post-stone state: all 5 PASS.
//!
//! Stays as permanent regression guard. Per the diagnostic-richness goal of
//! arc 233: errors gain source-coordinates context; let-bound values name
//! their binding site.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, FunctionBody, Value, ValueSnapshot};
use wat::value::{Provenance, TrackedValue};

// just-eval (rubric), the TrackedValue-preserving shape: `apply_function` collapses a fn call
// back to a bare `Value` (it's the fn-apply boundary, not the eval boundary), so it can't stand
// in for probes 3/4 here — the subject IS the raw eval-boundary TrackedValue/Provenance. Instead:
// fetch the fixture fn's OWN body AST and `eval_in_frozen` it directly, exactly as if that
// expression had been the top-level form — real span, no inline wat string.
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

// ─── Probe 1 — Literal{span} on i64 literal ─────────────────────────────────

#[test]
fn probe_1_int_literal_carries_literal_provenance() {
    let world = startup_beside(file!()).expect("startup");
    let tv: TrackedValue = eval_beside(&world, ":user::int-literal");

    assert!(
        matches!(tv.value(), Value::i64(42)),
        "42 should yield Value::i64(42); got {}",
        tv.value().type_name()
    );

    assert!(
        matches!(tv.provenance(), Provenance::Literal { .. }),
        "Stone 233.2.e: i64 literal must carry Provenance::Literal{{span}}; \
         got {:?}",
        tv.provenance()
    );
}

// ─── Probe 2 — Literal{span} on string literal ──────────────────────────────

#[test]
fn probe_2_string_literal_carries_literal_provenance() {
    let world = startup_beside(file!()).expect("startup");
    let tv: TrackedValue = eval_beside(&world, ":user::string-literal");

    assert!(
        matches!(tv.value(), Value::String(_)),
        "string literal should yield Value::String; got {}",
        tv.value().type_name()
    );

    assert!(
        matches!(tv.provenance(), Provenance::Literal { .. }),
        "Stone 233.2.e: string literal must carry Provenance::Literal{{span}}; \
         got {:?}",
        tv.provenance()
    );
}

// ─── Probe 3 — SymbolBound on let-bound symbol lookup ───────────────────────

#[test]
fn probe_3_let_bound_symbol_lookup_yields_symbol_bound_provenance() {
    let world = startup_beside(file!()).expect("startup");

    // `(let [x 42] x)` — bind x to 42; reference x in body.
    // The result's provenance should be SymbolBound (from the lookup),
    // NOT Literal{span of 42} (which was the RHS provenance).
    let tv: TrackedValue = eval_beside(&world, ":user::let-bound-lookup");

    assert!(
        matches!(tv.value(), Value::i64(42)),
        "let body should evaluate to 42; got {}",
        tv.value().type_name()
    );

    // The KEY assertion: provenance is SymbolBound (from env.lookup), not
    // Literal (which was the bound RHS's original provenance). The lookup
    // boundary REPLACES the stored provenance per sub-DESIGN Decision 2.
    assert!(
        matches!(tv.provenance(), Provenance::SymbolBound { .. }),
        "Stone 233.2.e: let-bound symbol lookup must yield SymbolBound \
         provenance; got {:?}",
        tv.provenance()
    );

    // The binding_span and head_span should both be non-default (i.e., they
    // were extracted from actual source positions).
    if let Provenance::SymbolBound { binding_span, head_span } = tv.provenance() {
        // Span::PartialEq always returns true per substrate contract (span.rs).
        // Compare fields directly to verify the two spans are genuinely distinct
        // source positions.
        assert!(
            binding_span.line != head_span.line || binding_span.col != head_span.col,
            "binding_span (let LHS x) and head_span (body x) should be \
             DISTINCT source positions; \
             binding={:?} head={:?}",
            binding_span, head_span
        );
    }
}

// ─── Probe 4 — SymbolBound from destructure pattern ─────────────────────────

#[test]
fn probe_4_destructure_slot_lookup_yields_symbol_bound_provenance() {
    let world = startup_beside(file!()).expect("startup");

    // Destructure: `(let [[a b] (tuple 1 2)] a)`. Slot `a` is bound at
    // its position in the LHS pattern. Lookup yields SymbolBound with
    // binding_span pointing at `a` in the pattern.
    let tv: TrackedValue = eval_beside(&world, ":user::destructure-lookup");

    assert!(
        matches!(tv.value(), Value::i64(1)),
        "destructured slot `a` should be 1; got {}",
        tv.value().type_name()
    );

    assert!(
        matches!(tv.provenance(), Provenance::SymbolBound { .. }),
        "Stone 233.2.e: destructure slot lookup must yield SymbolBound \
         provenance (per-name span tracking in Destructure pattern); \
         got {:?}",
        tv.provenance()
    );
}

// ─── Probe 5 — Literal{span} renders source-coordinates into Display ────────

#[test]
fn probe_5_literal_provenance_renders_source_coordinates() {
    // Construct a TrackedValue with Literal provenance + non-zero span.
    let span = wat::span::Span::new(Arc::new("test-source.wat".to_string()), 7, 13);
    let tv = TrackedValue::new(
        Value::i64(42),
        Provenance::Literal { span },
    );

    let snap = ValueSnapshot::of_tracked(&tv);
    let display = format!("{}", snap);
    assert_eq!(
        display,
        "wat::core::i64 `42` (from test-source.wat:7:13)",
        "Stone 233.2.e: Literal{{span}} provenance must render source-coordinates into Display"
    );
}
