//! vigilatum: 2026-06-06T04:56:04Z — vigilia 8-spell L1+L2=0 (first earned 2026-06-01T04:45:47Z; RE-EARNED diff-scoped at the 245 clear: the variadic rest-binder thread-through [ParsedFnSignature<N> minted, both type_complexity runes killed in combat; env_key uniform across all three converters; no strip-and-rewalk — one parse, threaded]; gates: variadic_define 16/16, lib 923/0/1, clippy-in-home empty)
//!
//! # Function — namespaced home for fn-form parsing, evaluation, and inference.
//!
//! ## Why this module exists
//!
//! Stone 241.18a — first stepping stone of the bar-raise chain (241.18a-g).
//! Mints `src/function/` as the dedicated namespaced home for the fn-form
//! machinery, per `feedback_namespaced_home_vigilia_gate` REMARKABLE bar.
//!
//! The substrate previously spread fn-form concerns across two large files:
//! - `src/runtime.rs` — `eval_fn` + `parse_fn_signature`
//! - `src/check.rs` — `infer_fn` + `parse_fn_signature_for_check` + `_diag`
//!
//! This home collects all three concerns (parse, eval, infer) for the
//! `:wat::core::fn` form under one roof. The design mirrors the established
//! namespaced-home convention (argspec/, comms/, remedy/) — one domain per home;
//! sub-files by concern within the domain.
//!
//! ## Depends on
//!
//! - `src/argspec/` — canonical triple parsing routed through `parse_argspec_triples`
//! - `src/runtime.rs` — `Function`, `Value`, `Environment`, `RuntimeError`,
//!   `synthesize_fn_body`
//! - `src/types.rs` — `parse_type_expr_with_span`, `TypeError`, `TypeExpr`
//! - `src/check.rs` — `CheckEnv`, `CheckError`, `CheckResult`, `InferCtx`,
//!   `Subst`, `infer`, `unify`, `apply_subst`, `format_type`
//!
//! ## Test home scope
//!
//! `tests/function/stone18a.rs` covers fn-form check-pass preservation (C01, C02).
//! `tests/function/stone18a_errors.rs` covers check-tier error paths (E01-E06).
//!
//! `eval_fn` runtime-eval path is exercised by integration tests using
//! `invoke_user_main` (~20 sites including `wat_arc201_signature_of_fn.rs`,
//! `wat_dispatch_e1_vec.rs`, `wat_recursive_patterns.rs`, etc.).
//!
//! `peel_metadata_preamble` metadata-present branch is exercised by
//! `tests/probe_arc241_stone6_def_metadata_map.rs` and
//! `tests/probe_arc241_stone7_metadata_of_reflection.rs` via `defn` macro
//! expansion to `(fn {meta} [args] -> :ret body)`.
//!
//! `parse_fn_signature_for_check`'s primary call site — the `:ensure :fn`
//! defclause validation in `src/check.rs` (the sole caller of
//! `parse_fn_signature_for_check`) — is exercised by
//! `tests/probe_arc237_stone3_guard_ensure.rs`; `tests/function/` focuses on
//! fn-form preservation + error contracts; cross-arc coverage stays in its
//! arc's probe.

mod eval;
mod infer;
mod metadata;
mod parse;
mod subsume;

/// The canonical form head for all `:wat::core::fn` error messages.
/// Declared once; all sub-modules reference this constant so the literal
/// never drifts.
pub(in crate::function) const FN_HEAD: &str = ":wat::core::fn";

// Arc 109 Stone — the defclause-into-function-home stone added the `defclause`/`extend-type`/
// `derive` parser re-exports (`parse` list, below) and the `defclause` call-dispatch
// re-exports (`eval` list, below), plus the new `subsume` sub-module — home-internal only,
// nothing outside `src/function/` reaches it. No new home minted; this fills out the
// existing one.
pub(crate) use eval::{eval_call_to_defclause, eval_call_to_defclause_with_vals, eval_fn, select_defclause_clause};
pub(crate) use infer::infer_fn;
pub(crate) use metadata::peel_type_binder;
pub(crate) use parse::{
    parse_defclause_form, parse_derive_form, parse_extend_type_form,
    parse_fn_signature, parse_fn_signature_for_check,
};
