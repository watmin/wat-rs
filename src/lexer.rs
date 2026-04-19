//! S-expression lexer — text → tokens.
//!
//! Handles keyword-path tokens (e.g. `:wat/algebra/Atom`), the colon-quoting
//! rule (`:Atom<Holon>` legal, `:Atom<:Holon>` illegal), string / numeric /
//! bool literals, parens/brackets, comments, quote / unquote / unquote-splicing.
//!
//! This module is a stub until the lexer task lands.
