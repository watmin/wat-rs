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

// PROBE (arc 296 stone A disconfirming probe) — proves a re-exported derive
// macro with a helper attribute resolves as `#[derive(wat_edn::…)]` from a
// dependent crate, with no dependency cycle (wat-reader → wat-edn →
// wat-to-edn-derive is acyclic). Deleted when the real ToEdn derive lands.
#[cfg(test)]
mod probe_arc296_stone_a {
    #[derive(wat_edn::ProbeToEdn)]
    #[to_edn(namespace = "probe")]
    #[allow(dead_code)]
    struct ProbeSpan {
        line: i64,
    }

    #[test]
    fn reexported_derive_resolves_from_a_dependent_with_no_cycle() {
        use wat_edn::ProbeToEdn;
        let v = ProbeSpan { line: 3 }.probe_to_edn();
        assert!(
            matches!(v, wat_edn::OwnedValue::Nil),
            "the re-exported derive emitted a working impl"
        );
    }
}
