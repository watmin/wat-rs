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
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Provenance, TrackedValue, Value, ValueSnapshot};

// ─── Probe 1 — Literal{span} on i64 literal ─────────────────────────────────

#[test]
fn probe_1_int_literal_carries_literal_provenance() {
    let world = startup_from_source(
        "(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("42").expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

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
    let world = startup_from_source(
        "(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("\"hello\"").expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

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
    let world = startup_from_source(
        "(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");

    // `(let [x 42] x)` — bind x to 42; reference x in body.
    // The result's provenance should be SymbolBound (from the lookup),
    // NOT Literal{span of 42} (which was the RHS provenance).
    let ast = wat::parse_one!("(:wat::core::let [x 42] x)").expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

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
        assert_ne!(
            binding_span, head_span,
            "binding_span (let LHS x) and head_span (body x) should be \
             DISTINCT source positions; both = {:?}",
            binding_span
        );
    }
}

// ─── Probe 4 — SymbolBound from destructure pattern ─────────────────────────

#[test]
fn probe_4_destructure_slot_lookup_yields_symbol_bound_provenance() {
    let world = startup_from_source(
        "(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");

    // Destructure: `(let [[a b] (tuple 1 2)] a)`. Slot `a` is bound at
    // its position in the LHS pattern. Lookup yields SymbolBound with
    // binding_span pointing at `a` in the pattern.
    let ast = wat::parse_one!("(:wat::core::let [[a b] (:wat::core::tuple 1 2)] a)")
        .expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

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
    let span = wat::span::Span {
        file: Arc::new("test-source.wat".to_string()),
        line: 7,
        col: 13,
    };
    let tv = TrackedValue::new(
        Value::i64(42),
        Provenance::Literal { span },
    );

    let snap = ValueSnapshot::of_tracked(&tv);
    let display = format!("{}", snap);

    // Display impl renders provenance into the string. For Literal, we
    // expect line + col coordinates to appear. The exact format is the
    // substrate's choice; we just assert source-coordinate context surfaces.
    assert!(
        display.contains("7") || display.contains("13") || display.contains("test-source.wat"),
        "Stone 233.2.e: Literal{{span}} provenance must render \
         source-coordinates into Display; got `{}`",
        display
    );
}
