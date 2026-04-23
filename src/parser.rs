//! S-expression parser — tokens → `WatAST`.
//!
//! Recursive descent over the s-expression grammar. Produces a uniform
//! `WatAST` tree: literals are their respective variants, keywords and
//! symbols are leaves, parenthesized forms are `List` nodes. Dispatch
//! on head keyword (`:wat::core::define`, `:wat::holon::...`, etc.)
//! happens at later passes, not here.
//!
//! Two entry points:
//! - [`parse_one`] — parse a single top-level form; errors if there's
//!   trailing content.
//! - [`parse_all`] — parse a sequence of top-level forms; errors on
//!   unclosed parens or unexpected closers.

use crate::ast::WatAST;
use crate::identifier::Identifier;
use crate::lexer::{lex, LexError, SpannedToken, Token};
use crate::span::Span;
use std::fmt;
use std::sync::Arc;

/// Parse error.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Lex failure — the input couldn't be tokenized.
    Lex(LexError),
    /// A `)` was found with no matching `(`.
    UnexpectedRParen,
    /// An opening `(` was never closed before end of input.
    UnclosedParen,
    /// `parse_one` expected exactly one form; got trailing content after
    /// the first complete form.
    TrailingContent,
    /// `parse_one` expected a form but the input was empty (all whitespace).
    Empty,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lex(e) => write!(f, "lex error: {}", e),
            ParseError::UnexpectedRParen => write!(f, "unexpected ')'"),
            ParseError::UnclosedParen => write!(f, "unclosed '('"),
            ParseError::TrailingContent => {
                write!(f, "trailing content after single top-level form")
            }
            ParseError::Empty => write!(f, "empty input — expected a form"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError::Lex(e)
    }
}

/// Parse the input as a single top-level `WatAST` form.
///
/// Errors if the input is empty, if it contains more than one top-level
/// form, or if any lex/parse rule is violated. Uses `<test>` as the
/// span file label — callers with a real path use
/// [`parse_one_with_file`].
pub fn parse_one(src: &str) -> Result<WatAST, ParseError> {
    parse_one_with_file(src, "<test>")
}

/// Parse the input as a sequence of top-level `WatAST` forms. Uses
/// `<test>` as the span file label — callers with a real path use
/// [`parse_all_with_file`].
pub fn parse_all(src: &str) -> Result<Vec<WatAST>, ParseError> {
    parse_all_with_file(src, "<test>")
}

/// [`parse_one`] with a span-label for diagnostics. Arc 016 slice 1.
pub fn parse_one_with_file(src: &str, file: &str) -> Result<WatAST, ParseError> {
    let file_arc = Arc::new(file.to_string());
    let tokens = lex(src, file_arc)?;
    let mut cursor = Cursor::new(&tokens);
    let node = match cursor.parse_form()? {
        Some(node) => node,
        None => return Err(ParseError::Empty),
    };
    if cursor.peek().is_some() {
        return Err(ParseError::TrailingContent);
    }
    Ok(node)
}

/// [`parse_all`] with a span-label for diagnostics. Arc 016 slice 1.
pub fn parse_all_with_file(src: &str, file: &str) -> Result<Vec<WatAST>, ParseError> {
    let file_arc = Arc::new(file.to_string());
    let tokens = lex(src, file_arc)?;
    let mut cursor = Cursor::new(&tokens);
    let mut out = Vec::new();
    while let Some(node) = cursor.parse_form()? {
        out.push(node);
    }
    Ok(out)
}

/// Internal token cursor.
struct Cursor<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(tokens: &'a [SpannedToken]) -> Self {
        Cursor { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&'a SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&'a SpannedToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// Parse one form. Returns `Ok(None)` if input is exhausted.
    /// Returns an error if the next token is an unexpected `)`.
    fn parse_form(&mut self) -> Result<Option<WatAST>, ParseError> {
        let st = match self.advance() {
            Some(t) => t,
            None => return Ok(None),
        };
        let span = st.span.clone();
        match &st.token {
            Token::LParen => {
                let list = self.parse_list_body()?;
                Ok(Some(WatAST::List(list, span)))
            }
            Token::RParen => Err(ParseError::UnexpectedRParen),
            Token::Int(n) => Ok(Some(WatAST::IntLit(*n, span))),
            Token::Float(x) => Ok(Some(WatAST::FloatLit(*x, span))),
            Token::Bool(b) => Ok(Some(WatAST::BoolLit(*b, span))),
            Token::Str(s) => Ok(Some(WatAST::StringLit(s.clone(), span))),
            Token::Keyword(k) => Ok(Some(WatAST::Keyword(k.clone(), span))),
            Token::Symbol(s) => Ok(Some(WatAST::Symbol(Identifier::bare(s.clone()), span))),
            Token::Quasiquote => self.parse_reader_macro(":wat::core::quasiquote", span),
            Token::Unquote => self.parse_reader_macro(":wat::core::unquote", span),
            Token::UnquoteSplicing => self.parse_reader_macro(":wat::core::unquote-splicing", span),
        }
    }

    /// A reader macro (`` ` `` / `,` / `,@`) wraps the following form.
    /// `` `X `` → `(:wat::core::quasiquote X)`, etc. The synthesized
    /// head-keyword and list inherit the reader macro's span; the
    /// inner form keeps its own.
    fn parse_reader_macro(
        &mut self,
        head_keyword: &str,
        span: Span,
    ) -> Result<Option<WatAST>, ParseError> {
        let inner = self.parse_form()?.ok_or(ParseError::Empty)?;
        Ok(Some(WatAST::List(
            vec![WatAST::Keyword(head_keyword.to_string(), span.clone()), inner],
            span,
        )))
    }

    /// Parse the body of a list — `(` already consumed. Accumulates child
    /// forms until the matching `)`.
    fn parse_list_body(&mut self) -> Result<Vec<WatAST>, ParseError> {
        let mut children = Vec::new();
        loop {
            match self.peek().map(|st| &st.token) {
                Some(Token::RParen) => {
                    self.advance();
                    return Ok(children);
                }
                Some(_) => match self.parse_form()? {
                    Some(child) => children.push(child),
                    None => unreachable!(
                        "parse_form returned None but peek() had a token"
                    ),
                },
                None => return Err(ParseError::UnclosedParen),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::WatAST;

    fn kw(s: &str) -> WatAST {
        WatAST::keyword(s.to_string())
    }
    fn sym(s: &str) -> WatAST {
        WatAST::symbol(Identifier::bare(s))
    }
    fn str_lit(s: &str) -> WatAST {
        WatAST::string(s.to_string())
    }
    fn list(items: Vec<WatAST>) -> WatAST {
        WatAST::list(items)
    }

    #[test]
    fn atom_literals() {
        // Tests rely on WatAST's structural PartialEq, which uses
        // Span::eq (always-true). Constructing expected with
        // Span::unknown() still matches the parser's real spans.
        assert_eq!(parse_one("42").unwrap(), WatAST::int(42));
        assert_eq!(parse_one("-1").unwrap(), WatAST::int(-1));
        assert_eq!(parse_one("2.5").unwrap(), WatAST::float(2.5));
        assert_eq!(parse_one("true").unwrap(), WatAST::bool(true));
        assert_eq!(parse_one("false").unwrap(), WatAST::bool(false));
        assert_eq!(parse_one("\"hello\"").unwrap(), str_lit("hello"));
        assert_eq!(parse_one(":foo").unwrap(), kw(":foo"));
        assert_eq!(parse_one("x").unwrap(), sym("x"));
    }

    #[test]
    fn empty_list() {
        assert_eq!(parse_one("()").unwrap(), list(vec![]));
    }

    #[test]
    fn simple_list() {
        assert_eq!(
            parse_one("(a b c)").unwrap(),
            list(vec![sym("a"), sym("b"), sym("c")])
        );
    }

    #[test]
    fn nested_list() {
        assert_eq!(
            parse_one("(a (b c) d)").unwrap(),
            list(vec![
                sym("a"),
                list(vec![sym("b"), sym("c")]),
                sym("d")
            ])
        );
    }

    #[test]
    fn algebra_core_atom() {
        assert_eq!(
            parse_one(r#"(:wat::holon::Atom "role")"#).unwrap(),
            list(vec![kw(":wat::holon::Atom"), str_lit("role")])
        );
    }

    #[test]
    fn algebra_core_bind_with_atoms() {
        // The MVP target shape.
        let src = r#"(:wat::holon::Bind (:wat::holon::Atom "role") (:wat::holon::Atom "filler"))"#;
        let expected = list(vec![
            kw(":wat::holon::Bind"),
            list(vec![kw(":wat::holon::Atom"), str_lit("role")]),
            list(vec![kw(":wat::holon::Atom"), str_lit("filler")]),
        ]);
        assert_eq!(parse_one(src).unwrap(), expected);
    }

    #[test]
    fn algebra_core_thermometer() {
        assert_eq!(
            parse_one("(:wat::holon::Thermometer 0.5 0.0 1.0)").unwrap(),
            list(vec![
                kw(":wat::holon::Thermometer"),
                WatAST::FloatLit(0.5, Span::unknown()),
                WatAST::FloatLit(0.0, Span::unknown()),
                WatAST::FloatLit(1.0, Span::unknown()),
            ])
        );
    }

    #[test]
    fn algebra_core_blend_negative_weight() {
        assert_eq!(
            parse_one("(:wat::holon::Blend a b 1 -1)").unwrap(),
            list(vec![
                kw(":wat::holon::Blend"),
                sym("a"),
                sym("b"),
                WatAST::IntLit(1, Span::unknown()),
                WatAST::IntLit(-1, Span::unknown()),
            ])
        );
    }

    #[test]
    fn define_signature_shape() {
        // Just verifying the shape survives parsing as a uniform List.
        // Dispatch to a Define node happens in a later pass.
        let src = "(:wat::core::define (:my::app::amplify (x :wat::holon::HolonAST) (y :wat::holon::HolonAST) (s :f64) -> :wat::holon::HolonAST) (:wat::holon::Blend x y 1 s))";
        let parsed = parse_one(src).unwrap();
        // First child must be the :wat::core::define keyword.
        if let WatAST::List(items, _) = &parsed {
            assert_eq!(items[0], kw(":wat::core::define"));
        } else {
            panic!("expected top-level List");
        }
    }

    #[test]
    fn parse_all_multiple_forms() {
        let forms = parse_all(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::core::load-file! "wat/holon/Subtract.wat")
            "#,
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn parse_all_ignores_comments_and_whitespace() {
        let forms = parse_all(
            r#"
            ;; comment
            42
            ;; another comment
            "hello"
            "#,
        )
        .unwrap();
        assert_eq!(forms, vec![WatAST::IntLit(42, Span::unknown()), str_lit("hello")]);
    }

    #[test]
    fn unexpected_rparen_at_start() {
        assert!(matches!(parse_one(")"), Err(ParseError::UnexpectedRParen)));
    }

    #[test]
    fn extra_rparen_after_complete_form_is_trailing() {
        // `(a))` — `(a)` parses fine; the extra `)` is trailing content.
        assert!(matches!(
            parse_one("(a))"),
            Err(ParseError::TrailingContent)
        ));
    }

    #[test]
    fn unexpected_rparen_inside_list() {
        // `(a ))` — inner ) closes the list; outer ) is then at top-level
        // via parse_all, which treats it as UnexpectedRParen.
        assert!(matches!(
            parse_all("(a)) foo"),
            Err(ParseError::UnexpectedRParen)
        ));
    }

    #[test]
    fn unclosed_paren() {
        assert!(matches!(parse_one("("), Err(ParseError::UnclosedParen)));
        assert!(matches!(parse_one("(a b"), Err(ParseError::UnclosedParen)));
        assert!(matches!(
            parse_one("(a (b)"),
            Err(ParseError::UnclosedParen)
        ));
    }

    #[test]
    fn empty_input_errors_in_parse_one() {
        assert!(matches!(parse_one(""), Err(ParseError::Empty)));
        assert!(matches!(parse_one("   "), Err(ParseError::Empty)));
        assert!(matches!(parse_one("; comment"), Err(ParseError::Empty)));
    }

    #[test]
    fn empty_input_ok_in_parse_all() {
        assert_eq!(parse_all("").unwrap(), vec![]);
        assert_eq!(parse_all("   ").unwrap(), vec![]);
    }

    #[test]
    fn trailing_content_rejected_by_parse_one() {
        assert!(matches!(
            parse_one("1 2"),
            Err(ParseError::TrailingContent)
        ));
    }

    #[test]
    fn lex_error_surfaces_as_parse_error() {
        // A lex error must surface as ParseError::Lex. Use the
        // unclosed-bracket-in-keyword error — whitespace inside an
        // unclosed `(` in a keyword body.
        let e = parse_one(":fn(T ").unwrap_err();
        assert!(matches!(e, ParseError::Lex(_)));
    }

    #[test]
    fn internal_colons_lex_as_single_keyword() {
        // Under the colon-quote model, `:` is the symbol-literal reader
        // macro — one leading `:` marks the start; internal `::` is
        // just the Rust path separator, pushed as body characters.
        assert_eq!(
            parse_one(":wat::core::load-file!").unwrap(),
            kw(":wat::core::load-file!")
        );
        assert_eq!(
            parse_one(":rust::crossbeam_channel::Sender<T>").unwrap(),
            kw(":rust::crossbeam_channel::Sender<T>")
        );
    }

    #[test]
    fn keyword_with_parens_inside() {
        // :fn(T,U)->R — internal parens must parse as a single keyword.
        assert_eq!(parse_one(":fn(T,U)->R").unwrap(), kw(":fn(T,U)->R"));
    }

    // ─── Quasiquote reader macros ───────────────────────────────────────

    #[test]
    fn quasiquote_wraps_following_form() {
        assert_eq!(
            parse_one("`foo").unwrap(),
            list(vec![kw(":wat::core::quasiquote"), sym("foo")])
        );
    }

    #[test]
    fn quasiquote_over_list() {
        // `(a b c) → (:wat::core::quasiquote (a b c))
        let expected = list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![sym("a"), sym("b"), sym("c")]),
        ]);
        assert_eq!(parse_one("`(a b c)").unwrap(), expected);
    }

    #[test]
    fn unquote_wraps_following_form() {
        assert_eq!(
            parse_one(",x").unwrap(),
            list(vec![kw(":wat::core::unquote"), sym("x")])
        );
    }

    #[test]
    fn unquote_splicing_wraps_following_form() {
        assert_eq!(
            parse_one(",@xs").unwrap(),
            list(vec![kw(":wat::core::unquote-splicing"), sym("xs")])
        );
    }

    #[test]
    fn quasiquote_with_unquote_inside() {
        // `(:wat::holon::Bind ,x ,y) — classic macro template shape.
        let expected = list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![
                kw(":wat::holon::Bind"),
                list(vec![kw(":wat::core::unquote"), sym("x")]),
                list(vec![kw(":wat::core::unquote"), sym("y")]),
            ]),
        ]);
        assert_eq!(
            parse_one("`(:wat::holon::Bind ,x ,y)").unwrap(),
            expected
        );
    }

    #[test]
    fn quasiquote_with_unquote_splicing_inside() {
        let expected = list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![
                kw(":wat::holon::Bundle"),
                list(vec![kw(":wat::core::unquote-splicing"), sym("xs")]),
            ]),
        ]);
        assert_eq!(
            parse_one("`(:wat::holon::Bundle ,@xs)").unwrap(),
            expected
        );
    }

    #[test]
    fn reader_macro_without_following_form_errors() {
        assert!(matches!(parse_one("`"), Err(ParseError::Empty)));
        assert!(matches!(parse_one(","), Err(ParseError::Empty)));
        assert!(matches!(parse_one(",@"), Err(ParseError::Empty)));
    }

    #[test]
    fn parametric_keyword_survives_in_call() {
        let src = "(foo :Vec<T>)";
        assert_eq!(
            parse_one(src).unwrap(),
            list(vec![sym("foo"), kw(":Vec<T>")])
        );
    }
}
