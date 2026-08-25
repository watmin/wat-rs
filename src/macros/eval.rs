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
//! `dispatch_keyword_head_value` (runtime.rs: fn dispatch_keyword_head / fn dispatch_keyword_head_value). Every head NOT
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
//! Surface names (`macro_eval`, `validate_pure_total`, `RefusedInMacro`) were
//! RATIFIED by an intueri cast (arc 249.2b; rationale recorded in
//! `error.rs:51` at the `RefusedInMacro` variant). Arc 249 Stone 249.2b-i.
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
) -> Result<crate::value::TrackedValue, MacroError> {
    // Pre-walk validation — DEFAULT-DENY purity gate.
    validate_pure_total(form)?;
    macro_eval_pre_validated(form, env, sym)
}

/// Evaluate a form that has ALREADY been validated by `validate_pure_total`
/// at definition time (via `validate_macro_definition` in `parse_defmacro_form`).
///
/// WHY this variant exists: `expand_program_body` passes the immutable definition
/// body to `macro_eval`. That body was validated ONCE at definition time (the
/// hoist — arc 249 stone O). Re-running `validate_pure_total` on every invocation
/// would be redundant. Substituted forms in `unquote_argument` and
/// `splice_argument` are NOT definition-body forms — they carry fresh AST built
/// at call time and MUST be validated; those sites use `macro_eval` (with
/// validation).
///
/// INVARIANT: callers must guarantee the form was validated by
/// `validate_pure_total` before calling this function. The only sanctioned
/// call site is `expand_program_body`.
pub(super) fn macro_eval_pre_validated(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, MacroError> {
    // Delegate to the existing evaluator — no new interpreter.
    // Thread `e.span` (the runtime's precise failing-site span) into MacroError.
    // Arc 298.2: every span is real; always use e.span directly.
    crate::runtime::eval(form, env, sym).map_err(|e| {
        let span = e.span().clone();
        // Arc 258 Stone 258.2b: MacroAbort surfaces clean — user message only,
        // no "macro_eval: runtime::eval failed:" prefix noise.
        // Arc 296: non-MacroAbort failures carry the typed RuntimeError cause
        // instead of collapsing to a prose string.
        match e.kind() {
            crate::runtime::RuntimeErrorKind::MacroAbort { message } => MacroError {
                span,
                kind: MacroErrorKind::MalformedTemplate { reason: message.clone() },
            },
            _ => MacroError {
                span,
                kind: MacroErrorKind::MacroEvalRuntimeFailed { cause: Box::new(e) },
            },
        }
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
pub(super) fn validate_pure_total(form: &WatAST) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, span) => {
            match items.first() {
                Some(WatAST::Keyword(head, _)) => {
                    // Pure literal data: skip entirely.
                    // Arc 294.b — `:wat::holon::literal` is also pure data; skip.
                    if head == ":wat::core::quote" || head == ":wat::holon::literal" {
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
                        // BEWARE: `fn` forms are NOT blanket-opaque here. A fn in a
                        // program body can be INVOKED at expand time (blessed HOFs —
                        // map, foldl — take fns), so its body is expand-time code and
                        // MUST be pure-validated. Blanket fn-opacity reopens F5 (a
                        // kernel-send hidden in a HOF'd fn body would fire during
                        // expansion). Caught + reverted at the 245 long-tail scoring.
                        //
                        // THE ONE sound contextual exception: a literal fn form as
                        // the argument of `:wat::runtime::signature-of-fn`. That verb
                        // only CREATES the closure (pure) and reads its SIGNATURE —
                        // the body never executes on this path, so its heads (user
                        // fns destined for the expansion's runtime code) are not
                        // expand-time code. Shared macro-purity infra (arc 249) — not
                        // run-threads-specific, even though run-threads (since retired)
                        // was the originating consumer that reflected on its caller's
                        // coordinator/factory fns this way.
                        if head == ":wat::runtime::signature-of-fn" {
                            for child in items.iter().skip(1) {
                                let is_literal_fn = matches!(
                                    child,
                                    WatAST::List(fn_items, _)
                                        if matches!(
                                            fn_items.first(),
                                            Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn"
                                        )
                                );
                                if !is_literal_fn {
                                    validate_pure_total(child)?;
                                }
                            }
                            return Ok(());
                        }
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
        // Arc 257 slice 1 — Map/Set literals are pure (recurse into k/v and elements).
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                validate_pure_total(k)?;
                validate_pure_total(v)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for item in items {
                validate_pure_total(item)?;
            }
            Ok(())
        }
        // Leaf nodes — no sub-forms to check.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        // Arc 300 stone B — rational literal is a leaf, same as int/float.
        | WatAST::RationalLit(_, _)
        // Arc 300 stone C1 — bigint literal is a leaf too.
        | WatAST::BigIntLit(_, _)
        // Arc 300 stone D — char literal is a leaf too.
        | WatAST::CharLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => Ok(()),
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
///
/// rune:solvere(load-bearing-coupling) — qq depth-walk is mirrored in 3 sites
/// (walk_template / validate_quasiquote_template / walk_quasiquote); the depth
/// rule (nested +1, fire-at-depth-1, peel-deeper) is one contract that must
/// change in all three in sync; a unifying visitor would obscure three readable
/// single-purpose walkers.
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
        // Arc 257 slice 1 — Map/Set template nodes: walk all children at the same depth.
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                validate_quasiquote_template(k, depth)?;
                validate_quasiquote_template(v, depth)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
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
// (runtime.rs: fn dispatch_keyword_head / fn dispatch_keyword_head_value). Effectful heads (`:wat::kernel::*`, IO, spawning,
// time `now`, random UUIDs, signal queries, `:wat::core::apply`,
// `:wat::core::eval-ast!`, etc.) are NOT present here — they are DENIED by default.
//
// The suite teaches completeness: a false-refusal (a pure head missing from this
// list) makes a stdlib test RED. Add it here. A missing effectful head is harmless
// (stays denied).
//
// Arc 249 Stone 249.2b-i — F5 CLOSED: this allow-list is the gate.
//
// rune:struere(invariant-coupling) — this allow-list mirrors the pure-total arm of
// fn dispatch_keyword_head / fn dispatch_keyword_head_value (runtime.rs); the suite enforces completeness
// (default-deny makes over-restriction the only drift direction).
fn is_pure_total(head: &str) -> bool {
    matches!(
        head,
        // ── Integer arithmetic (pure, total, wrapping) ─────────────────
        // mod/rem/quot route through the same eval_i64_arith dispatch as
        // `/` (runtime.rs) — div-by-zero is a deterministic located abort
        // (RuntimeErrorKind::DivisionByZero), never a panic, same as `/`.
        ":wat::core::i64::+"
        | ":wat::core::i64::-"
        | ":wat::core::i64::*"
        | ":wat::core::i64::/"
        | ":wat::core::i64::mod"
        | ":wat::core::i64::rem"
        | ":wat::core::i64::quot"

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
        // Dual-spelled pairs (e.g. i64::to-f64 / i64/to-f64; string::concat /
        // String/concat): both surface spellings route to the same dispatch arm;
        // both are listed here intentionally so either spelling in a macro
        // program-body is allowed.
        | ":wat::core::i64::to-string"
        | ":wat::core::i64::to-f64"
        | ":wat::core::i64/to-f64"
        | ":wat::core::i64/to-string"
        | ":wat::core::f64::to-string"
        | ":wat::core::f64::to-i64"
        | ":wat::core::bool::to-string"
        | ":wat::string::to-i64"
        | ":wat::string::to-f64"
        | ":wat::string::to-bool"

        // ── Keyword / symbol ops (pure) ────────────────────────────────
        | ":wat::core::keyword/to-string"
        | ":wat::core::keyword/from-string"  // pure constructor (routed via dispatch_keyword_head)

        // ── Macro diagnostics (pure: deterministic abort, no IO) ────────
        // Arc 258 Stone 258.2b — first-class macro-abort. Aborts expansion
        // with a user diagnostic. Pure: no IO, deterministic (same message
        // → same MacroError); the deliberate abort is safe at expand time.
        | ":wat::core::macro-error"

        // Arc 278 §4 — `:wat::kernel::macro-call-site`: reads the expand-time
        // `MACRO_CALL_SITE` thread-local (the CURRENT macro invocation's own
        // source span, pushed by `expand_macro_call`) and returns a
        // spliceable Frame-constructor FORM. Pure + deterministic PER
        // EXPANSION (same invocation → same span, every time it's read
        // during that invocation's expansion) and does no IO — it is the
        // `log`-macro's per-log-line `emitted-from` primitive, so it must be
        // permitted in a macro body for that macro to ever exist.
        | ":wat::kernel::macro-call-site"

        // ── String ops (pure) ─────────────────────────────────────────
        | ":wat::string::concat"
        // Arc 284 — pure-total interpolation intrinsic: same {name} + :name val grammar as
        // the format macro, but interpolates at call time → expand-time-legal in macro bodies.
        | ":wat::string::interpolate"
        | ":wat::core::String/concat"
        | ":wat::string::contains?"
        | ":wat::core::String/contains?"
        | ":wat::string::starts-with?"
        | ":wat::core::String/starts-with?"
        | ":wat::string::ends-with?"
        | ":wat::core::String/ends-with?"
        | ":wat::string::length"
        // Arc 279.1 — subs is on is_pure_total: the `format` macro walks the template
        // character-by-character at expand time (length + subs i (i+1)) to collapse the
        // `{{`/`}}` literal-brace escape. Char-indexed + total-deterministic (out-of-range
        // is a deterministic abort, like split's empty-sep refusal). "Does a macro need it?" — yes.
        | ":wat::string::subs"
        | ":wat::string::trim"
        | ":wat::string::to-lowercase"
        | ":wat::string::to-uppercase"
        // Arc 209 naming-conversion — pascal->kebab is on is_pure_total (the defservice macro
        // calls it at expand time to derive fn names). Arc 278 #16.2 — to-uppercase joins it:
        // serve-op-arms calls it at expand time to derive the `<OP>-MAX-REQUEST-BYTES` const
        // keyword from the kebab op name.
        // Arc 265 — pascal->kebab-in (namespace-scoped) is also on is_pure_total: the defservice
        // macro calls it at expand time to derive fn names using the namespace's declared acronyms.
        // Arc 293 S2 — kebab->pascal-in joins it: `defservice … :satisfies` derives the surface's
        // PascalCase Op/Reply variant names from the kebab :impls op names at expand time.
        | ":wat::string::pascal->kebab"
        | ":wat::string::pascal->kebab-in"
        | ":wat::string::kebab->pascal-in"
        | ":wat::string::split"
        | ":wat::string::join"
        | ":wat::core::String/empty?"

        // ── Type inspection (pure) ─────────────────────────────────────
        | ":wat::core::type"
        | ":wat::core::conforms?"
        | ":wat::core::subtype?"
        | ":wat::core::record?"

        // ── Control flow ──────────────────────────────────────────────
        | ":wat::core::if"
        | ":wat::core::match"
        | ":wat::core::let"
        | ":wat::core::do"
        | ":wat::core::fn"

        // ── ReadOutcome constructors (arc 170) ─────────────────────────
        // `read-string` became TOTAL, so every call site — including the handful INSIDE
        // program-body defmacros that parse at expand time (`wat/core.wat`'s interpolate
        // family) — now names these two heads. Constructing a variant of a `Purity::Pure`
        // enum is pure and total by construction: it allocates, it cannot diverge, and it
        // touches nothing outside its own fields. Their ABSENCE was the false-refusal this
        // list's own comment predicts ("a pure head missing from this list makes a stdlib
        // test RED. Add it here") — it fired as 2530 reds off one MalformedDefmacro root.
        | ":wat::core::ReadOutcome::Forms"
        | ":wat::core::ReadOutcome::Malformed"
        // …and reading the headline off the cause an expand-time Malformed arm caught.
        // `:wat::core::Error` is a Record-natured SURFACE (`wat/core.wat`), so this is a
        // plain field read on pure data — total, allocation-only, no effect.
        | ":wat::core::Error/message"

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
        // Stone 118.B4-0 — `nth` is now a Rust intrinsic (was a wat `defclause`, never a
        // candidate for this list). Legitimate now, for the same reason as its siblings above:
        // it reads no state, performs no effect, and its out-of-range raise is a deterministic
        // located abort — not disqualifying (see `i64::/`, admitted above for div-by-zero).
        | ":wat::core::nth"
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
        | ":wat::core::range"
        | ":wat::core::take"
        | ":wat::core::drop"
        | ":wat::core::reverse"
        | ":wat::core::sort'"  // rune:lint(retired-name) — live prime (arc 251 comparator-sort primitive); wat-level sort/sort-by wrap it
        | ":wat::core::find-last-index"

        // ── Arc 118.2a — the clojure-named lazy/eager HOF surface (wat/seq.wat).
        // `map`/`filter`/`take`/`drop` above now return Stream; these are the pure
        // eager materializers + reduce built over them. All pure/total (no IO, no
        // randomness, no channels) — safe at macro-expansion time. Needed by
        // `:wat::rete::defrule` (arc 278.5) and other program-body macros that
        // consume the now-lazy HOFs.
        | ":wat::core::mapv"
        | ":wat::core::filterv"
        | ":wat::core::into"
        | ":wat::core::doall"
        | ":wat::core::dorun"
        | ":wat::core::reduce"
        | ":wat::core::count"
        | ":wat::core::stream->vec"
        | ":wat::core::stream->pvec"
        | ":wat::core::reduce-stream"

        // ── Arc 118.2a — `:wat::stream::*` primitives (type + cons/lazy/empty).
        // The `filter` defclause + the new materializers above are built on these;
        // a program-body macro composing a lazy pipeline directly needs them too.
        | ":wat::stream::cons"
        | ":wat::stream::lazy"
        | ":wat::stream::empty"

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

        | ":wat::core::struct->form"
        | ":wat::core::forms"
        | ":wat::core::show"
        // Arc 279 — unquoted display: String→itself, i64/f64/bool→digits.
        // The format macro emits (:wat::core::str <val>) per placeholder at
        // runtime. It is NOT called at expand time, but listed here so any
        // future macro that needs unquoted display in its own program-body
        // can use it. Pure: deterministic, no IO.
        | ":wat::core::str"

        // ── Homoiconic WatAST bridge (arc 251.5a; pure-total node walk/build) ──
        // The read→walk→rebuild→write spine: parse, decompose, kind-preserving
        // rebuild, serialize — all deterministic, errors-as-values, no IO. The fence
        // is default-DENY; these pure-total ops were minted (arc 251.5a) AFTER the
        // arc-249 whitelist was written, so they were simply never added — not because
        // they are impure (they are not; same category as the holon AST ops above),
        // but because no defmacro needed to walk a binder Vector node until now (arc
        // 209 Stone C.1's defservice is the first). Admitting them unblocks every
        // node-walking defmacro, the homoiconic point of having the tooling at all.
        | ":wat::core::read-string"
        | ":wat::core::write-forms"
        // Arc 278 Stone 2 — the sift Predicate's `sieve-pred` capture macro calls
        // `ast->source` at expand time (captures the user's `(fn …)` form, prints it
        // verbatim into the `Sieve::Predicate` String field). Pure ∧ deterministic —
        // same category as its siblings (write-forms/ast-name/ast->children) above;
        // simply never added when Stone 1 minted it (grounded gap, brief-flagged).
        | ":wat::core::ast->source"
        | ":wat::core::ast->children"
        | ":wat::core::with-children"
        | ":wat::core::ast-kind"
        | ":wat::core::ast-name"
        | ":wat::core::ast-span"
        | ":wat::core::ast-end-span"
        // Arc 109 β-ii-c — same category as its ast-* siblings just above (pure,
        // total, structural node walk; no IO): "which of these type-param name
        // nodes appear anywhere in this AST?" `defservice` calls it at expand
        // time to compute each generated companion type's OWN param subset
        // instead of stamping the service's full param list onto every one.
        | ":wat::core::type-params-used-in"
        // Arc 109 stone (`BRIEF-STONE-type-equal-the-missing-door.md`) — the missing door:
        // `defservice` and its siblings live entirely in macro bodies and had no way to ask
        // "are these two DECLARED type spellings the same type?" — `TypeExpr` derives `Eq` in
        // Rust, at check time, but a macro body could not reach it and fell back to rendering
        // both sides to strings and comparing text (the defect that blocked ②-iii's `:peers`
        // migration). Pure ∧ deterministic ∧ total structural comparison via `parse_type_node`
        // (raises on a non-type node rather than returning a silently-wrong `false`); same
        // category as `type-params-used-in` immediately above. F5 is default-deny, so an
        // intrinsic minted FOR a macro body but missing here is refused at DEFINITION.
        | ":wat::core::type-equal?"
        | ":wat::core::symbol-node"
        // Arc 274.1 — capture-proof binder for program-body macros. A macro needs it to create
        // scoped symbols that cannot collide with caller variables. "Does a macro need it?" → YES.
        | ":wat::core::fresh-symbol"
        | ":wat::core::keyword-node"
        | ":wat::core::keyword/to-symbol"
        | ":wat::core::keyword/to-type-form"
        // Arc 109 Stone ②-i — Colon-mode sibling of keyword/to-type-form (Room 3); same
        // pure/deterministic category, just the rust-ish `:wat::core::` head spelling.
        | ":wat::core::keyword/to-type-form-colon"

        // ── Form-shape predicates (pure over WatAST form-values) ──────
        // core form-shape predicate over WatAST::List; distinct from
        // :wat::holon::is-List? (a classifier over HolonAST). The name
        // diverges on purpose — the form-vs-holon distinction is the
        // reason this exists. Do not "harmonize" the two names.
        | ":wat::core::List?"

        // ── Runtime reflection (pure read-only; no IO, no side effects) ─
        // Shared infra for type-driven macros to reflect on fn signatures
        // at macro expand time (arc 249 stone 249.2b-i; originating
        // consumer was arc 170 D2's `run-threads` macro, since retired —
        // this allow-list entry is NOT run-threads-specific).
        // signature-of-fn: fn → HolonAST (pure; reads from fn value, no IO)
        // extract-arg-names: HolonAST → Vector<Keyword> (pure; structural walk)
        // extract-arg-types: HolonAST → Vector<Keyword> (pure; structural walk)
        | ":wat::runtime::signature-of-fn"
        | ":wat::runtime::extract-arg-names"
        | ":wat::runtime::extract-arg-types"

        // Arc 170 Strike B — field-names-of / field-types-of: type-kw → the
        // frozen runtime type registry (`sym.types`, an `Option<Arc<TypeEnv>>`
        // populated once at freeze time) → AggregateDef.fields. Same category
        // as signature-of-fn immediately above (read-only reflection off
        // already-frozen registry state, no IO, no mutation, deterministic —
        // see eval_field_names_of / eval_field_types_of, runtime.rs:11593-11662).
        // Arg resolution mirrors the arc-166 eval_lookup_define pattern (a
        // literal Keyword is read directly, not through eval_inner) — no
        // fn-literal special-casing needed (the sole arg is a type keyword,
        // never a `fn` form), unlike signature-of-fn above.
        | ":wat::runtime::field-names-of"
        | ":wat::runtime::field-types-of"
    )
}
