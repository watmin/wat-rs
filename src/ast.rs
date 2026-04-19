//! `WatAST` — the language-surface AST the parser produces.
//!
//! Distinct from `holon::HolonAST`. `WatAST` represents everything the
//! s-expression grammar admits at parse time: literals, keyword-path
//! tokens, bare symbols, parenthesized forms. Classification into higher
//! forms (`Define`, `Lambda`, `Struct`, `UpperCall`, macro invocations,
//! …) happens at later passes (macro-expansion, name-resolution,
//! lowering) dispatching on the head of a `List` whose first element is
//! a `Keyword`.
//!
//! Standard Lisp parser discipline: parse to a uniform tree; interpret
//! structure at semantic passes, not at lex/parse time.
//!
//! Source positions (spans) are not carried on this MVP version.
//! Added in a follow-up refinement when error messages need them.

/// The parsed source tree. One variant per terminal kind plus a `List`
/// variant for any parenthesized form.
#[derive(Debug, Clone, PartialEq)]
pub enum WatAST {
    /// Integer literal, as in `42`, `-1`, `0`. Fits in `i64`.
    IntLit(i64),

    /// Floating-point literal, as in `3.14`, `-0.5`, `1e10`.
    FloatLit(f64),

    /// Boolean literal, as in `true` or `false`.
    BoolLit(bool),

    /// String literal, as in `"hello"` — quotes stripped, escape sequences
    /// applied.
    StringLit(String),

    /// Keyword token, as in `:foo`, `:wat/algebra/Atom`,
    /// `:List<Holon>`, `:fn(T,U)->R`. The leading `:` is part of the
    /// stored string. Used both as keyword literals (payloads for wat
    /// keyword atoms) and as keyword-path references (heads of calls,
    /// type annotations). Distinguished by context at later passes.
    ///
    /// The spec's colon-quoting rule (`:Atom<Holon>` legal,
    /// `:Atom<:Holon>` illegal) is enforced at the lexer — an internal
    /// `:` produces a lex error before this node can carry it.
    Keyword(String),

    /// Bare identifier, as in `x`, `role`, `tmp`. Used in `let` bindings,
    /// `lambda` parameter names, `match` patterns — the only places the
    /// language admits bare names. Keyword paths always carry the
    /// leading `:`; anything without one is a `Symbol`.
    Symbol(String),

    /// Parenthesized form `(head arg1 arg2 ...)`. Also covers
    /// empty list `()`. The first child is typically the head —
    /// a `Keyword` for language or algebra calls, a `Symbol` for
    /// bare-scoped lambda/let invocation.
    List(Vec<WatAST>),
}
