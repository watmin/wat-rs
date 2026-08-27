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

use wat::freeze::startup_from_file;
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
    // ⛔ STOP-1 FINDING (arc 296 Stone L) — NOT migrated. The message claims the F5 purity
    // gate rejects the impure computed-unquote. Grounded via `./target/release/wat --check`:
    // the ACTUAL error is `#wat.macro/MalformedDefmacro {:reason "expected return-type
    // keyword after `->`"}` — a signature-PARSE failure on this fixture's return-type
    // spelling `(:AST :- [:wat::holon::HolonAST])`, which the working fixtures in this same
    // directory (e.g. probe_arc241_stone17_defmacro_canonical_c01.wat,
    // probe_arc249_macro_engine_regression.wat) spell as a bare keyword (`:wat::WatAST`).
    // The fixture's return-type syntax appears stale relative to what defmacro's parser
    // accepts today, so this test currently fails BEFORE it ever reaches the F5
    // impurity-rejection logic it claims to exercise. Left as a bare `is_err()` rather than
    // asserting a discriminant for a code path this fixture never reaches — migrating it
    // would fabricate grounding, and rewriting the message would erase the discovery.
    assert!(
        result.is_err(),
        "an impure computed-unquote `~(:wat::kernel::stopped?)` MUST be rejected by the \
         fenced macro-eval engine (F5 closure); at HEAD it runs — the impurity hole"
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
    // ⛔ STOP-1 FINDING (arc 296 Stone L) — NOT migrated, same defect class as
    // `mint_impure_computed_unquote_rejected` above. The message claims a hygiene-bound
    // refusal (gate E) fires. Grounded via `--check`: the ACTUAL error is
    // `#wat.macro/MalformedDefmacro {:reason "expected return-type keyword after `->`"}` —
    // this fixture ALSO spells its return type `(:AST :- [:wat::holon::HolonAST])`, which
    // fails at signature-parse time, before the hygiene-bound logic under test ever runs.
    // Left as a bare `is_err()`; migrating it would assert a discriminant for logic this
    // fixture never reaches.
    assert!(
        result.is_err(),
        "a program body whose quasiquote introduces a literal let-binder name must be \
         refused (hygiene bound gate E, arc 249 stone 249.2b-ii)"
    );
}
