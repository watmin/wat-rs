//! S-expression parser — tokens → `WatAST`.
//!
//! Recursive descent over the s-expression grammar. Produces a uniform
//! `WatAST` tree: literals are their respective variants, keywords and
//! symbols are leaves, parenthesized forms are `List` nodes. Dispatch
//! on head keyword (`:wat::core::define`, `:wat::holon::...`, etc.)
//! happens at later passes, not here.
//!
//! Two entry points:
//! - [`parse_one!`] — macro that parses a single top-level form and
//!   auto-captures the call-site Rust file:line as the span label.
//! - [`parse_all!`] — macro that parses a sequence of top-level forms
//!   and auto-captures the call-site Rust file:line as the span label.
//!
//! Production callers with real source paths use
//! [`parse_one_with_file`] / [`parse_all_with_file`] directly.

use crate::ast::WatAST;
use crate::identifier::{Identifier, BOUND_NAMESPACE};
use crate::lexer::{lex, LexError, SpannedToken, Token};
use crate::span::Span;
use std::fmt;
use std::sync::Arc;

/// Parse error. Pattern A (Stone 243.7d): span at the outer struct
/// level; variant data in `ParseErrorKind`. Every constructor demands
/// the span — silent omission is uncompilable.
#[derive(Clone, PartialEq)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

/// Variant data for [`ParseError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Lex failure — the input couldn't be tokenized.
    Lex(LexError),
    /// A `)` was found with no matching `(`. Span is the location of
    /// the unmatched `)` so the user can jump straight to it instead
    /// of bisecting the file by hand.
    UnexpectedRParen,
    /// An opening `(` was never closed before end of input. Span is
    /// the location of the orphan `(` (not end-of-file) — points the
    /// user at the form they forgot to close.
    UnclosedParen,
    /// A `]` was found with no matching `[`. Arc 167 slice 1.
    UnexpectedRBracket,
    /// An opening `[` was never closed before end of input. Arc 167
    /// slice 1.
    UnclosedBracket,
    /// A `}` was found with no matching `{`. Arc 169 slice 1.
    UnexpectedRBrace,
    /// An opening `{` was never closed before end of input. Arc 169
    /// slice 1.
    UnclosedBrace,
    /// A struct-destructure brace-form carried a non-Symbol child.
    /// Arc 169 slice 1 — parse-time shape rule: every child of `{}`
    /// in struct-destructure position must be a bare Symbol, and at
    /// A brace-form `{...}` was used as a map literal but violated
    /// the pinned `HashMap<Keyword, HolonAST>` shape. Arc 214 P2:
    ///
    /// - Key in even-indexed position was not a Keyword.
    /// - Body had an odd number of forms (must alternate key/value).
    /// - First child was neither Keyword (map literal) nor bare Symbol
    ///   (struct destructure) — i.e., an integer, list, etc.
    MalformedBraceLiteral {
        /// Diagnostic naming the offending shape.
        reason: String,
    },
    /// `parse_one` expected exactly one form; got trailing content after
    /// the first complete form. Span points at the first trailing token.
    TrailingContent,
    /// `parse_one` expected a form but the input was empty (all whitespace).
    Empty,
    /// Arc 251 Stone 251.8a-ii — a symbol token spelled with the reserved
    /// `$bound` namespace segment (`identifier::BOUND_NAMESPACE`) as its
    /// namespace, e.g. `$bound/x`. Only the substrate's own binder
    /// construction may produce a `$bound`-namespaced symbol; user source
    /// writing it is refused HERE, at the reader — option D over option A
    /// in DESIGN-STONE-251.8-symbol-proper.md §251.8a-ii: no downstream
    /// pass can ever be handed a forged one, because the namespace is never
    /// constructed at all, rather than constructed-then-checked.
    ForgedBinderNamespace {
        /// The offending spelling, verbatim, for the error message.
        spelling: String,
    },
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::Lex(e) => write!(f, "lex error: {}", e),
            ParseErrorKind::UnexpectedRParen => write!(f, "unexpected ')'"),
            ParseErrorKind::UnclosedParen => write!(f, "unclosed '('"),
            ParseErrorKind::UnexpectedRBracket => write!(f, "unexpected ']'"),
            ParseErrorKind::UnclosedBracket => write!(f, "unclosed '['"),
            ParseErrorKind::UnexpectedRBrace => write!(f, "unexpected '}}'"),
            ParseErrorKind::UnclosedBrace => write!(f, "unclosed '{{'"),
            ParseErrorKind::MalformedBraceLiteral { reason } => write!(
                f,
                "malformed brace-literal: {}",
                reason
            ),
            ParseErrorKind::TrailingContent => {
                write!(f, "trailing content (parse_one expected a single top-level form)")
            }
            ParseErrorKind::Empty => write!(f, "empty input — expected a form"),
            ParseErrorKind::ForgedBinderNamespace { spelling } => write!(
                f,
                "`{spelling}` uses the reserved `{BOUND_NAMESPACE}` namespace — substrate-minted \
                 for local binders, user source may not write it. A local is written bare \
                 (e.g. `x`, not `{BOUND_NAMESPACE}/x`).",
            ),
        }
    }
}

impl fmt::Debug for ParseError {
    // Stone B (arc 296): Debug emits EDN (to_edn(), since to_wire_edn is in the
    // `wat` crate which depends on `wat-reader` — no reverse dep allowed).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use wat_edn::ToEdn;
        f.write_str(&wat_edn::write(&self.to_edn()))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use wat_edn::ToEdn;
        f.write_str(&wat_edn::write(&self.to_edn()))
    }
}

impl std::error::Error for ParseError {}

// ─── ToEdn ──────────────────────────────────────────────────────────────────
//
// The impl lives in `wat-reader` (where `ParseError` is defined) rather than
// in the `wat` crate because `ToEdn` is now in `wat-edn`. The orphan rule
// forbids implementing a foreign trait for a foreign type; keeping the impl
// in the crate that owns the type satisfies the rule.
//
// The `"wat.parse"` namespace literal mirrors `crate::error_ns::PARSE` in
// the `wat` crate. They must stay in sync.

impl wat_edn::ToEdn for ParseError {
    /// `#wat.parse/<VariantName> {:span {…} <variant fields>}` — Pattern A:
    /// span at the outer struct. The `Lex` variant nests the underlying
    /// `LexError` message as `:cause`; every other variant is structureless.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use std::borrow::Cow;
        use wat_edn::{Keyword, OwnedValue, Tag};

        let (variant, mut fields): (&str, Vec<(OwnedValue, OwnedValue)>) = match &self.kind {
            ParseErrorKind::Lex(e) => (
                "Lex",
                vec![(
                    OwnedValue::Keyword(Keyword::new("cause")),
                    OwnedValue::String(Cow::Owned(e.to_string())),
                )],
            ),
            ParseErrorKind::UnexpectedRParen => ("UnexpectedRParen", vec![]),
            ParseErrorKind::UnclosedParen => ("UnclosedParen", vec![]),
            ParseErrorKind::UnexpectedRBracket => ("UnexpectedRBracket", vec![]),
            ParseErrorKind::UnclosedBracket => ("UnclosedBracket", vec![]),
            ParseErrorKind::UnexpectedRBrace => ("UnexpectedRBrace", vec![]),
            ParseErrorKind::UnclosedBrace => ("UnclosedBrace", vec![]),
            ParseErrorKind::MalformedBraceLiteral { reason } => (
                "MalformedBraceLiteral",
                vec![(
                    OwnedValue::Keyword(Keyword::new("reason")),
                    OwnedValue::String(Cow::Owned(reason.clone())),
                )],
            ),
            ParseErrorKind::TrailingContent => ("TrailingContent", vec![]),
            ParseErrorKind::Empty => ("Empty", vec![]),
            ParseErrorKind::ForgedBinderNamespace { spelling } => (
                "ForgedBinderNamespace",
                vec![(
                    OwnedValue::Keyword(Keyword::new("spelling")),
                    OwnedValue::String(Cow::Owned(spelling.clone())),
                )],
            ),
        };
        // Append the span — always present (arc 298.2: every span is real).
        fields.push((
            OwnedValue::Keyword(Keyword::new("span")),
            self.span.to_edn(),
        ));
        OwnedValue::Tagged(
            Tag::ns("wat.parse", variant),
            Box::new(OwnedValue::Map(fields)),
        )
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError { span: crate::rust_caller_span!(), kind: ParseErrorKind::Lex(e) }
    }
}

/// Parse one form, auto-capturing the call-site Rust source location as
/// the span file label. Use in tests and any call site where a real
/// path is not available. Production code with a real path calls
/// [`parse_one_with_file`] directly.
///
/// Expands to `parse_one_with_file(src, concat!(file!(), ":", line!()))`.
#[macro_export]
macro_rules! parse_one {
    ($src:expr $(,)?) => {
        $crate::parser::parse_one_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

/// Parse all forms, auto-capturing the call-site Rust source location as
/// the span file label. Use in tests and any call site where a real
/// path is not available. Production code with a real path calls
/// [`parse_all_with_file`] directly.
///
/// Expands to `parse_all_with_file(src, concat!(file!(), ":", line!()))`.
#[macro_export]
macro_rules! parse_all {
    ($src:expr $(,)?) => {
        $crate::parser::parse_all_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

/// [`parse_one!`] with an explicit span-label for diagnostics. Arc 016 slice 1.
pub fn parse_one_with_file(src: &str, file: &str) -> Result<WatAST, ParseError> {
    let file_arc = Arc::new(file.to_string());
    let tokens = lex(src, file_arc)?;
    let mut cursor = Cursor::new(&tokens);
    let node = match cursor.parse_form()? {
        Some(node) => node,
        None => return Err(ParseError { span: crate::rust_caller_span!(), kind: ParseErrorKind::Empty }),
    };
    if let Some(tok) = cursor.peek() {
        return Err(ParseError { span: tok.span.clone(), kind: ParseErrorKind::TrailingContent });
    }
    Ok(node)
}

/// [`parse_all!`] with an explicit span-label for diagnostics. Arc 016 slice 1.
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
                // Arc 281 — thread the close span so the List node covers open..close.
                let (list, close_span) = self.parse_list_body(span.clone())?;
                let (end_l, end_c) = close_span.end.as_ref()
                    .map(|p| (p.line, p.col))
                    .unwrap_or((close_span.line, close_span.col));
                let node_span = Span::with_end(
                    span.file.clone(), span.line, span.col,
                    end_l, end_c,
                );
                Ok(Some(WatAST::List(list, node_span)))
            }
            Token::RParen => Err(ParseError { span, kind: ParseErrorKind::UnexpectedRParen }),
            Token::LBracket => {
                // Arc 167 slice 1 — bracketed forms parse as
                // `WatAST::Vector`. `[]` parses as an empty Vector
                // (distinct from `()`, which is Unit / empty List).
                // Arc 281 — thread the close span so the Vector node covers open..close.
                let (items, close_span) = self.parse_vector_body(span.clone())?;
                let (end_l, end_c) = close_span.end.as_ref()
                    .map(|p| (p.line, p.col))
                    .unwrap_or((close_span.line, close_span.col));
                let node_span = Span::with_end(
                    span.file.clone(), span.line, span.col,
                    end_l, end_c,
                );
                Ok(Some(WatAST::Vector(items, node_span)))
            }
            Token::RBracket => Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBracket }),
            Token::LBrace => {
                // Arc 257.2 — all `{…}` parse to `WatAST::Map` via
                // `parse_map_literal_body`. Binder-position interpretation
                // (keys-destructure, hash-destructure) is the runtime/check
                // job using `WatAST::classify_map_destructure`. The old
                // BraceKind dispatch and the two StructPattern-body parsers
                // are deleted here; `{x y z}` (odd-arity) is now a clean
                // parse error from `parse_map_literal_body`.
                // Arc 281 — thread the close span so the Map node covers open..close.
                let (items, close_span) = self.parse_brace_body(span.clone())?;
                let (end_l, end_c) = close_span.end.as_ref()
                    .map(|p| (p.line, p.col))
                    .unwrap_or((close_span.line, close_span.col));
                let node_span = Span::with_end(
                    span.file.clone(), span.line, span.col,
                    end_l, end_c,
                );
                self.parse_map_literal_body(items, node_span)
            }
            Token::RBrace => Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBrace }),
            Token::LHashBrace => {
                // Arc 215 stone 1 — `#{x y z ...}` set literal.
                // Arc 257 slice 1: produces a native `WatAST::Set` node
                // directly — no longer desugared to a `(:wat::core::HashSet
                // ...)` constructor-call List. T inferred by check.rs from
                // element types.
                // Arc 281 — thread the close span so the Set node covers open..close.
                let (items, close_span) = self.parse_brace_body(span.clone())?;
                let (end_l, end_c) = close_span.end.as_ref()
                    .map(|p| (p.line, p.col))
                    .unwrap_or((close_span.line, close_span.col));
                let node_span = Span::with_end(
                    span.file.clone(), span.line, span.col,
                    end_l, end_c,
                );
                self.parse_hashset_literal_body(items, node_span)
            }
            Token::Int(n) => Ok(Some(WatAST::IntLit(*n, span))),
            Token::Float(x) => Ok(Some(WatAST::FloatLit(*x, span))),
            // Arc 300 stone B — rational literal, numeric-literal lane (NOT
            // desugar; see DESIGN-STONE-rational-B-runtime.md). Already
            // reduced + normalized by the lexer.
            Token::Rational(r) => Ok(Some(WatAST::RationalLit(r.clone(), span))),
            // Arc 300 stone C1 — bigint literal, numeric-literal lane (mirrors
            // the Rational arm immediately above, one type over).
            Token::BigInt(n) => Ok(Some(WatAST::BigIntLit(n.clone(), span))),
            Token::Bool(b) => Ok(Some(WatAST::BoolLit(*b, span))),
            Token::Str(s) => Ok(Some(WatAST::StringLit(s.clone(), span))),
            Token::Keyword(k) => Ok(Some(WatAST::Keyword(k.clone(), span))),
            Token::Symbol(s) if s == "nil" => Ok(Some(WatAST::NilLit(span))),
            // Arc 251 Stone 251.8a-ii — refuse a symbol whose NAMESPACE
            // SEGMENT (everything before the `/`) is exactly the reserved
            // `$bound` namespace. A namespace check, not a whole-token
            // equality like the `nil` arm above: `$bound/x`, `$bound/y`, …
            // are all refused, while a bare `$bound` (no `/`, an ordinary
            // binder name) and `$boundary`/`$bound2/x` (share the prefix
            // but not the segment boundary) are untouched. Reads
            // BOUND_NAMESPACE — the literal "$bound" is not re-spelled.
            Token::Symbol(s)
                if s.strip_prefix(BOUND_NAMESPACE).is_some_and(|rest| rest.starts_with('/')) =>
            {
                Err(ParseError {
                    span,
                    kind: ParseErrorKind::ForgedBinderNamespace { spelling: s.clone() },
                })
            }
            Token::Symbol(s) => Ok(Some(WatAST::Symbol(Identifier::bare(s.clone()), span))),
            Token::Quasiquote => self.parse_reader_macro(":wat::core::quasiquote", span),
            Token::Quote => self.parse_reader_macro(":wat::core::quote", span),
            // Arc 294.b — `#holon <form>` → `(:wat::holon::literal <form>)`.
            Token::HolonLiteral => self.parse_reader_macro(":wat::holon::literal", span),
            Token::Unquote => self.parse_reader_macro(":wat::core::unquote", span),
            Token::UnquoteSplicing => self.parse_reader_macro(":wat::core::unquote-splicing", span),
            // Arc 220 slice 2 — `\c` character literal reader macro.
            // Arc 300 stone D — joins the scalar-literal lane (mirrors the
            // BigInt/Rational arms above, one type over) rather than
            // desugaring into a `(:wat::core::char/of "x")` call. Named
            // chars (`\newline` etc.) are resolved by the lexer before this
            // point; the parser sees only the resolved `char` value. The
            // `:wat::core::char` verb keeps working as a real runtime
            // String→char conversion; it simply stops being the reader's
            // parse target.
            Token::Char(c) => Ok(Some(WatAST::CharLit(*c, span))),
        }
    }

    /// A reader macro (`` ` `` / `~` / `~@`) wraps the following form.
    /// `` `X `` → `(:wat::core::quasiquote X)`, etc. The synthesized
    /// head-keyword and list inherit the reader macro's span; the
    /// inner form keeps its own. Arc 172 slice 1: unquote source
    /// characters changed from `,`/`,@` to `~`/`~@`.
    fn parse_reader_macro(
        &mut self,
        head_keyword: &str,
        span: Span,
    ) -> Result<Option<WatAST>, ParseError> {
        let inner = self.parse_form()?.ok_or(ParseError { span: crate::rust_caller_span!(), kind: ParseErrorKind::Empty })?;
        Ok(Some(WatAST::List(
            vec![WatAST::Keyword(head_keyword.to_string(), span.clone()), inner],
            span,
        )))
    }

    /// Parse the body of a list — `(` already consumed. Accumulates child
    /// forms until the matching `)`. `open_span` is the location of the
    /// `(` that was consumed; surfaced in `UnclosedParen` errors so the
    /// reader can jump to the orphan opener instead of bisecting the
    /// file by paren-counting.
    ///
    /// Arc 281 — returns the close `)` token's span alongside the children
    /// so `parse_form` can build a node span covering open..close.
    fn parse_list_body(&mut self, open_span: Span) -> Result<(Vec<WatAST>, Span), ParseError> {
        let mut children = Vec::new();
        loop {
            match self.peek().map(|st| &st.token) {
                Some(Token::RParen) => {
                    let close_span = self.advance().expect("peeked").span.clone();
                    return Ok((children, close_span));
                }
                Some(Token::RBracket) => {
                    // Arc 167 slice 1 — a `]` inside a list body
                    // is a delimiter mismatch. Surface as
                    // `UnexpectedRBracket` pointing at the `]`.
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBracket });
                }
                Some(Token::RBrace) => {
                    // Arc 169 slice 1 — a `}` inside a list body
                    // is a delimiter mismatch. Surface as
                    // `UnexpectedRBrace` pointing at the `}`.
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBrace });
                }
                Some(_) => match self.parse_form()? {
                    Some(child) => children.push(child),
                    None => unreachable!(
                        "parse_form returned None but peek() had a token"
                    ),
                },
                None => return Err(ParseError { span: open_span, kind: ParseErrorKind::UnclosedParen }),
            }
        }
    }

    /// Parse the body of a vector — `[` already consumed. Accumulates
    /// child forms until the matching `]`. `open_span` is the location
    /// of the `[`; surfaced in `UnclosedBracket` errors so the reader
    /// can jump to the orphan opener. Arc 167 slice 1.
    ///
    /// Arc 281 — returns the close `]` token's span alongside the children
    /// so `parse_form` can build a node span covering open..close.
    fn parse_vector_body(&mut self, open_span: Span) -> Result<(Vec<WatAST>, Span), ParseError> {
        let mut children = Vec::new();
        loop {
            match self.peek().map(|st| &st.token) {
                Some(Token::RBracket) => {
                    let close_span = self.advance().expect("peeked").span.clone();
                    return Ok((children, close_span));
                }
                Some(Token::RParen) => {
                    // A `)` inside a vector body is a delimiter
                    // mismatch — surface as `UnexpectedRParen`.
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRParen });
                }
                Some(Token::RBrace) => {
                    // Arc 169 slice 1 — a `}` inside a vector body
                    // is a delimiter mismatch.
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBrace });
                }
                Some(_) => match self.parse_form()? {
                    Some(child) => children.push(child),
                    None => unreachable!(
                        "parse_form returned None but peek() had a token"
                    ),
                },
                None => return Err(ParseError { span: open_span, kind: ParseErrorKind::UnclosedBracket }),
            }
        }
    }

    /// Parse the body of a brace-form — `{` already consumed. Mirrors
    /// `parse_vector_body`'s shape; emits an `UnclosedBrace` error if
    /// EOF arrives before a matching `}`. Cross-delimiter mismatches
    /// (`)` or `]` inside a `{...}` body) surface as the corresponding
    /// `Unexpected*` errors. Arc 169 slice 1.
    ///
    /// Arc 214 P2 — this method is content-agnostic; it accumulates all
    /// child forms and returns them. The LBrace arm in `parse_form`
    /// dispatches the result to either `parse_map_literal_body` or
    /// `parse_struct_destructure_body` based on the first child's shape.
    ///
    /// Arc 281 — returns the close `}` token's span alongside the children
    /// so `parse_form` can build a node span covering open..close.
    fn parse_brace_body(&mut self, open_span: Span) -> Result<(Vec<WatAST>, Span), ParseError> {
        let mut children = Vec::new();
        loop {
            match self.peek().map(|st| &st.token) {
                Some(Token::RBrace) => {
                    let close_span = self.advance().expect("peeked").span.clone();
                    return Ok((children, close_span));
                }
                Some(Token::RParen) => {
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRParen });
                }
                Some(Token::RBracket) => {
                    let span = self.peek().expect("guard").span.clone();
                    return Err(ParseError { span, kind: ParseErrorKind::UnexpectedRBracket });
                }
                Some(_) => match self.parse_form()? {
                    Some(child) => children.push(child),
                    None => unreachable!(
                        "parse_form returned None but peek() had a token"
                    ),
                },
                None => return Err(ParseError { span: open_span, kind: ParseErrorKind::UnclosedBrace }),
            }
        }
    }

    /// Arc 214 P2 — map literal semantic path. Called when the brace
    /// body is empty (empty map) or begins with a Keyword (map literal).
    ///
    /// Arc 215 stone 1 — replaces pinned `:wat::holon::HolonAST` V-type and
    /// Arc 257 slice 1 — `{k0 v0 k1 v1 ...}` map literal. Produces a
    /// first-class `WatAST::Map(pairs, span)` node (no eager desugar to a
    /// `(:wat::core::HashMap …)` constructor call).
    ///
    /// Validation rules:
    /// - Body length must be even (alternating key/value pairs).
    /// - Any value shape is accepted as a key at parse time; check.rs
    ///   handles type uniformity (K-inference + unification).
    ///
    /// Previously (arc 215 stone 2) this synthesized a `(:wat::core::HashMap
    /// :wat::type::Infer :wat::type::Infer k v …)` constructor-call List.
    /// That eager desugar made the AST non-EDN (a function-call form, not a
    /// map literal). Arc 257 fixes this at the source: the parser now emits
    /// the native `Map` node directly. K/V inference is unchanged — check.rs
    /// still starts from fresh type vars and unifies against the actual keys
    /// and values.
    fn parse_map_literal_body(
        &self,
        items: Vec<WatAST>,
        open_span: Span,
    ) -> Result<Option<WatAST>, ParseError> {
        // Even-count rule: body must alternate key/value pairs.
        if !items.len().is_multiple_of(2) {
            return Err(ParseError {
                span: open_span,
                kind: ParseErrorKind::MalformedBraceLiteral {
                    reason: format!(
                        "map-literal body must alternate key + value pairs; got {} forms",
                        items.len()
                    ),
                },
            });
        }

        // Build the pairs vec. Odd arity is unrepresentable in Vec<(k,v)>
        // by construction; the even-arity check above is the safety net.
        let mut pairs: Vec<(WatAST, WatAST)> = Vec::with_capacity(items.len() / 2);
        let mut i = 0;
        while i < items.len() {
            let key = items[i].clone();
            let val = items[i + 1].clone();
            pairs.push((key, val));
            i += 2;
        }
        Ok(Some(WatAST::Map(pairs, open_span)))
    }

    /// Arc 257 slice 1 — `#{x y z ...}` set literal. Produces a first-class
    /// `WatAST::Set(items, span)` node (no eager desugar to a
    /// `(:wat::core::HashSet …)` constructor call).
    ///
    /// T is inferred by check.rs from the first element; subsequent
    /// elements must unify against T. Empty `#{}` produces
    /// `WatAST::Set([], span)` — T stays as a fresh type variable until
    /// the set is used in a typed context.
    ///
    /// Previously (arc 215 stone 1) this synthesized a `(:wat::core::HashSet
    /// :wat::type::Infer x y z …)` constructor-call List. Arc 257 produces
    /// the native `Set` node directly, making the AST EDN-conformant.
    fn parse_hashset_literal_body(
        &self,
        items: Vec<WatAST>,
        open_span: Span,
    ) -> Result<Option<WatAST>, ParseError> {
        Ok(Some(WatAST::Set(items, open_span)))
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
        // Tests rely on WatAST's structural PartialEq, which skips spans.
        // A parsed node and a synthetic `WatAST::int(42)` compare equal
        // regardless of where they came from.
        assert_eq!(crate::parse_one!("42").unwrap(), WatAST::int(42));
        assert_eq!(crate::parse_one!("-1").unwrap(), WatAST::int(-1));
        assert_eq!(crate::parse_one!("2.5").unwrap(), WatAST::float(2.5));
        assert_eq!(crate::parse_one!("true").unwrap(), WatAST::bool(true));
        assert_eq!(crate::parse_one!("false").unwrap(), WatAST::bool(false));
        assert_eq!(crate::parse_one!("\"hello\"").unwrap(), str_lit("hello"));
        assert_eq!(crate::parse_one!(":foo").unwrap(), kw(":foo"));
        assert_eq!(crate::parse_one!("x").unwrap(), sym("x"));
    }

    #[test]
    fn empty_list() {
        assert_eq!(crate::parse_one!("()").unwrap(), list(vec![]));
    }

    #[test]
    fn simple_list() {
        assert_eq!(
            crate::parse_one!("(a b c)").unwrap(),
            list(vec![sym("a"), sym("b"), sym("c")])
        );
    }

    #[test]
    fn nested_list() {
        assert_eq!(
            crate::parse_one!("(a (b c) d)").unwrap(),
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
            crate::parse_one!(r#"(:wat::holon::Atom "role")"#).unwrap(),
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
        assert_eq!(crate::parse_one!(src).unwrap(), expected);
    }

    #[test]
    fn algebra_core_thermometer() {
        assert_eq!(
            crate::parse_one!("(:wat::holon::Thermometer 0.5 0.0 1.0)").unwrap(),
            list(vec![
                kw(":wat::holon::Thermometer"),
                WatAST::FloatLit(0.5, crate::rust_caller_span!()),
                WatAST::FloatLit(0.0, crate::rust_caller_span!()),
                WatAST::FloatLit(1.0, crate::rust_caller_span!()),
            ])
        );
    }

    #[test]
    fn algebra_core_blend_negative_weight() {
        assert_eq!(
            crate::parse_one!("(:wat::holon::Blend a b 1 -1)").unwrap(),
            list(vec![
                kw(":wat::holon::Blend"),
                sym("a"),
                sym("b"),
                WatAST::IntLit(1, crate::rust_caller_span!()),
                WatAST::IntLit(-1, crate::rust_caller_span!()),
            ])
        );
    }

    #[test]
    fn define_signature_shape() {
        // Just verifying the shape survives parsing as a uniform List.
        // Dispatch to a Define node happens in a later pass.
        // Stone 241.11 — :wat::core::defn is the surviving function-binding form.
        let src = "(:wat::core::defn :my::app::amplify [x <- :wat::holon::HolonAST y <- :wat::holon::HolonAST s <- :f64] -> :wat::holon::HolonAST (:wat::holon::Blend x y 1 s))";
        let parsed = crate::parse_one!(src).unwrap();
        // First child must be the :wat::core::defn keyword.
        if let WatAST::List(items, _) = &parsed {
            assert_eq!(items[0], kw(":wat::core::defn"));
        } else {
            panic!("expected top-level List");
        }
    }

    #[test]
    fn parse_all_multiple_forms() {
        let forms = crate::parse_all!(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::load-file! "wat/holon/Subtract.wat")
            "#
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn parse_all_ignores_comments_and_whitespace() {
        let forms = crate::parse_all!(
            r#"
            ;; comment
            42
            ;; another comment
            "hello"
            "#
        )
        .unwrap();
        assert_eq!(forms, vec![WatAST::IntLit(42, crate::rust_caller_span!()), str_lit("hello")]);
    }

    #[test]
    fn unexpected_rparen_at_start() {
        assert!(matches!(crate::parse_one!(")"), Err(ParseError { kind: ParseErrorKind::UnexpectedRParen, .. })));
    }

    #[test]
    fn extra_rparen_after_complete_form_is_trailing() {
        // `(a))` — `(a)` parses fine; the extra `)` is trailing content.
        assert!(matches!(
            crate::parse_one!("(a))"),
            Err(ParseError { kind: ParseErrorKind::TrailingContent, .. })
        ));
    }

    #[test]
    fn unexpected_rparen_inside_list() {
        // `(a ))` — inner ) closes the list; outer ) is then at top-level
        // via parse_all, which treats it as UnexpectedRParen.
        assert!(matches!(
            crate::parse_all!("(a)) foo"),
            Err(ParseError { kind: ParseErrorKind::UnexpectedRParen, .. })
        ));
    }

    #[test]
    fn unclosed_paren() {
        assert!(matches!(crate::parse_one!("("), Err(ParseError { kind: ParseErrorKind::UnclosedParen, .. })));
        assert!(matches!(crate::parse_one!("(a b"), Err(ParseError { kind: ParseErrorKind::UnclosedParen, .. })));
        assert!(matches!(
            crate::parse_one!("(a (b)"),
            Err(ParseError { kind: ParseErrorKind::UnclosedParen, .. })
        ));
    }

    #[test]
    fn empty_input_errors_in_parse_one() {
        assert!(matches!(crate::parse_one!(""), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
        assert!(matches!(crate::parse_one!("   "), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
        assert!(matches!(crate::parse_one!("; comment"), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
    }

    #[test]
    fn empty_input_ok_in_parse_all() {
        assert_eq!(crate::parse_all!("").unwrap(), vec![]);
        assert_eq!(crate::parse_all!("   ").unwrap(), vec![]);
    }

    #[test]
    fn trailing_content_rejected_by_parse_one() {
        assert!(matches!(
            crate::parse_one!("1 2"),
            Err(ParseError { kind: ParseErrorKind::TrailingContent, .. })
        ));
    }

    #[test]
    fn lex_error_surfaces_as_parse_error() {
        // A lex error must surface as ParseError::Lex. Use the
        // unclosed-bracket-in-keyword error — whitespace inside an
        // unclosed `(` in a keyword body.
        let e = crate::parse_one!(":fn(T ").unwrap_err();
        assert!(matches!(e, ParseError { kind: ParseErrorKind::Lex(_), .. }));
    }

    #[test]
    fn internal_colons_lex_as_single_keyword() {
        // Under the colon-quote model, `:` is the symbol-literal reader
        // macro — one leading `:` marks the start; internal `::` is
        // just the Rust path separator, pushed as body characters. This
        // is about `::`, not about angle brackets — the `<T>` on the
        // second case was incidental (arc 109 class-1 rule: a real
        // subject that is NOT the type-head permission must survive).
        // Angle brackets are gone, so the input drops the now-illegal
        // generic suffix and keeps testing internal `::`.
        assert_eq!(
            crate::parse_one!(":wat::load-file!").unwrap(),
            kw(":wat::load-file!")
        );
        assert_eq!(
            crate::parse_one!(":rust::crossbeam_channel::Sender").unwrap(),
            kw(":rust::crossbeam_channel::Sender")
        );
    }

    #[test]
    fn keyword_with_parens_inside() {
        // :fn(T,U)->R — internal parens are still legal keyword-body
        // characters, but a comma inside them is retired (arc 109): this
        // legacy multi-arg fn-type spelling now errors. A comma-free
        // parenthesized body still lexes as a single keyword.
        assert!(crate::parse_one!(":fn(T,U)->R").is_err());
        assert_eq!(crate::parse_one!(":fn(T)->R").unwrap(), kw(":fn(T)->R"));
    }

    // ─── Quasiquote reader macros ───────────────────────────────────────

    #[test]
    fn quasiquote_wraps_following_form() {
        assert_eq!(
            crate::parse_one!("`foo").unwrap(),
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
        assert_eq!(crate::parse_one!("`(a b c)").unwrap(), expected);
    }

    // ─── Quote reader macro ─────────────────────────────────────────────

    #[test]
    fn quote_wraps_following_form() {
        // 'foo → (:wat::core::quote foo)
        assert_eq!(
            crate::parse_one!("'foo").unwrap(),
            list(vec![kw(":wat::core::quote"), sym("foo")])
        );
    }

    #[test]
    fn quote_over_list() {
        // '(a b c) → (:wat::core::quote (a b c))
        assert_eq!(
            crate::parse_one!("'(a b c)").unwrap(),
            list(vec![
                kw(":wat::core::quote"),
                list(vec![sym("a"), sym("b"), sym("c")]),
            ])
        );
    }

    #[test]
    fn quote_does_not_disturb_keyword_body_apostrophe() {
        // Arc 171 invariant: `'` inside keyword body stays absorbed by lex_keyword.
        // `:wat::core::op'2` is a single keyword token, NOT a quote of `(:wat::core::op 2)`.
        assert_eq!(
            crate::parse_one!(":wat::core::op'2").unwrap(),
            kw(":wat::core::op'2")
        );
    }

    #[test]
    fn unquote_wraps_following_form() {
        // Arc 172 slice 1: source character changed from `,x` to `~x`.
        assert_eq!(
            crate::parse_one!("~x").unwrap(),
            list(vec![kw(":wat::core::unquote"), sym("x")])
        );
    }

    #[test]
    fn unquote_splicing_wraps_following_form() {
        // Arc 172 slice 1: source characters changed from `,@xs` to `~@xs`.
        assert_eq!(
            crate::parse_one!("~@xs").unwrap(),
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
        // Arc 172 slice 1: `,x ,y` → `~x ~y`.
        assert_eq!(
            crate::parse_one!("`(:wat::holon::Bind ~x ~y)").unwrap(),
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
        // Arc 172 slice 1: `,@xs` → `~@xs`.
        assert_eq!(
            crate::parse_one!("`(:wat::holon::Bundle ~@xs)").unwrap(),
            expected
        );
    }

    #[test]
    fn reader_macro_without_following_form_errors() {
        // Arc 172 slice 1: `,` is now whitespace (no longer a reader macro);
        // `~` and `~@` are the canonical unquote characters.
        assert!(matches!(crate::parse_one!("`"), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
        assert!(matches!(crate::parse_one!("~"), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
        assert!(matches!(crate::parse_one!("~@"), Err(ParseError { kind: ParseErrorKind::Empty, .. })));
    }

    #[test]
    fn parametric_keyword_survives_in_call() {
        // Arc 109 "annihilate the angle bracket" — THE PERMISSION IS
        // GONE: `:Vec<T>` no longer survives ANYWHERE, including as an
        // argument keyword in a call form. Re-pointed as a refusal
        // control: the lex error surfaces through the parser as
        // `ParseErrorKind::Lex`, same mechanism as
        // `lex_error_surfaces_as_parse_error` above.
        let src = "(foo :Vec<T>)";
        let e = crate::parse_one!(src).unwrap_err();
        assert!(matches!(
            e,
            ParseError {
                kind: ParseErrorKind::Lex(LexError {
                    kind: crate::lexer::LexErrorKind::AngleTypeHeadInName,
                    ..
                }),
                ..
            }
        ));
    }

    // ─── Stone 251.8a-ii — the $bound namespace is unforgeable ──────────
    //
    // `$bound` is the reserved namespace the substrate mints for local
    // binders (`identifier::BOUND_NAMESPACE`). A user writing it explicitly
    // — `$bound/x` — must be refused HERE, at the reader, before any
    // downstream pass can hold a forged binder symbol.

    #[test]
    fn bound_namespace_symbol_is_refused_at_parse() {
        let e = crate::parse_one!("$bound/x").unwrap_err();
        assert!(
            matches!(e, ParseError { kind: ParseErrorKind::ForgedBinderNamespace { .. }, .. }),
            "expected ForgedBinderNamespace, got {e:?}"
        );
    }

    #[test]
    fn bound_namespace_error_names_the_spelling_and_is_located() {
        let e = crate::parse_one!("$bound/x").unwrap_err();

        // EXACT, not `contains` — and on `e.kind`, NOT `e`.
        //
        // `ParseError`'s own Display renders the structured EDN form
        // (`#wat.parse/ForgedBinderNamespace {:spelling … :span …}`), so the
        // loose `e.to_string().contains("$bound/x")` this replaces was matching
        // the `:spelling` FIELD and never read the prose at all — it would have
        // passed with the teaching message deleted entirely. The prose lives on
        // `ParseErrorKind`. Byte-identical per docs/CONVENTIONS.md § 'Test
        // idioms' (a scalar -> assert_eq!); the EDN form is not golden'd here
        // because its :span carries a file:line that moves with this file.
        assert_eq!(
            e.kind.to_string(),
            "`$bound/x` uses the reserved `$bound` namespace — substrate-minted for local binders, \
             user source may not write it. A local is written bare (e.g. `x`, not `$bound/x`).",
        );

        // LOCATED — and this must be a DIFFERENTIAL, not a shape check. The
        // previous form here asserted `!format!("{:?}", e.span).is_empty()`,
        // which is vacuous: a Debug rendering of ANY span is non-empty, so it
        // passed whether or not the error carried a real location. What proves
        // location is that the span MOVES with the offending token: the same
        // forged symbol at a later column reports a later column.
        let early = crate::parse_one!("$bound/x").unwrap_err();
        let later = crate::parse_one!("   $bound/x").unwrap_err();
        assert_ne!(
            (early.span.line, early.span.col),
            (later.span.line, later.span.col),
            "the span must track the offending token, not be a constant",
        );
    }

    #[test]
    fn bound_namespace_check_is_a_namespace_check_not_whole_token_equality() {
        // Trap door named in EXPECTATIONS: copying `nil`'s bare-spelling-
        // equality shape too literally gives a check that only fires on the
        // exact token `$bound` with nothing after it. `$bound/anything` must
        // also be refused, not just the literal 3-char-longer token.
        assert!(matches!(
            crate::parse_one!("$bound/anything-else"),
            Err(ParseError { kind: ParseErrorKind::ForgedBinderNamespace { .. }, .. })
        ));
    }

    #[test]
    fn bound_namespace_positive_control_dollar_still_works() {
        // Row 3 — the one that matters most. Without this, the probe cannot
        // tell "refused `$bound/`" from "refused `$`". `$` is an ordinary
        // identifier character; only the NAMESPACE `$bound` is reserved.
        assert_eq!(crate::parse_one!("$x").unwrap(), sym("$x"));
        // A bare token spelled exactly `$bound`, with no `/` after it, is
        // just an ordinary binder name — not a forged namespace segment.
        assert_eq!(crate::parse_one!("$bound").unwrap(), sym("$bound"));
        // `:foo$impl`-style keyword — `$impl` is a macro-minted name SUFFIX
        // inside a keyword, never a namespace. Untouched by this stone: the
        // Keyword token type isn't even in the arm this stone changes.
        assert_eq!(crate::parse_one!(":foo$impl").unwrap(), kw(":foo$impl"));
        assert_eq!(
            crate::parse_one!(":probe::work$impl").unwrap(),
            kw(":probe::work$impl")
        );
    }

    #[test]
    fn bound_namespace_lookalike_but_not_a_slash_boundary_still_works() {
        // `$boundary` shares the `$bound` PREFIX but is not the reserved
        // namespace segment (no `/` right after `$bound`) — must parse clean.
        assert_eq!(crate::parse_one!("$boundary").unwrap(), sym("$boundary"));
        assert_eq!(crate::parse_one!("$bound2/x").unwrap(), sym("$bound2/x"));
    }
}
