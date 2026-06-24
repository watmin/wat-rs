//! `wat-reader` — the wat reader front-end.
//!
//! A closed, std-only set: span + identifier + AST + lexer + parser.
//! Depended on by both the main `wat` crate and `wat-macros` (proc-macro)
//! so discovery can use the REAL parser. Extracted so the hand-rolled
//! lexer in `wat-macros/src/discover.rs` can be eliminated.

pub mod span;
pub mod identifier;
pub mod ast;
pub mod lexer;
pub mod parser;

// Convenience re-exports at crate root
pub use ast::WatAST;
pub use identifier::{fresh_scope, Identifier, ScopeId};
pub use parser::{parse_all_with_file, parse_one_with_file, ParseError, ParseErrorKind};
pub use span::Span;
