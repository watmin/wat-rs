//! S-expression lexer — text → tokens.
//!
//! Produces a flat `Vec<Token>` that the parser consumes. Handles:
//!
//! - **Parens** `(` `)` — structural, single-character tokens.
//! - **Numeric literals** — `42`, `-1`, `3.14`, `-0.5`, `1e10`. Tries
//!   `i64` first, falls back to `f64`.
//! - **Bool literals** — `true` / `false`.
//! - **String literals** — `"..."` with `\"`, `\\`, `\n`, `\t`, `\r`
//!   escapes. Quotes stripped before emission.
//! - **Keyword tokens** — start with `:`, followed by any chars matching
//!   a keyword-path or parametric type keyword. Example: `:wat/algebra/Atom`,
//!   `:List<Holon>`, `:fn(T,U)->R`. The only brackets wat has are `(` and
//!   `)`, and they're the only ones this lexer tracks (because `(` and
//!   `)` can appear inside a keyword — as in `:fn(T,U)->R` — and the
//!   lexer must distinguish an internal matched pair from the outer `)`
//!   that closes the enclosing form). Every other character is plain:
//!   `<`, `>`, `/`, `-`, `,`, digits, letters — all just characters in
//!   the keyword's string. A keyword ends at whitespace at paren-depth 0
//!   or at an unmatched `)`. Rejects internal `:` per the colon-quoting
//!   rule — a keyword carries exactly one leading `:`. Whitespace inside
//!   an unclosed `(` is a lex error (malformed keyword).
//! - **Bare symbols** — any non-keyword, non-numeric, non-bool, non-paren,
//!   non-string token.
//! - **Line comments** — `;` to end-of-line — skipped.
//!
//! - **Reader macros** — `` ` `` (quasiquote), `,` (unquote), `,@`
//!   (unquote-splicing). The parser rewrites each to a list-form with
//!   a `:wat/core/quasiquote` / `:wat/core/unquote` / `:wat/core/unquote-splicing`
//!   head, so downstream passes see uniform `List` nodes.
//!
//! Future extensions (not in MVP): character literals `#\a`,
//! block comments.

use std::fmt;

/// A single lexical token.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// Integer literal.
    Int(i64),
    /// Floating-point literal.
    Float(f64),
    /// Boolean literal.
    Bool(bool),
    /// String literal — quotes stripped, escapes applied.
    Str(String),
    /// Keyword token — leading `:` included.
    Keyword(String),
    /// Bare identifier.
    Symbol(String),
    /// Quasiquote `` ` `` reader macro. Parser rewrites to
    /// `(:wat/core/quasiquote X)` wrapping the following form.
    Quasiquote,
    /// Unquote `,` reader macro. Parser rewrites to
    /// `(:wat/core/unquote X)`.
    Unquote,
    /// Unquote-splicing `,@` reader macro. Parser rewrites to
    /// `(:wat/core/unquote-splicing X)`.
    UnquoteSplicing,
}

/// Byte offset into the source string. Used by [`LexError`] variants
/// to point at the offending character. Full source spans (start..end
/// pairs) are not tracked — a single offset is enough for the
/// line/column reconstruction a diagnostic needs.
pub type Position = usize;

/// Lex error with byte offset.
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar(char, Position),
    UnterminatedString(Position),
    UnknownEscape(char, Position),
    InvalidNumber(String, Position),
    /// Keyword contains an internal `:` — the colon-quoting rule says
    /// a keyword has exactly one leading `:` and no others.
    InternalColon(Position),
    /// Whitespace inside an unclosed `(` or `<` in a keyword. The spec
    /// forbids internal whitespace in keywords; if we hit one while
    /// brackets are still open, the keyword is malformed.
    UnclosedBracketInKeyword(Position),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(c, p) => {
                write!(f, "unexpected character {:?} at byte {}", c, p)
            }
            LexError::UnterminatedString(p) => {
                write!(f, "unterminated string literal starting at byte {}", p)
            }
            LexError::UnknownEscape(c, p) => {
                write!(f, "unknown escape sequence \\{} at byte {}", c, p)
            }
            LexError::InvalidNumber(s, p) => {
                write!(f, "invalid numeric literal {:?} at byte {}", s, p)
            }
            LexError::InternalColon(p) => write!(
                f,
                "keyword contains an internal ':' at byte {} — a keyword carries exactly one leading ':'",
                p
            ),
            LexError::UnclosedBracketInKeyword(p) => write!(
                f,
                "whitespace inside unclosed bracket in keyword at byte {} — keywords cannot contain whitespace",
                p
            ),
        }
    }
}

impl std::error::Error for LexError {}

/// Tokenize a wat source string.
///
/// Returns the full token stream or the first lex error encountered.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Line comment — `;` to end of line
        if c == ';' {
            while i < bytes.len() && bytes[i] as char != '\n' {
                i += 1;
            }
            continue;
        }

        // Parens
        if c == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }

        // Quasiquote reader macros — `, ,, ,@`.
        if c == '`' {
            tokens.push(Token::Quasiquote);
            i += 1;
            continue;
        }
        if c == ',' {
            // `,@` or just `,`.
            if i + 1 < bytes.len() && bytes[i + 1] as char == '@' {
                tokens.push(Token::UnquoteSplicing);
                i += 2;
            } else {
                tokens.push(Token::Unquote);
                i += 1;
            }
            continue;
        }

        // String literal
        if c == '"' {
            let (s, next) = lex_string(src, i)?;
            tokens.push(Token::Str(s));
            i = next;
            continue;
        }

        // Keyword token
        if c == ':' {
            let (kw, next) = lex_keyword(src, i)?;
            tokens.push(Token::Keyword(kw));
            i = next;
            continue;
        }

        // Numeric literal or symbol — disambiguate by leading char
        if c.is_ascii_digit() || (c == '-' && is_numeric_start_at(bytes, i + 1)) {
            let (tok, next) = lex_numeric_or_symbol(src, i)?;
            tokens.push(tok);
            i = next;
            continue;
        }

        // Bare symbol — anything else until a break character
        let (sym, next) = lex_symbol(src, i);
        let tok = match sym.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Symbol(sym),
        };
        tokens.push(tok);
        i = next;
    }

    Ok(tokens)
}

/// True if the byte at `i` starts a numeric literal (ascii digit or `.`
/// followed by digit — allow `-.5`-style but not a bare `-`).
fn is_numeric_start_at(bytes: &[u8], i: usize) -> bool {
    if i >= bytes.len() {
        return false;
    }
    let c = bytes[i] as char;
    c.is_ascii_digit() || (c == '.' && i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit())
}

/// Characters that end a bare symbol or unquoted numeric.
fn is_symbol_break(c: char) -> bool {
    c.is_whitespace() || c == '(' || c == ')' || c == '"' || c == ';'
}

/// Lex a string literal starting at `start` (pointing at the opening `"`).
fn lex_string(src: &str, start: usize) -> Result<(String, usize), LexError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[start] as char, '"');
    let mut out = String::new();
    let mut i = start + 1;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '"' {
            return Ok((out, i + 1));
        }
        if c == '\\' {
            i += 1;
            if i >= bytes.len() {
                return Err(LexError::UnterminatedString(start));
            }
            let esc = bytes[i] as char;
            match esc {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                _ => return Err(LexError::UnknownEscape(esc, i)),
            }
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }

    Err(LexError::UnterminatedString(start))
}

/// Lex a keyword token starting at `start` (pointing at `:`).
///
/// Tracks paren depth because `(` and `)` can appear inside a keyword
/// (as in `:fn(T,U)->R`). `)` at paren_depth 0 ends the keyword — that
/// closer belongs to the enclosing form. Every other character
/// (including `<`, `>`, `/`, `-`, `,`) is just pushed as-is. Rejects
/// internal `:`. Whitespace inside an unclosed `(` is an error.
fn lex_keyword(src: &str, start: usize) -> Result<(String, usize), LexError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[start] as char, ':');
    let mut out = String::new();
    out.push(':');
    let mut i = start + 1;
    let mut paren_depth = 0i32;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c.is_whitespace() {
            if paren_depth > 0 {
                return Err(LexError::UnclosedBracketInKeyword(i));
            }
            break;
        }

        match c {
            '(' => {
                paren_depth += 1;
                out.push(c);
            }
            ')' => {
                if paren_depth == 0 {
                    // Unmatched `)` — belongs to the enclosing form.
                    break;
                }
                paren_depth -= 1;
                out.push(c);
            }
            ':' => {
                // No internal ':' per the colon-quoting rule.
                return Err(LexError::InternalColon(i));
            }
            '"' | ';' => {
                // These never appear inside a keyword.
                break;
            }
            _ => out.push(c),
        }

        i += 1;
    }

    Ok((out, i))
}

/// Lex a numeric literal (int or float) or a leading-`-` symbol.
fn lex_numeric_or_symbol(src: &str, start: usize) -> Result<(Token, usize), LexError> {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && !is_symbol_break(bytes[i] as char) {
        i += 1;
    }
    let raw = &src[start..i];

    // Try integer first.
    if let Ok(n) = raw.parse::<i64>() {
        return Ok((Token::Int(n), i));
    }
    // Then float.
    if let Ok(x) = raw.parse::<f64>() {
        return Ok((Token::Float(x), i));
    }
    Err(LexError::InvalidNumber(raw.to_string(), start))
}

/// Lex a bare symbol (including bools `true` / `false`, which the caller
/// re-classifies).
fn lex_symbol(src: &str, start: usize) -> (String, usize) {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && !is_symbol_break(bytes[i] as char) {
        i += 1;
    }
    (src[start..i].to_string(), i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(lex("").unwrap(), vec![]);
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(lex("   \n\t ").unwrap(), vec![]);
    }

    #[test]
    fn parens() {
        assert_eq!(
            lex("()").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
        assert_eq!(
            lex("( )").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            lex("; a comment\n()").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
        assert_eq!(
            lex("(;; inline\n)").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
    }

    #[test]
    fn int_positive() {
        assert_eq!(lex("42").unwrap(), vec![Token::Int(42)]);
    }

    #[test]
    fn int_negative() {
        assert_eq!(lex("-1").unwrap(), vec![Token::Int(-1)]);
    }

    #[test]
    fn float_positive() {
        assert_eq!(lex("3.14").unwrap(), vec![Token::Float(3.14)]);
    }

    #[test]
    fn float_negative() {
        assert_eq!(lex("-0.5").unwrap(), vec![Token::Float(-0.5)]);
    }

    #[test]
    fn bool_literals() {
        assert_eq!(lex("true").unwrap(), vec![Token::Bool(true)]);
        assert_eq!(lex("false").unwrap(), vec![Token::Bool(false)]);
    }

    #[test]
    fn string_basic() {
        assert_eq!(lex("\"hello\"").unwrap(), vec![Token::Str("hello".into())]);
    }

    #[test]
    fn string_escapes() {
        assert_eq!(
            lex(r#""line1\nline2""#).unwrap(),
            vec![Token::Str("line1\nline2".into())]
        );
        assert_eq!(
            lex(r#""quote \"mark\"""#).unwrap(),
            vec![Token::Str("quote \"mark\"".into())]
        );
    }

    #[test]
    fn string_unterminated() {
        assert!(matches!(
            lex("\"oops"),
            Err(LexError::UnterminatedString(_))
        ));
    }

    #[test]
    fn keyword_simple() {
        assert_eq!(
            lex(":foo").unwrap(),
            vec![Token::Keyword(":foo".into())]
        );
    }

    #[test]
    fn keyword_path() {
        assert_eq!(
            lex(":wat/algebra/Atom").unwrap(),
            vec![Token::Keyword(":wat/algebra/Atom".into())]
        );
    }

    #[test]
    fn keyword_parametric_type() {
        assert_eq!(
            lex(":List<Holon>").unwrap(),
            vec![Token::Keyword(":List<Holon>".into())]
        );
        assert_eq!(
            lex(":HashMap<K,V>").unwrap(),
            vec![Token::Keyword(":HashMap<K,V>".into())]
        );
        assert_eq!(
            lex(":fn(T,U)->R").unwrap(),
            vec![Token::Keyword(":fn(T,U)->R".into())]
        );
    }

    #[test]
    fn keyword_ends_at_unmatched_closer() {
        // The `)` here closes the enclosing form, not the keyword.
        let toks = lex("(:foo)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":foo".into()),
                Token::RParen
            ]
        );
    }

    #[test]
    fn keyword_internal_colon_rejected() {
        assert!(matches!(
            lex(":Atom<:Holon>"),
            Err(LexError::InternalColon(_))
        ));
    }

    #[test]
    fn keyword_gt_operator_path() {
        // `:wat/core/>` — the greater-than function at a keyword path.
        // The trailing `>` has no matching `<`, so it's a plain char.
        assert_eq!(
            lex(":wat/core/>").unwrap(),
            vec![Token::Keyword(":wat/core/>".into())]
        );
    }

    #[test]
    fn keyword_fn_type_with_arrow() {
        // `:fn(T,U)->R` — the `->` arrow has a `>` at angle_depth 0,
        // which must be pushed as a plain char, not treated as a closer.
        assert_eq!(
            lex(":fn(T,U)->R").unwrap(),
            vec![Token::Keyword(":fn(T,U)->R".into())]
        );
    }

    #[test]
    fn keyword_nested_parametric_with_fn_type() {
        // `:HashMap<String,fn(i32)->i32>` — the outer `<>` nests a
        // `fn(...)->...` type. The `->` is inside the `<>`.
        assert_eq!(
            lex(":HashMap<String,fn(i32)->i32>").unwrap(),
            vec![Token::Keyword(":HashMap<String,fn(i32)->i32>".into())]
        );
    }

    #[test]
    fn symbol_bare() {
        assert_eq!(lex("x").unwrap(), vec![Token::Symbol("x".into())]);
        assert_eq!(lex("hello").unwrap(), vec![Token::Symbol("hello".into())]);
    }

    #[test]
    fn symbol_with_dashes() {
        assert_eq!(
            lex("my-var").unwrap(),
            vec![Token::Symbol("my-var".into())]
        );
    }

    #[test]
    fn algebra_core_call_tokens() {
        // The MVP target: tokenize the hello-world algebra-core call.
        let toks = lex(r#"(:wat/algebra/Bind (:wat/algebra/Atom "role") (:wat/algebra/Atom "filler"))"#).unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat/algebra/Bind".into()),
                Token::LParen,
                Token::Keyword(":wat/algebra/Atom".into()),
                Token::Str("role".into()),
                Token::RParen,
                Token::LParen,
                Token::Keyword(":wat/algebra/Atom".into()),
                Token::Str("filler".into()),
                Token::RParen,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn thermometer_numeric_args() {
        let toks = lex("(:wat/algebra/Thermometer 0.5 0.0 1.0)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat/algebra/Thermometer".into()),
                Token::Float(0.5),
                Token::Float(0.0),
                Token::Float(1.0),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn blend_with_negative_weight() {
        let toks = lex("(:wat/algebra/Blend a b 1 -1)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat/algebra/Blend".into()),
                Token::Symbol("a".into()),
                Token::Symbol("b".into()),
                Token::Int(1),
                Token::Int(-1),
                Token::RParen,
            ]
        );
    }
}
