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
//! - **Keyword tokens** — start with `:`, followed by a body that is a
//!   **literal Rust path**. Examples:
//!     - `:wat::core::load!`
//!     - `:wat::holon::Atom`
//!     - `:crossbeam_channel::Sender<T>`
//!     - `:Vec<T>`, `:HashMap<K,V>`, `:Option<T>`
//!     - `:fn(T,U)->R`
//!     - `:(T,U)` — a tuple-literal type.
//!
//!   **The `:` is wat's symbol-literal reader macro** — exactly one
//!   leading `:` marks the start of a symbol literal; everything after
//!   is the body. The body contains the literal Rust syntax you want to
//!   name: module paths use `::` (Rust's path separator), type
//!   parameters use `<T>`, function types use `fn(args)->ret`, tuples
//!   use `(T,U)`. No translation — what you write IS the Rust form.
//!
//!   The only brackets wat has are `(` and `)`, and the lexer tracks
//!   their depth inside a keyword body so an internal balanced pair
//!   (`:fn(T,U)->R` or `:(i64,String)`) doesn't get cut short by the
//!   `)` that closes the enclosing form. Every other character is
//!   plain: `<`, `>`, `/`, `-`, `,`, `:`, `::`, digits, letters — all
//!   just body characters. A keyword ends at whitespace at paren-depth
//!   0, or at an unmatched `)`, or at a `"` / `;` (which can't appear
//!   inside a keyword). Whitespace inside an unclosed `(` is a lex
//!   error (malformed keyword).
//!
//! - **Bare symbols** — any non-keyword, non-numeric, non-bool, non-paren,
//!   non-string token.
//! - **Line comments** — `;` to end-of-line — skipped.
//!
//! - **Reader macros** — `` ` `` (quasiquote), `,` (unquote), `,@`
//!   (unquote-splicing). The parser rewrites each to a list-form with
//!   a `:wat::core::quasiquote` / `:wat::core::unquote` / `:wat::core::unquote-splicing`
//!   head, so downstream passes see uniform `List` nodes.
//!
//! Future extensions (not in MVP): character literals `#\a`,
//! block comments.

use crate::span::Span;
use std::fmt;
use std::sync::Arc;

/// A lexed token paired with its source span.
///
/// Arc 016 slice 1. Emitted by [`lex`] for every token; the parser
/// reads the span to attach to the AST node it constructs from the
/// token.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

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
    /// `(:wat::core::quasiquote X)` wrapping the following form.
    Quasiquote,
    /// Unquote `,` reader macro. Parser rewrites to
    /// `(:wat::core::unquote X)`.
    Unquote,
    /// Unquote-splicing `,@` reader macro. Parser rewrites to
    /// `(:wat::core::unquote-splicing X)`.
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
    /// Whitespace inside an unclosed `(` in a keyword. The spec forbids
    /// internal whitespace in keywords; if we hit one while parens are
    /// still open, the keyword is malformed.
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
/// Returns the full token stream (with per-token source spans) or the
/// first lex error encountered. `file` labels every emitted span — use
/// the source path when known, `<test>` / `<eval>` / `<synthetic>` for
/// ad-hoc parses.
pub fn lex(src: &str, file: Arc<String>) -> Result<Vec<SpannedToken>, LexError> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    let line_starts = compute_line_starts(src);

    let span_at = |pos: usize| -> Span {
        let (line, col) = line_col(src, &line_starts, pos);
        Span::new(file.clone(), line, col)
    };

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
            tokens.push(SpannedToken { token: Token::LParen, span: span_at(i) });
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(SpannedToken { token: Token::RParen, span: span_at(i) });
            i += 1;
            continue;
        }

        // Quasiquote reader macros — `, ,, ,@`.
        if c == '`' {
            tokens.push(SpannedToken { token: Token::Quasiquote, span: span_at(i) });
            i += 1;
            continue;
        }
        if c == ',' {
            // `,@` or just `,`.
            let s = span_at(i);
            if i + 1 < bytes.len() && bytes[i + 1] as char == '@' {
                tokens.push(SpannedToken { token: Token::UnquoteSplicing, span: s });
                i += 2;
            } else {
                tokens.push(SpannedToken { token: Token::Unquote, span: s });
                i += 1;
            }
            continue;
        }

        // String literal
        if c == '"' {
            let start = i;
            let (s, next) = lex_string(src, i)?;
            tokens.push(SpannedToken { token: Token::Str(s), span: span_at(start) });
            i = next;
            continue;
        }

        // Keyword token
        if c == ':' {
            let start = i;
            let (kw, next) = lex_keyword(src, i)?;
            tokens.push(SpannedToken { token: Token::Keyword(kw), span: span_at(start) });
            i = next;
            continue;
        }

        // Numeric literal or symbol — disambiguate by leading char
        if c.is_ascii_digit() || (c == '-' && is_numeric_start_at(bytes, i + 1)) {
            let start = i;
            let (tok, next) = lex_numeric_or_symbol(src, i)?;
            tokens.push(SpannedToken { token: tok, span: span_at(start) });
            i = next;
            continue;
        }

        // Bare symbol — anything else until a break character
        let start = i;
        let (sym, next) = lex_symbol(src, i);
        let tok = match sym.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Symbol(sym),
        };
        tokens.push(SpannedToken { token: tok, span: span_at(start) });
        i = next;
    }

    Ok(tokens)
}

/// Precompute byte offsets of every line start (offset 0 + every byte
/// after `\n`). Used by [`line_col`] for O(log n) line lookup.
fn compute_line_starts(src: &str) -> Vec<usize> {
    let mut out = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            out.push(i + 1);
        }
    }
    out
}

/// Map a byte offset to 1-indexed (line, col). `col` counts chars from
/// the start of the line (handles multi-byte UTF-8).
fn line_col(src: &str, line_starts: &[usize], byte_pos: usize) -> (i64, i64) {
    // Binary search for the greatest line_start <= byte_pos.
    let line_idx = match line_starts.binary_search(&byte_pos) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let line_start = line_starts[line_idx];
    let col = src[line_start..byte_pos].chars().count();
    ((line_idx + 1) as i64, (col + 1) as i64)
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
///
/// Iterates characters (not bytes) so multi-byte UTF-8 sequences
/// round-trip into the output `String` unchanged. The previous
/// byte-at-a-time implementation corrupted non-ASCII input by treating
/// each individual byte as a Latin-1 `char` and re-encoding it as
/// UTF-8; `"héllo"` (6 bytes in source) became 8 bytes in the
/// resulting String. Arc 008 slice 3.
fn lex_string(src: &str, start: usize) -> Result<(String, usize), LexError> {
    debug_assert_eq!(&src[start..start + 1], "\"");
    let mut out = String::new();
    let rest = &src[start + 1..];
    let mut chars = rest.char_indices();

    while let Some((offset, c)) = chars.next() {
        if c == '"' {
            // Byte position in `src` one past the closing quote.
            return Ok((out, start + 1 + offset + c.len_utf8()));
        }
        if c == '\\' {
            let (esc_offset, esc) =
                chars.next().ok_or(LexError::UnterminatedString(start))?;
            match esc {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                _ => return Err(LexError::UnknownEscape(esc, start + 1 + esc_offset)),
            }
            continue;
        }
        out.push(c);
    }

    Err(LexError::UnterminatedString(start))
}

/// Lex a keyword token starting at `start` (pointing at `:`).
///
/// The `:` is the symbol-literal reader macro; everything that follows
/// is the body — a literal Rust path. Tracks paren depth because `(`
/// and `)` appear inside keyword bodies (as in `:fn(T,U)->R` and
/// `:(i64,String)`). An unmatched `)` ends the keyword — that closer
/// belongs to the enclosing form. Internal `:` and `::` are body
/// characters (Rust's path separator); the leading `:` is the only
/// one that marks "symbol starts here."
///
/// Every other character (including `<`, `>`, `/`, `-`, `,`, `!`, `?`)
/// is pushed as-is. Whitespace inside an unclosed `(` is an error.
/// `"` and `;` terminate the keyword — they never appear inside one.
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

    /// Strip spans from a lex_tokens() result — lexer tests assert on token
    /// shape, not positions. A dedicated arc-016 slice covers the
    /// span-carrying behavior.
    fn lex_tokens(src: &str) -> Result<Vec<Token>, LexError> {
        let spanned = lex(src, Arc::new("<test>".to_string()))?;
        Ok(spanned.into_iter().map(|s| s.token).collect())
    }

    #[test]
    fn empty_input() {
        assert_eq!(lex_tokens("").unwrap(), vec![]);
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(lex_tokens("   \n\t ").unwrap(), vec![]);
    }

    #[test]
    fn parens() {
        assert_eq!(
            lex_tokens("()").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
        assert_eq!(
            lex_tokens("( )").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            lex_tokens("; a comment\n()").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
        assert_eq!(
            lex_tokens("(;; inline\n)").unwrap(),
            vec![Token::LParen, Token::RParen]
        );
    }

    #[test]
    fn int_positive() {
        assert_eq!(lex_tokens("42").unwrap(), vec![Token::Int(42)]);
    }

    #[test]
    fn int_negative() {
        assert_eq!(lex_tokens("-1").unwrap(), vec![Token::Int(-1)]);
    }

    #[test]
    fn float_positive() {
        assert_eq!(lex_tokens("2.5").unwrap(), vec![Token::Float(2.5)]);
    }

    #[test]
    fn float_negative() {
        assert_eq!(lex_tokens("-0.5").unwrap(), vec![Token::Float(-0.5)]);
    }

    #[test]
    fn bool_literals() {
        assert_eq!(lex_tokens("true").unwrap(), vec![Token::Bool(true)]);
        assert_eq!(lex_tokens("false").unwrap(), vec![Token::Bool(false)]);
    }

    #[test]
    fn string_basic() {
        assert_eq!(lex_tokens("\"hello\"").unwrap(), vec![Token::Str("hello".into())]);
    }

    #[test]
    fn string_escapes() {
        assert_eq!(
            lex_tokens(r#""line1\nline2""#).unwrap(),
            vec![Token::Str("line1\nline2".into())]
        );
        assert_eq!(
            lex_tokens(r#""quote \"mark\"""#).unwrap(),
            vec![Token::Str("quote \"mark\"".into())]
        );
    }

    #[test]
    fn string_unterminated() {
        assert!(matches!(
            lex_tokens("\"oops"),
            Err(LexError::UnterminatedString(_))
        ));
    }

    #[test]
    fn string_preserves_multibyte_utf8() {
        // "héllo" is 6 UTF-8 bytes (h=1, é=2, l=1, l=1, o=1). The
        // lexer must round-trip it byte-exact — the pre-arc-008 byte-
        // at-a-time loop corrupted it to 8 bytes by treating each
        // byte as a Latin-1 char and re-encoding. Arc 008 slice 3.
        let got = lex_tokens("\"héllo\"").unwrap();
        assert_eq!(got, vec![Token::Str("héllo".into())]);
        if let Token::Str(s) = &got[0] {
            assert_eq!(s.len(), 6, "héllo should be 6 UTF-8 bytes");
        }

        // CJK and emoji exercise 3- and 4-byte sequences.
        let got = lex_tokens("\"日本語 🦀\"").unwrap();
        assert_eq!(got, vec![Token::Str("日本語 🦀".into())]);

        // Escape handling adjacent to multi-byte chars.
        let got = lex_tokens(r#""héllo\nworld""#).unwrap();
        assert_eq!(got, vec![Token::Str("héllo\nworld".into())]);
    }

    #[test]
    fn keyword_simple() {
        assert_eq!(
            lex_tokens(":foo").unwrap(),
            vec![Token::Keyword(":foo".into())]
        );
    }

    #[test]
    fn keyword_path() {
        assert_eq!(
            lex_tokens(":wat::holon::Atom").unwrap(),
            vec![Token::Keyword(":wat::holon::Atom".into())]
        );
    }

    #[test]
    fn keyword_parametric_type() {
        assert_eq!(
            lex_tokens(":Vec<wat::holon::HolonAST>").unwrap(),
            vec![Token::Keyword(":Vec<wat::holon::HolonAST>".into())]
        );
        assert_eq!(
            lex_tokens(":HashMap<K,V>").unwrap(),
            vec![Token::Keyword(":HashMap<K,V>".into())]
        );
        assert_eq!(
            lex_tokens(":fn(T,U)->R").unwrap(),
            vec![Token::Keyword(":fn(T,U)->R".into())]
        );
    }

    #[test]
    fn keyword_ends_at_unmatched_closer() {
        // The `)` here closes the enclosing form, not the keyword.
        let toks = lex_tokens("(:foo)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":foo".into()),
                Token::RParen
            ]
        );
    }

    // ─── Colon-quote model: :: is the Rust path separator ──────────────

    #[test]
    fn keyword_double_colon_path() {
        // :: is the canonical namespace separator. The leading : is
        // the symbol-quote; everything after is literal Rust.
        assert_eq!(
            lex_tokens(":wat::core::load!").unwrap(),
            vec![Token::Keyword(":wat::core::load!".into())]
        );
        assert_eq!(
            lex_tokens(":wat::holon::Atom").unwrap(),
            vec![Token::Keyword(":wat::holon::Atom".into())]
        );
        assert_eq!(
            lex_tokens(":my::vocab::foo").unwrap(),
            vec![Token::Keyword(":my::vocab::foo".into())]
        );
    }

    #[test]
    fn keyword_crate_path() {
        // Rust crate paths embed directly — no translation.
        assert_eq!(
            lex_tokens(":rust::crossbeam_channel::Sender<T>").unwrap(),
            vec![Token::Keyword(":rust::crossbeam_channel::Sender<T>".into())]
        );
        assert_eq!(
            lex_tokens(":std::sync::mpsc::Receiver<String>").unwrap(),
            vec![Token::Keyword(":std::sync::mpsc::Receiver<String>".into())]
        );
    }

    #[test]
    fn keyword_division_operator_path() {
        // The division operator's full path: :: separator + / name.
        // Unambiguous: separator is ::, name is /.
        assert_eq!(
            lex_tokens(":wat::core::/").unwrap(),
            vec![Token::Keyword(":wat::core::/".into())]
        );
    }

    #[test]
    fn keyword_tuple_literal_type() {
        // :( opens a tuple-literal type expression.
        assert_eq!(
            lex_tokens(":(i64,String)").unwrap(),
            vec![Token::Keyword(":(i64,String)".into())]
        );
        assert_eq!(
            lex_tokens(":(Holon,wat::holon::HolonAST,Holon)").unwrap(),
            vec![Token::Keyword(":(Holon,wat::holon::HolonAST,Holon)".into())]
        );
    }

    #[test]
    fn keyword_unit_type() {
        // :() is the unit type — also the empty tuple.
        assert_eq!(
            lex_tokens(":()").unwrap(),
            vec![Token::Keyword(":()".into())]
        );
    }

    #[test]
    fn keyword_vec_parametric() {
        // :Vec<T> — Rust's collection name.
        assert_eq!(
            lex_tokens(":Vec<T>").unwrap(),
            vec![Token::Keyword(":Vec<T>".into())]
        );
        assert_eq!(
            lex_tokens(":Vec<wat::holon::HolonAST>").unwrap(),
            vec![Token::Keyword(":Vec<wat::holon::HolonAST>".into())]
        );
    }

    #[test]
    fn keyword_gt_operator_path() {
        // `:wat::core::>` — the greater-than function at a keyword path.
        // The trailing `>` has no matching `<`, so it's a plain char.
        assert_eq!(
            lex_tokens(":wat::core::>").unwrap(),
            vec![Token::Keyword(":wat::core::>".into())]
        );
    }

    #[test]
    fn keyword_fn_type_with_arrow() {
        // `:fn(T,U)->R` — the `->` arrow has a `>` at angle_depth 0,
        // which must be pushed as a plain char, not treated as a closer.
        assert_eq!(
            lex_tokens(":fn(T,U)->R").unwrap(),
            vec![Token::Keyword(":fn(T,U)->R".into())]
        );
    }

    #[test]
    fn keyword_nested_parametric_with_fn_type() {
        // `:HashMap<String,fn(i32)->i32>` — the outer `<>` nests a
        // `fn(...)->...` type. The `->` is inside the `<>`.
        assert_eq!(
            lex_tokens(":HashMap<String,fn(i32)->i32>").unwrap(),
            vec![Token::Keyword(":HashMap<String,fn(i32)->i32>".into())]
        );
    }

    #[test]
    fn symbol_bare() {
        assert_eq!(lex_tokens("x").unwrap(), vec![Token::Symbol("x".into())]);
        assert_eq!(lex_tokens("hello").unwrap(), vec![Token::Symbol("hello".into())]);
    }

    #[test]
    fn symbol_with_dashes() {
        assert_eq!(
            lex_tokens("my-var").unwrap(),
            vec![Token::Symbol("my-var".into())]
        );
    }

    #[test]
    fn algebra_core_call_tokens() {
        // The MVP target: tokenize the hello-world algebra-core call.
        let toks = lex_tokens(r#"(:wat::holon::Bind (:wat::holon::Atom "role") (:wat::holon::Atom "filler"))"#).unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat::holon::Bind".into()),
                Token::LParen,
                Token::Keyword(":wat::holon::Atom".into()),
                Token::Str("role".into()),
                Token::RParen,
                Token::LParen,
                Token::Keyword(":wat::holon::Atom".into()),
                Token::Str("filler".into()),
                Token::RParen,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn thermometer_numeric_args() {
        let toks = lex_tokens("(:wat::holon::Thermometer 0.5 0.0 1.0)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat::holon::Thermometer".into()),
                Token::Float(0.5),
                Token::Float(0.0),
                Token::Float(1.0),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn blend_with_negative_weight() {
        let toks = lex_tokens("(:wat::holon::Blend a b 1 -1)").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Keyword(":wat::holon::Blend".into()),
                Token::Symbol("a".into()),
                Token::Symbol("b".into()),
                Token::Int(1),
                Token::Int(-1),
                Token::RParen,
            ]
        );
    }
}
