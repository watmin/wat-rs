//! Special-form doc entry for `:wat::config::set-eval-redef!` — arc 255 Stone 1a-ε, shape ③,
//! `:wat::config::set-redef!`'s sibling. See `config_set_redef.rs`'s module doc for the full
//! STOP-3 finding (two real freeze-time processors, one per structural position) — identical
//! here, since `collect_entry_file_inner` and `register_runtime_defs_form` each handle both
//! setters in the same arms.
//!
//! The `role = eval` impl fn for THIS fqdn is `config_set_redef.rs`'s
//! `eval_config_set_eval_redef` — a separate fn from `set-redef!`'s own
//! `eval_config_set_redef`, NOT one shared fn stacked with two attributes: `role = eval`
//! codegens a dispatch shim named from the fn alone
//! (`wat_special_form_impl.rs`'s `emit`), so two `role = eval` attributes on one fn would mint
//! the same shim name twice — a duplicate-definition error. See `config_set_redef.rs`'s doc on
//! `eval_config_set_redef` for the full mechanism. Both fns' bodies are the identical
//! `Ok(Value::Unit)` no-op the shared arm at `runtime.rs:2120` already performs.

use wat_macros::wat_special_form;

/// Opt in (or out of) permitting a runtime *evaluated* `def`-equivalent (the eval-time path
/// `:wat::config::set-redef!` does not cover) to redefine an already-bound top-level name:
/// `(:wat::config::set-eval-redef! true)` / `(:wat::config::set-eval-redef! false)`. Same
/// freeze-time processing, same eval-arm no-op, same STOP-3 two-processor finding as
/// `:wat::config::set-redef!` (`config_set_redef.rs`) — every ground below is that row's,
/// restated for this FQDN because `render-doc` reads per-entry, not per-argument-shared-fn.
///
/// **Category ground —** identical reasoning to `:wat::config::set-redef!`'s row: this form
/// flips a process-global boolean (`sym.eval_redef_allowed`) that no value the caller holds
/// addresses — `:Ambient`'s own prose, not `:Declaration`'s (no name is registered for lookup).
/// `Ambient`.
///
/// **Purity ground —** measured directly: `:wat::config::set-eval-redef!` shares
/// `:wat::config::set-redef!`'s literal eval arm (`runtime.rs:2120`,
/// `":wat::config::set-redef!" | ":wat::config::set-eval-redef!" => Ok(Value::Unit)`) — the SAME
/// non-firing-`println`-probe evidence applies verbatim (the arm ignores `args`, whichever of
/// the two heads reached it). `Pure`.
///
/// **Determinism ground —** identical: the eval arm consults nothing and returns the same
/// `Value::Unit` unconditionally. `Deterministic`.
///
/// **Totality ground —** identical: `Ok(Value::Unit)`, no match, no fallible sub-call, defined
/// unconditionally for every input that reaches it. `Total`.
///
/// **Expand-time ground —** identical: no sub-forms evaluated, no runtime-only state consulted.
/// `Legal`.
///
/// @added 1.0.0
/// @Category Ambient
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::config::set-eval-redef! <bool>)
/// @ret :wat::core::nil always `Unit` — the flag update already happened at freeze time; the eval arm ignores its argument
/// @example (:wat::config::set-eval-redef! true) #=> nil
#[wat_special_form(":wat::config::set-eval-redef!")]
pub(crate) struct ConfigSetEvalRedef;
