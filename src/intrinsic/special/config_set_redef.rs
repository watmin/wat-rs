//! Special-form doc entry for `:wat::config::set-redef!` — arc 255 Stone 1a-ε, shape ③ of
//! `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`'s three: a form
//! that DOES reach the evaluator and returns `Unit`, because its real work (updating the
//! runtime's redef-opt-in flag) already happened at freeze time.
//!
//! ⚠ STOP-3 measured TWO real freeze-time processors, each covering a DIFFERENT structural
//! position of this same form, not one honest primary:
//!
//! - **Leading position** (the ordinary, documented usage — a setter at the top of the entry
//!   file, before any non-setter form): `collect_entry_file_inner` (`src/config.rs:319`) scans
//!   the entry file's leading forms, parses this setter's bool argument, and folds it into the
//!   `Config` it returns; `freeze.rs:1280` then copies `config.redef_allowed` onto
//!   `bundle.symbols.redef_allowed` BEFORE `check_program` runs (the comment there: "so that
//!   `CheckEnv::from_symbols` sees the correct redef_allowed flag").
//! - **Any later position** (legal but unusual — this form is one of the eight
//!   `RUNTIME_DECLARATION_HEADS`, `src/declare/parse.rs:136`, so it is NOT restricted to the
//!   leading position the way `collect_entry_file_inner`'s own scan is):
//!   `register_runtime_defs_form` (`src/declare/register.rs:1770`) mutates
//!   `sym.redef_allowed` directly whenever this form reaches it — measured with a probe
//!   (`wat-scripts/scratch-pad/1a-epsilon-probe/probe-nonleading-setredef.wat`): a
//!   `set-redef!` placed AFTER an earlier top-level `defn` is accepted (does not raise
//!   `SetterAfterNonSetter` — that check is only reachable from within
//!   `collect_entry_file_inner`'s own leading-setter loop, which has already `break`d out by
//!   then) and the program runs to completion, meaning the form fell through
//!   `collect_entry_file_inner` entirely and was processed by `register_runtime_defs_form`
//!   instead.
//!
//! Both are real; neither alone is honest. `role = declare` is annotated on both (STOP-3's
//! "annotate both and say so" branch), on THEIR OWN functions, in THEIR OWN files
//! (`src/config.rs`, `src/declare/register.rs`) — not duplicated here.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// Opt in (or out of) permitting `def` to redefine an already-bound top-level name for the rest
/// of this program: `(:wat::config::set-redef! true)` / `(:wat::config::set-redef! false)`.
/// Ordinarily written as a leading entry-file setter (see the module doc's STOP-3 finding for
/// where it is actually processed); by the time evaluation reaches this form — whichever
/// position it appeared at — the flag has already been committed, and the eval arm
/// (`runtime.rs:2120`) returns `Ok(Value::Unit)` without inspecting its argument at all
/// (measured with a probe that puts a `println` inside the argument expression —
/// `wat-scripts/scratch-pad/1a-epsilon-probe/probe-noeval-args.wat` — and the println never
/// fires: the eval arm does not even evaluate its own sub-form).
///
/// **Category ground —** contrast `:wat::core::use!`'s row (`use_form.rs`), which DOES fit
/// `:Declaration` (it registers a name, `:rust::...::path`, that other call sites look up).
/// This form registers no name at all: it flips a single process-global boolean on the
/// `SymbolTable` carrier (`sym.redef_allowed`) that gates a DIFFERENT, unrelated mechanism's
/// behavior (whether a later `def` may rebind an existing name) — no value the caller holds
/// addresses that flag, and nothing is looked up BY this form's own identity the way a `def`
/// binding or a `use!` dependency is. That is exactly `:Ambient`'s own prose: "Reads or writes
/// process-global state that no value the caller holds addresses" — the same shape its own
/// named examples (`sigusr1?`, `reset-sigusr1!`, …) have: a global mode toggle, not a named
/// registration. `Ambient`. They — this row and `use!`'s — do NOT share a category.
///
/// **Purity ground —** measured directly: `:wat::config::set-redef!` appears in
/// `src/runtime.rs` exactly once as a real eval arm (`dispatch_keyword_head_value`, `:2120`,
/// `=> Ok(Value::Unit)`), reached (not refused) at expression position — `Unevaluated` would be
/// a lie about a form that measurably evaluates. The arm ignores `args` completely (proven
/// above by the non-firing `println` probe), so no sub-form's effect can leak through either.
/// Unconditionally free of effect at the moment it runs. `Pure`.
///
/// **Determinism ground —** the eval arm consults nothing (no clock, no entropy, no read of
/// `env`/`sym`) and returns the same `Value::Unit` unconditionally for any argument that
/// reached it. `Deterministic`.
///
/// **Totality ground —** read directly: the eval arm is `Ok(Value::Unit)` with no match, no
/// unwrap, no fallible sub-call — defined unconditionally for every input that reaches it, the
/// strongest form of `Total` (it does not even inspect its argument). `Total`.
///
/// **Expand-time ground —** the eval arm evaluates none of its own sub-forms and consults no
/// runtime-only state to produce its unconditional `Ok(Value::Unit)` — legal unconditionally,
/// the same shape `fn`'s own `Legal` ruling argues. `Legal`.
///
/// @added 1.0.0
/// @Category Ambient
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::config::set-redef! <bool>)
/// @ret :wat::core::nil always `Unit` — the flag update already happened at freeze time; the eval arm ignores its argument
/// @example (:wat::config::set-redef! true) #=> nil
#[wat_special_form(":wat::config::set-redef!")]
pub(crate) struct ConfigSetRedef;

/// Arc 255 Stone 1a-ε — the `role = eval` pointer for `:wat::config::set-redef!`. NOT stacked
/// with `:wat::config::set-eval-redef!`'s own pointer (`eval_config_set_eval_redef`, below,
/// same file) on one shared fn: unlike `role = check`/`role = declare` (whose stacking
/// precedent — `infer_boolean_shortcircuit`, `check.rs:15553` — genuinely is one fn, two
/// FQDNs), `role = eval` codegens a NAMED dispatch shim
/// (`wat_special_form_impl.rs`'s `emit`, `format_ident!("__wat_special_form_eval_{}", fn_ident)`)
/// derived from the fn's own name alone, not the FQDN — stacking two `role = eval` attributes on
/// ONE fn would mint the identical shim name twice, a duplicate-definition compile error.
/// Measured by reading `emit`, not assumed: `check`/`declare` emit no shim at all (their `else`
/// branch is literally `(TokenStream2::new(), quote!{None})`), which is exactly why stacking
/// THOSE is safe and stacking `eval`/`tail` is not. Two separate, identically-bodied fns instead
/// — this is a new, standalone delegate carrying the canonical `NativeHandler` signature purely
/// to host the registration, the same move `fn_form.rs`'s `eval_fn_form` makes for
/// `:wat::core::fn`. Its body is the identical no-op the existing shared arm
/// (`runtime.rs:2120`, untouched, STOP-5) already performs.
#[wat_special_form_impl(":wat::config::set-redef!", role = eval)]
fn eval_config_set_redef(
    _args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    Ok(Value::Unit)
}

/// Arc 255 Stone 1a-ε — the `role = eval` pointer for `:wat::config::set-eval-redef!`.
/// `eval_config_set_redef`'s sibling, above, one function up — see its doc for why this is a
/// SEPARATE fn rather than a second `#[wat_special_form_impl]` stacked on the same one (a
/// shim-name collision, not a style preference). Identical body: both FQDNs share the exact
/// same eval arm (`runtime.rs:2120`).
#[wat_special_form_impl(":wat::config::set-eval-redef!", role = eval)]
fn eval_config_set_eval_redef(
    _args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    Ok(Value::Unit)
}
