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
//!     - `:wat::load-file!`
//!     - `:wat::holon::Atom`
//!     - `:crossbeam_channel::Sender<T>`
//!     - `:wat::core::Vector<T>`, `:wat::core::HashMap<K,V>`, `:wat::core::Option<T>`
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
//!   plain: `<`, `>`, `/`, `-`, `'`, `:`, `::`, digits, letters — all
//!   just body characters. `'` (apostrophe) is the canonical dispatch /
//!   discriminator separator (arc 171). `,` at depth 0 (outside `(...)`
//!   or `<...>`) is rejected; commas inside tuple or parametric type
//!   positions remain valid. A keyword ends at whitespace at paren-depth
//!   0, or at an unmatched `)`, or at a `"` / `;` (which can't appear
//!   inside a keyword). Whitespace inside an unclosed `(` is a lex
//!   error (malformed keyword).
//!
//! - **Bare symbols** — any non-keyword, non-numeric, non-bool, non-paren,
//!   non-string token.
//! - **Line comments** — `;` to end-of-line — skipped.
//!
//! - **Reader macros** — `` ` `` (quasiquote), `~` (unquote), `~@`
//!   (unquote-splicing). The parser rewrites each to a list-form with
//!   a `:wat::core::quasiquote` / `:wat::core::unquote` / `:wat::core::unquote-splicing`
//!   head, so downstream passes see uniform `List` nodes.
//!   Comma (`,`) is whitespace per EDN spec; it carries no token
//!   at the main-lex-loop level (arc 172 slice 1).
//!
//! - **Character literals** — `\c` / `\newline` / `\return` / `\space` /
//!   `\tab` / `\uNNNN` per arc 220 (Clojure/EDN convention, BMP-only).
//!   Note: the pre-arc-220 doc comment listed `#\a` as a future extension —
//!   that was Common-Lisp/Scheme-style and WRONG per the wat-rs lineage.
//!   Wat is clojure-on-rust; the form is `\c`, not `#\a`.
//! - **Block comments** — not yet implemented.

use crate::span::Span;
use num_rational::BigRational;
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
    /// `[` — opens a bracketed form. Arc 167 slice 1: produces
    /// `WatAST::Vector` at the parser layer; vectors are admitted
    /// only in binding-syntax positions (fn/defn sigs, future
    /// let-bindings). At value position the parser still produces
    /// the Vector node; eval/check error there.
    LBracket,
    /// `]`
    RBracket,
    /// `{` — opens a brace-form. Arc 257: produces `WatAST::Map`
    /// at the parser layer. In value position it is a map literal;
    /// in `:wat::core::let` / match binder position it is interpreted
    /// as a destructure (`{:keys [..]}` or `{var :field ..}`) via
    /// `WatAST::classify_map_destructure`. Out-of-position uses are
    /// rejected with a clean MalformedForm.
    LBrace,
    /// `}`
    RBrace,
    /// `#{` — opens a set literal. Arc 215 stone 1: `#{x y z ...}`
    /// desugars to `(:wat::core::HashSet :wat::type::Infer x y z ...)`
    /// at parse time; T is inferred from the element types by check.rs.
    LHashBrace,
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
    /// Quote `'` reader macro. Parser rewrites to
    /// `(:wat::core::quote X)` wrapping the following form.
    /// Arc 220 Slice 3 (Clojure precedent — `'(1 2 3)` form-start).
    /// Distinct from arc 171's keyword-body `'` discriminator (which is
    /// absorbed by `lex_keyword` and never reaches this top-level token).
    Quote,
    /// `#holon` reader tag (arc 294.b). Parser rewrites to
    /// `(:wat::holon::literal X)` wrapping the following form.
    /// Enables heterogeneous EDN maps/sets/vectors to be typed as
    /// `Hologram` without monomorphic literal inference. Span covers
    /// the 6 chars `#holon`; the following form keeps its own span.
    HolonLiteral,
    /// Unquote `~` reader macro. Parser rewrites to
    /// `(:wat::core::unquote X)`. Arc 172 slice 1: source character
    /// changed from `,` to `~`; variant name unchanged.
    Unquote,
    /// Unquote-splicing `~@` reader macro. Parser rewrites to
    /// `(:wat::core::unquote-splicing X)`. Arc 172 slice 1: source
    /// characters changed from `,@` to `~@`; variant name unchanged.
    UnquoteSplicing,
    /// Character literal — `\c` form. Arc 220 slice 2 (Clojure/EDN
    /// convention). Examples: `\a` (letter a), `\newline` (newline),
    /// `\space` (space), `\tab` (tab), `\return` (carriage return),
    /// `\uNNNN` (Unicode BMP escape). BMP-only (U+0000–U+FFFF).
    Char(char),
    /// Rational literal — `<int>/<int>` form (arc 300 stone B). Always a
    /// GENUINE ratio, already reduced to lowest terms with the sign on the
    /// numerator and denominator > 0 — mirrors Stone A's `wat-edn`
    /// normalization exactly (`crates/wat-edn/src/lexer.rs::lex_number`).
    /// A literal whose denominator reduces to 1 (`4/2`) becomes
    /// [`Token::Int`] instead, never this variant — so a `Rational` here
    /// never holds an integer-valued ratio.
    Rational(BigRational),
    /// Arbitrary-precision integer literal — `<int>N` form (arc 300 stone
    /// C1). Mirrors `wat-edn`'s `N`-suffix lexing
    /// (`crates/wat-edn/src/lexer.rs::lex_number`'s `Token::BigInt` branch)
    /// and Clojure's `1N` BigInteger literal. Unlike the `/` rational path
    /// above, this NEVER reduces to `Token::Int` even when the value fits
    /// in i64 — `1N` is always bigint (clj: `(class 1N)` is
    /// `clojure.lang.BigInt` regardless of magnitude).
    BigInt(num_bigint::BigInt),
}

/// Byte offset into the source string. Used by [`LexError`] variants
/// to point at the offending character. Full source spans (start..end
/// pairs) are not tracked — a single offset is enough for the
/// line/column reconstruction a diagnostic needs.
pub type Position = usize;

/// Lex error. Pattern A (Stone 243.7e): position at the outer struct level;
/// variant data in [`LexErrorKind`]. Every constructor demands the position
/// so silent omission is uncompilable.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub position: Position,
    pub kind: LexErrorKind,
}

/// Variant data for [`LexError`]. The byte position lives in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    UnexpectedChar(char),
    UnterminatedString,
    UnknownEscape(char),
    InvalidNumber(String),
    /// Whitespace inside an unclosed `(` in a keyword. The spec forbids
    /// internal whitespace in keywords; if we hit one while parens are
    /// still open, the keyword is malformed.
    UnclosedBracketInKeyword,
    /// Comma inside a keyword body at depth 0 (not inside `(...)` or
    /// `<...>`). Comma as keyword-body separator was retired in arc 171;
    /// `'` (apostrophe) is now the canonical dispatch / discriminator
    /// separator. Example: `:wat::core::op'2` (arity),
    /// `:wat::core::op'i64'i64` (type-discriminator). The legacy `,2` /
    /// `,i64-f64` shape was swept in arc 171 slice 2 (~440 sites).
    CommaInKeywordBody,
    /// Invalid character literal. Arc 220 slice 2: `\c` form error
    /// (empty body, supplementary-plane codepoint, unknown named char,
    /// or backslash followed by whitespace).
    InvalidChar(String),
    /// Raw control character in source. Stone 249 scope-closure:
    /// identifier names must never contain U+0001 (the env-key separator
    /// byte); the lexer rejects ALL raw control characters (except the
    /// permitted structural whitespace `\t`, `\n`, `\r`) so this
    /// invariant is ENFORCED by the lexer, not merely conventional.
    /// The codepoint is reported as its `u32` value for diagnostics.
    ControlCharacterInSource { codepoint: u32 },
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::UnexpectedChar(c) => {
                write!(f, "unexpected character {:?}", c)
            }
            LexErrorKind::UnterminatedString => {
                write!(f, "unterminated string literal")
            }
            LexErrorKind::UnknownEscape(c) => {
                write!(f, "unknown escape sequence \\{}", c)
            }
            LexErrorKind::InvalidNumber(s) => {
                write!(f, "invalid numeric literal {:?}", s)
            }
            LexErrorKind::UnclosedBracketInKeyword => write!(
                f,
                "whitespace inside unclosed bracket in keyword — keywords cannot contain whitespace"
            ),
            LexErrorKind::CommaInKeywordBody => write!(
                f,
                "comma inside keyword body retired (arc 109 \"the comma dies in the reader\", \
                closing the arc 171 carve-out): a comma can never appear in a keyword body, at \
                any depth. For an arity/discriminator suffix use apostrophe `'` — \
                `:wat::core::op'2` (arity), `:wat::core::op'i64'i64` (type-discriminator). For a \
                tuple type use the `:-` binder form — `(:wat::core::Tuple :- [T1 T2 T3])` \
                instead of `:(T1,T2,T3)`. For a function type use the `:->` arrow form — \
                `[T1 T2 :-> R]` instead of `:fn(T1,T2)->R` (see wat/cache.wat, wat/spawn.wat)."
            ),
            LexErrorKind::InvalidChar(msg) => write!(
                f,
                "invalid character literal: {}",
                msg
            ),
            LexErrorKind::ControlCharacterInSource { codepoint } => write!(
                f,
                "raw control character U+{:04X} in source (only \\t, \\n, \\r are permitted \
                 as structural whitespace; all other control bytes are rejected to enforce \
                 the env-key separator invariant)",
                codepoint
            ),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lex error at byte {}: {}", self.position, self.kind)
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

    // Arc 281 — build a span with both start and end stamped.
    // `start_i` is the byte index of the first char of the token;
    // `end_i` is the byte index one past the last char of the token.
    let span_with_end = |start_i: usize, end_i: usize| -> Span {
        let (sl, sc) = line_col(src, &line_starts, start_i);
        let (el, ec) = line_col(src, &line_starts, end_i);
        Span::with_end(file.clone(), sl, sc, el, ec)
    };

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Raw control character rejection (Stone 249 scope-closure).
        // `\t` (0x09), `\n` (0x0A), `\r` (0x0D) are permitted structural
        // whitespace and are consumed by `c.is_whitespace()` above.
        // ALL other control characters (C0 control range 0x00–0x1F plus
        // DEL 0x7F) are rejected here — BEFORE any token dispatch arm —
        // so the check fires regardless of token context (identifier,
        // keyword, symbol, etc.). This makes the claim "the lexer never
        // produces U+0001 in an identifier name" ENFORCED, not conventional.
        if c.is_control() {
            return Err(LexError {
                position: i,
                kind: LexErrorKind::ControlCharacterInSource { codepoint: c as u32 },
            });
        }

        // Line comment — `;` to end of line
        if c == ';' {
            while i < bytes.len() && bytes[i] as char != '\n' {
                i += 1;
            }
            continue;
        }

        // Parens — single-char; end_i = i + 1.
        if c == '(' {
            tokens.push(SpannedToken { token: Token::LParen, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(SpannedToken { token: Token::RParen, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }

        // Brackets — arc 167 slice 1. Emit `LBracket` / `RBracket`
        // tokens which the parser turns into `WatAST::Vector`.
        if c == '[' {
            tokens.push(SpannedToken { token: Token::LBracket, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }
        if c == ']' {
            tokens.push(SpannedToken { token: Token::RBracket, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }

        // Braces — arc 257. Emit `LBrace` / `RBrace` tokens
        // which the parser turns into `WatAST::Map`.
        //
        // Arc 294.b — `#holon` reader tag desugars to `(:wat::holon::literal X)`.
        // Must check BEFORE `#{` (which also starts with `#`) and before the
        // bare-symbol fallthrough.  Only matches when the 5 chars following `#`
        // spell "holon" AND the char immediately after is a delimiter
        // (whitespace, `(`, `[`, `{`, `)`, `]`, `}`, `"`, `;`, `,`, `#`) or
        // EOF — so `#holonx` (no delimiter) falls through to the symbol path.
        if c == '#'
            && i + 6 <= bytes.len()
            && &bytes[i + 1..i + 6] == b"holon"
            && (i + 6 >= bytes.len()
                || is_symbol_break(bytes[i + 6] as char)
                || bytes[i + 6] == b'#')
        {
            tokens.push(SpannedToken { token: Token::HolonLiteral, span: span_with_end(i, i + 6) });
            i += 6;
            continue;
        }
        // Arc 215 stone 1 — `#{` two-character prefix emits `LHashBrace`
        // (set literal). Must check BEFORE plain `{` so `#{` is not
        // split into `Symbol("#")` + `LBrace`.
        if c == '#' && i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
            tokens.push(SpannedToken { token: Token::LHashBrace, span: span_with_end(i, i + 2) });
            i += 2;
            continue;
        }
        if c == '{' {
            tokens.push(SpannedToken { token: Token::LBrace, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }
        if c == '}' {
            tokens.push(SpannedToken { token: Token::RBrace, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }

        // Quasiquote reader macros — `` ` ``, `~`, `~@`.
        // Arc 172 slice 1: comma (`,`) retired as Unquote/UnquoteSplicing
        // token at the main-lex-loop level; it is now treated as EDN
        // whitespace (see whitespace arm above). Tilde (`~`) and `~@`
        // are the canonical Clojure-style unquote characters.
        if c == ',' {
            // Comma is whitespace per EDN spec — skip silently.
            i += 1;
            continue;
        }
        if c == '`' {
            tokens.push(SpannedToken { token: Token::Quasiquote, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }
        // Quote reader macro — `'` at top-level token boundary.
        // Arc 220 Slice 3: `'foo` → `(:wat::core::quote foo)`.
        // The keyword-body `'` discriminator (arc 171) is absorbed inside
        // `lex_keyword` before this point and never reaches this branch.
        if c == '\'' {
            tokens.push(SpannedToken { token: Token::Quote, span: span_with_end(i, i + 1) });
            i += 1;
            continue;
        }
        if c == '~' {
            // `~@` or just `~`.
            if i + 1 < bytes.len() && bytes[i + 1] as char == '@' {
                tokens.push(SpannedToken { token: Token::UnquoteSplicing, span: span_with_end(i, i + 2) });
                i += 2;
            } else {
                tokens.push(SpannedToken { token: Token::Unquote, span: span_with_end(i, i + 1) });
                i += 1;
            }
            continue;
        }

        // String literal — end_i = `next` (one past the closing `"`).
        if c == '"' {
            let start = i;
            let (s, next) = lex_string(src, i)?;
            tokens.push(SpannedToken { token: Token::Str(s), span: span_with_end(start, next) });
            i = next;
            continue;
        }

        // Character literal — arc 220 slice 2.
        // `\c` / `\newline` / `\space` / `\tab` / `\return` / `\uNNNN`.
        // Clojure/EDN form. Must check BEFORE bare-symbol fallthrough
        // so `\a` is not consumed as a symbol.
        if c == '\\' {
            let start = i;
            let (ch, next) = lex_char(src, i)?;
            tokens.push(SpannedToken { token: Token::Char(ch), span: span_with_end(start, next) });
            i = next;
            continue;
        }

        // Non-ASCII token-start byte — clean refusal, never a panic (arc 300
        // stone reader-unicode-parity). wat source is a narrower grammar than
        // EDN — a Unicode symbol isn't wat source, and wat-reader doesn't try
        // to parse it (that's `wat-edn`'s job, the clj-parity target). Must
        // sit AFTER the string/char-literal dispatch above (so `"héllo"` and
        // `\é` are unaffected — those are legitimate UTF-8 content reached via
        // their own ASCII-byte dispatch) and BEFORE keyword/symbol dispatch
        // below, so the byte-wise `lex_keyword` / `lex_symbol` scanners never
        // see a multi-byte lead byte. `c` here is `bytes[i] as char` (a Latin-1
        // widen, not a real decode); re-decode the true scalar from `src` for
        // the error (`i` is a valid char boundary — every prior token fully
        // consumed its bytes).
        if !c.is_ascii() {
            let real = src[i..].chars().next().unwrap_or(c);
            return Err(LexError { position: i, kind: LexErrorKind::UnexpectedChar(real) });
        }

        // Keyword token — end_i = `next` (one past the last keyword char).
        if c == ':' {
            let start = i;
            let (kw, next) = lex_keyword(src, i)?;
            tokens.push(SpannedToken { token: Token::Keyword(kw), span: span_with_end(start, next) });
            i = next;
            continue;
        }

        // Numeric literal or symbol — disambiguate by leading char.
        // end_i = `next` (one past the last char of the numeric/symbol).
        if c.is_ascii_digit() || (c == '-' && is_numeric_start_at(bytes, i + 1)) {
            let start = i;
            let (tok, next) = lex_numeric_or_symbol(src, i)?;
            tokens.push(SpannedToken { token: tok, span: span_with_end(start, next) });
            i = next;
            continue;
        }

        // Bare symbol — anything else until a break character.
        // end_i = `next` (one past the last char of the symbol).
        let start = i;
        let (sym, next) = lex_symbol(src, i);
        let tok = match sym.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Symbol(sym),
        };
        tokens.push(SpannedToken { token: tok, span: span_with_end(start, next) });
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
    c.is_whitespace()
        || c == '('
        || c == ')'
        || c == '['
        || c == ']'
        || c == '{'
        || c == '}'
        || c == '"'
        || c == ';'
        || c == ','  // Arc 172 slice 1: comma is EDN whitespace; it
                     // terminates a symbol scan so `a,b` reads as `a` `,` `b`.
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
                chars.next().ok_or(LexError { position: start, kind: LexErrorKind::UnterminatedString })?;
            match esc {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                _ => return Err(LexError { position: start + 1 + esc_offset, kind: LexErrorKind::UnknownEscape(esc) }),
            }
            continue;
        }
        out.push(c);
    }

    Err(LexError { position: start, kind: LexErrorKind::UnterminatedString })
}

/// Lex a character literal starting at `start` (pointing at `\`).
///
/// Handles the Clojure/EDN `\c` form (arc 220 slice 2):
/// - Named: `\newline`, `\return`, `\space`, `\tab`
/// - Unicode escape: `\uNNNN` (exactly 4 hex digits; BMP range 0000–FFFF)
/// - Single non-alphanumeric char: `\(`, `\;`, `\é`, etc.
/// - Single alphanumeric char: `\a`, `\1`, etc. (after named-char check)
///
/// BMP-only: supplementary-plane codepoints (U+10000–U+10FFFF) are rejected
/// with `LexError::InvalidChar`, inheriting Stone 218.6b discipline.
///
/// Returns `(char_value, next_byte_offset)`.
///
/// Shape adapted verbatim from `crates/wat-edn/src/lexer.rs:288-355`.
fn lex_char(src: &str, start: usize) -> Result<(char, usize), LexError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[start] as char, '\\');
    let mut pos = start + 1; // skip the backslash

    // Backslash cannot be followed by end-of-input or whitespace.
    if pos >= bytes.len() {
        return Err(LexError { position: start, kind: LexErrorKind::InvalidChar("empty char literal".into()) });
    }
    let first = bytes[pos];
    if (first as char).is_whitespace() {
        return Err(LexError { position: start, kind: LexErrorKind::InvalidChar(
            "backslash followed by whitespace".into(),
        ) });
    }

    // Single non-alphanumeric character (e.g. `\(`, `\;`, `\é`).
    // Alphanumeric bodies fall through to the named-char / unicode-escape path.
    if !(first as char).is_ascii_alphanumeric() {
        // Decode one UTF-8 scalar at pos.
        let rest = &src[pos..];
        let c = rest.chars().next().ok_or_else(|| {
            LexError { position: start, kind: LexErrorKind::InvalidChar("incomplete UTF-8 sequence".into()) }
        })?;
        if (c as u32) > 0xFFFF {
            return Err(LexError { position: start, kind: LexErrorKind::InvalidChar(
                format!(
                    "\\{}: supplementary-plane codepoint U+{:X} not supported; \
                     wat char literals are BMP-only (U+0000–U+FFFF)",
                    c, c as u32
                ),
            ) });
        }
        pos += c.len_utf8();
        return Ok((c, pos));
    }

    // Read ASCII-alphanumeric body (a name like "newline", "u00A0", or a single letter/digit).
    let body_start = pos;
    while pos < bytes.len() && (bytes[pos] as char).is_ascii_alphanumeric() {
        pos += 1;
    }
    let body = &src[body_start..pos];

    // 1. Named char literal?
    match body {
        "newline" => return Ok(('\n', pos)),
        "return"  => return Ok(('\r', pos)),
        "space"   => return Ok((' ', pos)),
        "tab"     => return Ok(('\t', pos)),
        _ => {}
    }

    // 2. `\uNNNN` Unicode escape (exactly 4 hex digits)?
    if body.len() == 5 && body.starts_with('u') {
        let hex = &body[1..];
        let acc = u32::from_str_radix(hex, 16).map_err(|_| {
            LexError { position: start, kind: LexErrorKind::InvalidChar(format!("\\{}: invalid hex escape", body)) }
        })?;
        let c = char::from_u32(acc).ok_or_else(|| {
            LexError { position: start, kind: LexErrorKind::InvalidChar(
                format!("\\{}: not a valid Unicode scalar value", body),
            ) }
        })?;
        // Supplementary plane check (structurally impossible with 4 hex digits,
        // but explicit per BMP-only discipline).
        if (c as u32) > 0xFFFF {
            return Err(LexError { position: start, kind: LexErrorKind::InvalidChar(
                format!(
                    "\\{}: supplementary-plane codepoint U+{:X} not supported; \
                     wat char literals are BMP-only",
                    body, c as u32
                ),
            ) });
        }
        return Ok((c, pos));
    }

    // 3. Single character (`\a`, `\1`, etc.)?
    let mut chars = body.chars();
    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            return Ok((c, pos));
        }
    }

    // Unrecognised body (e.g. `\xyz`, `\newlines`).
    Err(LexError { position: start, kind: LexErrorKind::InvalidChar(
        format!("unrecognised char literal \\{}", body),
    ) })
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
/// `'` (apostrophe) is the canonical separator inside keyword bodies
/// for arity suffixes and type discriminators (e.g. `:wat::core::op'2`,
/// `:wat::core::op'i64'i64`). It also appears as a primed type-head
/// suffix: `:wat::kernel::Thread<I,O>` — the `'` marks the primed
/// variant and is immediately followed by `<` opening the generic params.
/// `,` is rejected with `LexError::CommaInKeywordBody` at ANY depth
/// (arc 109 "the comma dies in the reader" — closes the depth ≥ 1
/// carve-out arc 171 left open for `(...)`/`<...>`). A comma can never
/// enter a keyword body again. Whitespace inside an unclosed `(` or
/// `<` is an error. `"` and `;` terminate the keyword — they never
/// appear inside one.
fn lex_keyword(src: &str, start: usize) -> Result<(String, usize), LexError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[start] as char, ':');
    let mut out = String::new();
    out.push(':');
    let mut i = start + 1;
    let mut paren_depth = 0i32;
    // Arc 072 — track `<>` depth alongside `()` so type-keyword
    // expressions like `:Result<(i64,i64),i64>` and (with the
    // user's intuitive whitespace) `:Result<(i64,i64), i64>` don't
    // silently truncate at the space inside the brackets.
    //
    // Operator `<` / `>` (e.g., `:wat::core::<`, `:wat::core::>=`)
    // appear in keyword paths AFTER `::` and must NOT be treated as
    // bracket openers. Disambiguation: `<` increments depth only
    // when preceded by an alphanumeric or `_` (a type-head name like
    // `Result<` or `Vec<`) or by `'` (a primed type head like
    // `Thread'<` — arc 214). `>` decrements only when angle_depth >
    // 0. The `'<` combination is unambiguous: operator `<` always
    // follows `::`, and arc-171 discriminator apostrophes come AFTER
    // an op name (`<'2`, `op'i64'i64`) — so `'` before `<` can only
    // open the params of a primed type head. Pre-arc-072 the lexer
    // ignored angle brackets entirely, so whitespace inside `<...>`
    // truncated the keyword and the downstream type checker saw a
    // malformed Result with one arg — surfacing as opaque
    // "fresh-var unsolved" errors at pattern-arm sites.
    let mut angle_depth = 0i32;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c.is_whitespace() {
            if paren_depth > 0 || angle_depth > 0 {
                return Err(LexError { position: i, kind: LexErrorKind::UnclosedBracketInKeyword });
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
            // Arc 167 slice 2 follow-on — `]` terminates a keyword
            // when no `<...>` or `(...)` is currently open. The
            // flat-shape fn signature `[x <- :T]` puts a keyword
            // immediately before `]`; without this break, the
            // keyword silently absorbs the `]` and the parser sees
            // an unclosed bracket. Mirrors the `)` early-break above.
            ']' => {
                if angle_depth > 0 || paren_depth > 0 {
                    // Inside a parametric type expression — `]` is
                    // not legal here in current keyword grammar, but
                    // preserve the byte so downstream parsing
                    // surfaces a clearer error than a silent break.
                    out.push(c);
                } else {
                    // Unmatched `]` — belongs to the enclosing
                    // vector form.
                    break;
                }
            }
            '[' => {
                // `[` is not legal inside a keyword (no parametric
                // type uses `[`). Same treatment as `]`: when no
                // bracket grouping is open, the `[` belongs to the
                // enclosing form (a vector opener following an
                // unspaced keyword).
                if angle_depth > 0 || paren_depth > 0 {
                    out.push(c);
                } else {
                    break;
                }
            }
            // Arc 169 slice 1 — same treatment as `[` / `]`. `{`
            // and `}` are not legal inside a keyword (no parametric
            // type uses braces); when no `(...)` / `<...>` grouping
            // is open, a brace belongs to the enclosing form.
            '{' | '}' => {
                if angle_depth > 0 || paren_depth > 0 {
                    out.push(c);
                } else {
                    break;
                }
            }
            '<' => {
                // Type-head `<` follows an alphanumeric (`Result<`,
                // `Vec<`, ...) or an apostrophe on a primed type head
                // (`Thread'<`, `Process'<`). Operator `<` follows `::`
                // — the previous emitted char is `:`, never alphanumeric
                // or `'` for a path. Use the last char in `out` to
                // decide.
                //
                // Arc 214 — `'` is valid as a type-head-final char:
                // `Thread'<I,O>` has `'` immediately before `<`. This is
                // unambiguous: operator `<` in a keyword path always
                // follows `::` (`:wat::core::<`), and arc-171
                // discriminator apostrophes come AFTER an op name
                // (`<'2`, `op'i64'i64`) — so `'<` can only be a primed
                // type head opening its params.
                let prev_alpha = out
                    .chars()
                    .last()
                    .map(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
                    .unwrap_or(false);
                if prev_alpha {
                    angle_depth += 1;
                }
                out.push(c);
            }
            '>' => {
                // Closes a previously-opened type-head `<`. Operator
                // `>` and `>=` would have left angle_depth at 0
                // because their `<`/`>` followed `::`.
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
                out.push(c);
            }
            '"' | ';' => {
                // These never appear inside a keyword.
                break;
            }
            ',' => {
                // Arc 109 "the comma dies in the reader" — a comma can
                // never enter a keyword body again, at ANY depth. Arc
                // 171 already killed it at depth 0 (the arity/discriminator
                // separator shape); this retires the depth ≥ 1 carve-out
                // that survived for parametric types (`:HashMap<K,V>`)
                // and tuple/fn types (`:(A,B,C)`, `:fn(T,U)->R`) — those
                // move to the `:-` binder form and `[T U :-> R]` arrow
                // form respectively (both already live in the stdlib:
                // `wat/cache.wat`, `wat/spawn.wat`). Outside a keyword
                // body, `,` remains ordinary EDN whitespace — unaffected.
                return Err(LexError { position: i, kind: LexErrorKind::CommaInKeywordBody });
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
    // Rational literal: `<int>/<int>` (arc 300 stone B). `raw` is already
    // the whole `"1/2"` span (the scan above stopped at a symbol-break
    // char, and `/` is not one — see `is_symbol_break`). Mirrors Stone A's
    // DONE normalization at the data layer (`wat-edn`'s `lex_number` +
    // parser): split on `/`, parse both sides as `BigInt`, reduce via
    // `BigRational`, den==1 reduces to an Integer (clj Long), `/0` is a
    // clean error, never a panic.
    if let Some(slash) = raw.find('/') {
        let (numer_s, rest) = raw.split_at(slash);
        let denom_s = &rest[1..];
        if !numer_s.is_empty()
            && !denom_s.is_empty()
            && numer_s.bytes().all(|b| b.is_ascii_digit() || b == b'-' || b == b'+')
            && denom_s.bytes().all(|b| b.is_ascii_digit())
        {
            if let (Ok(numer), Ok(denom)) =
                (numer_s.parse::<num_bigint::BigInt>(), denom_s.parse::<num_bigint::BigInt>())
            {
                if denom == num_bigint::BigInt::from(0) {
                    return Err(LexError {
                        position: start,
                        kind: LexErrorKind::InvalidNumber("divide by zero".to_string()),
                    });
                }
                let ratio = BigRational::new(numer, denom);
                if ratio.is_integer() {
                    let n = ratio.numer();
                    return match n.to_string().parse::<i64>() {
                        Ok(v) => Ok((Token::Int(v), i)),
                        Err(_) => Err(LexError {
                            position: start,
                            kind: LexErrorKind::InvalidNumber(format!(
                                "{} exceeds i64 (runtime BigInt out of scope)",
                                n
                            )),
                        }),
                    };
                }
                return Ok((Token::Rational(ratio), i));
            }
        }
    }
    // BigInt literal: `<int>N` suffix (arc 300 stone C1). Mirrors wat-edn's
    // `N`-suffix lexing (`crates/wat-edn/src/lexer.rs`'s `Token::BigInt`
    // branch): strip the trailing `N`, parse the body as `num_bigint::BigInt`.
    // Never reduces to `Token::Int` — `1N` is always bigint (contrast the
    // `/` rational path above, which DOES reduce den==1 to Integer).
    if let Some(body) = raw.strip_suffix('N') {
        let digits = body.strip_prefix('-').unwrap_or(body);
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            if let Ok(n) = body.parse::<num_bigint::BigInt>() {
                return Ok((Token::BigInt(n), i));
            }
        }
    }
    // Then float.
    if let Ok(x) = raw.parse::<f64>() {
        return Ok((Token::Float(x), i));
    }
    Err(LexError { position: start, kind: LexErrorKind::InvalidNumber(raw.to_string()) })
}

/// Lex a bare symbol (including bools `true` / `false`, which the caller
/// re-classifies).
fn lex_symbol(src: &str, start: usize) -> (String, usize) {
    let bytes = src.as_bytes();
    let mut i = start;
    // Arc 271 — track `<...>` depth so a multi-type-param generic method name
    // (a bare Symbol like `combine<A,B>`) keeps the comma that EDN treats as
    // whitespace (`is_symbol_break`). This mirrors `lex_keyword`'s angle handling
    // (which is why generic FNS `:foldl<T,Acc>` — keyword names — already worked):
    // `<` opens a type-head only when preceded by an alphanumeric / `_` / `'`
    // (`make<`, `Thread'<`), NEVER for a leading/operator `<` (`<-`, `<`, `<=`),
    // so binder/arrow symbols are unaffected. While `angle_depth > 0`, a comma is
    // retained instead of breaking the scan; at depth 0 it breaks as before.
    let mut angle_depth = 0i32;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '<' {
            let prev_type_head = i > start && {
                let p = bytes[i - 1] as char;
                p.is_ascii_alphanumeric() || p == '_' || p == '\''
            };
            if prev_type_head {
                angle_depth += 1;
            }
            i += 1;
        } else if c == '>' {
            if angle_depth > 0 {
                angle_depth -= 1;
            }
            i += 1;
        } else if c == ',' && angle_depth > 0 {
            // Inside `<...>` the comma separates type params — keep it.
            i += 1;
        } else if is_symbol_break(c) {
            break;
        } else {
            i += 1;
        }
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
            Err(LexError { kind: LexErrorKind::UnterminatedString, .. })
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
        // `:HashMap<K,V>` and `:fn(T,U)->R` — the comma-carrying
        // multi-param shapes — are retired (arc 109); see
        // `keyword_comma_in_angle_brackets_rejected` / `keyword_fn_type_with_arrow`.
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
            lex_tokens(":wat::load-file!").unwrap(),
            vec![Token::Keyword(":wat::load-file!".into())]
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
        // :( opens a tuple-literal type expression. The comma-carrying
        // multi-element shape is retired (arc 109) — see
        // `keyword_comma_in_parens_rejected`; destination is the `:-`
        // binder form, `(:wat::core::Tuple :- [T1 T2])`.
        assert!(matches!(
            lex_tokens(":(i64,String)"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
        assert!(matches!(
            lex_tokens(":(Holon,wat::holon::HolonAST,Holon)"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
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
        // `:fn(T,U)->R` — retired (arc 109): a comma can never enter a
        // keyword body, so a multi-arg legacy fn type now rejects at the
        // first comma. Destination: `[T U :-> R]` (wat/cache.wat,
        // wat/spawn.wat already use this arrow form).
        assert!(matches!(
            lex_tokens(":fn(T,U)->R"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
    }

    #[test]
    fn keyword_nested_parametric_with_fn_type() {
        // `:HashMap<String,fn(i32)->i32>` — both the outer `<>` comma and
        // the nested `fn(...)` comma are retired (arc 109); the first one
        // reached (the outer `<>` comma) is what fires.
        assert!(matches!(
            lex_tokens(":HashMap<String,fn(i32)->i32>"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
    }

    // ─── Arc 171 slice 1 — apostrophe as keyword-body separator ──────────

    #[test]
    fn keyword_apostrophe_arity_suffix() {
        // A — `:wat::core::op'2` parses as a single keyword.
        // Apostrophe falls through to the _ arm in lex_keyword (no
        // functional change needed; this test proves it and pins the
        // behaviour).
        assert_eq!(
            lex_tokens(":wat::core::op'2").unwrap(),
            vec![Token::Keyword(":wat::core::op'2".into())]
        );
    }

    #[test]
    fn keyword_apostrophe_multi_discriminator() {
        // B — `:wat::core::op'i64'i64` parses as a single keyword
        // (multi-apostrophe; both separators absorbed in one token).
        assert_eq!(
            lex_tokens(":wat::core::op'i64'i64").unwrap(),
            vec![Token::Keyword(":wat::core::op'i64'i64".into())]
        );
    }

    #[test]
    fn keyword_apostrophe_full_op_table() {
        // arc 237 Stone 237.8a — mixed-type op variants (cross-numeric)
        // DELETED under THE DECISION (`feedback_no_implicit_coercion`).
        // Only the same-type variants (op'f64'f64, op'i64'i64) remain
        // as lexer-level test coverage.
        {
            let kw = &":wat::core::op'f64'f64";
            assert_eq!(
                lex_tokens(kw).unwrap(),
                vec![Token::Keyword((*kw).into())]
            );
        }
    }

    #[test]
    fn keyword_apostrophe_after_parametric_close() {
        // D — apostrophe after `>` (outside `<...>`) is pushed as-is.
        // Arc 109 retired the comma-carrying multi-param spelling; use a
        // single-param generic (no comma) to keep exercising the
        // apostrophe-after-close mechanic this test is actually about.
        assert_eq!(
            lex_tokens(":HashMap<String>'snapshot").unwrap(),
            vec![Token::Keyword(":HashMap<String>'snapshot".into())]
        );
    }

    // ─── Arc 214 — primed type-head with generic params ──────────────────

    #[test]
    fn keyword_primed_generic_single_param() {
        // Arc 214 fix: `'` before `<` is a valid type-head-final char.
        // `:wat::kernel::Thread'<I>` lexes as a single keyword. (The
        // two-param comma-carrying form this test used pre-arc-109 is now
        // covered by `keyword_comma_in_angle_brackets_rejected` below —
        // it must ERROR, not lex clean.)
        assert_eq!(
            lex_tokens(":wat::kernel::Thread'<wat::core::i64>").unwrap(),
            vec![Token::Keyword(":wat::kernel::Thread'<wat::core::i64>".into())]
        );
    }

    #[test]
    fn keyword_unprimed_generic_single_param_control() {
        // Control: unprimed single-param generic, comma-free.
        assert_eq!(
            lex_tokens(":wat::kernel::Thread<wat::core::nil>").unwrap(),
            vec![Token::Keyword(":wat::kernel::Thread<wat::core::nil>".into())]
        );
    }

    #[test]
    fn keyword_comma_in_body_rejected_depth_0() {
        // E — `,N` style is rejected at depth 0 (arc 171 closure, unchanged
        // by arc 109). Comma as keyword-body separator retired; `'` is
        // canonical.
        assert!(matches!(
            lex_tokens(":wat::core::op,2"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
        assert!(matches!(
            lex_tokens(":foo,bar"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
    }

    #[test]
    fn keyword_comma_in_parens_rejected() {
        // Arc 109 "the comma dies in the reader" — the `:(A,B,C)` tuple
        // shape used to lex clean (comma valid at paren_depth > 0). Now
        // ANY comma in a keyword body is CommaInKeywordBody, regardless
        // of depth. Destination: `(:wat::core::Tuple :- [T1 T2 T3])`.
        assert!(matches!(
            lex_tokens(":(i64,String)"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
    }

    #[test]
    fn keyword_comma_in_angle_brackets_rejected() {
        // Arc 109 — same retirement for the `:HashMap<K,V>` parametric
        // shape (comma valid at angle_depth > 0 pre-arc-109). Destination:
        // `:HashMap :- [K V]` (angle-brackets-to-binder, arc 109 stone ③).
        assert!(matches!(
            lex_tokens(":HashMap<K,V>"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
        assert!(matches!(
            lex_tokens(":wat::kernel::Thread<wat::core::i64,wat::core::i64>"),
            Err(LexError { kind: LexErrorKind::CommaInKeywordBody, .. })
        ));
    }

    #[test]
    fn keyword_single_param_generic_and_tuple_still_lex() {
        // Additive-refusal check (STOP-2): a comma-free bracketed keyword
        // body — the shapes that never relied on the retired permission —
        // is untouched.
        assert_eq!(
            lex_tokens(":Vec<i64>").unwrap(),
            vec![Token::Keyword(":Vec<i64>".into())]
        );
        assert_eq!(
            lex_tokens(":(i64)").unwrap(),
            vec![Token::Keyword(":(i64)".into())]
        );
    }

    #[test]
    fn keyword_apostrophe_only_no_body() {
        // F — `:'foo` is the shortest apostrophe-inside-keyword case.
        // `'` is not a terminator, so it becomes part of the keyword
        // body; `foo` continues. Honest delta: no quote-shorthand
        // collision because `lex_keyword` is only entered after `:` is
        // already consumed by the main lex loop; `'` seen here is
        // always body content, never a Lisp/Clojure quote reader-macro.
        assert_eq!(
            lex_tokens(":'foo").unwrap(),
            vec![Token::Keyword(":'foo".into())]
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

    // ─── Arc 172 slice 1 — Clojure-style tilde unquote; comma as whitespace ──

    #[test]
    fn tilde_produces_unquote() {
        // A — `~foo` → Unquote token followed by Symbol("foo").
        // Source character changed from `,` to `~` (arc 172 slice 1);
        // Token::Unquote variant name unchanged.
        assert_eq!(
            lex_tokens("~foo").unwrap(),
            vec![Token::Unquote, Token::Symbol("foo".into())]
        );
    }

    #[test]
    fn tilde_at_produces_unquote_splicing() {
        // B — `~@xs` → UnquoteSplicing token followed by Symbol("xs").
        // Source characters changed from `,@` to `~@` (arc 172 slice 1);
        // Token::UnquoteSplicing variant name unchanged.
        assert_eq!(
            lex_tokens("~@xs").unwrap(),
            vec![Token::UnquoteSplicing, Token::Symbol("xs".into())]
        );
    }

    #[test]
    fn comma_is_whitespace_top_level() {
        // C — `(a , b)` → same as `(a b)`: comma between elements is whitespace.
        assert_eq!(
            lex_tokens("(a , b)").unwrap(),
            vec![
                Token::LParen,
                Token::Symbol("a".into()),
                Token::Symbol("b".into()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn comma_is_whitespace_inside_list() {
        // D — `(a, b, c)` → `(a b c)`: trailing comma after each element.
        assert_eq!(
            lex_tokens("(a, b, c)").unwrap(),
            vec![
                Token::LParen,
                Token::Symbol("a".into()),
                Token::Symbol("b".into()),
                Token::Symbol("c".into()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn old_comma_unquote_no_longer_works() {
        // E — `` `(a ,b c) `` parses as quasiquote-of-list with bare symbols.
        // Comma is whitespace; `,b` is just `b`.
        // This test verifies lexer-level token stream: backtick, lparen, a, b, c, rparen.
        assert_eq!(
            lex_tokens("`(a ,b c)").unwrap(),
            vec![
                Token::Quasiquote,
                Token::LParen,
                Token::Symbol("a".into()),
                Token::Symbol("b".into()),
                Token::Symbol("c".into()),
                Token::RParen,
            ]
        );
    }

    // ─── Stone 249 scope-closure — control character rejection ───────────────
    //
    // The env-key separator is U+0001 (SOH). The lexer enforces the invariant
    // "no identifier name contains U+0001" by REJECTING all raw control
    // characters at the lex dispatch point. These tests prove the enforcement.

    #[test]
    fn control_character_u0001_in_source_is_rejected() {
        // U+0001 (SOH) — the env-key separator byte — must be rejected.
        // If the lexer absorbed it into a symbol name the separator invariant
        // would collapse; ENFORCEMENT via rejection is the strongest rung.
        let src = "\u{1}foo";
        assert!(
            matches!(
                lex_tokens(src),
                Err(LexError {
                    kind: LexErrorKind::ControlCharacterInSource { codepoint: 1 },
                    ..
                })
            ),
            "U+0001 (SOH) in source must produce ControlCharacterInSource"
        );
    }

    #[test]
    fn control_character_general_rejected() {
        // A range of other C0 control characters (BEL, BS, ESC, FS, GS, RS, US)
        // must also be rejected.
        for &cp in &[0x07u32, 0x08, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F] {
            let src = format!("{}x", char::from_u32(cp).unwrap());
            assert!(
                matches!(
                    lex_tokens(&src),
                    Err(LexError {
                        kind: LexErrorKind::ControlCharacterInSource { .. },
                        ..
                    })
                ),
                "control character U+{:04X} must be rejected", cp
            );
        }
    }

    #[test]
    fn permitted_whitespace_still_lexes_fine() {
        // \t (0x09), \n (0x0A), \r (0x0D) are structural whitespace — they must
        // NOT be rejected by the control-character gate. Consuming them as
        // whitespace is sufficient; the resulting token stream should be empty.
        assert_eq!(lex_tokens("\t\n\r").unwrap(), vec![]);
        // They also work inside token sequences.
        assert_eq!(
            lex_tokens("(\t)\n(\r)").unwrap(),
            vec![
                Token::LParen, Token::RParen,
                Token::LParen, Token::RParen,
            ]
        );
    }
}
