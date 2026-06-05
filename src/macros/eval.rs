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
//! `(:wat::core::quasiquote X)` and `(:wat::core::quote X)` are skipped
//! entirely (X is template/literal data, not evaluable code). This mirrors
//! how `expand_form` skips them (expand.rs:~85).
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
///   - head == `quasiquote` or `quote` → return Ok (skip contents; data, not code)
///   - head is on the blessed allow-list → recurse into args
///   - otherwise → `Err(MacroError { kind: RefusedInMacro { head } })`
///
/// Non-List nodes and Lists without a Keyword head are walked recursively
/// (their children may contain keyword-headed sub-forms).
pub(crate) fn validate_pure_total(form: &WatAST) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, span) => {
            match items.first() {
                Some(WatAST::Keyword(head, _)) => {
                    // Data forms: skip recursion into their contents.
                    // These are template/literal data, not evaluable macro code.
                    if head == ":wat::core::quasiquote" || head == ":wat::core::quote" {
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
    )
}
