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
/// EDN character set (Clojure dialect): alphanumeric + `. * + ! - _ ? $ % & = < > / '`.
/// The `:` and `#` bytes are NOT permitted in symbol bodies per EDN spec;
/// wat-rs uses `::` as its internal namespace separator but the wat-edn
/// substrate enforces strict-EDN on input. Constructors (`::ns`, `::try_ns`)
/// translate `::` → `.` at the boundary before storage.
///
/// wat is a Clojure dialect, and Clojure legally admits a trailing prime `'`
/// inside symbol/keyword BODIES (`x'`, `:wut'`) — the primed convention wat uses
/// for its service names (`echo'`, `mem-store'`). `'` is a legal Clojure body
/// character (only a LEADING `'` is the quote reader macro — see `is_symbol_start`,
/// which correctly omits it). So a primed keyword must survive the process-pipe wire.
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
                | b'/'
                | b'\''
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

/// Write a keyword body segment verbatim.
///
/// Arc 109 "the comma dies in the reader" retired the position-aware
/// `,` → `_` wire-escape swap (arc 170 REALIZATIONS-SLICE-1.md pass
/// 14 / arc 218 stone 218.1): a keyword body never contains `,` any
/// more (the lexer no longer accepts it as a body-continue char at
/// any bracket depth), so there is nothing left to escape.
///
/// Keyword body is all-ASCII (lexer enforces `is_symbol_continue` on
/// every byte after the first, and non-ASCII scalars pass through
/// their own decode path). A plain `push_str` is both simplest and
/// fastest — no allocation, single pass, matches the byte-for-byte
/// output the old per-byte loop produced for every body that
/// contained no comma (which, post-migration, is now all of them).
pub(crate) fn write_keyword_body_to<W: std::fmt::Write>(
    seg: &str,
    w: &mut W,
) -> std::fmt::Result {
    w.write_str(seg)
}

/// Split a namespaced body `"ns/name"` at the first `/`.
///
/// Returns `Some((ns, name))` when exactly one `/` acts as a separator,
/// `None` when no `/` is present (simple / unqualified body).
///
/// Used by the JSON bridge to decode keywords, symbols, and tagged
/// elements that carry an `ns/name` pair as a JSON string.
pub(crate) fn split_namespaced(body: &str) -> Option<(&str, &str)> {
    body.find('/').map(|slash| (&body[..slash], &body[slash + 1..]))
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

/// Translate a wat-rs `::` namespace separator to strict-EDN `.` form
/// and validate the first-character rule in one step. Returns the
/// translated namespace on success.
pub(crate) fn translate_and_validate_ns(ns: &str) -> Result<String, &'static str> {
    let translated = ns.replace("::", ".");
    validate_first_char(&translated)?;
    Ok(translated)
}

/// Spec: "A UUID. The tagged element is a canonical UUID string
/// representation." The canonical form is 8-4-4-4-12 lowercase
/// hexadecimal characters separated by hyphens.
///
/// `uuid::Uuid::parse_str` is more lenient (accepts simple-form,
/// URN-form, and braced-form). Strict EDN means strict canonical.
pub(crate) fn is_canonical_uuid(s: &str) -> bool {
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
        if !(is_dash || b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return false;
        }
    }
    true
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
