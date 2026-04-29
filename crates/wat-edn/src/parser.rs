//! Recursive-descent EDN parser.
//!
//! Builds [`Value`] from a [`Lexer`]. Built-in tags `#inst` and
//! `#uuid` are canonicalized to typed `Value::Inst` / `Value::Uuid`
//! at parse time. User tags surface as `Value::Tagged`.

use crate::error::{Error, ErrorKind, Result};
use crate::escapes::validate_first_char;
use crate::lexer::{Lexer, Token};
use crate::value::{Keyword, Symbol, Tag, Value};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use std::str::FromStr;
use uuid::Uuid;

/// Which kind of name-bearing token are we splitting? Picks the
/// error variant to wear on rejection. Also routes user-tag-specific
/// rules (e.g. namespace required for tags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyKind {
    Symbol,
    Keyword,
    Tag,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    /// One-token lookahead.
    peeked: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            peeked: None,
        }
    }

    /// Parse exactly one top-level form, then expect EOF.
    pub fn parse_top(mut self) -> Result<Value<'a>> {
        let v = self.parse_value()?;
        match self.next_token()? {
            Token::Eof => Ok(v),
            other => Err(self.unexpected(other)),
        }
    }

    /// Parse all top-level forms until EOF.
    pub fn parse_all(mut self) -> Result<Vec<Value<'a>>> {
        let mut out = Vec::new();
        loop {
            match self.peek_token()? {
                Token::Eof => return Ok(out),
                _ => out.push(self.parse_value()?),
            }
        }
    }

    // ─── Token plumbing ─────────────────────────────────────────

    fn next_token(&mut self) -> Result<Token<'a>> {
        if let Some(t) = self.peeked.take() {
            return Ok(t);
        }
        self.lexer.next_token()
    }

    fn peek_token(&mut self) -> Result<&Token<'a>> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    fn unexpected(&self, t: Token) -> Error {
        Error::at(
            self.lexer.pos(),
            ErrorKind::Other(format!("unexpected token {:?}", t)),
        )
    }

    // ─── Value parsing ─────────────────────────────────────────

    fn parse_value(&mut self) -> Result<Value<'a>> {
        self.parse_value_inner(false)
    }

    /// Inner parse with `discarding` flag. When true, built-in tag
    /// validators (`#inst`, `#uuid`) skip semantic interpretation per
    /// spec L267 ("a reader should not call ... handlers during the
    /// processing of the element to be discarded"). The returned
    /// Value will be thrown away.
    fn parse_value_inner(&mut self, discarding: bool) -> Result<Value<'a>> {
        // Discards at this position: consume `#_ <form>` pairs as trivia.
        self.skip_discards()?;

        let pos = self.lexer.pos();
        let t = self.next_token()?;
        match t {
            Token::Nil => Ok(Value::Nil),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::Integer(i) => Ok(Value::Integer(i)),
            Token::Float(f) => Ok(Value::Float(f)),
            Token::BigInt(s) => {
                let n = BigInt::from_str(s).map_err(|_| {
                    Error::at(pos, ErrorKind::InvalidNumber(format!("{}N", s)))
                })?;
                Ok(Value::BigInt(Box::new(n)))
            }
            Token::BigDec(s) => {
                let n = BigDecimal::from_str(s).map_err(|_| {
                    Error::at(pos, ErrorKind::InvalidNumber(format!("{}M", s)))
                })?;
                Ok(Value::BigDec(Box::new(n)))
            }
            // c: Cow<'a, str> from the lexer — fast path is Borrowed
            // (zero-copy slice into the input), Owned only when the
            // lexer hit an escape sequence. Either way, no allocation
            // here — Value::String just wraps the existing Cow.
            Token::String(c) => Ok(Value::String(c)),
            Token::Char(c) => Ok(Value::Char(c)),
            Token::Keyword(s) => {
                let (namespace, name) = parse_namespaced(s, BodyKind::Keyword)
                    .map_err(|kind| Error::at(pos, kind))?;
                Ok(Value::Keyword(Keyword::from_parts_unchecked(namespace, name)))
            }
            Token::Symbol(s) => {
                let (namespace, name) = parse_namespaced(s, BodyKind::Symbol)
                    .map_err(|kind| Error::at(pos, kind))?;
                Ok(Value::Symbol(Symbol::from_parts_unchecked(namespace, name)))
            }
            Token::Tag(s) => self.parse_tagged(s, pos, discarding),
            Token::LParen => self.parse_list(),
            Token::LBracket => self.parse_vector(),
            Token::LBrace => self.parse_map(),
            Token::HashLBrace => self.parse_set(),
            Token::HashUnderscore => unreachable!("skip_discards consumed it"),
            Token::RParen | Token::RBracket | Token::RBrace | Token::Eof => {
                Err(Error::at(pos, ErrorKind::UnexpectedEof))
            }
        }
    }

    /// Loop while the next token is `#_`: consume it, parse-and-discard
    /// the following form. Per spec L267, handlers must NOT execute
    /// during a discard — `parse_value_inner(true)` propagates the
    /// flag so built-in `#inst` / `#uuid` validators skip semantic
    /// interpretation of the discarded body.
    fn skip_discards(&mut self) -> Result<()> {
        while matches!(self.peek_token()?, Token::HashUnderscore) {
            self.next_token()?; // consume HashUnderscore
            let _ = self.parse_value_inner(true)?;
        }
        Ok(())
    }

    fn parse_list(&mut self) -> Result<Value<'a>> {
        // Pre-size to avoid the 0→4→8→16 reallocation cascade on
        // common-shaped EDN. Most lists/vectors have <= 8 elements.
        let mut items = Vec::with_capacity(8);
        loop {
            self.skip_discards()?;
            match self.peek_token()? {
                Token::RParen => {
                    self.next_token()?;
                    return Ok(Value::List(items));
                }
                Token::Eof => return Err(Error::at(self.lexer.pos(), ErrorKind::UnclosedList)),
                _ => items.push(self.parse_value()?),
            }
        }
    }

    fn parse_vector(&mut self) -> Result<Value<'a>> {
        let mut items = Vec::with_capacity(8);
        loop {
            self.skip_discards()?;
            match self.peek_token()? {
                Token::RBracket => {
                    self.next_token()?;
                    return Ok(Value::Vector(items));
                }
                Token::Eof => return Err(Error::at(self.lexer.pos(), ErrorKind::UnclosedVector)),
                _ => items.push(self.parse_value()?),
            }
        }
    }

    fn parse_map(&mut self) -> Result<Value<'a>> {
        let mut entries: Vec<(Value, Value)> = Vec::with_capacity(8);
        loop {
            self.skip_discards()?;
            match self.peek_token()? {
                Token::RBrace => {
                    self.next_token()?;
                    return Ok(Value::Map(entries));
                }
                Token::Eof => return Err(Error::at(self.lexer.pos(), ErrorKind::UnclosedMap)),
                _ => {
                    let k = self.parse_value()?;
                    self.skip_discards()?;
                    if matches!(self.peek_token()?, Token::RBrace | Token::Eof) {
                        return Err(Error::at(self.lexer.pos(), ErrorKind::OddMapElements));
                    }
                    let v = self.parse_value()?;
                    entries.push((k, v));
                }
            }
        }
    }

    fn parse_set(&mut self) -> Result<Value<'a>> {
        let mut items = Vec::with_capacity(8);
        loop {
            self.skip_discards()?;
            match self.peek_token()? {
                Token::RBrace => {
                    self.next_token()?;
                    return Ok(Value::Set(items));
                }
                Token::Eof => return Err(Error::at(self.lexer.pos(), ErrorKind::UnclosedSet)),
                _ => items.push(self.parse_value()?),
            }
        }
    }

    fn parse_tagged(&mut self, tag_body: &str, pos: usize, discarding: bool) -> Result<Value<'a>> {
        // A tag must be followed by an element. If the next token closes
        // a containing collection or hits EOF, the tag is dangling.
        let body = {
            let next = self.peek_token()?;
            if matches!(
                next,
                Token::Eof | Token::RParen | Token::RBracket | Token::RBrace
            ) {
                return Err(Error::at(
                    pos,
                    ErrorKind::TagWithoutElement(tag_body.into()),
                ));
            }
            self.parse_value_inner(discarding)?
        };

        // Under #_ discard, the entire tagged element is thrown away;
        // skip semantic interpretation (built-in handlers and user-tag
        // namespace validation) so a discarded `#inst "bad"` parses as
        // a discarded form rather than erroring on the bad date.
        if discarding {
            return Ok(Value::Nil);
        }

        // Built-in tags
        match tag_body {
            "inst" => match &body {
                Value::String(s) => {
                    let dt: DateTime<Utc> = DateTime::parse_from_rfc3339(s)
                        .map_err(|e| {
                            Error::at(
                                pos,
                                ErrorKind::InvalidInst(format!("{}: {}", s, e)),
                            )
                        })?
                        .with_timezone(&Utc);
                    Ok(Value::Inst(dt))
                }
                _ => Err(Error::at(
                    pos,
                    ErrorKind::InvalidInst(format!(
                        "expected string body, got {}",
                        body.type_name()
                    )),
                )),
            },
            "uuid" => match body.as_str() {
                Some(s) => {
                    if !is_canonical_uuid(s) {
                        return Err(Error::at(
                            pos,
                            ErrorKind::InvalidUuid(format!(
                                "{}: not in canonical 8-4-4-4-12 hyphenated form",
                                s
                            )),
                        ));
                    }
                    let u = Uuid::parse_str(s).map_err(|e| {
                        Error::at(pos, ErrorKind::InvalidUuid(format!("{}: {}", s, e)))
                    })?;
                    Ok(Value::Uuid(u))
                }
                None => Err(Error::at(
                    pos,
                    ErrorKind::InvalidUuid(format!(
                        "expected string body, got {}",
                        body.type_name()
                    )),
                )),
            },
            other => {
                let (namespace, name) = parse_namespaced(other, BodyKind::Tag)
                    .map_err(|k| Error::at(pos, k))?;
                let ns = namespace.ok_or_else(|| {
                    Error::at(pos, ErrorKind::UserTagMissingNamespace(other.into()))
                })?;

                // wat-edn-internal sentinels for non-finite floats (mirror
                // of writer.rs::write_float). Recognized so f64 round-trips
                // through write→parse without losing NaN / Inf.
                if ns == "wat-edn.float" {
                    match name {
                        "nan" => return Ok(Value::Float(f64::NAN)),
                        "inf" => return Ok(Value::Float(f64::INFINITY)),
                        "neg-inf" => return Ok(Value::Float(f64::NEG_INFINITY)),
                        _ => {}
                    }
                }

                Ok(Value::Tagged(
                    Tag::from_parts_unchecked(ns, name),
                    Box::new(body),
                ))
            }
        }
    }
}

/// Split a symbol/keyword/tag body on the optional `/` namespace
/// separator. Returns `(namespace, name)` as borrowed slices into
/// `body` — no allocation in this function. The caller materializes
/// into `Symbol::from_parts`/`Keyword::from_parts`/`Tag::from_parts`,
/// which uses `CompactString` and inlines short identifiers.
///
/// `kind` selects the error variant used on rejection AND honors the
/// spec's kind-specific rules: keywords may not begin with `::` and
/// `:/` is illegal (whereas `/` alone is a legal bare symbol).
fn parse_namespaced(
    body: &str,
    kind: BodyKind,
) -> std::result::Result<(Option<&str>, &str), ErrorKind> {
    let wrap = |msg: String| -> ErrorKind {
        match kind {
            BodyKind::Symbol => ErrorKind::InvalidSymbol(msg),
            BodyKind::Keyword => ErrorKind::InvalidKeyword(msg),
            BodyKind::Tag => ErrorKind::InvalidTag(msg),
        }
    };

    if body.is_empty() {
        return Err(wrap("empty".into()));
    }

    // `/` is a legal SYMBOL on its own; not a legal keyword/tag body.
    if body == "/" {
        match kind {
            BodyKind::Symbol => return Ok((None, "/")),
            BodyKind::Keyword => return Err(wrap(":/ is not a legal keyword".into())),
            // Lexer never emits a tag body of "/" (tag must start with
            // an alphabetic byte), but the symmetry holds the rule.
            BodyKind::Tag => unreachable!("lexer rejects tag bodies starting with /"),
        }
    }

    if let Some(idx) = body.find('/') {
        let ns = &body[..idx];
        let name = &body[idx + 1..];
        if ns.is_empty() {
            return Err(wrap(format!("empty prefix in {}", body)));
        }
        if name.is_empty() {
            return Err(wrap(format!("empty name in {}", body)));
        }
        if name.contains('/') {
            return Err(wrap(format!("more than one / in {}", body)));
        }
        validate_first_char(ns).map_err(|m| wrap(format!("prefix in {}: {}", body, m)))?;
        validate_first_char(name).map_err(|m| wrap(format!("name in {}: {}", body, m)))?;
        Ok((Some(ns), name))
    } else {
        validate_first_char(body).map_err(|m| wrap(format!("{}: {}", body, m)))?;
        Ok((None, body))
    }
}

/// Spec: "A UUID. The tagged element is a canonical UUID string
/// representation." The canonical form is 8-4-4-4-12 lowercase
/// hexadecimal characters separated by hyphens.
///
/// `uuid::Uuid::parse_str` is more lenient (accepts simple-form,
/// URN-form, and braced-form). Strict EDN means strict canonical.
fn is_canonical_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let expect_dash = matches!(i, 8 | 13 | 18 | 23);
        let is_dash = b == b'-';
        if expect_dash != is_dash {
            return false;
        }
        if !is_dash && !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

// validate_first_char lives in `crate::escapes` so the rule is owned
// once and shared with the lexer's symbol-start check.

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Value {
        Parser::new(s).parse_top().unwrap()
    }

    #[test]
    fn primitives() {
        assert_eq!(p("nil"), Value::Nil);
        assert_eq!(p("true"), Value::Bool(true));
        assert_eq!(p("false"), Value::Bool(false));
        assert_eq!(p("42"), Value::Integer(42));
        assert_eq!(p("-7"), Value::Integer(-7));
        assert_eq!(p("3.14"), Value::Float(3.14));
    }

    #[test]
    fn collections() {
        assert_eq!(
            p("[1 2 3]"),
            Value::Vector(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
        assert_eq!(
            p("(1 2 3)"),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
        assert_eq!(
            p(r#"{"a" 1 "b" 2}"#),
            Value::Map(vec![
                (Value::String("a".into()), Value::Integer(1)),
                (Value::String("b".into()), Value::Integer(2)),
            ])
        );
        assert_eq!(
            p("#{1 2 3}"),
            Value::Set(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
    }

    #[test]
    fn nested() {
        let v = p("[[1 2] [3 4]]");
        assert_eq!(
            v,
            Value::Vector(vec![
                Value::Vector(vec![Value::Integer(1), Value::Integer(2)]),
                Value::Vector(vec![Value::Integer(3), Value::Integer(4)]),
            ])
        );
    }

    #[test]
    fn keywords_and_symbols() {
        assert_eq!(p(":foo"), Value::Keyword(Keyword::new("foo")));
        assert_eq!(p(":ns/foo"), Value::Keyword(Keyword::ns("ns", "foo")));
        assert_eq!(p("foo"), Value::Symbol(Symbol::new("foo")));
        assert_eq!(p("ns/foo"), Value::Symbol(Symbol::ns("ns", "foo")));
    }

    #[test]
    fn user_tags() {
        let v = p("#myapp/Person {:name \"Fred\"}");
        match v {
            Value::Tagged(tag, body) => {
                assert_eq!(tag, Tag::ns("myapp", "Person"));
                assert!(matches!(*body, Value::Map(_)));
            }
            _ => panic!("expected Tagged"),
        }
    }

    #[test]
    fn user_tag_missing_namespace_errors() {
        let r = Parser::new("#bareTag 42").parse_top();
        match r {
            Err(Error::Parse {
                kind: ErrorKind::UserTagMissingNamespace(_),
                ..
            }) => {}
            other => panic!("expected UserTagMissingNamespace, got {:?}", other),
        }
    }

    #[test]
    fn discard() {
        assert_eq!(
            p("[1 #_999 2 3]"),
            Value::Vector(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
    }

    #[test]
    fn inst_canonicalizes() {
        let v = p(r#"#inst "2026-04-26T14:30:00Z""#);
        assert!(matches!(v, Value::Inst(_)));
    }

    #[test]
    fn uuid_canonicalizes() {
        let v = p(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#);
        assert!(matches!(v, Value::Uuid(_)));
    }

    #[test]
    fn bigint_and_bigdec() {
        let v = p("123456789012345678901234567890N");
        assert!(matches!(v, Value::BigInt(_)));
        let v = p("3.14M");
        assert!(matches!(v, Value::BigDec(_)));
    }

    #[test]
    fn parse_all_returns_multiple() {
        let vs = Parser::new("1 2 3").parse_all().unwrap();
        assert_eq!(
            vs,
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]
        );
    }
}
