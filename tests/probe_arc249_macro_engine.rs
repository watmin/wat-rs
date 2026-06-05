//! FM-2-bis probe for Arc 249 Stone 249.2b — the total-pure macro-eval engine.
//!
//! The capability: a macro body is a TOTAL, PURE combinator program evaluated at
//! expand time (not just a quasiquote template). Mechanism: a fenced `macro_eval`
//! dispatch (blessed pure-combinator heads only, reusing the pure `eval_*` helpers)
//! replaces `runtime::eval` in the computed-unquote pipeline AND evaluates whole
//! macro bodies. Four-questions verdict: ONE body kind (a program); a bare
//! quasiquote is the degenerate program; `expand_template`'s "must be quasiquote"
//! gate is DELETED.
//!
//! ROW STATUS:
//!   - A REGRESSION (GREEN at HEAD + after): an existing PURE computed-unquote
//!     `~(:wat::core::i64::+ …)` still expands identically (behavior preserved
//!     through the reroute).
//!   - B/C/D MINT (RED at HEAD; `#[ignore]`'d):
//!       B — F5 closure: an IMPURE computed-unquote `~(:wat::kernel::…)` must be
//!           REJECTED. At HEAD it runs (the impurity hole) → startup succeeds → RED.
//!       C — minimal program body: a macro body that is an `if` (not a bare
//!           quasiquote) must expand. At HEAD `expand_template` rejects it → RED.
//!       D — fold-shaped program body (the real new power): a macro body that
//!           folds over its variadic args to build a nested form. RED at HEAD.
//!
//! Disconfirm at HEAD:  cargo test --release --test probe_arc249_macro_engine -- --ignored
//! Done when all pass with zero `#[ignore]`.
//! Run: cargo test --release --test probe_arc249_macro_engine

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Eval a bool-returning `:user::compute` with body `body`, after sibling `decls`.
fn eval_bool_with(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::bool {body})",
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

/// True if startup (parse + macro-expand + check) ACCEPTS the program.
fn startup_ok(src: &str) -> bool {
    startup_from_source(&with_nil_main(src), None, Arc::new(InMemoryLoader::new())).is_ok()
}

// ═══════════════════════════════════════════════════════════════════════════
// A — REGRESSION: a PURE computed-unquote still expands. GREEN at HEAD + after.
// `~(:wat::core::i64::+ 1 2)` evaluates at expand time to 3; the macro expands to
// (:wat::core::i64::+ 3 10) → 13.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn regression_pure_computed_unquote_preserved() {
    let decls = "(:wat::core::defmacro :my::pure-cu [] -> :AST<wat::holon::HolonAST> \
                 `(:wat::core::i64::+ ~(:wat::core::i64::+ 1 2) 10))";
    let body = "(:wat::core::= (:my::pure-cu) 13)";
    assert_eq!(eval_bool_with(decls, body).unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// B — F5 CLOSURE: an IMPURE computed-unquote must be REJECTED by the fenced engine.
// `~(:wat::kernel::stopped?)` names a kernel head. At HEAD the unsandboxed eval
// RUNS it (the F5 hole) → startup SUCCEEDS → this assertion (startup must FAIL)
// is RED. After the engine: the kernel head isn't in `macro_eval` → expansion
// errors → startup fails → green.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_impure_computed_unquote_rejected() {
    let src = "(:wat::core::defmacro :my::impure-cu [] -> :AST<wat::holon::HolonAST> \
               `~(:wat::kernel::stopped?))\n\
               (:wat::core::defn :user::probe [] -> :wat::core::bool (:my::impure-cu))";
    assert!(
        !startup_ok(src),
        "an impure computed-unquote `~(:wat::kernel::stopped?)` MUST be rejected by the \
         fenced macro-eval engine (F5 closure); at HEAD it runs — the impurity hole"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// C — MINIMAL PROGRAM BODY: a macro body that is an `if` (not a bare quasiquote).
// At HEAD `expand_template` rejects any non-quasiquote body → startup fails → RED.
// After: `macro_eval` evaluates the `if`, returns the chosen quasiquote → expands.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "arc 249.2b: macro-eval engine not built — un-ignore after it lands"]
fn mint_program_body_if() {
    // body is `(if (= 1 1) `(...) `(...))` — a program, not a bare quasiquote.
    let decls = "(:wat::core::defmacro :my::pick [x <- :wat::holon::HolonAST] \
                 -> :AST<wat::holon::HolonAST> \
                 (:wat::core::if (:wat::core::= 1 1) -> :AST<wat::holon::HolonAST> \
                   `(:wat::core::i64::+ ~x 1) \
                   `(:wat::core::i64::+ ~x 2)))";
    let body = "(:wat::core::= (:my::pick 10) 11)";
    assert_eq!(eval_bool_with(decls, body).unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// D — FOLD-SHAPED PROGRAM BODY (the real new power the template layer cannot
// express): a variadic macro whose body FOLDS over its args to build a nested
// form. `(:my::sum 1 2 3)` → `(i64::+ (i64::+ (i64::+ 0 1) 2) 3)` → 6.
// Quasiquote is the form-builder; `foldl` is the logic. RED at HEAD.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "arc 249.2b: macro-eval engine not built — un-ignore after it lands"]
fn mint_program_body_fold() {
    let decls = "(:wat::core::defmacro :my::sum [& nums <- :AST<wat::holon::Holons>] \
                 -> :AST<wat::holon::HolonAST> \
                 (:wat::core::foldl \
                   (:wat::core::fn [acc <- :wat::holon::HolonAST n <- :wat::holon::HolonAST] \
                      -> :wat::holon::HolonAST `(:wat::core::i64::+ ~acc ~n)) \
                   `0 \
                   nums))";
    let body = "(:wat::core::= (:my::sum 1 2 3) 6)";
    assert_eq!(eval_bool_with(decls, body).unwrap(), Value::bool(true));
}
