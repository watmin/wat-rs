//! `:wat::kernel::` source intrinsics — arc 255 home #8b
//! (255.1c-split-the-remainder, carved from `kernel_remainder.rs`). Originally four
//! verbs, ONE subject: the program reading a fact about its OWN source —
//! a form's lexical position, the live call stack, the in-flight macro
//! expansion, or a fn value's own reconstructible forms. Excursus 003 F2 adds a
//! fifth, `error-site`, `here`'s DERIVED sibling (see its own section below). All
//! five are `@Category Reflection`.
//!
//! All five delegate to a `pub fn` that already existed before this carve
//! (`crate::kernel::source::eval_kernel_here`, `crate::kernel::source::eval_kernel_call_site`,
//! `crate::kernel::source::eval_kernel_macro_call_site`, `crate::kernel::source::eval_kernel_error_site`,
//! or, for `fn-forms`, `crate::closure_extract::eval_kernel_fn_forms`) — see `kernel/mod.rs` for
//! the tier-wide "bodies do not live here" claim this home is an instance of.
//!
//! ## The five, and why each lands clean
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
//! - **`error-site`** (`src/kernel/source.rs::eval_kernel_error_site`, beside `here`)
//!   answers "the location an error minted HERE should carry": `here`'s own
//!   `list_span` when that span is already user source, otherwise the SAME
//!   derivation `RuntimeError::new` applies to every raised error — the
//!   innermost `CALL_STACK` frame whose file is user source
//!   (`crate::value::signal::derive_primary_location_and_frames`, D4/F2). A
//!   stdlib fault mint site calls this in place of `here` so a RETURNED
//!   `:wat::core::Error` locates at the user's line, never the stdlib's —
//!   never a second copy of D4's rule (GF2b). `@Determinism Nondeterministic`,
//!   same reasoning as `call-site`: the stdlib branch reads ambient
//!   `CALL_STACK` state, so the same stdlib mint site answers differently
//!   depending on which user call reached it this time.
//! - **`fn-forms`** (`src/closure_extract.rs:508`) calls `extract_closure`,
//!   reconstructing a fn value's own source form and walking its body for
//!   transitive deps — the program turning a piece of itself back into
//!   inspectable source. `@Determinism Deterministic`: no I/O, no mutation;
//!   the same fn value + name deterministically reconstructs the same forms.
//!
//! All five are `@Purity Pure` — no I/O, no mutation; each reads
//! Rust-side state the runtime already maintains about the program itself.
//!
//! ## Gate coverage
//!
//! `here`, `call-site`, `macro-call-site`, `error-site` carry registered
//! `TypeScheme`s (`check.rs`, near `18908`/`20652`/`20669`/`18921`) —
//! gate LIVE. `fn-forms` does not; `check.rs`'s `infer_kernel_fn_forms`
//! (`:10406`) is the real authority — gate SKIPS, a bespoke `infer_list` arm
//! carrying a `//` maintainer comment naming it.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::here)` → `:wat::core::Span`. Returns the source
/// coordinate of the `(here)` form itself — `{file, line, col, end}` (`end`
/// is `Some` here: the call form's own span is a real wat-parsed range).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @ret     :wat::core::Span the call form's own source coordinate
/// @example (:wat::i64::> (:wat::core::Span/line (:wat::kernel::here)) 0) #=> true
// Registered `TypeScheme` — `check.rs:16158` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:16256`
// `eval_kernel_here` returns `value_from_span(list_span.clone())` — the
// program reading its OWN source position. Clean fit, no argument needed.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: `list_span`
// is a lexical fact of the AST node, fixed at parse time — no I/O, no
// mutation, and the same call form always yields the same Span.
#[wat_intrinsic(":wat::kernel::here")]
pub(crate) fn eval_kernel_here(
    env: &Environment, // rune:lint(unused-env) — reads only the call form's own span
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::kernel::source::eval_kernel_here(&[], list_span)
}

/// `(:wat::kernel::error-site)` → `:wat::core::Span`. `here`'s DERIVED sibling —
/// excursus 003 F2. Returns the location an error minted HERE should carry: this
/// call form's own source coordinate when it is already user source, otherwise the
/// innermost live wat call-stack frame whose file is user source — the SAME
/// derivation `RuntimeError::new` applies to every raised error, reached through
/// the SAME fn (`crate::value::signal::derive_primary_location_and_frames`), never
/// a second copy of the rule. When no frame is in user source, returns this call
/// form's own coordinate unchanged — exactly what the shared derivation returns in
/// that case.
///
/// A stdlib fault-mint site (`wat/cache.wat`, `wat/sqlite.wat`, `wat/query.wat`,
/// `wat/telemetry/*`, `Fault/of` in `wat/core.wat`) calls this in place of
/// `(:wat::kernel::here)` so a RETURNED `:wat::core::Error`-conforming value
/// locates at the user's own line, never the stdlib's — C-114's defect, this time
/// in a RETURNED value rather than a raised one. Called from user code, it answers
/// exactly as `here` would: user source is already the right answer.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @ret     :wat::core::Span the location an error minted here should carry
/// @example-norun (:wat::kernel::error-site) #=> #wat.core/Span{}
// Registered `TypeScheme` — `check.rs`, beside `:wat::kernel::here`'s — gate LIVE.
//
// Deciding line for `@Category Reflection`: `src/kernel/source.rs::eval_kernel_error_site`
// delegates to `crate::value::signal::derive_primary_location_and_frames` — the program
// reading a fact about its own source (which frame, if any, the loader read under user
// privilege), discarding the frames/elided halves raised errors need and it does not.
//
// Deciding line for `@Purity Pure`: reads only `is_user_source_file` (a thread-local set)
// and, on the stdlib branch, `CALL_STACK` (a Rust-side stack snapshot) — no I/O, no mutation.
//
// Deciding line for `@Determinism Nondeterministic`: mirrors `call-site`'s reasoning — the
// stdlib branch's answer is the live CALL_STACK's innermost user-source frame, so the SAME
// stdlib mint site answers differently depending on which user call reached it this time.
#[wat_intrinsic(":wat::kernel::error-site")]
pub(crate) fn eval_kernel_error_site(
    env: &Environment, // rune:lint(unused-env) — reads only the call form's own span + CALL_STACK
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::kernel::source::eval_kernel_error_site(&[], list_span)
}

/// `(:wat::kernel::call-site)` → `:wat::kernel::Frame`. Returns the caller's
/// `{file, line, symbol}` — the wat equivalent of Ruby's `caller` / Rust's
/// `Location::caller()`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
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
    crate::kernel::source::eval_kernel_call_site(&[], list_span)
}

/// `(:wat::kernel::macro-call-site)` → `:wat::WatAST`. The expand-time twin
/// of `call-site`: valid only inside a macro body; returns the macro
/// invocation's own source span as a SPLICEABLE `Frame'` constructor form.
///
/// **Expand-time ground —** reads the expand-time `MACRO_CALL_SITE` thread-local (the current
/// macro invocation's own source span, pushed by `expand_macro_call`) and returns a spliceable
/// Frame-constructor form. Pure and deterministic per expansion (same invocation → same span,
/// every time it's read during that invocation's expansion), and does no IO — the `log`
/// macro's per-log-line `emitted-from` primitive needs it, so it must be permitted in a macro
/// body. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a;
/// originally arc 278 §4); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
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
    crate::kernel::source::eval_kernel_macro_call_site(&[], list_span)
}

/// `(:wat::kernel::fn-forms f name)` → `(:wat::core::Vector :- [wat::WatAST])`.
/// Reifies a fn value (anonymous or named-by-reference) into a
/// self-contained program fragment that, evaluated in a fresh universe,
/// resolves `name` to a behaviorally-equivalent fn.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     f :wat::core::Fn the fn value to reify (or a keyword naming a registered fn)
/// @arg     name :wat::core::keyword the bind name the reified fn carries when the forms are later evaluated
/// @ret     (:wat::core::Vector :- [:wat::WatAST]) `prologue ++ [(def name entry-form)]`
/// @example (:wat::i64::> (:wat::core::length (:wat::kernel::fn-forms (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) :my-id)) 0) #=> true
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
