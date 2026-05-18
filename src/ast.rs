//! `WatAST` — the language-surface AST the parser produces.
//!
//! Distinct from `wat::holon::HolonAST`. `WatAST` represents everything the
//! s-expression grammar admits at parse time: literals, keyword-path
//! tokens, bare symbols, parenthesized forms. Classification into higher
//! forms (`Define`, `Fn`, `Struct`, `UpperCall`, macro invocations,
//! …) happens at later passes (macro-expansion, name-resolution,
//! lowering) dispatching on the head of a `List` whose first element is
//! a `Keyword`.
//!
//! Standard Lisp parser discipline: parse to a uniform tree; interpret
//! structure at semantic passes, not at lex/parse time.
//!
//! # Hygiene
//!
//! `Symbol` carries an [`Identifier`](crate::identifier::Identifier) —
//! a (name, scope-set) pair that lets lexical-scope lookups distinguish
//! `tmp` the user wrote from `tmp` a macro introduced. Fresh-parsed
//! symbols have empty scope sets; macro expansion (slice 5c) adds
//! scopes per Racket's sets-of-scopes model. Keywords (full paths)
//! carry no scope tracking — hygiene only matters for bare names.

use crate::identifier::Identifier;
use crate::span::Span;

/// The parsed source tree. One variant per terminal kind plus a `List`
/// variant for any parenthesized form.
///
/// Every variant carries a trailing [`Span`] with the source location
/// the node was parsed from. Span comparison is structural-transparent
/// (see [`crate::span`] module docs) — two nodes with the same
/// structure but different spans compare equal and hash identically.
#[derive(Debug, Clone, PartialEq)]
pub enum WatAST {
    /// Integer literal, as in `42`, `-1`, `0`. Fits in `i64`.
    IntLit(i64, Span),

    /// Floating-point literal, as in `3.14`, `-0.5`, `1e10`.
    FloatLit(f64, Span),

    /// Boolean literal, as in `true` or `false`.
    BoolLit(bool, Span),

    /// String literal, as in `"hello"` — quotes stripped, escape sequences
    /// applied.
    StringLit(String, Span),

    /// Keyword token, as in `:foo`, `:wat::holon::Atom`,
    /// `:wat::holon::Holons`, `:fn(T,U)->R`. The leading `:` is part of the
    /// stored string. Used both as keyword literals (payloads for wat
    /// keyword atoms) and as keyword-path references (heads of calls,
    /// type annotations). Distinguished by context at later passes.
    ///
    /// Keywords carry no scope tracking — their full-path spelling
    /// already disambiguates `:my::app::foo` from `:my::macro::foo`.
    Keyword(String, Span),

    /// Bare identifier, as in `x`, `role`, `tmp`. Used in `let` bindings,
    /// `fn` parameter names, `match` patterns — the only places the
    /// language admits bare names. The `Identifier` carries a scope
    /// set for macro hygiene (empty on fresh parse).
    Symbol(Identifier, Span),

    /// Parenthesized form `(head arg1 arg2 ...)`. Also covers
    /// empty list `()`. The first child is typically the head —
    /// a `Keyword` for language or algebra calls, a `Symbol` for
    /// bare-scoped fn/let invocation.
    List(Vec<WatAST>, Span),

    /// Bracketed form `[a b c ...]`. Also covers empty vector
    /// `[]`. Distinct from `List` at the AST level so consumers
    /// (slice 2's fn / defn signature parser; slice ≥3's let
    /// binding-block parser) can syntactically distinguish a
    /// vector from a list.
    ///
    /// Arc 167 slice 1 (additive substrate). Vectors are admitted
    /// only in **binding-syntax positions**; appearing at value
    /// position errors at eval/check time. The legal-position
    /// consumers are wired in slice 2 (`fn` / `defn` signatures)
    /// and arc 168 (`let`).
    Vector(Vec<WatAST>, Span),

    /// Braced form `{a b c ...}` — the struct-destructure binder
    /// shape. Each child is a bare `Symbol` that is BOTH the
    /// field-name (looked up against the struct type of the
    /// adjacent expression) AND the local binding-name in the
    /// enclosing let scope.
    ///
    /// Arc 169 slice 1 (additive substrate). Admitted only in
    /// `:wat::core::let` binding-position alongside a struct-typed
    /// expression; appearing anywhere else errors at parse / check
    /// time. The 12-word semantic rule: *bind the field's value to
    /// the field's name in this scope*.
    ///
    /// Empty `{}` and non-Symbol contents are rejected at PARSE
    /// time. Field-name validation against a registered struct's
    /// fields is the consumer's job at check / runtime — the
    /// parser does not consult any type registry.
    StructPattern(Vec<WatAST>, Span),
}

impl WatAST {
    /// Borrow the span this node was parsed from.
    pub fn span(&self) -> &Span {
        match self {
            WatAST::IntLit(_, s)
            | WatAST::FloatLit(_, s)
            | WatAST::BoolLit(_, s)
            | WatAST::StringLit(_, s)
            | WatAST::Keyword(_, s)
            | WatAST::Symbol(_, s)
            | WatAST::List(_, s)
            | WatAST::Vector(_, s)
            | WatAST::StructPattern(_, s) => s,
        }
    }

    /// Convenience constructors with [`Span::unknown`] — for
    /// synthetic forms / tests / runtime-constructed ASTs.
    pub fn int(n: i64) -> Self {
        WatAST::IntLit(n, Span::unknown())
    }
    pub fn float(x: f64) -> Self {
        WatAST::FloatLit(x, Span::unknown())
    }
    pub fn bool(b: bool) -> Self {
        WatAST::BoolLit(b, Span::unknown())
    }
    pub fn string(s: impl Into<String>) -> Self {
        WatAST::StringLit(s.into(), Span::unknown())
    }
    pub fn keyword(k: impl Into<String>) -> Self {
        WatAST::Keyword(k.into(), Span::unknown())
    }
    pub fn symbol(ident: Identifier) -> Self {
        WatAST::Symbol(ident, Span::unknown())
    }
    pub fn list(items: Vec<WatAST>) -> Self {
        WatAST::List(items, Span::unknown())
    }
    /// Synthetic Vector with [`Span::unknown`] — for tests and
    /// runtime-constructed bracketed forms.
    pub fn vector(items: Vec<WatAST>) -> Self {
        WatAST::Vector(items, Span::unknown())
    }
    /// Synthetic StructPattern with [`Span::unknown`] — for tests
    /// and runtime-constructed brace forms. Arc 169 slice 1.
    pub fn struct_pattern(items: Vec<WatAST>) -> Self {
        WatAST::StructPattern(items, Span::unknown())
    }

    /// The children of this AST node. Compound shapes return their
    /// `items`; leaves return an empty slice.
    ///
    /// Arc 212 (failure engineering applied at the walker layer).
    /// Walkers that recurse generically through the AST MUST use
    /// this method rather than pattern-matching on `WatAST::List`
    /// specifically. The motivation:
    ///
    /// - When `WatAST::Vector` was added in arc 167 slice 1, the
    ///   macro expand-time walker (`macros.rs::walk_template`) was
    ///   correctly updated to recurse into Vector children. The
    ///   runtime quasiquote walker (`runtime.rs::walk_quasiquote`)
    ///   was NOT updated; it skipped Vectors silently. Latent
    ///   substrate flaw for ~50 arcs; surfaced via t6 in arc 212.
    ///
    /// - The fix was tempting to ship per-walker (add a Vector arm
    ///   to each function that was missing one). But that produces
    ///   N copies of the same logic and N opportunities for the
    ///   same bug when the next AST variant lands. The honest fix
    ///   is at the substrate layer: own "what are the children of
    ///   an AST node?" here, so walkers can't get it wrong.
    ///
    /// When a NEW compound `WatAST` variant lands, update this
    /// method's match arm to include it. Every walker that descends
    /// via `children()` automatically benefits without per-walker
    /// audit.
    ///
    /// **The bug class is structurally eliminated** — failure
    /// engineering at the walker layer.
    pub fn children(&self) -> &[WatAST] {
        match self {
            WatAST::List(items, _)
            | WatAST::Vector(items, _)
            | WatAST::StructPattern(items, _) => items,
            _ => &[],
        }
    }
}

// wat_ast_to_source / wat_ast_program_to_source — RETIRED in arc
// 012 slice 3 (the task-#269 commit). Added in arc 011 to bridge
// the AST → source → subprocess boundary of the old hermetic-ast
// primitive. With fork-program-ast, the child inherits AST in
// memory via COW — no textual round-trip, no serializer needed.
// Zero remaining callers. If a future use case surfaces (pretty-
// printer, REPL history, or a :wat::core::ast-to-source stdlib
// primitive), reintroduce with that caller's concrete shape.
