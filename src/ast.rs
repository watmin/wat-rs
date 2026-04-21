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
//! # Hygiene
//!
//! `Symbol` carries an [`Identifier`](crate::identifier::Identifier) —
//! a (name, scope-set) pair that lets lexical-scope lookups distinguish
//! `tmp` the user wrote from `tmp` a macro introduced. Fresh-parsed
//! symbols have empty scope sets; macro expansion (slice 5c) adds
//! scopes per Racket's sets-of-scopes model. Keywords (full paths)
//! carry no scope tracking — hygiene only matters for bare names.

use crate::identifier::Identifier;

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

    /// Keyword token, as in `:foo`, `:wat::algebra::Atom`,
    /// `:Vec<holon::HolonAST>`, `:fn(T,U)->R`. The leading `:` is part of the
    /// stored string. Used both as keyword literals (payloads for wat
    /// keyword atoms) and as keyword-path references (heads of calls,
    /// type annotations). Distinguished by context at later passes.
    ///
    /// Keywords carry no scope tracking — their full-path spelling
    /// already disambiguates `:my::app::foo` from `:my::macro::foo`.
    Keyword(String),

    /// Bare identifier, as in `x`, `role`, `tmp`. Used in `let` bindings,
    /// `lambda` parameter names, `match` patterns — the only places the
    /// language admits bare names. The `Identifier` carries a scope
    /// set for macro hygiene (empty on fresh parse).
    Symbol(Identifier),

    /// Parenthesized form `(head arg1 arg2 ...)`. Also covers
    /// empty list `()`. The first child is typically the head —
    /// a `Keyword` for language or algebra calls, a `Symbol` for
    /// bare-scoped lambda/let invocation.
    List(Vec<WatAST>),
}

/// Serialize a [`WatAST`] as parseable source text.
///
/// The output is wat source the lexer + parser can round-trip to an
/// equivalent `WatAST`. Used by cross-process consumers that need a
/// textual form — notably
/// `:wat::kernel::run-sandboxed-hermetic-ast`, which writes the
/// serialized source to a tempfile the subprocess reads and parses.
///
/// Scope sets on `Symbol` are NOT preserved — serialization writes the
/// bare identifier name. Hygiene is an in-process concept; a fresh
/// process parses fresh and rebuilds scope sets from its own macro
/// pass. Programs that depend on cross-process scope identity cannot
/// be serialized losslessly and should stay in-process.
///
/// `FloatLit` uses `{:?}` formatting so integral floats round-trip
/// (`3.0` stays `3.0`, not `3`). `StringLit` re-escapes `\`, `"`, and
/// common control chars.
///
/// One top-level form per call. For multi-form programs, see
/// [`wat_ast_program_to_source`].
pub fn wat_ast_to_source(ast: &WatAST) -> String {
    let mut out = String::new();
    write_wat_ast(ast, &mut out);
    out
}

/// Serialize a sequence of top-level forms as a single program.
/// Each form is written via [`wat_ast_to_source`]; forms are joined
/// with newlines. Ready for `parse_all` on the receiving end.
pub fn wat_ast_program_to_source(forms: &[WatAST]) -> String {
    forms
        .iter()
        .map(wat_ast_to_source)
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_wat_ast(ast: &WatAST, out: &mut String) {
    match ast {
        WatAST::IntLit(n) => out.push_str(&n.to_string()),
        WatAST::FloatLit(x) => {
            // `{:?}` keeps the decimal point for integral floats —
            // `3.0` serializes as `3.0`, which parses back as FloatLit.
            // `{}` would emit `3`, which parses as IntLit and would
            // round-trip to a different variant.
            out.push_str(&format!("{:?}", x));
        }
        WatAST::BoolLit(b) => out.push_str(if *b { "true" } else { "false" }),
        WatAST::StringLit(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    other => out.push(other),
                }
            }
            out.push('"');
        }
        WatAST::Keyword(k) => out.push_str(k),
        WatAST::Symbol(ident) => out.push_str(&ident.name),
        WatAST::List(items) => {
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_wat_ast(item, out);
            }
            out.push(')');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_one;

    fn roundtrip(src: &str) -> String {
        let ast = parse_one(src).expect("parse");
        wat_ast_to_source(&ast)
    }

    #[test]
    fn roundtrip_int() {
        assert_eq!(roundtrip("42"), "42");
        assert_eq!(roundtrip("-1"), "-1");
        assert_eq!(roundtrip("0"), "0");
    }

    #[test]
    fn roundtrip_float_keeps_decimal_on_integral() {
        assert_eq!(roundtrip("3.14"), "3.14");
        // `3.0` MUST stay `3.0`; parsing `3` would be IntLit.
        assert_eq!(roundtrip("3.0"), "3.0");
        assert_eq!(roundtrip("-0.5"), "-0.5");
    }

    #[test]
    fn roundtrip_bool_and_keyword() {
        assert_eq!(roundtrip("true"), "true");
        assert_eq!(roundtrip("false"), "false");
        assert_eq!(roundtrip(":wat::core::define"), ":wat::core::define");
        assert_eq!(roundtrip(":Vec<holon::HolonAST>"), ":Vec<holon::HolonAST>");
    }

    #[test]
    fn roundtrip_string_with_escapes() {
        assert_eq!(roundtrip(r#""hello""#), r#""hello""#);
        assert_eq!(roundtrip(r#""with \"quotes\"""#), r#""with \"quotes\"""#);
        assert_eq!(roundtrip(r#""back\\slash""#), r#""back\\slash""#);
        assert_eq!(roundtrip(r#""line\none""#), r#""line\none""#);
    }

    #[test]
    fn roundtrip_list_nested() {
        assert_eq!(
            roundtrip("(:wat::core::i64::+ 1 (:wat::core::i64::* 2 3))"),
            "(:wat::core::i64::+ 1 (:wat::core::i64::* 2 3))"
        );
    }

    #[test]
    fn roundtrip_empty_list() {
        assert_eq!(roundtrip("()"), "()");
    }

    #[test]
    fn roundtrip_parse_idempotent() {
        // Serialize → parse → serialize must be stable.
        let src = r##"(:wat::core::define (:my::add (x :i64) (y :i64) -> :i64) (:wat::core::i64::+ x y))"##;
        let ast1 = parse_one(src).expect("parse 1");
        let s1 = wat_ast_to_source(&ast1);
        let ast2 = parse_one(&s1).expect("parse 2");
        let s2 = wat_ast_to_source(&ast2);
        assert_eq!(s1, s2);
    }

    #[test]
    fn program_to_source_joins_with_newlines() {
        let a = parse_one("(:a 1)").expect("parse a");
        let b = parse_one("(:b 2)").expect("parse b");
        let joined = wat_ast_program_to_source(&[a, b]);
        assert_eq!(joined, "(:a 1)\n(:b 2)");
    }
}
