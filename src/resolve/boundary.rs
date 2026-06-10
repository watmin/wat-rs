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
//!
//! Both passes ask the same question — "what is the argument-evaluation shape of
//! this head?" — and historically each answered it with its own `if`-chain. The
//! chains **drifted** (arc 251.1 ward): `normalize` silently lacked the
//! `match`/`cond`/`matches?` boundaries that `walk` carried, so the two encodings
//! of one language fact diverged. This module is the decomplected answer: one
//! [`Boundary`] classification, consulted by both. The traversal itself stays in
//! each pass (one borrows the tree to push errors, the other consumes it to
//! rebuild) — but the *classification* lives here exactly once.
//!
//! The payoff is structural, not stylistic: both passes match on [`Boundary`]
//! **exhaustively**, so adding a future special-form boundary is a compile error
//! in every pass until it is handled. Drift becomes unrepresentable.

/// The argument-evaluation shape of a special-form list head.
///
/// Each variant names which child regions are live **code** (to be resolved /
/// rewritten) and which are **data** (to be left untouched). The doc on each
/// variant is the single specification of that form's boundary; the passes
/// implement the traversal, but neither re-decides the classification.
pub(super) enum Boundary {
    /// `:wat::core::quote` / `:wat::core::forms` / `:wat::core::define` — every
    /// argument is captured as data; no child is code. (`define` is retired at
    /// the checker, but the resolver still must not walk its body.)
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
    /// Not a special-form boundary — every child is ordinary live code.
    Ordinary,
}

/// Classify a list head's argument-evaluation boundary shape.
///
/// This is the ONE place the boundary-head set is encoded. Both the call-head
/// resolution walk and the symbol-ref normalization pass route through it.
pub(super) fn quote_boundary(head: &str) -> Boundary {
    match head {
        ":wat::core::quote" | ":wat::core::forms" | ":wat::core::define" => Boundary::AllData,
        ":wat::core::quasiquote" => Boundary::Quasiquote,
        ":wat::form::matches?" => Boundary::MatchesSubject,
        ":wat::core::match" => Boundary::Match,
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
pub(super) fn is_unquote_escape(head: &str) -> bool {
    head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing"
}
