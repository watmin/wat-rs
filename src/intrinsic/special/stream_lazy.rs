//! Special-form doc entry for `:wat::stream::lazy` — arc 255 Stone 1a-zeta, the last three of
//! the special-form table. `eval_lazy_seq` (`src/runtime.rs`) does not fit the canonical
//! `NativeHandler` shape (three params, no `sym` — the same asymmetry `eval_quote`/`eval_fn`
//! hit), so a thin delegate lives here (mirrors `quote.rs`'s `eval_quote_form`). There is NO
//! tail arm for this row (`special_forms.rs`'s own table names none), so no `role = tail`.
//! The check arm did not exist as a named fn (inline in `check.rs`'s big `match k.as_str()`);
//! `infer_stream_lazy` below is that arm's body, moved verbatim (STOP-3).

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckError, CheckErrorKind, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};
use std::collections::HashMap;

/// `(:wat::stream::lazy <body>) -> (Stream :- [T])`. Capture-don't-eval: `<body>` is wrapped,
/// UNEVALUATED, in a 0-arg closure over the current environment and returned as a
/// `Stream::Thunk`; `<body>` runs only when the resulting stream is forced (`:wat::stream::
/// next` et al.), and — unlike the memoized form Stone 118.B3 retired — it is NOT memoized:
/// forcing the same cell twice runs `<body>` twice.
///
/// **Category ground —** `eval_lazy_seq` (`src/runtime.rs:6129`) constructs a NEW
/// `:wat::stream::Stream` value from what it is handed — the same "constructs a new Stream
/// value from what it is given" shape `eval_stream_empty_intrinsic`/`eval_stream_cons_intrinsic`
/// (`src/intrinsic/stream.rs`) already argue `Transform` for within this SAME `:wat::stream::*`
/// family ("cons ... A pure reshape: it stores exactly what it is handed"). `stream::lazy`
/// reshapes an unevaluated form into a lazily-forceable Stream value — the output is a form of
/// the input, deferred rather than immediately restructured, the same family both siblings
/// already occupy. `Transform`.
///
/// **Purity ground —** measured directly: `eval_lazy_seq` never calls `eval`/`eval_inner`/
/// `step_list` on `args[0]` — it wraps the raw `WatAST` node as a closure's
/// `FunctionBody::Wat`, the identical "wraps the raw node and returns" shape `quote.rs`'s own
/// `Pure` ground uses, and the identical "evaluates NONE of its sub-forms when it itself runs"
/// shape `fn_form.rs`'s own `Pure` ground uses for `fn`. Building the thunk is unconditionally
/// free of effect regardless of what `<body>` will later do WHEN FORCED — a different
/// evaluation event, at a different call site (`:wat::stream::next`), the identical sentence
/// `fn_form.rs`'s own `Purity` ground closes with. `Pure`.
///
/// **Determinism ground —** the same `<body>` + the same environment always produce a
/// structurally identical `Stream::Thunk`; `eval_lazy_seq` consults no clock, entropy, or
/// hygiene-tagging counter — only `env.clone()`, a cheap `Arc`/handle copy, the same
/// no-independent-variation shape `quasiquote.rs`'s own ground uses. (The LACK of memoization —
/// forcing the same cell twice runs `<body>` twice — is a property of that LATER forcing event,
/// not of this construction event; the same construction-vs-later-call boundary `fn_form.rs`'s
/// own `Purity` ground draws.) `Deterministic`.
///
/// **Totality ground —** `eval_lazy_seq`'s only fallible path is `args.len() != 1` →
/// `ArityMismatch`, the same fixed-arity carve-out `if`'s/`quote`'s own `Total`/`Preserving`
/// grounds use. Past that guard, construction (`Function { .. }`, then
/// `Stream::Thunk(LazyCell::new(thunk))`) is unconditional — nothing past the guard can fail,
/// the identical "past this gate, construction cannot fail" shape `quote.rs`'s own `Total`
/// ground uses. `Total`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` residue hand-list
/// names `":wat::stream::lazy"` literally (its "collection / sequence ops still on the
/// pre-registry dispatch path" group) — legal, pre-registration. `eval_lazy_seq` evaluates
/// NONE of its sub-forms at its own call site — it captures `<body>`, never runs it — the
/// identical shape `fn_form.rs`'s own `Legal` ground argues for `fn` (contrast `do`/`ann-form`/
/// `and`, which DO run real sub-forms at their own call site and are `Preserving`): `lazy`'s own
/// admission inside a macro body does not depend on `<body>`'s. `validate_pure_total`
/// (`macros/eval.rs`) still recurses into `<body>` afterward — the same conservative walk it
/// performs on a `fn`'s body (per its own `BEWARE` comment: a closure "can be INVOKED at expand
/// time" by a blessed HOF, and a lazy thunk built inside a macro body could equally be forced by
/// one during the same expansion) — that recursion is a property of the WALKER's defense in
/// depth, not evidence that `lazy`'s own admission depends on `<body>`'s, the identical
/// distinction `fn_form.rs` draws for `fn`. `Legal`.
///
/// @added 1.0.0
/// @Category Transform
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::stream::lazy <body>)
/// @ret (:wat::stream::Stream :- [T]) a Thunk cell that evaluates `<body>` when forced
/// @example (:wat::core::stream->vec [] (:wat::stream::lazy (:wat::stream::empty))) #=> []
#[wat_special_form(":wat::stream::lazy")]
pub(crate) struct StreamLazy;

/// Arc 255 Stone 1a-zeta — the `role = eval` pointer for `:wat::stream::lazy`. `eval_lazy_seq`
/// (`src/runtime.rs`) does not fit the canonical `NativeHandler` shape: three params (`args`,
/// `list_span`, `env`; no `sym`) — the same asymmetry `quote.rs`'s `eval_quote_form` and
/// `fn_form.rs`'s `eval_fn_form` hit for their own inner fns. STOP-3 forbids reshaping it (no
/// other caller to touch here, but the signature is the established idiom for this shape
/// regardless). This thin delegate takes the full four `NativeHandler` params, ignores `sym`
/// (`stream::lazy` never consults the symbol table — it captures the enclosing `Environment`),
/// and forwards to the untouched `eval_lazy_seq` (widened `fn` -> `pub(crate) fn` this stone so
/// this file can call it).
#[wat_special_form_impl(":wat::stream::lazy", role = eval)]
fn eval_lazy_seq_form(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_lazy_seq(args, list_span, env)
}

/// Arc 255 Stone 1a-zeta — the `role = check` pointer for `:wat::stream::lazy`. The inline arm
/// at `check.rs`'s `":wat::stream::lazy"` match key stays untouched in its OWN LOGIC (STOP-3:
/// extracted verbatim), moved wholesale to this named fn so the registry's `role = check`
/// annotation names real, reachable code rather than a fn nothing calls. `infer`/`unify`/
/// `apply_subst`/`format_type` are `check.rs` helpers reused verbatim (all already
/// `pub(crate)`/`pub`).
#[wat_special_form_impl(":wat::stream::lazy", role = check)]
pub(crate) fn infer_stream_lazy(
    _head: &str,
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 1 {
        local_errors.push(CheckError {
            span: head_span.clone(),
            kind: CheckErrorKind::ArityMismatch {
                callee: ":wat::stream::lazy".into(),
                expected: 1,
                got: args.len(),
            },
        });
        let t = fresh.fresh();
        let seq_ty = TypeExpr::Parametric {
            head: "wat::stream::Stream".into(),
            args: vec![t],
        };
        return if local_errors.is_empty() {
            CheckResult::ok(seq_ty)
        } else {
            CheckResult::partial_with(seq_ty, local_errors)
        };
    }
    // Type-check the body and unify it with (Stream :- [fresh_T]).
    let body_ty = crate::check::infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let elem = fresh.fresh();
    let seq_ty = TypeExpr::Parametric {
        head: "wat::stream::Stream".into(),
        args: vec![elem],
    };
    if let Some(bt) = body_ty {
        if crate::check::unify(&bt, &seq_ty, subst, env.types()).is_err() {
            local_errors.push(CheckError {
                span: args[0].span().clone(),
                kind: CheckErrorKind::TypeMismatch {
                    callee: ":wat::stream::lazy".into(),
                    param: "<body>".into(),
                    expected: "(wat::stream::Stream :- [T])".into(),
                    got: crate::check::format_type(&crate::check::apply_subst(&bt, subst)),
                },
            });
        }
    }
    let result = crate::check::apply_subst(&seq_ty, subst);
    if local_errors.is_empty() {
        CheckResult::ok(result)
    } else {
        CheckResult::partial_with(result, local_errors)
    }
}
