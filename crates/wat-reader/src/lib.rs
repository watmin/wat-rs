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

// Arc 296 stone A: real cross-crate derive guard.
//
// Proves the REAL `ToEdn` derive works from `wat-reader` (a crate that does
// NOT depend on `wat` or `wat-macros`). The cycle-break the stone exists for:
// wat-reader → wat-edn → wat-to-edn-derive (acyclic). This test persists as
// the guard until stone B makes Span itself a derive-tagged record.
#[cfg(test)]
mod real_derive_cross_crate {
    /// Minimal struct that derives `wat_edn::ToEdn` from `wat-reader`.
    /// No namespace attr → defaults to `"wat.kernel"` (the back-compat default).
    #[derive(wat_edn::ToEdn)]
    #[allow(dead_code)]
    struct ReaderProbe {
        name: String,
        line: i64,
    }

    #[test]
    fn real_derive_works_cross_crate_no_cycle() {
        use wat_edn::ToEdn;
        let v = ReaderProbe { name: "x".to_owned(), line: 7 }.to_edn();
        let edn = wat_edn::write(&v);
        assert_eq!(
            edn,
            r#"#wat.kernel/ReaderProbe {:name "x" :line 7}"#,
            "the real ToEdn derive emits a correct tagged record from wat-reader"
        );
    }
}
