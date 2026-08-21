//! `:wat::kernel::` source intrinsics — arc 255 home #8b
//! (255.1c-split-the-remainder, carved from `kernel_remainder.rs`). Four
//! verbs, ONE subject: the program reading a fact about its OWN source —
//! a form's lexical position, the live call stack, the in-flight macro
//! expansion, or a fn value's own reconstructible forms. All four are
//! `@Category Reflection`.
//!
//! All four delegate to a `pub fn` that already existed before this carve
//! (`crate::runtime::eval_kernel_here`, `crate::runtime::eval_kernel_call_site`,
//! `crate::runtime::eval_kernel_macro_call_site`, or, for `fn-forms`,
//! `crate::closure_extract::eval_kernel_fn_forms`) — see `kernel/mod.rs` for
//! the tier-wide "bodies do not live here" claim this home is an instance of.
//!
//! ## The four, and why each lands clean
//!
//! - **`here`** (`runtime.rs:16256`) returns `value_from_span(list_span.clone())`
//!   — the `(here)` FORM'S OWN source position, a lexical fact fixed at
//!   parse time, no runtime dependency. `@Determinism Deterministic`: the
//!   same call form always yields the same `Location`.
//! - **`call-site`** (`runtime.rs:25585`) reads `snapshot_call_stack().first()`
//!   — the wat call stack, a structure the program's own fn-calls maintain
//!   about themselves. `@Determinism Nondeterministic`, unlike `here`: the
//!   answer is the CALLING function's live invocation frame, so the same
//!   enclosing fn called from two different call sites answers differently
//!   depending on which call reached it this time — not fixed by this
//!   call's own (zero) arguments.
//! - **`macro-call-site`** (`runtime.rs:25648`) reads the `MACRO_CALL_SITE`
//!   thread-local top — the program interrogating its own in-flight macro
//!   expansion. Same `@Determinism Nondeterministic` reasoning as
//!   `call-site`: ambient expansion-stack state, no I/O, no mutation, but
//!   the answer depends on which macro invocation is currently expanding.
//! - **`fn-forms`** (`src/closure_extract.rs:508`) calls `extract_closure`,
//!   reconstructing a fn value's own source form and walking its body for
//!   transitive deps — the program turning a piece of itself back into
//!   inspectable source. `@Determinism Deterministic`: no I/O, no mutation;
//!   the same fn value + name deterministically reconstructs the same forms.
//!
//! All four are `@Purity Pure` — no I/O, no mutation; each reads
//! Rust-side state the runtime already maintains about the program itself.
//!
//! ## Gate coverage
//!
//! `here`, `call-site`, `macro-call-site` carry registered `TypeScheme`s
//! (`check.rs`, near `16158`/`17891`/`17908`) — gate LIVE. `fn-forms` does
//! not; `check.rs`'s `infer_kernel_fn_forms` (`:10406`) is the real
//! authority — gate SKIPS, a bespoke `infer_list` arm carrying a `//`
//! maintainer comment naming it.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::here)` → `:wat::kernel::Location`. Returns the source
/// coordinate of the `(here)` form itself — `{file, line, col}`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Reflection
/// @ret     :wat::kernel::Location the call form's own source coordinate
/// @example (:wat::core::i64::> (:wat::kernel::Location/line (:wat::kernel::here)) 0) #=> true
// Registered `TypeScheme` — `check.rs:16158` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:16256`
// `eval_kernel_here` returns `value_from_span(list_span.clone())` — the
// program reading its OWN source position. Clean fit, no argument needed.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: `list_span`
// is a lexical fact of the AST node, fixed at parse time — no I/O, no
// mutation, and the same call form always yields the same Location.
#[wat_intrinsic(":wat::kernel::here")]
pub(crate) fn eval_kernel_here(
    env: &Environment, // rune:lint(unused-env) — reads only the call form's own span
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_here(&[], list_span)
}

/// `(:wat::kernel::call-site)` → `:wat::kernel::Frame`. Returns the caller's
/// `{file, line, symbol}` — the wat equivalent of Ruby's `caller` / Rust's
/// `Location::caller()`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @ret     :wat::kernel::Frame the innermost enclosing wat fn-call's frame
/// @example-norun (:wat::kernel::call-site) #=> #wat.kernel/Frame{}
// Registered `TypeScheme` — `check.rs:17891` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:25585`
// `eval_kernel_call_site` reads `snapshot_call_stack().first()` — the wat
// call stack, a structure the program's own fn-calls maintain about
// themselves. The program interrogating itself. Clean fit.
//
// Deciding line for `@Purity Pure`: reads a Rust-side stack snapshot; no
// I/O, no mutation.
//
// Deciding line for `@Determinism Nondeterministic`: unlike `here` (whose
// answer is fixed by the call FORM's own lexical position), `call-site`'s
// answer is the CALLING function's live invocation frame — the same
// enclosing fn, called from two different call sites, answers differently
// depending on which call reached it THIS time. Depends on the runtime call
// path, not fixed by this call's own zero arguments.
#[wat_intrinsic(":wat::kernel::call-site")]
pub(crate) fn eval_kernel_call_site(
    env: &Environment, // rune:lint(unused-env) — reads only the wat call stack
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_call_site(&[], list_span)
}

/// `(:wat::kernel::macro-call-site)` → `:wat::WatAST`. The expand-time twin
/// of `call-site`: valid only inside a macro body; returns the macro
/// invocation's own source span as a SPLICEABLE `Frame'` constructor form.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @ret     :wat::WatAST a spliceable `(:wat::kernel::Frame' file line symbol)` form
/// @example-norun (:wat::kernel::macro-call-site) #=> #wat/WatAST{}
// Registered `TypeScheme` — `check.rs:17908` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:25648`
// `eval_kernel_macro_call_site` reads the `MACRO_CALL_SITE` thread-local top
// — the program interrogating its own in-flight macro expansion. Clean fit.
//
// Deciding line for `@Purity Pure` / `@Determinism Nondeterministic`: same
// reasoning as `call-site` — reads ambient expansion-stack state (no I/O, no
// mutation) whose answer depends on which macro invocation is currently
// expanding, not on this call's own (zero) arguments.
#[wat_intrinsic(":wat::kernel::macro-call-site")]
pub(crate) fn eval_kernel_macro_call_site(
    env: &Environment, // rune:lint(unused-env) — reads only the macro-expansion stack
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_macro_call_site(&[], list_span)
}

/// `(:wat::kernel::fn-forms f name)` → `:wat::core::Vector<wat::WatAST>`.
/// Reifies a fn value (anonymous or named-by-reference) into a
/// self-contained program fragment that, evaluated in a fresh universe,
/// resolves `name` to a behaviorally-equivalent fn.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Reflection
/// @arg     f :wat::core::Fn the fn value to reify (or a keyword naming a registered fn)
/// @arg     name :wat::core::keyword the bind name the reified fn carries when the forms are later evaluated
/// @ret     :wat::core::Vector<wat::WatAST> `prologue ++ [(def name entry-form)]`
/// @example (:wat::core::i64::> (:wat::core::length (:wat::kernel::fn-forms (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) :my-id)) 0) #=> true
// No registered `TypeScheme` — `check.rs`'s `infer_kernel_fn_forms`
// (`:10406`) is the real authority.
//
// Deciding line for `@Category Reflection`: `src/closure_extract.rs:508`
// `eval_kernel_fn_forms` calls `extract_closure`, reconstructing the fn's
// own source form and walking its body for transitive deps — the program
// turning a piece of itself back into inspectable source. Clean fit.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: no I/O,
// no mutation; `extract_closure` deterministically reconstructs the same
// forms from the same fn value + name every time.
#[wat_intrinsic(":wat::kernel::fn-forms")]
pub(crate) fn eval_kernel_fn_forms(
    f: &WatAST,
    name: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::closure_extract::eval_kernel_fn_forms(&[f.clone(), name.clone()], list_span, env, sym)
        .map_err(Into::into)
}
