//! Arc 109 Stone — the reflect home's EXPAND role: macroexpand.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`). `eval_macroexpand_1`
//! (one expansion step) and `eval_macroexpand` (fixpoint expansion, built on the former)
//! are the `:wat::core::macroexpand-1` / `:wat::core::macroexpand` special forms. Moved
//! verbatim out of `src/runtime.rs` (arc 109 reflect stone). Behaviour is unchanged; only
//! the location moved.
//!
//! Both items are bumped from private to `pub(crate)`: neither carries
//! `#[wat_intrinsic]` (they are special forms with no context-tail context, dispatched by
//! an explicit arm in `runtime.rs`'s `dispatch_keyword_head_value`), so the visibility
//! bump is required for that cross-module dispatch call, not a signature change.
//!
//! Siblings: `render.rs` (internal state → AST), `lookup.rs` (find a binding), `verbs.rs`
//! (the `*-of` API surface), `match.rs` (form matching).

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use std::sync::Arc;
use wat_macros::wat_special_form_impl;

// `eval_inner` is genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-2); it is the evaluator's own entry point.
use crate::runtime::eval_inner;

/// `(:wat::core::macroexpand-1 <wat::WatAST>) -> :wat::WatAST`. Arc 030.
/// One expansion step. If the input AST is a macro call (list with a
/// registered-macro keyword head), apply the macro's template and
/// return the result. Otherwise return the input unchanged.
///
/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::macroexpand-1`.
/// Annotated IN PLACE (signature already fits the canonical `NativeHandler` shape) — a
/// SEPARATE fn from `eval_macroexpand`'s own pointer (below), never stacked on it: `role = eval`
/// codegens a dispatch shim named from the fn identifier alone, so two FQDNs on one eval fn
/// would mint a duplicate symbol (`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-
/// so]]`) — moot here regardless, since one-step vs. fixpoint are genuinely different bodies.
/// See `intrinsic/special/macroexpand_1.rs` for the doc-only struct.
#[wat_special_form_impl(":wat::core::macroexpand-1", role = eval)]
pub(crate) fn eval_macroexpand_1(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::macroexpand-1";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let ast = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__WatAST(a) => (*a).clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::WatAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let registry = sym.macro_registry().ok_or(RuntimeError::new(
        args[0].span().clone(),
        RuntimeErrorKind::NoMacroRegistry { op: OP.into() },
    ))?;
    let expanded = crate::macros::expand_once(ast, registry, env, sym).map_err(|e| {
        RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::MacroExpansionFailed {
                op: OP.into(),
                cause: Box::new(e),
            },
        )
    })?;
    Ok(Value::wat__WatAST(Arc::new(expanded)))
}

/// `(:wat::core::macroexpand <wat::WatAST>) -> :wat::WatAST`. Arc 030.
/// Fixpoint expansion. Applies macroexpand-1 repeatedly until the AST
/// stops changing (bounded by EXPANSION_DEPTH_LIMIT to catch cycles).
///
/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::macroexpand`.
/// Annotated IN PLACE (signature already fits the canonical `NativeHandler` shape) — a
/// SEPARATE fn from `eval_macroexpand_1`'s own pointer, above (see its doc for why stacking is
/// moot here). See `intrinsic/special/macroexpand.rs` for the doc-only struct and the shared
/// `role = check` pointer.
#[wat_special_form_impl(":wat::core::macroexpand", role = eval)]
pub(crate) fn eval_macroexpand(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::macroexpand";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let mut ast = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__WatAST(a) => (*a).clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::WatAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let registry = sym.macro_registry().ok_or(RuntimeError::new(
        args[0].span().clone(),
        RuntimeErrorKind::NoMacroRegistry { op: OP.into() },
    ))?;
    for _ in 0..crate::macros::EXPANSION_DEPTH_LIMIT {
        let next = crate::macros::expand_once(ast.clone(), registry, env, sym).map_err(|e| {
            RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::MacroExpansionFailed {
                    op: OP.into(),
                    cause: Box::new(e),
                },
            )
        })?;
        if next == ast {
            return Ok(Value::wat__WatAST(Arc::new(next)));
        }
        ast = next;
    }
    // Fixpoint not reached — synthesise a typed ExpansionDepthExceeded cause so
    // the MacroExpansionFailed envelope always carries a Box<MacroError>.
    Err(RuntimeError::new(
        args[0].span().clone(),
        RuntimeErrorKind::MacroExpansionFailed {
            op: OP.into(),
            cause: Box::new(crate::macros::MacroError {
                span: args[0].span().clone(),
                kind: crate::macros::MacroErrorKind::ExpansionDepthExceeded {
                    limit: crate::macros::EXPANSION_DEPTH_LIMIT,
                },
            }),
        },
    )
    .into())
}
