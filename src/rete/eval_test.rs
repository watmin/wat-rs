//! Fenced expression eval under token bindings — `eval-test` / `eval_rhs_expr`.
//!
//! `where` / `:test` raise (not Clara). Shared by interpreted RHS operands.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value, ValueSnapshot};

/// Interpreter / differential for a fenced `:then` operand against one token's bindings.
/// Live caller is [`resolve_rhs_value`]. Compiled `RhsOp::Expr` runs `expr_ir::exec_value`
/// (`compiled_rhs::exec_compiled_rhs` never calls this). Mirrors [`build_test_env`]'s own
/// child-`Environment`-over-`bindings` construction exactly (the same one `eval_test_core`
/// uses for a `where` predicate): a fresh base `Environment` is correct here for the same
/// reason it is there — the only names a fenced `:then` expression may reference are its
/// `?vars` and `sym`'s registered functions.
pub(crate) fn eval_rhs_expr(
    expr: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let expr_env = build_test_env(bindings, &Environment::new());
    Ok(crate::runtime::eval_inner(expr, &expr_env, sym)?.value_owned())
}

// ─── Arc 278 Stone 6b-i: eval-test ────────────────────────────────────────────

/// Build the CHILD `Environment` a `where` predicate is evaluated in — one binding per `?var` the
/// token carries.
///
/// Extracted from [`eval_test_core`] (also used by [`eval_rhs_expr`]) so that
/// DESIGN-STONE-compiled-where's **Step 0** can time this block ALONE against the block plus the
/// `eval_inner` walk, without duplicating it in a test where it would drift from the real path
/// (`[[feedback_feasibility_probe_must_exercise_the_exact_mechanism]]` — a probe that does not walk
/// the exact substrate path production uses proves nothing). Pure extraction: no behaviour change.
///
/// Measured (Step 0, 2026-08-01): **122.5 ns/eval — 22.7% of a `where` evaluation** when this
/// interpreter was the TestNode path. Native TestNode fire is `exec_where` over `BindSpan`
/// (matching [`eval_test_core`]). Live callers: [`eval_test`] (dispatch), [`eval_rhs_expr`],
/// and kernel tests' differential.
pub(crate) fn build_test_env<B: Bindings + ?Sized>(bindings: &B, env: &Environment) -> Environment {
    crate::rete::kernel::census_count("filter:test-env-builds");
    let mut b = env.child();
    for (k, v) in bindings.iter() {
        let name = match k {
            Value::String(s) => s.as_str().to_string(),
            _ => continue, // non-string key: skip (should not occur in well-formed bindings)
        };
        crate::rete::kernel::census_count("filter:test-key-alloc");
        b = b.bind_unknown_span(name, TrackedValue::from(v.clone()));
    }
    b.build()
}

/// Core evaluator for a `where` predicate — callable without the `eval-test` dispatch wrapper.
///
/// Builds a CHILD `Environment` from `bindings` (keys are `Value::String("?x")`),
/// evaluates `expr` in it, and requires `Value::bool`. Live callers: [`eval_test`] (dispatch)
/// and kernel tests' differential. Native TestNode fire is `exec_where` over `BindSpan`,
/// not this interpreter.
///
/// Pass a fresh `env` (`&Environment::new()`) — the only names a `where` expression may
/// reference are its `?vars` (from `bindings`) and `sym`'s registered user functions.
///
/// Generic over [`Bindings`] — today's callers always pass a Token's `PMap` (a `:test` clause
/// evaluates after a join in the fixtures exercised so far), but a `:test` clause may in
/// principle sit right after a single condition (element-side), so the reader stays agnostic
/// rather than assuming a representation.
pub(crate) fn eval_test_core<B: Bindings + ?Sized>(
    expr: &WatAST,
    bindings: &B,
    // rune:purgare(trait-contract) — parent Environment is the eval_inner signature; callers pass empty
    env: &Environment,
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Build a CHILD Environment binding each ?var → value. The cost this carries, and why
    // DESIGN-STONE-compiled-where targets it, is documented on `build_test_env` itself — where
    // Step 0's timing arm calls the same body, so the two cannot drift.
    let test_env = build_test_env(bindings, env);

    // Evaluate the predicate expr in the test env; result MUST be bool.
    match crate::runtime::eval_inner(expr, &test_env, sym)?.value_owned() {
        Value::bool(x) => Ok(x),
        other => Err(RuntimeError::new(expr.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            })
        .into()),
    }
}

/// `(:wat::rete::eval-test expr bindings) -> :wat::core::bool`
///
/// Dispatch wrapper: evaluates the two args, extracts the `WatAST` and `PersistentMap`,
/// then delegates to `eval_test_core`. No behavior change from the previous monolithic
/// implementation — the core extraction is a refactor only.
///
/// The four-axis compile-condition fence (pure ∧ det ∧ total ∧ rete) proves `expr` safe
/// BEFORE a `where`/`:test` clause is compiled into a rule, so `eval_test_core` does not
/// re-run it on that path. ⚠ Corrected: that guarantee belongs to the RULE-COMPILE path
/// (`compile-condition`), not to this verb itself — `eval-test` is directly callable, and a
/// caller invoking it on an un-fenced `expr` gets exactly what `eval_inner` does with it, fence
/// or no fence. That is precisely why this verb cannot honestly be `Pure`/`Deterministic`: it
/// evaluates a caller-supplied expression in a fresh child `Environment`, and nothing at this
/// boundary bounds what that expression can do — the same "purity is the form's, like apply"
/// shape `:wat::stream::next` forcing a thunk is effectful for.
///
/// Arc 255 Stone P6-c-W5b — moved verbatim into `#[wat_intrinsic]` with its real (2) arity
/// declared; the hand-rolled `args.len() != 2` guard this wave retired lived right here.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      ControlFlow
/// @arg     expr :wat::WatAST the quoted predicate expression (from `:wat::core::quote`)
/// @arg     bindings :wat::core::PersistentMap the token's bound `?var`s, visible to `expr` as a fresh child `Environment`
/// @ret     :wat::core::bool `expr`'s result; raises if it is not a `:wat::core::bool`
/// @example-norun (:wat::rete::eval-test (:wat::core::quote (:wat::rete::i64::> ?t 20)) (:wat::core::PersistentMap "?t" 25))
#[wat_intrinsic(":wat::rete::eval-test")]
pub(crate) fn eval_test(
    expr: &WatAST,
    bindings: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Arg 0: evaluate → must be Value::wat__WatAST (a quoted expr from :wat::core::quote).
    let expr_val = crate::runtime::eval_inner(expr, env, sym)?.value_owned();
    let expr_ast = match expr_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(expr.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };

    // Arg 1: evaluate → must be Value::wat__core__PersistentMap.
    let bindings_val = crate::runtime::eval_inner(bindings, env, sym)?.value_owned();
    let map = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError::new(bindings.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::PersistentMap (the token's merged bindings)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };

    // Fresh env: where sees only ?vars + sym's user fns (same as `eval_rhs_expr`).
    Ok(Value::bool(eval_test_core(&expr_ast, &map, &Environment::new(), sym)?))
}
