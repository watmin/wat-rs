//! Diagnostic probe — ValueSnapshot in RuntimeError messages (arc 233 Stone 233.1).
//!
//! Three RuntimeError variants currently carry `&'static str` got/expected
//! fields that lose the actual value content:
//!
//!   NotCallable    { got: &'static str, span }
//!   TypeMismatch   { op, expected: &'static str, got: &'static str, span }
//!   BadCondition   { got: &'static str, span }
//!
//! These probes assert that error messages contain the RENDERED VALUE
//! (not just the type name). Currently FAIL — errors only carry type
//! names. After Stone 233.1 ships, errors carry
//! ValueSnapshot { type_name, rendered, provenance: Provenance::Unknown }
//! and the Display output includes the rendered content.
//!
//! v1 scope: just the rendered field. Provenance always Unknown.
//! (Stone 233.2 fills in Provenance::Literal/SymbolBound/RuntimeBuilt.)
//!
//! Renders come from src/runtime.rs:17382 `render_value(v, depth)` —
//! existing primitive that produces Display-shape output for any Value.
//!
//! IMPORTANT: these probes hit RUNTIME errors specifically. The
//! `CheckError::TypeMismatch` (src/check.rs) is a DIFFERENT enum with
//! its own fields and is OUT OF SCOPE for 233.1. Check-time errors
//! operate at the type level (not value level) so the "include the
//! rendered value" pattern doesn't apply directly.
//!
//! Outcomes:
//!   ALL PASS — Stone 233.1 complete; inline-value gap closed for the
//!              runtime path.
//!   ANY FAIL — specific RuntimeError variant's promotion incomplete.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1: NotCallable renders the offending keyword content ─────────────
//
// Symbol-bound keyword as head (NOT via apply) — currently triggers
// NotCallable because eval_list's regular dispatch doesn't handle
// Symbol-bound keyword Values.
//
// Error message should include ":wat::core::i64::+'2" (the rendered
// keyword content), not just "wat::core::keyword" (the type name).
#[test]
fn probe_1_not_callable_renders_offending_keyword() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::i64)
  (:wat::core::let
    [plus :wat::core::i64::+'2]
    (plus 2 3)))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 1: expected NotCallable; got {:?}", v),
        Err(e) => {
            println!("Probe 1 error: {}", e);
            assert!(
                e.contains("wat::core::keyword"),
                "Probe 1: error should mention the type name; got: {}",
                e
            );
            assert!(
                e.contains(":wat::core::i64::+'2"),
                "Probe 1: error should include the RENDERED keyword content; got: {}",
                e
            );
        }
    }
}

// ─── Probe 2: NotCallable renders runtime-built keyword content ─────────────
//
// Keyword built at runtime via keyword/from-string, then attempted as call
// head (NOT via apply). Should hit NotCallable with the runtime-built
// keyword value rendered in the error.
//
// This is the canonical "the source can't reveal the value; need inline
// render" case from INVENTORY § O's three-case table.
#[test]
fn probe_2_not_callable_renders_runtime_built_keyword() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::i64)
  (:wat::core::let
    [head (:wat::core::keyword/from-string "ns::nonexistent-verb")]
    (head 1 2)))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 2: expected NotCallable; got {:?}", v),
        Err(e) => {
            println!("Probe 2 error: {}", e);
            assert!(
                e.contains("wat::core::keyword"),
                "Probe 2: error should mention the type name; got: {}",
                e
            );
            assert!(
                e.contains("ns::nonexistent-verb") || e.contains(":ns::nonexistent-verb"),
                "Probe 2: error should include the RENDERED runtime-built keyword; got: {}",
                e
            );
        }
    }
}

// NOTE on RuntimeError::TypeMismatch + RuntimeError::BadCondition probes:
//
// Both are runtime-only errors, but constructing a wat program that reaches
// them WITHOUT being caught at check-time is non-trivial — the type-checker
// catches most mismatches statically (heterogeneous vec literals, non-bool
// conditions in `if`, etc. all check at compile-time).
//
// Runtime triggers exist (polymorphic dispatch arms, runtime-built values,
// apply-spread mismatches via runtime-built Vectors) but require careful
// fixture construction. Sonnet should add 2-3 additional probes during
// the Stone 233.1 sweep — they have more visibility into the dispatch
// substrate's runtime-error trigger patterns than this orchestrator-side
// probe pass.
//
// The Stone 233.1 BRIEF asks sonnet to:
//   - Promote `got`/`expected` `&'static str` fields to ValueSnapshot in
//     all three variants (NotCallable, TypeMismatch, BadCondition)
//   - Add probes covering TypeMismatch + BadCondition runtime triggers
//   - Existing tests that assert error message contents may need updates
//     (part of the sweep)
