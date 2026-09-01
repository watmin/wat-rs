//! Kernel sub-module mirroring `src/intrinsic/kernel/source.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Four items backing the edge
//! file's four `@Category Reflection` verbs — `eval_kernel_here` (`here`),
//! `eval_kernel_call_site` (`call-site`), `eval_kernel_macro_call_site`
//! (`macro-call-site`) — plus `bound_names`, a private helper serving the
//! source-position family: it supplies the field names for the
//! `:wat::spawn::Bound` record `eval_listener_prime`'s thread-tier arm
//! constructs (now in `src/kernel/resource.rs`), read from the
//! macro-generated `BOUND_FIELDS` const that stays in `runtime.rs` (its
//! `wat_field_names_from!` invocation is not itself one of this stone's 34
//! named items).
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::{value_from_frame_info, value_from_span, BOUND_FIELDS};
use crate::span::Span;
use crate::value::{snapshot_call_stack, EvalBreak, RuntimeError, RuntimeErrorKind, Value};
use std::sync::Arc;

/// `(:wat::kernel::here) -> :wat::kernel::Location` — arc 296.
///
/// Returns the source coordinate of the `(here)` form itself —
/// the `list_span` of the call site — as a `:wat::kernel::Location`
/// record `{file, line, col}`. Arity 0; any arguments are an error.
pub(crate) fn eval_kernel_here(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::here";
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(value_from_span(list_span.clone()))
}

/// `(:wat::kernel::call-site)` — nullary; returns the caller's
/// `:wat::kernel::Frame {file, line, symbol}` — the wat equivalent of
/// Ruby's `caller` / Rust's `Location::caller()`.
///
/// A native verb pushes no wat frame of its own (only wat fn-calls push,
/// via `FrameGuard`), so from inside this native verb
/// `snapshot_call_stack().first()` IS the caller's frame — the innermost
/// user call that invoked `(:wat::kernel::call-site)`. Mirrors the
/// mechanism `:wat::kernel::assertion-failed!` uses to find "where the
/// author wrote the assert" (src/assertion.rs).
pub(crate) fn eval_kernel_call_site(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::call-site";
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    match snapshot_call_stack().first().cloned() {
        Some(frame) => Ok(value_from_frame_info(frame)),
        // Arc 109 — an empty call stack cannot happen from wat: every wat
        // fn-call pushes a FrameGuard before evaluating its body, and all
        // wat runs inside a fn (there is no top-level `call-site` use in the
        // corpus). The old all-`None` Frame here MASKED that invariant with a
        // fabricated value; now Frame's fields are non-`Option` and a value is
        // always known, so refuse honestly instead — mirroring the sibling
        // `macro-call-site`, which likewise refuses on an empty stack rather
        // than fabricating a frame.
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "`:wat::kernel::call-site` was reached with an empty wat call \
                     stack (no enclosing fn frame). Every wat fn-call pushes a frame \
                     before evaluating its body; `call-site` is only meaningful inside \
                     a fn body, never at the top level."
                    .into(),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::macro-call-site)` — nullary; the EXPAND-TIME twin of
/// `:wat::kernel::call-site`. Meaningful only inside a macro body: returns
/// the macro INVOCATION's own source span — not the runtime call stack
/// (macro expansion runs before any wat fn-call happens, so `CALL_STACK` is
/// irrelevant here) — as a SPLICEABLE `:wat::kernel::Frame` constructor
/// FORM, `Value::wat__WatAST`, not a `Frame` value. A `Frame` VALUE cannot
/// cross `value_to_watast` (it errors on aggregates), so returning one would
/// make `~(:wat::kernel::macro-call-site)` fail to splice; a FORM round-trips
/// through `value_to_watast`'s `wat__WatAST` arm untouched.
///
/// The span comes from the `MACRO_CALL_SITE` thread-local stack
/// (`src/value/frame.rs`), pushed by `expand_macro_call`
/// (src/macros/expand.rs) for the duration of expanding each macro
/// invocation — read the TOP (innermost currently-expanding invocation;
/// nested macro expansion pushes/pops correctly).
///
/// Arc 278 §4 — DESIGN-telemetry-caller-and-capacity.md §4. Uses the
/// generated POSITIONAL PRIME constructor `:wat::kernel::Frame'` (arc 294
/// item 9a: machinery/generated code uses the prime ctor, never hand-rolled
/// kwargs) — mirrors `eval_struct_to_form`'s ctor-form-building convention.
/// Arc 109 — Frame's fields are non-`Option`, so the ctor form supplies bare
/// `file`/`line`/`symbol` values. `symbol` is the NAME of the macro being
/// expanded (threaded through `MacroCallSiteGuard`): at expand time there is no
/// enclosing runtime fn, but the macro itself is known, so its name is the
/// honest symbol — never absent.
pub(crate) fn eval_kernel_macro_call_site(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::macro-call-site";
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    let (call_site, macro_name) = match crate::value::current_macro_call_site() {
        Some(pair) => pair,
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "`:wat::kernel::macro-call-site` is only valid inside a macro body / \
                         at expand time (no macro invocation is currently being expanded on \
                         this thread)"
                        .into(),
                },
            )
            .into());
        }
    };
    let span = list_span.clone();
    let form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::Frame'".into(), span.clone()), // rune:lint(retired-name) — positional constructor idiom: Frame is the record, Frame' builds one
            // Arc 109 — Frame's fields are concrete (non-`Option`); the ctor
            // form supplies bare values, never `(Some …)`/`None` wrappers.
            // file: "<file>"
            WatAST::StringLit((*call_site.file).clone(), span.clone()),
            // line: <line>
            WatAST::IntLit(call_site.line, span.clone()),
            // symbol: "<macro name>" — at expand time there is no enclosing
            // runtime fn, but the macro BEING expanded IS known; its name is
            // the honest symbol (never absent).
            WatAST::StringLit(macro_name, span.clone()),
        ],
        span,
    );
    Ok(Value::wat__WatAST(Arc::new(form)))
}

pub(crate) fn bound_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(BOUND_FIELDS))
        .clone()
}
