//! Fenced expression eval under token bindings — `eval-test` / `eval_rhs_expr`.
//!
//! `where` / `:test` raise (not Clara). Shared by interpreted RHS operands.

use crate::ast::WatAST;
use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value, ValueSnapshot};
use crate::span::Span;

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
/// COUNTED, because this is the hot path: it runs for EVERY token × EVERY TestNode — 10,000 times
/// on node-share `[50 200]`, of which 98% are about to FAIL — and each pass allocates a child
/// `Environment` (`Arc<EnvCell>` + a `HashMap`) plus, per binding, a fresh `String` (`.to_string()`
/// on a key FIXED at rule-compile time), a `Span`, and a `Value` clone. Exactly the waste
/// `compiled_cond` was built to remove from the alpha path: *"two heap allocations rebuilding the
/// constant binding key on every call, including every call that is about to fail."*
///
/// Measured (Step 0, 2026-08-01): **122.5 ns/eval — 22.7% of a `where` evaluation.** The other
/// 77.3% is the `eval_inner` walk, which is why the stone is a full expression IR and not just
/// this block.
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

/// `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::core::bool`
///
/// Dispatch wrapper: evaluates the two args, extracts the `WatAST` and `PersistentMap`,
/// then delegates to `eval_test_core`. No behavior change from the previous monolithic
/// implementation — the core extraction is a refactor only.
///
/// Because the four-axis compile-condition fence (pure ∧ det ∧ total ∧ rete)
/// proves safety at compile time, eval-test does not re-run it.
pub(crate) fn eval_test(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Arity: exactly 2 args.
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            })
        .into());
    }

    // Arg 0: evaluate → must be Value::wat__WatAST (a quoted expr from :wat::core::quote).
    let expr_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let expr_ast = match expr_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };

    // Arg 1: evaluate → must be Value::wat__core__PersistentMap.
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let map = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
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
