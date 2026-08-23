//! Hand-rolled byte-level EDN lexer.
//!
//! Single-pass over the input. Tokens borrow string bodies from the
//! input where possible; only escape sequences in strings force
//! allocation. Comma is whitespace per spec — including inside a
//! keyword body: `,` never continues a keyword body, at any bracket
//! depth. (Arc 109 "the comma dies in the reader" — retires the
//! position-aware `,`-as-body-continue / `_`-as-wire-escape carve-out
//! that arc 170 slice 1f-W introduced; see git history for the prior
//! shape.)

use crate::error::{Error, ErrorKind, Result};
use crate::vocab::{
    decode_string_escape, hex_value, is_symbol_continue, is_symbol_start, is_whitespace,
    name_to_char,
};
use std::borrow::Cow;

/// Tokens emitted by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Nil,
    True,
    False,
    Integer(i64),
    Float(f64),
    /// Big integer (raw digits, optional leading sign, no `N` suffix).
    BigInt(&'a str),
    /// Big decimal (raw decimal, no `M` suffix).
    BigDec(&'a str),
    /// Rational literal (`<int>/<int>`): raw numerator digits (optional
    /// leading sign, same body a lone `Integer`/`BigInt` token would
    /// carry) and raw denominator digits (plain digit run, no sign —
    /// the EDN ratio grammar allows a sign only on the numerator).
    /// The lexer has already refused an all-zero denominator (`1/0`,
    /// `-5/0`) with a clean `Err`, so a `Rational` token's `denom` is
    /// guaranteed non-zero here — the parser reduces + normalizes
    /// (`4/2` → `Integer(2)`, `1/2` stays a `Rational`) without a
    /// zero-denominator case to guard against.
    Rational { numer: &'a str, denom: &'a str },
    String(Cow<'a, str>),
    Char(char),
    /// Keyword body (no leading `:`) plus its start-of-body byte
    /// offset in the original input (the `:` is at `body_start - 1`).
    ///
    /// Borrowed `Cow<'a, str>` is the fast path (zero-copy slice into
    /// `input`); non-ASCII scalars are the only case that force an
    /// owned body.
    ///
    /// `body_start` is kept so callers can compute a span pointing at
    /// a specific byte in the body even when the parser has already
    /// peeked past the token.
    Keyword { body: Cow<'a, str>, body_start: usize },
    /// Symbol body.
    Symbol(&'a str),
    /// Tag body (no leading `#`).
    Tag(&'a str),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    HashLBrace,
    HashUnderscore,
    Eof,
}

/// Hand-rolled byte-level lexer. Stored as `&[u8]`, not `&str`:
/// dispatch is byte-level and hot, multi-byte UTF-8 only matters at
/// slice extraction (validated then) and inside `\` char-literal
/// handling.
pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b) if is_whitespace(b) => self.pos += 1,
                Some(b';') => {
                    while let Some(b) = self.peek() {
                        self.pos += 1;
                        if b == b'\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>> {
        self.skip_trivia();
        let b = match self.peek() {
            None => return Ok(Token::Eof),
            Some(b) => b,
        };

        match b {
            b'(' => { self.pos += 1; Ok(Token::LParen) }
            b')' => { self.pos += 1; Ok(Token::RParen) }
            b'[' => { self.pos += 1; Ok(Token::LBracket) }
            b']' => { self.pos += 1; Ok(Token::RBracket) }
            b'{' => { self.pos += 1; Ok(Token::LBrace) }
            b'}' => { self.pos += 1; Ok(Token::RBrace) }
            b'"' => self.lex_string(),
            b'\\' => self.lex_char(),
            b':' => self.lex_keyword(),
            b'#' => self.lex_hash(),
            b'-' | b'+' => self.lex_signed(),
            b'0'..=b'9' => {
                let start = self.pos;
                self.lex_number(start)
            }
            // Non-ASCII lead byte in token-start position — admit it as a
            // symbol start (clj-parity: `clojure.edn` reads `😀`/`é`/`λ`/
            // `foo→bar` as Symbols). Previously dead-ended at `UnexpectedByte`;
            // `lex_symbol` now decodes the scalar (mirrors the `decode_utf8_char`
            // pattern the char-literal path already uses below).
            _ if is_symbol_start(b) || b >= 0x80 => {
                let start = self.pos;
                self.lex_symbol(start)
            }
            _ => Err(Error::at(self.pos, ErrorKind::UnexpectedByte(b))),
        }
    }

    // ─── Strings ────────────────────────────────────────────────

    fn lex_string(&mut self) -> Result<Token<'a>> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        let quote_start = self.pos;
        self.pos += 1;
        let body_start = self.pos;

        // Fast path: scan to closing quote without escapes.
        loop {
            match self.peek() {
                None => return Err(Error::at(quote_start, ErrorKind::UnclosedString)),
                Some(b'"') => {
                    let body = &self.input[body_start..self.pos];
                    let s = std::str::from_utf8(body)
                        .map_err(|e| Error::at(body_start, ErrorKind::Utf8(e.to_string())))?;
                    self.pos += 1;
                    return Ok(Token::String(Cow::Borrowed(s)));
                }
                Some(b'\\') => return self.lex_string_escaped(quote_start, body_start),
                Some(_) => self.pos += 1,
            }
        }
    }

    fn lex_string_escaped(&mut self, quote_start: usize, body_start: usize) -> Result<Token<'a>> {
        let mut out = String::with_capacity(self.pos - body_start);
        let prefix = std::str::from_utf8(&self.input[body_start..self.pos])
            .map_err(|e| Error::at(body_start, ErrorKind::Utf8(e.to_string())))?;
        out.push_str(prefix);

        loop {
            let chunk_start = self.pos;
            while let Some(b) = self.peek() {
                if b == b'"' || b == b'\\' {
                    break;
                }
                self.pos += 1;
            }
            if self.pos > chunk_start {
                let chunk = std::str::from_utf8(&self.input[chunk_start..self.pos])
                    .map_err(|e| Error::at(chunk_start, ErrorKind::Utf8(e.to_string())))?;
                out.push_str(chunk);
            }

            match self.peek() {
                None => return Err(Error::at(quote_start, ErrorKind::UnclosedString)),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(Token::String(Cow::Owned(out)));
                }
                Some(b'\\') => {
                    self.pos += 1;
                    self.process_escape(&mut out)?;
                }
                _ => unreachable!(),
            }
        }
    }

    fn process_escape(&mut self, out: &mut String) -> Result<()> {
        let escape_byte = self
            .advance()
            .ok_or_else(|| Error::at(self.pos, ErrorKind::UnclosedString))?;
        if let Some(c) = decode_string_escape(escape_byte) {
            out.push(c);
            return Ok(());
        }
        if escape_byte == b'u' {
            let acc = self.read_hex4()?;
            let c = char::from_u32(acc).ok_or_else(|| {
                Error::at(
                    self.pos,
                    ErrorKind::InvalidUnicode(format!("U+{:04X} is not a scalar value", acc)),
                )
            })?;
            out.push(c);
            return Ok(());
        }
        Err(Error::at(self.pos - 1, ErrorKind::InvalidEscape(escape_byte)))
    }

    fn read_hex4(&mut self) -> Result<u32> {
        let mut codepoint = 0u32;
        for _ in 0..4 {
            let h = self
                .advance()
                .ok_or_else(|| Error::at(self.pos, ErrorKind::InvalidUnicode("\\u truncated".into())))?;
            let v = hex_value(h).ok_or_else(|| {
                Error::at(
                    self.pos - 1,
                    ErrorKind::InvalidUnicode(format!("non-hex byte 0x{:02x}", h)),
                )
            })?;
            codepoint = (codepoint << 4) | (v as u32);
        }
        Ok(codepoint)
    }

    // ─── Characters ─────────────────────────────────────────────

    fn lex_char(&mut self) -> Result<Token<'a>> {
        debug_assert_eq!(self.peek(), Some(b'\\'));
        let start = self.pos;
        self.pos += 1;
        let body_start = self.pos;

        // Spec: "Backslash cannot be followed by whitespace."
        // Capture `first` here to avoid a second peek below.
        let first = match self.peek() {
            None => return Err(Error::at(start, ErrorKind::InvalidChar("empty".into()))),
            Some(b) if is_whitespace(b) => {
                return Err(Error::at(
                    start,
                    ErrorKind::InvalidChar("backslash followed by whitespace".into()),
                ))
            }
            Some(b) => b,
        };

        // Single non-alpha non-digit character (`\(`, `\;`, `\é`, etc.).
        // Alphanumeric bodies fall through to the named-char path below
        // (where `\newline`, `\space`, `\a`, `\1` resolve uniformly).
        if !first.is_ascii_alphanumeric() {
            let (c, byte_len) = decode_utf8_char(&self.input[self.pos..])
                .map_err(|e| Error::at(self.pos, ErrorKind::Utf8(e)))?;
            if (c as u32) > 0xFFFF {
                return Err(Error::at(
                    start,
                    ErrorKind::InvalidChar(format!(
                        "\\{}: supplementary-plane (U+{:X}) not supported; \
                         wat-edn char literals are BMP-only",
                        c, c as u32
                    )),
                ));
            }
            self.pos += byte_len;
            return Ok(Token::Char(c));
        }

        // Read alphanumeric body (a name like "newline", "u00A0", or single letter)
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() {
                self.pos += 1;
            } else {
                break;
            }
        }

        let body = &self.input[body_start..self.pos];
        let body_str = std::str::from_utf8(body)
            .map_err(|e| Error::at(body_start, ErrorKind::Utf8(e.to_string())))?;

        // 1. Named char literal? (`\newline`, `\space`, etc. — spec set
        //    plus extensions, all owned by vocab::NAMED_CHARS.)
        if let Some(c) = name_to_char(body_str) {
            return Ok(Token::Char(c));
        }
        // 2. `\uNNNN` Unicode escape?
        if body_str.len() == 5 && body_str.starts_with('u') {
            let acc = u32::from_str_radix(&body_str[1..], 16).map_err(|_| {
                Error::at(start, ErrorKind::InvalidChar(format!("\\{}", body_str)))
            })?;
            let c = char::from_u32(acc).ok_or_else(|| {
                Error::at(start, ErrorKind::InvalidChar(format!("\\{}: not a scalar", body_str)))
            })?;
            return Ok(Token::Char(c));
        }
        // 3. Single character? (`\a`, `\1`, etc.) — one iterator walk.
        let mut it = body_str.chars();
        if let Some(c) = it.next() {
            if it.next().is_none() {
                if (c as u32) > 0xFFFF {
                    return Err(Error::at(
                        start,
                        ErrorKind::InvalidChar(format!(
                            "\\{}: supplementary-plane (U+{:X}) not supported; \
                             wat-edn char literals are BMP-only",
                            body_str, c as u32
                        )),
                    ));
                }
                return Ok(Token::Char(c));
            }
        }
        Err(Error::at(start, ErrorKind::InvalidChar(body_str.into())))
    }

    // ─── Keywords ───────────────────────────────────────────────

    fn lex_keyword(&mut self) -> Result<Token<'a>> {
        debug_assert_eq!(self.peek(), Some(b':'));
        let start = self.pos;
        self.pos += 1;
        let body_start = self.pos;

        // Per spec: keyword cannot begin with `::`
        if self.peek() == Some(b':') {
            return Err(Error::at(
                start,
                ErrorKind::InvalidKeyword("keyword begins with :: ".into()),
            ));
        }

        // Comma is never a body-continue char: EDN whitespace rules
        // apply uniformly inside a keyword body, at any bracket depth.
        // (Arc 109 — the comma dies in the reader.)
        while let Some(b) = self.peek() {
            // Non-ASCII scalar: clj-parity admits it as a keyword-body
            // constituent (`:λ`, `:a😀`). Decode it (same
            // `decode_utf8_char` pattern as the char-literal and symbol
            // paths) so `self.pos` lands on the next char boundary, then
            // advance by its full byte width.
            if b >= 0x80 {
                let (_, byte_len) = decode_utf8_char(&self.input[self.pos..])
                    .map_err(|e| Error::at(self.pos, ErrorKind::Utf8(e)))?;
                self.pos += byte_len;
                continue;
            }
            if !is_symbol_continue(b) {
                break;
            }
            self.pos += 1;
        }

        if self.pos == body_start {
            return Err(Error::at(start, ErrorKind::InvalidKeyword("empty".into())));
        }

        let body = std::str::from_utf8(&self.input[body_start..self.pos])
            .map_err(|e| Error::at(body_start, ErrorKind::Utf8(e.to_string())))?;
        Ok(Token::Keyword { body: Cow::Borrowed(body), body_start })
    }

    // ─── Symbols ────────────────────────────────────────────────

    fn lex_symbol(&mut self, start: usize) -> Result<Token<'a>> {
        let first = self
            .peek()
            .ok_or_else(|| Error::at(start, ErrorKind::InvalidSymbol("empty".into())))?;

        if first >= 0x80 {
            // Non-ASCII lead byte: clj-parity admits any Unicode scalar as a
            // symbol-start char (`😀`, `é`, `λ`). Decode the full scalar
            // (same `decode_utf8_char` pattern as the char-literal path
            // above) so `self.pos` lands on the next char boundary, never
            // mid-sequence.
            let (_, byte_len) = decode_utf8_char(&self.input[self.pos..])
                .map_err(|e| Error::at(self.pos, ErrorKind::Utf8(e)))?;
            self.pos += byte_len;
        } else if !is_symbol_start(first) {
            return Err(Error::at(
                start,
                ErrorKind::InvalidSymbol(format!("invalid first byte 0x{:02x}", first)),
            ));
        } else {
            self.pos += 1;
        }

        // Spec: "If `-`, `+` or `.` are the first character, the second
        // character (if any) must be non-numeric." (`-N` / `+N` are routed
        // to lex_number via lex_signed; the `.` case lands here.)
        if matches!(first, b'-' | b'+' | b'.')
            && matches!(self.peek(), Some(b'0'..=b'9'))
        {
            return Err(Error::at(
                start,
                ErrorKind::InvalidSymbol(format!(
                    "{} cannot be followed by a digit",
                    first as char
                )),
            ));
        }

        while let Some(b) = self.peek() {
            if is_symbol_continue(b) {
                self.pos += 1;
            } else if b >= 0x80 {
                // Non-ASCII continuation scalar (clj-parity: `foo→bar` is one
                // Symbol). Decode + advance by the full char width.
                let (_, byte_len) = decode_utf8_char(&self.input[self.pos..])
                    .map_err(|e| Error::at(self.pos, ErrorKind::Utf8(e)))?;
                self.pos += byte_len;
            } else {
                break;
            }
        }

        let body = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|e| Error::at(start, ErrorKind::Utf8(e.to_string())))?;

        match body {
            "nil" => Ok(Token::Nil),
            "true" => Ok(Token::True),
            "false" => Ok(Token::False),
            _ => Ok(Token::Symbol(body)),
        }
    }

    fn lex_signed(&mut self) -> Result<Token<'a>> {
        // `-` or `+` followed by digit is a number; otherwise symbol.
        let start = self.pos;
        let next = self.peek_at(1);
        if matches!(next, Some(b'0'..=b'9')) {
            self.lex_number(start)
        } else {
            self.lex_symbol(start)
        }
    }

    // ─── Hash dispatch ─────────────────────────────────────────

    fn lex_hash(&mut self) -> Result<Token<'a>> {
        debug_assert_eq!(self.peek(), Some(b'#'));
        self.pos += 1;
        let next = match self.peek() {
            None => return Err(Error::at(self.pos - 1, ErrorKind::InvalidTag("# at EOF".into()))),
            Some(n) => n,
        };

        match next {
            b'{' => {
                self.pos += 1;
                Ok(Token::HashLBrace)
            }
            b'_' => {
                self.pos += 1;
                Ok(Token::HashUnderscore)
            }
            b'#' => self.lex_symbolic_value(),
            _ if next.is_ascii_alphabetic() => self.lex_tag(),
            _ => Err(Error::at(self.pos, ErrorKind::InvalidTag(format!("byte 0x{:02x}", next)))),
        }
    }

    /// EDN `##name` symbolic-value reader (clj-parity). `clojure.edn`
    /// recognizes exactly three symbolic values — `##Inf`, `##-Inf`,
    /// `##NaN` — as the IEEE-754 special floats; there is no generic
    /// fallback for an unrecognized `##name` (clj errors on `##foo`), so
    /// wat-edn must too.
    ///
    /// Entry: `self.pos` is on the *second* `#` (the first was already
    /// consumed by `lex_hash`'s dispatch `self.pos += 1`).
    fn lex_symbolic_value(&mut self) -> Result<Token<'a>> {
        let hash_start = self.pos - 1; // span for errors: the first '#'
        debug_assert_eq!(self.peek(), Some(b'#'));
        self.pos += 1; // consume the second '#'
        let name_start = self.pos;
        if matches!(self.peek(), Some(b'-')) {
            self.pos += 1;
        }
        while let Some(b) = self.peek() {
            if is_symbol_continue(b) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let name = std::str::from_utf8(&self.input[name_start..self.pos])
            .map_err(|e| Error::at(name_start, ErrorKind::Utf8(e.to_string())))?;
        match name {
            "Inf" => Ok(Token::Float(f64::INFINITY)),
            "-Inf" => Ok(Token::Float(f64::NEG_INFINITY)),
            "NaN" => Ok(Token::Float(f64::NAN)),
            _ => Err(Error::at(
                hash_start,
                ErrorKind::InvalidTag(format!("## {} is not a legal symbolic value", name)),
            )),
        }
    }

    fn lex_tag(&mut self) -> Result<Token<'a>> {
        let start = self.pos;
        // First char must be alphabetic per spec
        debug_assert!(self.peek().map(|b| b.is_ascii_alphabetic()).unwrap_or(false));
        self.pos += 1;
        while let Some(b) = self.peek() {
            if is_symbol_continue(b) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let body = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|e| Error::at(start, ErrorKind::Utf8(e.to_string())))?;
        Ok(Token::Tag(body))
    }

    // ─── Numbers ────────────────────────────────────────────────

    fn lex_number(&mut self, start: usize) -> Result<Token<'a>> {
        if matches!(self.peek(), Some(b'-' | b'+')) {
            self.pos += 1;
        }

        // Spec: "No integer other than 0 may begin with 0."
        // Reject `01`, `-007`, `+0123`. Allow `0`, `0.5`, `0e10`, `0M`, `0N`.
        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'0'..=b'9')) {
            return Err(Error::at(
                self.pos,
                ErrorKind::InvalidNumber("leading zero".into()),
            ));
        }

        // The dispatcher in next_token only routes here when the leading
        // byte (after optional sign) is a digit, so the digit-required
        // assertion is structural — keep as debug_assert.
        debug_assert!(matches!(self.peek(), Some(b'0'..=b'9')));

        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }

        // Ratio literal: `<int>/<digit+>`. Must be checked before the
        // float `.`/`e` paths below — a ratio has no fractional part,
        // exponent, or `M`/`N` suffix. Only fires when a digit
        // immediately follows the `/`; `1/x` (division-style symbol
        // use) leaves the `/` unconsumed so `next_token` re-dispatches
        // on it as its own token, same as any other punctuation run.
        if self.peek() == Some(b'/') && matches!(self.peek_at(1), Some(b'0'..=b'9')) {
            let numer = std::str::from_utf8(&self.input[start..self.pos])
                .expect("number body is ASCII");
            self.pos += 1; // consume '/'
            let denom_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            let denom = std::str::from_utf8(&self.input[denom_start..self.pos])
                .expect("number body is ASCII");
            // clj: "Divide by zero" — refuse here, before a Value ever
            // exists, so the parser never has to handle a zero denom.
            if denom.bytes().all(|b| b == b'0') {
                return Err(Error::at(
                    denom_start,
                    ErrorKind::InvalidNumber("divide by zero".into()),
                ));
            }
            return Ok(Token::Rational { numer, denom });
        }

        let mut is_float = false;

        if self.peek() == Some(b'.') {
            // Only treat `.` as decimal if followed by a digit (avoids
            // grabbing token-terminating punctuation).
            if matches!(self.peek_at(1), Some(b'0'..=b'9')) {
                is_float = true;
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'-' | b'+')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(Error::at(
                    self.pos,
                    ErrorKind::InvalidNumber("expected digit after exponent".into()),
                ));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        let body = std::str::from_utf8(&self.input[start..self.pos])
            .expect("number body is ASCII");

        // Suffix dispatch. NOTE: lexical-core was tried for f64/i64
        // parsing and benchmarked SLOWER than std on this crate's
        // workload (small numeric tokens; per-call setup cost exceeds
        // savings). std::str::parse stays.
        match self.peek() {
            Some(b'M') => {
                self.pos += 1;
                Ok(Token::BigDec(body))
            }
            Some(b'N') => {
                if is_float {
                    return Err(Error::at(
                        self.pos,
                        ErrorKind::InvalidNumber("N suffix on float".into()),
                    ));
                }
                self.pos += 1;
                Ok(Token::BigInt(body))
            }
            _ => {
                if is_float {
                    let f: f64 = body
                        .parse()
                        .map_err(|_| Error::at(start, ErrorKind::InvalidNumber(body.into())))?;
                    Ok(Token::Float(f))
                } else {
                    let i: i64 = body
                        .parse()
                        .map_err(|_| Error::at(start, ErrorKind::InvalidNumber(body.into())))?;
                    Ok(Token::Integer(i))
                }
            }
        }
    }
}

// Predicate helpers (`is_symbol_start`, `is_symbol_continue`,
// `is_whitespace`, `hex_value`) live in `crate::vocab` so the
// lexer and writer share one source of truth.

fn decode_utf8_char(bytes: &[u8]) -> std::result::Result<(char, usize), String> {
    if bytes.is_empty() {
        return Err("empty input".into());
    }
    let b0 = bytes[0];
    let len = if b0 < 0x80 {
        1
    } else if b0 < 0xC0 {
        return Err(format!("invalid UTF-8 lead byte 0x{:02x}", b0));
    } else if b0 < 0xE0 {
        2
    } else if b0 < 0xF0 {
        3
    } else if b0 < 0xF8 {
        4
    } else {
        return Err(format!("invalid UTF-8 lead byte 0x{:02x}", b0));
    };
    if bytes.len() < len {
        return Err("truncated UTF-8 sequence".into());
    }
    let s = std::str::from_utf8(&bytes[..len]).map_err(|e| e.to_string())?;
    let c = s.chars().next().ok_or("empty char")?;
    Ok((c, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(s: &str) -> Vec<Token<'_>> {
        let mut lx = Lexer::new(s);
        let mut out = Vec::new();
        loop {
            let t = lx.next_token().unwrap();
            if matches!(t, Token::Eof) {
                break;
            }
            out.push(t);
        }
        out
    }

    #[test]
    fn primitives() {
        assert_eq!(lex_all("nil"), vec![Token::Nil]);
        assert_eq!(lex_all("true false"), vec![Token::True, Token::False]);
        assert_eq!(lex_all("42 -7 +3"), vec![Token::Integer(42), Token::Integer(-7), Token::Integer(3)]);
        assert_eq!(lex_all("2.5"), vec![Token::Float(2.5)]);
        assert_eq!(lex_all("1e10"), vec![Token::Float(1e10)]);
        assert_eq!(lex_all("42N"), vec![Token::BigInt("42")]);
        assert_eq!(lex_all("2.5M"), vec![Token::BigDec("2.5")]);
    }

    #[test]
    fn strings() {
        assert_eq!(
            lex_all(r#""hello""#),
            vec![Token::String(std::borrow::Cow::Borrowed("hello"))]
        );
        let toks = lex_all(r#""a\nb""#);
        assert_eq!(toks.len(), 1);
        if let Token::String(c) = &toks[0] {
            assert_eq!(c.as_ref(), "a\nb");
        } else { panic!() }
        let toks = lex_all(r#""é""#);
        if let Token::String(c) = &toks[0] {
            assert_eq!(c.as_ref(), "é");
        } else { panic!() }
    }

    #[test]
    fn keywords_and_symbols() {
        assert_eq!(
            lex_all(":foo"),
            vec![Token::Keyword { body: Cow::Borrowed("foo"), body_start: 1 }]
        );
        assert_eq!(
            lex_all(":ns/foo"),
            vec![Token::Keyword { body: Cow::Borrowed("ns/foo"), body_start: 1 }]
        );
        assert_eq!(lex_all("foo"), vec![Token::Symbol("foo")]);
        assert_eq!(lex_all("ns/foo"), vec![Token::Symbol("ns/foo")]);
        assert_eq!(lex_all("foo.bar/baz"), vec![Token::Symbol("foo.bar/baz")]);
    }

    #[test]
    fn delimiters() {
        assert_eq!(
            lex_all("(){}[]#{}#_"),
            vec![
                Token::LParen, Token::RParen,
                Token::LBrace, Token::RBrace,
                Token::LBracket, Token::RBracket,
                Token::HashLBrace, Token::RBrace,
                Token::HashUnderscore,
            ]
        );
    }

    #[test]
    fn tags() {
        assert_eq!(lex_all("#inst"), vec![Token::Tag("inst")]);
        assert_eq!(lex_all("#myapp/Person"), vec![Token::Tag("myapp/Person")]);
        assert_eq!(
            lex_all("#wat.core/Vec<i64>"),
            vec![Token::Tag("wat.core/Vec<i64>")]
        );
    }

    #[test]
    fn comments_and_commas() {
        let toks = lex_all("1 ; this is a comment\n 2");
        assert_eq!(toks, vec![Token::Integer(1), Token::Integer(2)]);
        let toks = lex_all("[1, 2, 3]");
        assert_eq!(
            toks,
            vec![
                Token::LBracket,
                Token::Integer(1), Token::Integer(2), Token::Integer(3),
                Token::RBracket,
            ]
        );
    }

    #[test]
    fn characters() {
        assert_eq!(lex_all(r"\c"), vec![Token::Char('c')]);
        assert_eq!(lex_all(r"\newline"), vec![Token::Char('\n')]);
        assert_eq!(lex_all(r"\space"), vec![Token::Char(' ')]);
        assert_eq!(lex_all("\\é"), vec![Token::Char('é')]);
    }
}
