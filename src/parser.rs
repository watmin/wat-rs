//! S-expression parser — tokens → `WatAST`.
//!
//! Recursive descent over the s-expression grammar. Produces a uniform
//! `WatAST` tree: literals are their respective variants, keywords and
//! symbols are leaves, parenthesized forms are `List` nodes. Dispatch
//! on head keyword (`:wat::core::define`, `:wat::algebra::...`, etc.)
//! happens at later passes, not here.
//!
//! Two entry points:
//! - [`parse_one`] — parse a single top-level form; errors if there's
//!   trailing content.
//! - [`parse_all`] — parse a sequence of top-level forms; errors on
//!   unclosed parens or unexpected closers.

use crate::ast::WatAST;
use crate::identifier::Identifier;
use crate::lexer::{lex, LexError, Token};
use std::fmt;

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
/// form, or if any lex/parse rule is violated.
pub fn parse_one(src: &str) -> Result<WatAST, ParseError> {
    let tokens = lex(src)?;
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

/// Parse the input as a sequence of top-level `WatAST` forms.
pub fn parse_all(src: &str) -> Result<Vec<WatAST>, ParseError> {
    let tokens = lex(src)?;
    let mut cursor = Cursor::new(&tokens);
    let mut out = Vec::new();
    while let Some(node) = cursor.parse_form()? {
        out.push(node);
    }
    Ok(out)
}

/// Internal token cursor.
struct Cursor<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Cursor { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&'a Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// Parse one form. Returns `Ok(None)` if input is exhausted.
    /// Returns an error if the next token is an unexpected `)`.
    fn parse_form(&mut self) -> Result<Option<WatAST>, ParseError> {
        let tok = match self.advance() {
            Some(t) => t,
            None => return Ok(None),
        };
        match tok {
            Token::LParen => {
                let list = self.parse_list_body()?;
                Ok(Some(WatAST::List(list)))
            }
            Token::RParen => Err(ParseError::UnexpectedRParen),
            Token::Int(n) => Ok(Some(WatAST::IntLit(*n))),
            Token::Float(x) => Ok(Some(WatAST::FloatLit(*x))),
            Token::Bool(b) => Ok(Some(WatAST::BoolLit(*b))),
            Token::Str(s) => Ok(Some(WatAST::StringLit(s.clone()))),
            Token::Keyword(k) => Ok(Some(WatAST::Keyword(k.clone()))),
            Token::Symbol(s) => Ok(Some(WatAST::Symbol(Identifier::bare(s.clone())))),
            Token::Quasiquote => self.parse_reader_macro(":wat::core::quasiquote"),
            Token::Unquote => self.parse_reader_macro(":wat::core::unquote"),
            Token::UnquoteSplicing => self.parse_reader_macro(":wat::core::unquote-splicing"),
        }
    }

    /// A reader macro (`` ` `` / `,` / `,@`) wraps the following form.
    /// `` `X `` → `(:wat::core::quasiquote X)`, etc.
    fn parse_reader_macro(&mut self, head_keyword: &str) -> Result<Option<WatAST>, ParseError> {
        let inner = self.parse_form()?.ok_or(ParseError::Empty)?;
        Ok(Some(WatAST::List(vec![
            WatAST::Keyword(head_keyword.to_string()),
            inner,
        ])))
    }

    /// Parse the body of a list — `(` already consumed. Accumulates child
    /// forms until the matching `)`.
    fn parse_list_body(&mut self) -> Result<Vec<WatAST>, ParseError> {
        let mut children = Vec::new();
        loop {
            match self.peek() {
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
        WatAST::Keyword(s.to_string())
    }
    fn sym(s: &str) -> WatAST {
        WatAST::Symbol(Identifier::bare(s))
    }
    fn str_lit(s: &str) -> WatAST {
        WatAST::StringLit(s.to_string())
    }
    fn list(items: Vec<WatAST>) -> WatAST {
        WatAST::List(items)
    }

    #[test]
    fn atom_literals() {
        assert_eq!(parse_one("42").unwrap(), WatAST::IntLit(42));
        assert_eq!(parse_one("-1").unwrap(), WatAST::IntLit(-1));
        assert_eq!(parse_one("3.14").unwrap(), WatAST::FloatLit(3.14));
        assert_eq!(parse_one("true").unwrap(), WatAST::BoolLit(true));
        assert_eq!(parse_one("false").unwrap(), WatAST::BoolLit(false));
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
            parse_one(r#"(:wat::algebra::Atom "role")"#).unwrap(),
            list(vec![kw(":wat::algebra::Atom"), str_lit("role")])
        );
    }

    #[test]
    fn algebra_core_bind_with_atoms() {
        // The MVP target shape.
        let src = r#"(:wat::algebra::Bind (:wat::algebra::Atom "role") (:wat::algebra::Atom "filler"))"#;
        let expected = list(vec![
            kw(":wat::algebra::Bind"),
            list(vec![kw(":wat::algebra::Atom"), str_lit("role")]),
            list(vec![kw(":wat::algebra::Atom"), str_lit("filler")]),
        ]);
        assert_eq!(parse_one(src).unwrap(), expected);
    }

    #[test]
    fn algebra_core_thermometer() {
        assert_eq!(
            parse_one("(:wat::algebra::Thermometer 0.5 0.0 1.0)").unwrap(),
            list(vec![
                kw(":wat::algebra::Thermometer"),
                WatAST::FloatLit(0.5),
                WatAST::FloatLit(0.0),
                WatAST::FloatLit(1.0),
            ])
        );
    }

    #[test]
    fn algebra_core_blend_negative_weight() {
        assert_eq!(
            parse_one("(:wat::algebra::Blend a b 1 -1)").unwrap(),
            list(vec![
                kw(":wat::algebra::Blend"),
                sym("a"),
                sym("b"),
                WatAST::IntLit(1),
                WatAST::IntLit(-1),
            ])
        );
    }

    #[test]
    fn define_signature_shape() {
        // Just verifying the shape survives parsing as a uniform List.
        // Dispatch to a Define node happens in a later pass.
        let src = "(:wat::core::define (:my::app::amplify (x :Holon) (y :Holon) (s :f64) -> :Holon) (:wat::algebra::Blend x y 1 s))";
        let parsed = parse_one(src).unwrap();
        // First child must be the :wat::core::define keyword.
        if let WatAST::List(items) = &parsed {
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
            (:wat::core::load! "wat/std/Subtract.wat")
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
        assert_eq!(forms, vec![WatAST::IntLit(42), str_lit("hello")]);
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
            parse_one(":wat::core::load!").unwrap(),
            kw(":wat::core::load!")
        );
        assert_eq!(
            parse_one(":crossbeam_channel::Sender<T>").unwrap(),
            kw(":crossbeam_channel::Sender<T>")
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
        // `(:wat::algebra::Bind ,x ,y) — classic macro template shape.
        let expected = list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![
                kw(":wat::algebra::Bind"),
                list(vec![kw(":wat::core::unquote"), sym("x")]),
                list(vec![kw(":wat::core::unquote"), sym("y")]),
            ]),
        ]);
        assert_eq!(
            parse_one("`(:wat::algebra::Bind ,x ,y)").unwrap(),
            expected
        );
    }

    #[test]
    fn quasiquote_with_unquote_splicing_inside() {
        let expected = list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![
                kw(":wat::algebra::Bundle"),
                list(vec![kw(":wat::core::unquote-splicing"), sym("xs")]),
            ]),
        ]);
        assert_eq!(
            parse_one("`(:wat::algebra::Bundle ,@xs)").unwrap(),
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
        let src = "(foo :List<T>)";
        assert_eq!(
            parse_one(src).unwrap(),
            list(vec![sym("foo"), kw(":List<T>")])
        );
    }
}
