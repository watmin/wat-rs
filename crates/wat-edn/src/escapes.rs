//! Single source of truth for spec-level vocabulary shared between
//! the lexer and the writer. Adding a new named char or escape
//! lives here once; both directions update in sync.
//!
//! Found by the /sever ward (vocab duplication finding).

/// Named character literals. Bidirectional: lexer maps name → char,
/// writer maps char → name. Two columns in one table.
///
/// First four are spec-defined (`\space \newline \tab \return`).
/// Last two are wat-edn extensions (Clojure-aligned).
pub const NAMED_CHARS: &[(&str, char)] = &[
    ("newline", '\n'),
    ("space", ' '),
    ("tab", '\t'),
    ("return", '\r'),
    ("formfeed", '\u{000C}'),
    ("backspace", '\u{0008}'),
];

/// Lookup a named char literal: `name_to_char("newline") == Some('\n')`.
pub fn name_to_char(name: &str) -> Option<char> {
    NAMED_CHARS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
}

/// Reverse lookup: `char_to_name('\n') == Some("newline")`.
pub fn char_to_name(c: char) -> Option<&'static str> {
    NAMED_CHARS
        .iter()
        .find(|(_, ch)| *ch == c)
        .map(|(n, _)| *n)
}

/// Decode a single string-escape character (the byte after `\`)
/// into its in-string representation, or `None` if invalid.
///
/// Spec defines: `\t \r \n \\ \"`. wat-edn extends with `\b \f \/`
/// for JSON/Clojure compatibility.
#[inline]
pub fn decode_string_escape(byte: u8) -> Option<char> {
    match byte {
        b'"' => Some('"'),
        b'\\' => Some('\\'),
        b'/' => Some('/'),
        b'n' => Some('\n'),
        b't' => Some('\t'),
        b'r' => Some('\r'),
        b'b' => Some('\u{0008}'),
        b'f' => Some('\u{000C}'),
        _ => None,
    }
}

/// Reverse: encode a char as its `\X` escape sequence (without the
/// leading backslash), or `None` if it can be emitted literally.
///
/// Note: `\/` is NOT emitted on write (spec doesn't require it; it's
/// only accepted on read for JSON-compat). `\u{0008}` and `\u{000C}`
/// emit as `\b` and `\f` (extensions) so round-trip preserves them
/// without `\uXXXX` ceremony.
#[inline]
pub fn encode_string_escape(c: char) -> Option<&'static str> {
    match c {
        '"' => Some("\""),
        '\\' => Some("\\"),
        '\n' => Some("n"),
        '\r' => Some("r"),
        '\t' => Some("t"),
        '\u{0008}' => Some("b"),
        '\u{000C}' => Some("f"),
        _ => None,
    }
}

// ─── Symbol-character predicates ─────────────────────────────────
//
// Spec: "Symbols begin with a non-numeric character and can contain
// alphanumeric characters and `. * + ! - _ ? $ % & = < >`. If `-`,
// `+` or `.` are the first character, the second character (if any)
// must be non-numeric. Additionally, `: #` are allowed as constituent
// characters in symbols other than as the first character."
//
// `/` is also legal as a symbol body: spec allows the bare slash
// symbol AND the prefix-name separator inside a single symbol.

/// True if `b` may begin a symbol body.
#[inline]
pub fn is_symbol_start(b: u8) -> bool {
    b.is_ascii_alphabetic()
        || matches!(
            b,
            b'.' | b'*' | b'+' | b'!' | b'-' | b'_' | b'?' | b'$' | b'%' | b'&' | b'=' | b'<' | b'>' | b'/'
        )
}

/// True if `b` may continue a symbol body (after the first byte).
#[inline]
pub fn is_symbol_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'.' | b'*'
                | b'+'
                | b'!'
                | b'-'
                | b'_'
                | b'?'
                | b'$'
                | b'%'
                | b'&'
                | b'='
                | b'<'
                | b'>'
                | b':'
                | b'#'
                | b'/'
        )
}

/// True if `b` is EDN whitespace. Spec treats commas as whitespace.
#[inline]
pub fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',')
}

/// Hex digit decode: `0..=9 -> 0..=9`, `a..=f|A..=F -> 10..=15`.
#[inline]
pub fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Validate the first character of a symbol/keyword/tag name body.
///
/// Spec: "Symbols begin with a non-numeric character. If `-`, `+` or `.`
/// are the first character, the second character (if any) must be
/// non-numeric."
///
/// Returns the static reason on rejection; the caller wraps it in the
/// appropriate `ErrorKind` variant (Symbol/Keyword/Tag-flavored).
pub fn validate_first_char(s: &str) -> Result<(), &'static str> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Err("empty");
    }
    let first = bytes[0];
    if first.is_ascii_digit() {
        return Err("first character must be non-numeric");
    }
    if matches!(first, b'-' | b'+' | b'.') {
        if let Some(&second) = bytes.get(1) {
            if second.is_ascii_digit() {
                return Err("leading +/-/. cannot be followed by a digit");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_char_round_trip() {
        for (name, ch) in NAMED_CHARS {
            assert_eq!(name_to_char(name), Some(*ch));
            assert_eq!(char_to_name(*ch), Some(*name));
        }
    }

    #[test]
    fn decode_encode_string_escapes_consistent() {
        // Every encoder output must decode back to the same character.
        for c in ['"', '\\', '\n', '\r', '\t', '\u{0008}', '\u{000C}'] {
            let escaped = encode_string_escape(c).unwrap();
            // The encoded form is the body after `\`; for `\\` we get "\\"
            // which is two chars — read the first byte for round-trip.
            let first = escaped.as_bytes()[0];
            assert_eq!(decode_string_escape(first), Some(c));
        }
    }

    #[test]
    fn hex_decode() {
        assert_eq!(hex_value(b'0'), Some(0));
        assert_eq!(hex_value(b'9'), Some(9));
        assert_eq!(hex_value(b'a'), Some(10));
        assert_eq!(hex_value(b'F'), Some(15));
        assert_eq!(hex_value(b'g'), None);
    }
}
