//! Special-form argument-evaluation boundaries — the single source of truth.
//!
//! Several name-resolution passes must agree on WHICH list heads capture some
//! of their arguments as **data** (patterns, quoted forms, DSL markers) rather
//! than evaluating them as live **code**:
//!
//! - [`super::walk::check_form`] resolves call heads — it must not walk a quoted
//!   form's arguments or a `match` arm's pattern as if they were calls.
//! - [`super::normalize`] rewrites namespaced symbol refs (`wat.core/+` →
//!   `:wat::core::+`) — it must not rewrite a symbol sitting in a data position.
//! - [`crate::macros::expand::expand_form`] (arc 294 item 9a) — full-Lisp macro
//!   expansion recurses into every non-macro form's children; it must not treat a
//!   `matches?` pattern's aggregate-shaped head (e.g. `:test::PaperResolved`, now a
//!   kwargs companion macro post-flip) as a macro CALL, feeding raw DSL clauses to
//!   `kwargs-lower` as if they were kv-pairs.
//!
//! These passes ask the same question — "what is the argument-evaluation shape of
//! this head?" — and historically each answered it with its own `if`-chain. The
//! chains **drifted** (arc 251.1 ward): `normalize` silently lacked the
//! `match`/`cond`/`matches?` boundaries that `walk` carried, so the two encodings
//! of one language fact diverged. This module is the decomplected answer: one
//! [`Boundary`] classification, consulted by all three. The traversal itself stays
//! in each pass (one borrows the tree to push errors, one consumes it to rebuild,
//! one consumes it to macro-expand) — but the *classification* lives here exactly
//! once.
//!
//! The payoff is structural, not stylistic: `walk` and `normalize` match on
//! [`Boundary`] **exhaustively**, so adding a future special-form boundary is a
//! compile error in both until it is handled. `expand_form` consults only the
//! [`Boundary::MatchesSubject`] variant it needs (arc 294 item 9a) rather than
//! matching exhaustively — its quote/quasiquote/literal data-forms are already
//! handled by an earlier, differently-scoped check (they return the whole form
//! unexpanded, including any nested unquote-escapes a template caller means to
//! observe rather than execute; see `expand_form`'s doc), so folding them into
//! this same classification is future work, not this fix's scope.

/// The argument-evaluation shape of a special-form list head.
///
/// Each variant names which child regions are live **code** (to be resolved /
/// rewritten) and which are **data** (to be left untouched). The doc on each
/// variant is the single specification of that form's boundary; the passes
/// implement the traversal, but neither re-decides the classification.
pub(crate) enum Boundary {
    /// `:wat::core::quote` / `:wat::core::forms` / `:wat::holon::literal` — every
    /// argument is captured as data; no child is code.
    AllData,
    /// `:wat::core::quasiquote` — the template argument (`items[1]`) is data
    /// EXCEPT inside `:wat::core::unquote` / `:wat::core::unquote-splicing`
    /// escapes, which are live code.
    Quasiquote,
    /// `:wat::form::matches?` — only the subject (`items[1]`) is code; the
    /// pattern (`items[2..]`) is DSL data owned by the matches? grammar walker.
    MatchesSubject,
    /// `:wat::core::match` — the scrutinee (`items[1]`) and each arm body are
    /// code; the `-> :T` return annotation (`items[2..=3]`) and each arm's
    /// pattern are data.
    Match,
    /// `:wat::rete::make-rule` (arc 278 task #78 —
    /// DESIGN-STONE-where-bodies-expand-at-compile-time.md). `items[1]` (the
    /// rule name) is ordinary code. `items[2]` (the quoted `:when` vector) is
    /// DATA, EXCEPT the body of each `(:wat::rete::where …)` form inside it,
    /// which is CODE. `items[3]` (the quoted `:then` vector) is data — the RHS
    /// is a separate question (task #61: derived fact fields are copies only).
    ///
    /// `make-rule`, not `defrule`: a census of rule producers
    /// (`defrule`'s template, `sift-rules-defsvc`'s generator, hand-built rule
    /// literals, and direct `make-rule` calls) found `make-rule` is the one
    /// door all four funnel through; hooking `defrule` alone would silently
    /// miss the other three.
    MakeRule,
    /// Not a special-form boundary — every child is ordinary live code.
    Ordinary,
}

/// Classify a list head's argument-evaluation boundary shape.
///
/// This is the ONE place the boundary-head set is encoded. Both the call-head
/// resolution walk and the symbol-ref normalization pass route through it.
pub(crate) fn quote_boundary(head: &str) -> Boundary {
    match head {
        // Arc 294.b — body is data (same as quote); no symbol resolution inside.
        ":wat::core::quote" | ":wat::core::forms" | ":wat::holon::literal" => Boundary::AllData,
        ":wat::core::quasiquote" => Boundary::Quasiquote,
        ":wat::form::matches?" => Boundary::MatchesSubject,
        ":wat::core::match" => Boundary::Match,
        ":wat::rete::make-rule" => Boundary::MakeRule,
        _ => Boundary::Ordinary,
    }
}

/// True if `head` is a quasiquote escape — the one place inside a
/// [`Boundary::Quasiquote`] template where data gives way to live code.
///
/// The escape-head set is a language fact, encoded here exactly once so the two
/// quasiquote-template descents (`super::quote::check_quasiquote_template` —
/// borrow + resolve; `super::normalize::normalize_quasiquote_template` — consume
/// + rewrite) cannot drift on which heads open an escape.
///
/// Arc 278 — widened `pub(super)` → `pub(crate)`: `crate::closure_extract`'s free-symbol
/// walk is a THIRD quasiquote-template descent and must use the same language fact, or it
/// re-creates the drift this function exists to prevent.
pub(crate) fn is_unquote_escape(head: &str) -> bool {
    head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing"
}

/// True if `head` is the rete `where` head — the one place inside a
/// [`Boundary::MakeRule`] call's quoted `:when` vector where a condition's DATA
/// gives way to a live-code BODY (arc 278 task #78).
///
/// Encoded here exactly once so the three `make-rule` descents (`super::walk`'s
/// `check_make_rule_when`, `super::normalize`'s `normalize_make_rule_when`, and
/// `crate::macros::expand::expand_make_rule_when`) cannot drift on which head
/// opens the code region — the same discipline [`is_unquote_escape`] gives the
/// quasiquote escape set.
///
/// `pub(crate)`, not `pub(super)`: unlike the quasiquote escape (used only
/// inside `resolve`), the macro expander (`crate::macros::expand`) is a
/// different module and must consult this too — the whole point is a `where`
/// body is finally macro-expanded, not just resolved/normalized.
pub(crate) fn is_where_form(head: &str) -> bool {
    head == ":wat::rete::where"
}
