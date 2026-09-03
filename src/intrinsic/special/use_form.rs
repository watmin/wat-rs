//! Special-form doc entry for `:wat::core::use!` — arc 255 Stone 1a-ε, shape ③ of
//! `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`'s three: a
//! form that DOES reach the evaluator and returns `Unit`, because its real work (validating
//! and recording the dependency) already happened at the resolve pass, before evaluation ever
//! starts.
//!
//! ⚠ STOP-4 measured FALSE, not confirmed true — see `@Category`'s ground below and the
//! stone's report: `use!` is NOT freeze-processor-less. `collect_use_declarations`
//! (`src/resolve/rust_use.rs:13`), Pass 1 of `resolve_references` (`src/resolve/walk.rs:29`,
//! called from `freeze.rs` step 7), scans every top-level form for `(:wat::core::use!
//! :rust::...)`, validates the keyword against the build-time rust-deps registry, and records
//! it into a program-global `UseDeclarations` set that Pass 2 consults for every `:rust::*`
//! call head in the program. `special_forms.rs`'s own arity table names this same fact in its
//! section header: "─── Resolve-pass declaration ───". `role = declare` is annotated there,
//! not invented here.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};
use std::collections::HashMap;

/// Declare that the program depends on the Rust-shimmed symbol `<path>` (a `:rust::...`
/// keyword) — e.g. `(:wat::core::use! :rust::cache::Lru)`. Resolved entirely at the resolve
/// pass (`collect_use_declarations`, `src/resolve/rust_use.rs`): the keyword is checked against
/// the build-time `RustDepsRegistry` and, if present, recorded into a program-global
/// `UseDeclarations` set — "one `use!` anywhere enables the symbol everywhere" (the pass's own
/// comment, `resolve/walk.rs:39`). By the time evaluation reaches this form the declaration has
/// already done its job; the eval arm (`runtime.rs:2947`) returns `Ok(Value::Unit)` without
/// inspecting its argument at all.
///
/// **Category ground —** `collect_use_declarations` REGISTERS a program-level entity — the
/// dependency on `:rust::...::path` — exactly `:Declaration`'s own prose ("registers a
/// program-level entity ... visible to everything after it"): pass 2's call-head resolution
/// (`check_form`, same file) treats every `:rust::*` call head as legal only if some `use!`
/// somewhere recorded it, the same relationship `def` has to the names it binds. The one
/// wrinkle against a literal reading of "after it" — pass 1 collects every top-level `use!` in
/// the program before pass 2 checks any call head, so a `use!` technically also covers
/// references written textually BEFORE it — does not point at a different category: `:Ambient`
/// (process-global state no value addresses) does not fit either, because what is registered
/// IS a value-addressed name (the `:rust::...` path), looked up by name, not a bare flag; and
/// `:CheckGate` (a call site refusing itself at check time) does not fit because `use!` does not
/// refuse itself — it ENABLES other call sites. `Declaration` is the closest honest fit, and
/// it is `:wat::config::set-redef!`/`set-eval-redef!`'s sibling row's own contrast case (see
/// `config_set_redef.rs`): those flip a process-global mode flag no value addresses (`Ambient`);
/// this one registers a name (`Declaration`). They do NOT share a category.
///
/// **Purity ground —** measured directly, the same method `config_set_redef.rs`'s row uses:
/// `:wat::core::use!` appears in `src/runtime.rs` exactly once as a real eval arm
/// (`dispatch_keyword_head_value`, `:2947`, `=> Ok(Value::Unit)`), reached (not refused) at
/// expression position, so `Unevaluated` would be a lie about a form that measurably evaluates.
/// The arm ignores `args` completely — it never calls `eval`/`step_list` on the keyword
/// argument, so no sub-form's effect (there is none to have; the argument is a bare keyword
/// literal, never itself an evaluated expression in this position) can leak through either.
/// Unconditionally free of effect at the moment it runs. `Pure`.
///
/// **Determinism ground —** the eval arm consults nothing — no clock, no entropy, no read of
/// `env`/`sym` state — and returns the same `Value::Unit` unconditionally, for any argument
/// that reached it (malformed arguments are refused earlier, at check/resolve). `Deterministic`.
///
/// **Totality ground —** read directly: the eval arm is `Ok(Value::Unit)` with no match, no
/// unwrap, no fallible sub-call — defined unconditionally for every input that reaches it, the
/// strongest form of `Total` (it does not even inspect its argument to fail on a bad shape).
/// `Total`.
///
/// **Expand-time ground —** the eval arm evaluates none of its own sub-forms (the keyword
/// argument is read, never evaluated) and consults no runtime-only state (no clock, no spawn,
/// no submitted-form evaluation) to produce its unconditional `Ok(Value::Unit)` — legal
/// unconditionally, the same shape `fn`'s own `Legal` ruling argues
/// (`fn_form.rs`: "unconditionally free ... regardless of what happens elsewhere"). `Legal`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::core::use! <rust-path>)
/// @ret :wat::core::nil always `Unit` — the declaration's real work (validating and recording the dependency) already happened at the resolve pass; the eval arm ignores its argument
/// @example (:wat::core::use! :rust::cache::Lru) #=> nil
#[wat_special_form(":wat::core::use!")]
pub(crate) struct Use;

/// Arc 255 Stone 1a-ε — the `role = eval` pointer for `:wat::core::use!`. The dispatch arm at
/// `runtime.rs:2947` (`=> Ok(Value::Unit)`) stays untouched (STOP-5); this is a new, standalone
/// delegate carrying the canonical `NativeHandler` signature purely to host the registration —
/// the same move `fn_form.rs`'s `eval_fn_form` makes for `:wat::core::fn`. Its body is the
/// identical no-op the existing arm already performs.
#[wat_special_form_impl(":wat::core::use!", role = eval)]
fn eval_use_form(
    _args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    Ok(Value::Unit)
}

/// Arc 255 Stone 1a-ε — the `role = check` pointer for `:wat::core::use!`. The inline arm at
/// `check.rs:4766` stays untouched (STOP-5); this is a new, standalone delegate carrying
/// `infer_config_set_bool`'s own signature shape purely to host the registration. Its body is
/// the identical no-op-returning-Unit-type the existing arm already performs: "the type checker
/// treats it as a no-op returning `:()`. The argument is a keyword path; we don't recurse into
/// it" (the existing arm's own comment).
#[wat_special_form_impl(":wat::core::use!", role = check)]
pub(crate) fn infer_use_form(
    _head: &str,
    _args: &[WatAST],
    _head_span: &Span, // rune:lint(unused-span) — infallible — no error path
    _env: &CheckEnv,
    _locals: &HashMap<String, TypeExpr>,
    _fresh: &mut InferCtx,
    _subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    CheckResult::ok(TypeExpr::Tuple(vec![]))
}
