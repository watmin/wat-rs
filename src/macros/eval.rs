//! Fenced macro evaluator — DEFAULT-DENY pure-combinator restriction of `runtime::eval`.
//!
//! `macro_eval(form, env, sym)`:
//!   1. `validate_pure_total(form)?`  — walk the form; any keyword head not on the
//!      blessed allow-list (or not handled by the quote/quasiquote data-skip) → Err.
//!   2. `crate::runtime::eval(form, env, sym)` — run the already-validated form
//!      through the existing evaluator. No new interpreter, no duplicated arithmetic.
//!
//! # DEFAULT-DENY
//!
//! The allow-list is the PURE-TOTAL subset of `dispatch_keyword_head` /
//! `dispatch_keyword_head_value` (runtime.rs:5295 / 5318). Every head NOT
//! explicitly blessed is refused. This means:
//!   - a newly-added effectful prim is AUTOMATICALLY refused (the F5 class
//!     structurally cannot recur) — it stays denied until someone deliberately blesses it.
//!   - a forgotten pure head causes a false-refusal (a RED test), never a
//!     silently-admitted effect. The suite teaches completeness.
//!
//! # Skipped forms (data, not code)
//!
//! `(:wat::core::quote X)` is skipped entirely (X is pure literal data, never evaluated).
//!
//! `(:wat::core::quasiquote X)` is NOT simply skipped: `X` is a template that is
//! **data** (the literal parts) PLUS **code** (the `~`/`~@` unquote sub-expressions).
//! `validate_pure_total` descends into `X` tracking quasiquote depth (mirroring
//! `walk_quasiquote`'s depth logic): nested `quasiquote` bumps depth +1; an
//! `unquote` or `unquote-splicing` that *fires* at depth 1 means its sub-expression
//! is real code — `validate_pure_total` recurses into it. Literal template nodes
//! and material below depth 1 (inside a nested quasiquote) stay skipped (data).
//! This closes the F5-redux hole: an impure computed unquote in a program-body
//! quasiquote is refused before eval, not silently executed at expand time.
//!
//! # Load-bearing invariant — expand runs BEFORE user-defn registration
//!
//! `validate_pure_total` gates keyword *heads*. A blessed HOF (`map`/`foldl`/…)
//! takes a function ARG; an inline `(:wat::core::fn …)` lambda's body IS
//! validated (the walk recurses into it), but a bare reference to a top-level
//! user `defn` passed as that arg would NOT be head-checked. That vector is
//! closed not here but by the **freeze pipeline order**: `expand_all`
//! (freeze.rs:~745/751 — where this evaluator runs) precedes
//! `register_defines` (freeze.rs:~807), so **user/stdlib `defn`s are not yet
//! registered at expand time** — a reference to one does not resolve (it
//! errors), it cannot run an impure body. Only blessed builtins + inline
//! lambdas (body-validated) are reachable.
//!
//! **If that order ever changes** (defn-registration moved before macro
//! expansion), this evaluator alone no longer guarantees totality/purity — a
//! gate at `apply_function` (refuse applying a top-level user `defn` in macro
//! mode; the DESIGN's "gate 2") becomes necessary. Mark this dependency before
//! touching the freeze order.
//!
//! # Provenance
//!
//! Surface names (`macro_eval`, `validate_pure_total`, `RefusedInMacro`) are
//! PROPOSED — owed an intueri cast by the orchestrator (arc 249.2b
//! BRIEF-STONE-249.2b-i.md § naming).
//!
//! Arc 249 Stone 249.2b-i — F5 is now CLOSED here (gated by this module).

use crate::ast::WatAST;
use crate::runtime::{Environment, SymbolTable};

use super::error::{MacroError, MacroErrorKind};

// ─── Public surface ──────────────────────────────────────────────────────────

/// Fenced expand-time evaluator: validate purity, then delegate to the
/// existing `runtime::eval`. Closes F5 (arc 249 finding): any impure keyword
/// head in `form` is refused before eval is invoked.
///
/// Returns `Err(MacroError)` (wrapping `MacroErrorKind::RefusedInMacro`) for
/// any keyword-headed sub-form whose head is not on the blessed allow-list.
/// Returns `Err(MacroError { kind: MalformedTemplate })` if the underlying
/// `runtime::eval` fails.
pub(crate) fn macro_eval(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::runtime::TrackedValue, MacroError> {
    // Pre-walk validation — DEFAULT-DENY purity gate.
    validate_pure_total(form)?;
    // Delegate to the existing evaluator — no new interpreter.
    crate::runtime::eval(form, env, sym).map_err(|e| MacroError {
        span: form.span().clone(),
        kind: MacroErrorKind::MalformedTemplate {
            reason: format!("macro_eval: runtime::eval failed: {}", e),
        },
    })
}

// ─── Validator ───────────────────────────────────────────────────────────────

/// Walk `form` recursively. At each `List` with a `Keyword` head:
///   - head == `quote` → return Ok (skip contents; pure literal data, never evaluated)
///   - head == `quasiquote` → descend into the template tracking depth (see below)
///   - head is on the blessed allow-list → recurse into args
///   - otherwise → `Err(MacroError { kind: RefusedInMacro { head } })`
///
/// Non-List nodes and Lists without a Keyword head are walked recursively
/// (their children may contain keyword-headed sub-forms).
///
/// # Quasiquote depth tracking (mirrors `walk_quasiquote`)
///
/// A quasiquote template mixes data (literal template nodes) and code
/// (`~`/`~@` unquote sub-expressions). When `validate_pure_total` enters a
/// `(:wat::core::quasiquote X)`, it calls `validate_quasiquote_template(X, 1)`.
/// That helper walks `X` at depth 1:
///   - nested `(:wat::core::quasiquote …)` → bump depth +1, recurse (data below)
///   - `(:wat::core::unquote E)` or `(:wat::core::unquote-splicing E)` at depth 1
///     → `E` is real code; recurse `validate_pure_total(E)`
///   - same unquote forms at depth > 1 → peel depth, recurse (still inside a
///     nested quasiquote; E is data at this level)
///   - everything else → recurse `validate_quasiquote_template` (template data)
pub(crate) fn validate_pure_total(form: &WatAST) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, span) => {
            match items.first() {
                Some(WatAST::Keyword(head, _)) => {
                    // Pure literal data: skip entirely.
                    if head == ":wat::core::quote" {
                        return Ok(());
                    }
                    // Quasiquote: descend into the template with depth tracking.
                    // The template mixes data (literal nodes) and code (unquote
                    // sub-expressions); validate only the code parts.
                    if head == ":wat::core::quasiquote" {
                        if let Some(template) = items.get(1) {
                            validate_quasiquote_template(template, 1)?;
                        }
                        return Ok(());
                    }
                    // Check against the blessed allow-list.
                    if is_pure_total(head) {
                        // Recurse into all args (skip the head keyword itself).
                        for child in items.iter().skip(1) {
                            validate_pure_total(child)?;
                        }
                        Ok(())
                    } else {
                        Err(MacroError {
                            span: span.clone(),
                            kind: MacroErrorKind::RefusedInMacro { head: head.clone() },
                        })
                    }
                }
                // List with non-Keyword head (e.g. a nested list, or no head):
                // recurse into all children.
                _ => {
                    for child in items {
                        validate_pure_total(child)?;
                    }
                    Ok(())
                }
            }
        }
        // Vectors: recurse into elements (may contain keyword-headed sub-forms).
        WatAST::Vector(items, _) => {
            for child in items {
                validate_pure_total(child)?;
            }
            Ok(())
        }
        // Leaf nodes — no sub-forms to check.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _)
        | WatAST::StructPattern(_, _) => Ok(()),
    }
}

/// Depth-tracked walk of a quasiquote template, called by `validate_pure_total`
/// when it encounters `(:wat::core::quasiquote X)`. Mirrors the depth logic of
/// `walk_quasiquote` in runtime.rs:
///
///   - nested `quasiquote` → bump depth +1 (still inside data; recurse)
///   - `unquote` / `unquote-splicing` at depth 1 → E is real code; validate it
///   - same forms at depth > 1 → peel depth (still data within nested quasiquote)
///   - everything else → recurse at current depth (template data node)
///
/// Arc 249 Stone 249.3a — closes the F5-redux hole: an impure computed unquote
/// inside a program-body quasiquote is refused here, before `runtime::eval` runs.
fn validate_quasiquote_template(form: &WatAST, depth: u32) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                // Nested quasiquote: bump depth, recurse into template.
                if head == ":wat::core::quasiquote" {
                    if let Some(inner) = items.get(1) {
                        validate_quasiquote_template(inner, depth + 1)?;
                    }
                    return Ok(());
                }
                // Unquote or unquote-splicing.
                if head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing" {
                    if depth == 1 {
                        // Fires at this level: E is real code. Validate it fully.
                        if let Some(code) = items.get(1) {
                            validate_pure_total(code)?;
                        }
                    } else {
                        // Still inside a nested quasiquote: peel depth, stay in template mode.
                        if let Some(inner) = items.get(1) {
                            validate_quasiquote_template(inner, depth - 1)?;
                        }
                    }
                    return Ok(());
                }
            }
            // Plain list in the template: walk children at the same depth.
            for child in items {
                validate_quasiquote_template(child, depth)?;
            }
            Ok(())
        }
        WatAST::Vector(items, _) => {
            for child in items {
                validate_quasiquote_template(child, depth)?;
            }
            Ok(())
        }
        // Leaf nodes in the template — no sub-forms to check.
        _ => Ok(()),
    }
}

// ─── Blessed allow-list — DEFAULT-DENY ───────────────────────────────────────
//
// ONLY the pure-total subset of `dispatch_keyword_head` / `dispatch_keyword_head_value`
// (runtime.rs:5295 / 5318). Effectful heads (`:wat::kernel::*`, IO, spawning,
// time `now`, random UUIDs, signal queries, `:wat::core::apply`,
// `:wat::core::eval-ast!`, etc.) are NOT present here — they are DENIED by default.
//
// The suite teaches completeness: a false-refusal (a pure head missing from this
// list) makes a stdlib test RED. Add it here. A missing effectful head is harmless
// (stays denied).
//
// Arc 249 Stone 249.2b-i — F5 CLOSED: this allow-list is the gate.
fn is_pure_total(head: &str) -> bool {
    matches!(
        head,
        // ── Integer arithmetic (pure, total, wrapping) ─────────────────
        ":wat::core::i64::+"
        | ":wat::core::i64::-"
        | ":wat::core::i64::*"
        | ":wat::core::i64::/"

        // ── Integer comparison ─────────────────────────────────────────
        | ":wat::core::i64::>"
        | ":wat::core::i64::<"
        | ":wat::core::i64::>="
        | ":wat::core::i64::<="

        // ── Float arithmetic (pure, IEEE 754) ─────────────────────────
        | ":wat::core::f64::+"
        | ":wat::core::f64::-"
        | ":wat::core::f64::*"
        | ":wat::core::f64::/"
        | ":wat::core::f64::abs"
        | ":wat::core::f64::max"
        | ":wat::core::f64::min"
        | ":wat::core::f64::round"
        | ":wat::core::f64::clamp"
        | ":wat::core::f64::max-of"
        | ":wat::core::f64::min-of"

        // ── Float comparison ───────────────────────────────────────────
        | ":wat::core::f64::>"
        | ":wat::core::f64::<"
        | ":wat::core::f64::>="
        | ":wat::core::f64::<="

        // ── Polymorphic equality / relational ─────────────────────────
        | ":wat::core::="
        | ":wat::core::not="

        // ── Boolean logic ─────────────────────────────────────────────
        | ":wat::core::and"
        | ":wat::core::or"
        | ":wat::core::not"

        // ── Scalar conversions (pure) ──────────────────────────────────
        | ":wat::core::i64::to-string"
        | ":wat::core::i64::to-f64"
        | ":wat::core::i64/to-f64"
        | ":wat::core::i64/to-string"
        | ":wat::core::f64::to-string"
        | ":wat::core::f64::to-i64"
        | ":wat::core::bool::to-string"
        | ":wat::core::string::to-i64"
        | ":wat::core::string::to-f64"
        | ":wat::core::string::to-bool"

        // ── Keyword / symbol ops (pure) ────────────────────────────────
        | ":wat::core::keyword/to-string"
        | ":wat::core::keyword/from-string"  // pure constructor (routed via dispatch_keyword_head)

        // ── String ops (pure) ─────────────────────────────────────────
        | ":wat::core::string::concat"
        | ":wat::core::String/concat"
        | ":wat::core::string::contains?"
        | ":wat::core::String/contains?"
        | ":wat::core::string::starts-with?"
        | ":wat::core::String/starts-with?"
        | ":wat::core::string::ends-with?"
        | ":wat::core::String/ends-with?"
        | ":wat::core::string::length"
        | ":wat::core::string::trim"
        | ":wat::core::string::split"
        | ":wat::core::string::join"
        | ":wat::core::String/empty?"

        // ── Type inspection (pure) ─────────────────────────────────────
        | ":wat::core::type"
        | ":wat::core::conforms?"
        | ":wat::core::subtype?"
        | ":wat::core::record?"

        // ── Control flow ──────────────────────────────────────────────
        | ":wat::core::if"
        | ":wat::core::cond"
        | ":wat::core::match"
        | ":wat::core::let"
        | ":wat::core::do"
        | ":wat::core::fn"

        // ── Collections — constructors ─────────────────────────────────
        | ":wat::core::Vector"
        | ":wat::core::Tuple"
        | ":wat::core::HashMap"
        | ":wat::core::HashSet"

        // ── Collections — polymorphic intrinsics ─────────────────────
        | ":wat::core::length"
        | ":wat::core::empty?"
        | ":wat::core::contains?"
        | ":wat::core::conj"
        | ":wat::core::get"
        | ":wat::core::assoc"
        | ":wat::core::first"
        | ":wat::core::second"
        | ":wat::core::third"
        | ":wat::core::last"
        | ":wat::core::rest"

        // ── Collections — per-type ops ────────────────────────────────
        | ":wat::core::Vector/length"
        | ":wat::core::Vector/empty?"
        | ":wat::core::Vector/contains?"
        | ":wat::core::Vector/get"
        | ":wat::core::Vector/conj"
        | ":wat::core::Vector/concat"
        | ":wat::core::HashMap/length"
        | ":wat::core::HashMap/empty?"
        | ":wat::core::HashMap/contains-key?"
        | ":wat::core::HashMap/get"
        | ":wat::core::HashMap/assoc"
        | ":wat::core::HashMap/dissoc"
        | ":wat::core::HashMap/keys"
        | ":wat::core::HashMap/values"
        | ":wat::core::HashSet/length"
        | ":wat::core::HashSet/empty?"
        | ":wat::core::HashSet/contains?"
        | ":wat::core::HashSet/conj"

        // ── Collections — HOFs (bounded iteration over finite lists) ──
        | ":wat::core::map"
        | ":wat::core::filter"
        | ":wat::core::foldl"
        | ":wat::core::foldr"
        | ":wat::core::range"
        | ":wat::core::take"
        | ":wat::core::drop"
        | ":wat::core::reverse"
        | ":wat::core::sort-by"
        | ":wat::core::find-last-index"

        // ── Option / Result (pure unwrappers, no effects) ────────────
        | ":wat::core::Option/expect"
        | ":wat::core::Option/try"
        | ":wat::core::Result/expect"
        | ":wat::core::Result/try"

        // ── Math (pure functions, deterministic) ─────────────────────
        | ":wat::std::math::ln"
        | ":wat::std::math::log"
        | ":wat::std::math::exp"
        | ":wat::std::math::sqrt"
        | ":wat::std::math::sin"
        | ":wat::std::math::cos"
        | ":wat::std::math::pi"

        // ── Statistics (pure over closed data) ───────────────────────
        | ":wat::std::stat::mean"
        | ":wat::std::stat::variance"
        | ":wat::std::stat::stddev"

        // ── Holon AST / form construction (pure; no IO) ──────────────
        | ":wat::holon::Atom"
        | ":wat::holon::Bind"
        | ":wat::holon::Bundle"
        | ":wat::holon::Permute"
        | ":wat::holon::Thermometer"
        | ":wat::holon::Blend"
        | ":wat::holon::to-wat"
        | ":wat::holon::from-wat"
        | ":wat::holon::to-holon"
        | ":wat::holon::from-holon"
        | ":wat::holon::statement-length"
        | ":wat::holon::Bundle/children"
        | ":wat::holon::Bundle/first"
        | ":wat::holon::Bind/left"
        | ":wat::holon::Bind/right"
        | ":wat::holon::extract-classifier"
        | ":wat::holon::leaf"
        | ":wat::holon::is?"
        | ":wat::holon::is-Map?"
        | ":wat::holon::is-Set?"
        | ":wat::holon::is-Vector?"
        | ":wat::holon::is-List?"
        | ":wat::holon::is-Tuple?"
        | ":wat::holon::is-Symbol?"
        | ":wat::holon::is-Keyword?"
        | ":wat::holon::is-Tag?"
        | ":wat::holon::is-Nil?"

        // ── Quasiquote / unquote (pure form-builders) ─────────────────
        // Note: quasiquote and quote are SKIPPED in validate_pure_total
        // (data, not code). Listed here only for completeness / in case
        // runtime::eval is given a raw quasiquote that validate_pure_total
        // allowed through the skip path; the skip path does not add the
        // head to the blessed set, so they land in is_pure_total only if
        // explicitly listed (harmless).
        | ":wat::core::quasiquote"
        | ":wat::core::quote"
        | ":wat::core::struct->form"
        | ":wat::core::forms"
        | ":wat::core::show"

        // ── Form-shape predicates (pure over WatAST form-values) ──────
        // core form-shape predicate over WatAST::List; distinct from
        // :wat::holon::is-List? (a classifier over HolonAST). The name
        // diverges on purpose — the form-vs-holon distinction is the
        // reason this exists. Do not "harmonize" the two names.
        | ":wat::core::List?"
    )
}
