//! Embedded wat — the one literal reader shared by the inline-wat LINT
//! (`tests/lint/no_inlined_wat_in_tests.rs`) and the codemod-reach DRIVER
//! (`src/bin/wat-fix-rust.rs`), arc 255 stone 255.80 (X2: "no new Rust
//! parsing library — factor the tested extractor into one shared module").
//!
//! This module is the single source of truth for "what is a Rust string
//! literal, and what text does it decode to" — escapes, raw strings with
//! `#`-count, `\` line continuations, char literals vs lifetimes, nested
//! block comments. Unlike the lint's original `extract_string_literals`
//! (which only needed the DECODED text), this version also records, for
//! every decoded character, the raw `char`-offset range in the source it
//! came from — the offset map 255.80 needs to splice a codemod's edits
//! (computed in DECODED coordinates) back into the RAW Rust source.
//!
//! The decoded↔raw correspondence is established per decoded char: decoding
//! an escape sequence (`\n`, `\x41`, `\u{2764}`, …) consumes >1 raw char and
//! emits exactly 1 decoded char, so that decoded char's raw range spans the
//! whole escape. A `\`-newline line continuation consumes raw chars and
//! emits NO decoded char at all — it simply contributes no entry, so the
//! next decoded char's raw range starts after it. This is why a splice is
//! self-checking: a changed region whose raw range (taken from its first and
//! last decoded char's raw bounds) contains an escape ANYWHERE inside it —
//! including a swallowed continuation — can never compare equal to the old
//! decoded text (the decode shrank or transformed something), so the
//! "raw slice == old decoded text" check the driver requires refuses it
//! automatically, with no separate gap-tracking needed.
//!
//! All offsets are `char` indices (`Vec<char>` positions), not byte
//! offsets — consistent with `wat::span`'s own `Pos` convention and safe
//! across multi-byte UTF-8 without extra bookkeeping.

/// A single decoded escape from a Rust string literal, plus how many source
/// chars it consumed (so the caller can advance its cursor past the escape
/// sequence). Moved verbatim from `tests/lint/no_inlined_wat_in_tests.rs`.
fn decode_escape(chars: &[char], backslash_at: usize) -> (Option<char>, usize) {
    let n = chars.len();
    let Some(&kind) = chars.get(backslash_at + 1) else {
        return (None, 1);
    };
    match kind {
        'n' => (Some('\n'), 2),
        't' => (Some('\t'), 2),
        'r' => (Some('\r'), 2),
        '\\' => (Some('\\'), 2),
        '\'' => (Some('\''), 2),
        '"' => (Some('"'), 2),
        '0' => (Some('\0'), 2),
        'x' => {
            let start = backslash_at + 2;
            let mut end = start;
            while end < n && end < start + 2 && chars[end].is_ascii_hexdigit() {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let ch = u8::from_str_radix(&hex, 16).ok().map(|v| v as char);
            (ch, end - backslash_at)
        }
        'u' => {
            if chars.get(backslash_at + 2) != Some(&'{') {
                return (None, 2);
            }
            let start = backslash_at + 3;
            let mut end = start;
            while end < n && chars[end] != '}' {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let consumed = if end < n { end + 1 - backslash_at } else { end - backslash_at };
            let ch = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
            (ch, consumed)
        }
        // Line continuation: `\` immediately followed by a newline elides the newline and any
        // leading whitespace on the next line — no character is emitted.
        '\n' => {
            let mut end = backslash_at + 2;
            while end < n && (chars[end] == ' ' || chars[end] == '\t' || chars[end] == '\n' || chars[end] == '\r') {
                end += 1;
            }
            (None, end - backslash_at)
        }
        other => (Some(other), 2),
    }
}

/// One extracted string literal: its decoded content, plus — for every
/// decoded char — the `[start, end)` RAW char-index range (into the source
/// `Vec<char>` the literal was extracted from) that produced it.
///
/// `raw_quote_start`/`raw_quote_end` bound the literal's content region in
/// raw coordinates (exclusive of the delimiting quotes/`r#"`/`"#`), handy
/// for a caller that wants the literal's own span without walking the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralSpan {
    pub decoded: String,
    /// `char_map[i] == (raw_start, raw_end)` for the i-th decoded char.
    pub char_map: Vec<(usize, usize)>,
    pub raw_quote_start: usize,
    pub raw_quote_end: usize,
    pub is_raw_string: bool,
}

impl LiteralSpan {
    /// The raw `[start, end)` char range in the source that the decoded
    /// slice `decoded[lo..hi]` (char indices into `self.decoded`, NOT byte
    /// offsets) came from. `lo == hi` (an empty slice) has no raw range of
    /// its own — callers splicing an empty region should use an adjacent
    /// char's boundary instead.
    pub fn raw_range_for(&self, lo: usize, hi: usize) -> Option<(usize, usize)> {
        if lo >= hi || hi > self.char_map.len() {
            return None;
        }
        let start = self.char_map[lo].0;
        let end = self.char_map[hi - 1].1;
        Some((start, end))
    }
}

/// Extract every string literal's content AND decoded↔raw offset map from a
/// chunk of Rust source, skipping `//` line comments and `/* … */` block
/// comments (which nest), and recognizing char literals (`'x'`, `'"'`,
/// `'\''`) so an embedded quote can't be mistaken for a string open; bare
/// lifetimes (`'a`) are left untouched. `raw_src_chars` must be the SAME
/// `Vec<char>` the caller will later splice into (offsets are indices into
/// it).
pub fn extract_literal_spans(raw_src_chars: &[char]) -> Vec<LiteralSpan> {
    let chars = raw_src_chars;
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // `//` line comment.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // `/* … */` block comment — nests.
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            let mut depth = 1usize;
            while i < n && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Raw string: `r"…"`, `r#"…"#`, `r##"…"##`, … — the head-count of `#` must match on close.
        if c == 'r' || c == 'R' {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let content_start = j + 1;
                let mut k = content_start;
                let mut closed_at = None;
                while k < n {
                    if chars[k] == '"' {
                        let mut m = k + 1;
                        let mut h = 0usize;
                        while h < hashes && chars.get(m) == Some(&'#') {
                            h += 1;
                            m += 1;
                        }
                        if h == hashes {
                            closed_at = Some((k, m));
                            break;
                        }
                    }
                    k += 1;
                }
                match closed_at {
                    Some((close_start, resume)) => {
                        let decoded: String = chars[content_start..close_start].iter().collect();
                        let char_map = (content_start..close_start).map(|p| (p, p + 1)).collect();
                        out.push(LiteralSpan {
                            decoded,
                            char_map,
                            raw_quote_start: content_start,
                            raw_quote_end: close_start,
                            is_raw_string: true,
                        });
                        i = resume;
                    }
                    None => {
                        // Unterminated raw string — no more literals to find in this file.
                        i = n;
                    }
                }
                continue;
            }
            // `r`/`R` not followed by a raw-string opener (an identifier, `r#ident`, etc.) — fall
            // through and let the char be scanned normally below.
        }

        // Char literal: `'x'`, `'\n'`, `'"'`, `'\''`, `'\u{2764}'`, … Distinguished from a bare
        // lifetime (`'a`, `'static`) by actually closing with a matching `'`.
        if c == '\'' {
            if chars.get(i + 1) == Some(&'\\') {
                let (_, consumed) = decode_escape(chars, i + 1);
                let after = i + 1 + consumed;
                if chars.get(after) == Some(&'\'') {
                    i = after + 1;
                    continue;
                }
                // Not actually a closed char literal (e.g. a lifetime that happens to precede a
                // backslash elsewhere) — treat the quote as ordinary and move on one char.
            } else if chars.get(i + 2) == Some(&'\'') {
                i += 3;
                continue;
            }
            // Bare lifetime (`'a`, `'de`, `'static`) — not a literal; leave the identifier for
            // normal scanning.
            i += 1;
            continue;
        }

        // Regular string literal.
        if c == '"' {
            i += 1;
            let content_start = i;
            let mut decoded = String::new();
            let mut char_map = Vec::new();
            while i < n {
                let cc = chars[i];
                if cc == '"' {
                    i += 1;
                    break;
                }
                if cc == '\\' {
                    let (decoded_ch, consumed) = decode_escape(chars, i);
                    if let Some(ch) = decoded_ch {
                        decoded.push(ch);
                        char_map.push((i, i + consumed));
                    }
                    i += consumed;
                    continue;
                }
                decoded.push(cc);
                char_map.push((i, i + 1));
                i += 1;
            }
            let raw_quote_end = i - 1; // position of the closing `"`
            out.push(LiteralSpan {
                decoded,
                char_map,
                raw_quote_start: content_start,
                raw_quote_end,
                is_raw_string: false,
            });
            continue;
        }

        i += 1;
    }

    out
}

/// `format!`-template placeholders (`{ns}`, `{}`, `{fire_fn}`, …) aren't wat
/// syntax. LENGTH-PRESERVING variant of the lint's own `replace_placeholders`
/// (which shrinks a placeholder to a fixed 6-char `__ph__` and so cannot be
/// used where decoded-char offsets must still line up with the untouched
/// `char_map` afterwards): every non-nested `{…}` span is replaced by a
/// same-length run of `x` so the decoded text's char COUNT — and therefore
/// every index into it — never moves. `{{` / `}}` (format!'s literal-brace
/// escapes) are left untouched rather than mistaken for an opening brace.
pub fn replace_placeholders_preserving_len(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            out.push('{');
            out.push('{');
            i += 2;
            continue;
        }
        if chars[i] == '}' && chars.get(i + 1) == Some(&'}') {
            out.push('}');
            out.push('}');
            i += 2;
            continue;
        }
        if chars[i] == '{' {
            if let Some(rel_close) = chars[i..].iter().position(|&c| c == '}') {
                let close = i + rel_close;
                let inner_has_brace = chars[i + 1..close].contains(&'{');
                if !inner_has_brace {
                    let len = close - i + 1;
                    for _ in 0..len {
                        out.push('x');
                    }
                    i = close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans_for(src: &str) -> Vec<LiteralSpan> {
        let chars: Vec<char> = src.chars().collect();
        extract_literal_spans(&chars)
    }

    #[test]
    fn plain_literal_offset_map_is_one_to_one() {
        let src = r#"let x = "ab";"#;
        let spans = spans_for(src);
        assert_eq!(spans.len(), 1);
        let s = &spans[0];
        assert_eq!(s.decoded, "ab");
        // "ab" starts right after the opening quote.
        let quote_at = src.find('"').unwrap();
        assert_eq!(s.char_map, vec![(quote_at + 1, quote_at + 2), (quote_at + 2, quote_at + 3)]);
    }

    #[test]
    fn escape_offset_map_spans_the_whole_escape() {
        let src = r#"let x = "a\nb";"#;
        let spans = spans_for(src);
        let s = &spans[0];
        assert_eq!(s.decoded, "a\nb");
        let chars: Vec<char> = src.chars().collect();
        for (idx, &(raw_start, raw_end)) in s.char_map.iter().enumerate() {
            let raw: String = chars[raw_start..raw_end].iter().collect();
            if idx == 1 {
                assert_eq!(raw, "\\n");
            } else {
                assert_eq!(raw.chars().count(), 1);
            }
        }
    }

    #[test]
    fn raw_string_offset_map_is_one_to_one_with_hash_count() {
        let src = r###"let a = r#"(:wat::core::+ 1 2)"#;"###;
        let spans = spans_for(src);
        assert_eq!(spans.len(), 1);
        let s = &spans[0];
        assert_eq!(s.decoded, "(:wat::core::+ 1 2)");
        assert!(s.is_raw_string);
        let chars: Vec<char> = src.chars().collect();
        assert_eq!(chars[s.raw_quote_start..s.raw_quote_end].iter().collect::<String>(), s.decoded);
    }

    #[test]
    fn line_continuation_elides_with_no_decoded_char_and_no_map_entry() {
        // The raw gap (`\` + newline + leading whitespace) must not appear as a decoded char,
        // and the next decoded char's raw range must start AFTER the whole elided run.
        let src = "let x = \"line one \\\n     line two\";";
        let spans = spans_for(src);
        let s = &spans[0];
        assert_eq!(s.decoded, "line one line two");
        assert_eq!(s.char_map.len(), s.decoded.chars().count());
        // Find the index of the space right after "one" in the decoded text — the char AFTER
        // it ('l' of "line two") must map to a raw offset strictly past the continuation.
        let space_idx = s.decoded.find("one ").unwrap() + "one ".len();
        let (gap_end_raw, _) = s.char_map[space_idx];
        let backslash_raw = src.find('\\').unwrap();
        assert!(gap_end_raw > backslash_raw);
    }

    #[test]
    fn raw_range_for_refuses_implicitly_when_an_escape_sits_inside() {
        // "a\nb" decoded is "a\nb" (3 chars); asking for the raw range covering the ESCAPE
        // char (decoded index 1) must span the 2-char `\n` — longer than the 1-char decoded
        // slice it stands for, which is exactly what makes the driver's "raw slice == old
        // decoded text" self-check fail for a region that swallows an escape.
        let src = r#""a\nb""#;
        let spans = spans_for(src);
        let s = &spans[0];
        let (raw_start, raw_end) = s.raw_range_for(1, 2).unwrap();
        assert_eq!(raw_end - raw_start, 2); // `\n`, two raw chars for one decoded char
    }

    #[test]
    fn placeholder_substitution_preserves_length() {
        let s = "(:wat::core::defmacro {bare_name} [x <- wat.type/i64])";
        let out = replace_placeholders_preserving_len(s);
        assert_eq!(out.chars().count(), s.chars().count());
        assert!(!out.contains('{'));
    }

    #[test]
    fn placeholder_substitution_respects_double_brace_escapes() {
        let s = "{{literal}} {real}";
        let out = replace_placeholders_preserving_len(s);
        assert_eq!(out.chars().count(), s.chars().count());
        assert!(out.starts_with("{{literal}}"));
        assert!(!out[11..].contains('{'));
    }

    #[test]
    fn extractor_skips_comments_like_the_lint_did() {
        let src = r#"
            // (:wat::core::char) — a line comment, not a literal
            /* (:wat::core::also-not-a-literal) */
            let x = "(:wat::core::+ 1 2)";
        "#;
        let spans = spans_for(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].decoded, "(:wat::core::+ 1 2)");
    }

    #[test]
    fn extractor_does_not_choke_on_quote_char_literals() {
        let src = r#"if c == '"' { let s = "(:wat::core::+ 1 2)"; }"#;
        let spans = spans_for(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].decoded, "(:wat::core::+ 1 2)");
    }
}
