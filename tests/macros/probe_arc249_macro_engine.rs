//! FM-2-bis probe for Arc 249 Stone 249.2b — the total-pure macro-eval engine.
//!
//! ROW STATUS:
//!   - A REGRESSION (GREEN): a PURE computed-unquote still expands identically.
//!   - B MINT: an IMPURE computed-unquote must be REJECTED.
//!   - C MINT: a macro body that is an `if` (not a bare quasiquote) must expand.
//!   - D MINT: a fold-shaped program body must expand.
//!   - E HYGIENE BOUND: a program body with literal binder in quasiquote is REFUSED.
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_macro_engine

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside_value` because this probe drives several distinct co-located fixtures from one `.rs`.)
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval")
}

// ═══════════════════════════════════════════════════════════════════════════
// A — REGRESSION
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn regression_pure_computed_unquote_preserved() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_regression.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// B — F5 CLOSURE: an IMPURE computed-unquote must be REJECTED.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_impure_computed_unquote_rejected() {
    let result = startup_from_file("tests/macros/probe_arc249_macro_engine_impure.wat.bad");
    // Phase 3 (296-L) repair: the fixture's return-type spelling `(:AST :- [:wat::holon::HolonAST])`
    // failed defmacro SIGNATURE PARSING (`MalformedDefmacro "expected return-type keyword after
    // \`->\`"`) before the F5 purity gate under test ever ran. Respelled to `:wat::WatAST` — the
    // bare keyword defmacro's own parser requires for a macro's return type (a macro always
    // expands to a form) — which is also what the working fixtures in this directory use
    // (`probe_arc249_macro_engine_regression.wat`). The gate now fires for real.
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError { kind: MacroErrorKind::RefusedInMacro { head }, .. })
            if head == ":wat::kernel::stopped?"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// C — MINIMAL PROGRAM BODY: a macro body that is an `if` (not a bare quasiquote).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_program_body_if() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_prog_if.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// D — FOLD-SHAPED PROGRAM BODY.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_program_body_fold() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_prog_fold.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// E — HYGIENE BOUND.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn hygiene_bound_program_body_literal_binder_refused() {
    let result = startup_from_file("tests/macros/probe_arc249_macro_engine_hygiene.wat.bad");
    // Phase 3 (296-L) repair: same defect class as `mint_impure_computed_unquote_rejected`
    // above — the outer defmacro's param type `:wat::holon::HolonAST` and return type
    // `(:AST :- [:wat::holon::HolonAST])` both failed defmacro's own signature constraint (a
    // macro param/return always binds/produces a FORM, so it must be `:wat::WatAST`), before
    // the hygiene-bound logic under test ever ran. Respelled the outer signature to
    // `:wat::WatAST` (leaving the inner `if`'s unrelated `-> (:AST :- [...])` annotation
    // alone — that's a value-level type ascription on the `if`, not part of defmacro's own
    // signature). The gate now fires for real.
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder },
            ..
        }) if macro_name == ":my::capturing" && binder == "tmp"
    );
}
