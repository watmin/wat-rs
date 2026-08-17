//! Single-pass EDN writer. `write_to` appends to an existing buffer
//! (preferred for large outputs); `write` returns a fresh String.
//!
//! # Position-aware keyword body wire encoding (arc 170 slice 1f-W)
//!
//! When writing a keyword body, the writer tracks bracket depth (`<`
//! increments, `>` decrements). At depth ≥ 1 (inside a parametric
//! type-arg list like `:Foo<A,B>`), every `,` is emitted as `_` —
//! the wire-escape rule from REALIZATIONS-SLICE-1.md pass 14
//! (locked 2026-05-10).
//!
//! Mirror of [`super::lexer::Lexer::new_wire`]'s `_` → `,` decode.
//! Round-trip property: `Parser::new_wire(write(k)).parse_top() == k` for any keyword
//! `k`, including parametric forms with commas at any depth.
//!
//! Outside `<...>` (depth 0), keyword body chars pass verbatim:
//! `_` stays `_` (preserves `:rust::*` Rust-mirror convention; no
//! `,` is legal at depth 0 because EDN treats `,` as whitespace).

use crate::vocab::{char_to_name, encode_string_escape, write_keyword_body_to};
use crate::value::{Keyword, Symbol, Tag, Value};
use chrono::SecondsFormat;
use std::fmt::Write;

// ─── Pretty-printing ────────────────────────────────────────────

/// Pretty-print an EDN `Value` to a `String`. Uses 2-space indent;
/// scalar containers stay on one line, nested collections break per
/// element. Maps put each `key value` pair on its own line.
pub fn write_pretty(v: &Value) -> String {
    let mut out = String::with_capacity(128);
    write_pretty_indented(v, &mut out, 0);
    out
}

const INDENT: &str = "  ";

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str(INDENT);
    }
}

/// True if the value inlines without breaking in a pretty-printed collection.
/// BigInt and BigDec are atomic (no sub-elements; print inline as `42N`/`3.14M`)
/// and are correctly treated as inline values for pretty-print inlining.
///
/// WHY `Value::Tagged` is absent: the tagged variant has its own dedicated arm in
/// `write_pretty_indented` (see lines below) that writes `#ns/name <body>` inline or
/// with a newline depending on the body — it never needs the generic "break each
/// element" path that this predicate guards against.
fn is_inline_value(v: &Value) -> bool {
    matches!(
        v,
        Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float(_)
            | Value::BigInt(_)
            | Value::BigDec(_)
            | Value::Rational(_)
            | Value::String(_)
            | Value::Char(_)
            | Value::Symbol(_)
            | Value::Keyword(_)
            | Value::Inst(_)
            | Value::Uuid(_)
    )
}

/// True if every element inlines (so we can inline a small collection).
fn all_inline(items: &[Value]) -> bool {
    items.iter().all(is_inline_value)
}

fn write_pretty_indented(v: &Value, out: &mut String, level: usize) {
    match v {
        Value::List(items) | Value::Vector(items) | Value::Set(items) => {
            let (open, close) = match v {
                Value::List(_) => ("(", ")"),
                Value::Vector(_) => ("[", "]"),
                Value::Set(_) => ("#{", "}"),
                _ => unreachable!(),
            };
            if items.is_empty() {
                out.push_str(open);
                out.push_str(close);
            } else if items.len() <= 8 && all_inline(items) {
                // Inline small scalar-only collections.
                out.push_str(open);
                let mut first = true;
                for item in items {
                    if !first {
                        out.push(' ');
                    }
                    write_to(item, out);
                    first = false;
                }
                out.push_str(close);
            } else {
                out.push_str(open);
                out.push('\n');
                let inner = level + 1;
                for (i, item) in items.iter().enumerate() {
                    push_indent(out, inner);
                    write_pretty_indented(item, out, inner);
                    if i + 1 < items.len() {
                        out.push('\n');
                    }
                }
                out.push('\n');
                push_indent(out, level);
                out.push_str(close);
            }
        }
        Value::Map(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
            } else {
                out.push('{');
                out.push('\n');
                let inner = level + 1;
                for (i, (k, val)) in entries.iter().enumerate() {
                    push_indent(out, inner);
                    write_pretty_indented(k, out, inner);
                    out.push(' ');
                    write_pretty_indented(val, out, inner);
                    if i + 1 < entries.len() {
                        out.push('\n');
                    }
                }
                out.push('\n');
                push_indent(out, level);
                out.push('}');
            }
        }
        Value::Tagged(tag, body) => {
            // #ns/name <body>  — tag and body on same line if body
            // is scalar, otherwise newline + indent for body.
            out.push('#');
            out.push_str(tag.namespace());
            out.push('/');
            out.push_str(tag.name());
            out.push(' ');
            write_pretty_indented(body, out, level);
        }
        // Scalars: defer to write_to.
        _ => write_to(v, out),
    }
}

// ─── Identifier writers ─────────────────────────────────────────
//
// Direct `push_str` to the caller's buffer. The `Display` impls in
// value.rs route through `fmt::Formatter`, which adds vtable cost;
// in the hot writer path we already own a `&mut String` and skip
// the formatter machinery. The two paths emit byte-equivalent
// output — locked by the equivalence tests in
// `tests/display_equivalence.rs`.

#[inline]
fn write_symbol(s: &Symbol, out: &mut String) {
    if let Some(ns) = s.namespace() {
        out.push_str(ns);
        out.push('/');
    }
    out.push_str(s.name());
}

#[inline]
fn write_keyword(k: &Keyword, out: &mut String) {
    out.push(':');
    if let Some(ns) = k.namespace() {
        write_keyword_body_to(ns, out).expect("String fmt::Write is infallible");
        out.push('/');
    }
    write_keyword_body_to(k.name(), out).expect("String fmt::Write is infallible");
}

#[inline]
fn write_tag(t: &Tag, out: &mut String) {
    // Tag::namespace is unconditional per spec — no Option to peek.
    out.push('#');
    out.push_str(t.namespace());
    out.push('/');
    out.push_str(t.name());
}

/// Write `v` as EDN, returning a fresh `String`.
pub fn write(v: &Value) -> String {
    let mut out = String::with_capacity(64);
    write_to(v, &mut out);
    out
}

// rune:purgare(future-fixture) — buffer-reuse ergonomic retained for
// the future Clojure-IPC bridge per crates/wat-edn/docs/IPC-BRIDGE.md:95;
// no current external caller. Symmetric with the actively-consumed
// `write` fn. The append-to-existing-buffer shape is the canonical
// Rust pattern for output composition. This rune retires when the IPC
// bridge ships and write_to gains a real caller (per purgare SKILL:
// "rune retires when the downstream lands").
/// Append `v` as EDN to `out`. Reuses caller-owned buffer.
pub fn write_to(v: &Value, out: &mut String) {
    match v {
        Value::Nil => out.push_str("nil"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Integer(i) => write!(out, "{}", i).unwrap(),
        Value::BigInt(n) => write!(out, "{}N", n).unwrap(),
        Value::Float(f) => write_float(*f, out),
        Value::BigDec(n) => write!(out, "{}M", n).unwrap(),
        Value::Rational(n) => write!(out, "{}/{}", n.numer(), n.denom()).unwrap(),
        Value::String(s) => write_string(s, out),
        Value::Char(c) => write_char(*c, out),
        Value::Symbol(s) => write_symbol(s, out),
        Value::Keyword(k) => write_keyword(k, out),
        Value::List(items) => write_seq(items, '(', ')', out),
        Value::Vector(items) => write_seq(items, '[', ']', out),
        Value::Set(items) => {
            out.push('#');
            write_seq(items, '{', '}', out);
        }
        Value::Map(entries) => write_map(entries, out),
        Value::Tagged(tag, body) => {
            write_tag(tag, out);
            out.push(' ');
            write_to(body, out);
        }
        Value::Inst(dt) => {
            // Standard EDN form: #inst "RFC3339"
            out.push_str("#inst \"");
            out.push_str(&dt.to_rfc3339_opts(SecondsFormat::AutoSi, true));
            out.push('"');
        }
        Value::Uuid(u) => {
            write!(out, "#uuid \"{}\"", u).unwrap();
        }
    }
}

fn write_float(f: f64, out: &mut String) {
    // EDN doesn't define NaN or ±Infinity. wat-edn emits namespaced
    // sentinel tags that its own reader recognizes, so f64 round-trips
    // through write→parse even for non-finite values. Other EDN readers
    // see them as ordinary user tags and can pass through, ignore, or
    // install a handler.
    //
    // Shape: `#wat.core.f64/{NaN,+Inf,-Inf} []` — arc 294.l. The body is
    // an empty vector, not `nil`: arc 278 A.0 retired the bare-nil unit
    // variant body (`#tag []` is the only legal shape), and wat-edn's own
    // sentinel is not exempt just because parser.rs intercepts it before
    // substrate tag dispatch ever sees it.
    if f.is_nan() {
        out.push_str("#wat.core.f64/NaN []");
        return;
    }
    if f.is_infinite() {
        if f.is_sign_negative() {
            out.push_str("#wat.core.f64/-Inf []");
        } else {
            out.push_str("#wat.core.f64/+Inf []");
        }
        return;
    }
    // `Debug` (unlike `Display`) is documented to emit the shortest
    // representation that round-trips exactly, always keeps a `.0` on whole
    // floats, and switches to exponent form at large magnitudes (proven by
    // run, this stone: 1e16 -> "1e16", 1e200 -> "1e200") — so it always
    // contains a `.` or an `e` and is never lexable as an EDN integer, at
    // every finite magnitude. That collapses the old `1e16`-bounded special
    // case entirely; there is no longer a boundary to special-case.
    write!(out, "{:?}", f).unwrap();
}

/// Write a quoted EDN string with escapes. Fast path uses
/// `memchr::memchr` to skip clean ASCII chunks in one move; only
/// chunks containing escape-required bytes hit the per-char path.
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    let bytes = s.as_bytes();
    let mut start = 0;

    // Bytes requiring escape: `"`, `\`, and any C0 control byte (< 0x20).
    // memchr3 finds `"` and `\` cheaply; we then check the chunk for
    // any control bytes before pushing it whole.
    while start < bytes.len() {
        let next_special =
            memchr::memchr3(b'"', b'\\', 0x00, &bytes[start..]).map(|n| start + n);
        // Find first control byte in [start..end_clean) too.
        let end_clean = next_special.unwrap_or(bytes.len());
        let mut chunk_end = end_clean;
        // Scan the prospective clean chunk for any C0 control byte.
        for (i, &b) in bytes[start..end_clean].iter().enumerate() {
            if b < 0x20 {
                chunk_end = start + i;
                break;
            }
        }
        // Push the clean chunk in one shot.
        if chunk_end > start {
            // SAFETY: bytes[start..chunk_end] is a valid UTF-8 slice
            // because we only stop at ASCII bytes (0x00..0x20, 0x22, 0x5C),
            // none of which are mid-UTF-8 continuation bytes.
            out.push_str(std::str::from_utf8(&bytes[start..chunk_end]).expect("ascii-or-utf8 chunk"));
            start = chunk_end;
            if start >= bytes.len() {
                break;
            }
        }
        // Escape one byte (or multibyte char if we somehow stopped on one).
        let b = bytes[start];
        if let Some(esc) = encode_string_escape(b as char) {
            out.push('\\');
            out.push_str(esc);
            start += 1;
        } else if b < 0x20 {
            write!(out, "\\u{:04X}", b as u32).unwrap();
            start += 1;
        } else {
            // Unreachable: memchr3 stops on `"`, `\`, or NUL — all handled
            // by encode_string_escape (NUL falls through to the b<0x20
            // branch above). Anything else is a control byte caught by the
            // linear scan in the loop head, also `b<0x20`.
            unreachable!("write_string fallback: byte 0x{:02x} should have been escaped", b)
        }
    }

    out.push('"');
}

fn write_char(c: char, out: &mut String) {
    out.push('\\');
    if let Some(name) = char_to_name(c) {
        out.push_str(name);
        return;
    }
    let cp = c as u32;
    // wat-edn aligns to BMP-only chars for cross-language interop
    // (clojure.edn/read rejects supplementary-plane char literals).
    // Surface the constraint at write time rather than emitting a form
    // downstream readers can't consume.
    if cp > 0xFFFF {
        panic!(
            "wat-edn char literal U+{:X} is supplementary-plane; \
             wat-edn aligns to BMP-only (U+0000..=U+FFFF) for \
             cross-language EDN interop",
            cp
        );
    }
    // BMP control bytes (< 0x20) and DEL (0x7F) → \uXXXX (exactly 4
    // hex digits per spec).
    if cp < 0x20 || cp == 0x7F {
        write!(out, "u{:04X}", cp).unwrap();
        return;
    }
    // BMP non-control non-printable still fits in 4 digits.
    if !(0x20..=0x7E).contains(&cp) {
        write!(out, "u{:04X}", cp).unwrap();
        return;
    }
    // Printable ASCII → literal.
    out.push(c);
}

fn write_seq(items: &[Value], open: char, close: char, out: &mut String) {
    out.push(open);
    let mut first = true;
    for item in items {
        if !first {
            out.push(' ');
        }
        write_to(item, out);
        first = false;
    }
    out.push(close);
}

fn write_map(entries: &[(Value, Value)], out: &mut String) {
    out.push('{');
    let mut first = true;
    for (k, v) in entries {
        if !first {
            out.push(' ');
        }
        write_to(k, out);
        out.push(' ');
        write_to(v, out);
        first = false;
    }
    out.push('}');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Keyword, Symbol, Tag};

    #[test]
    fn primitives() {
        assert_eq!(write(&Value::Nil), "nil");
        assert_eq!(write(&Value::Bool(true)), "true");
        assert_eq!(write(&Value::Bool(false)), "false");
        assert_eq!(write(&Value::Integer(42)), "42");
        assert_eq!(write(&Value::Integer(-7)), "-7");
        assert_eq!(write(&Value::Float(2.5)), "2.5");
        assert_eq!(write(&Value::Float(42.0)), "42.0"); // forced
    }

    #[test]
    fn strings() {
        assert_eq!(write(&Value::String("hello".into())), r#""hello""#);
        assert_eq!(write(&Value::String("a\nb".into())), r#""a\nb""#);
        assert_eq!(write(&Value::String("é".into())), r#""é""#);
    }

    #[test]
    fn keywords_and_symbols() {
        assert_eq!(write(&Value::Keyword(Keyword::new("foo"))), ":foo");
        assert_eq!(
            write(&Value::Keyword(Keyword::ns("ns", "foo"))),
            ":ns/foo"
        );
        assert_eq!(write(&Value::Symbol(Symbol::new("foo"))), "foo");
    }

    /// wat is a Clojure dialect: a trailing prime `'` is a legal symbol/keyword
    /// BODY character (`:wut'`, `x'`, wat's primed service names `echo'`). Such a
    /// value must survive the process-pipe wire — parse via `parse_owned`, write
    /// back identically. (Regression for arc 278 s2s: a primed keyword died on the
    /// wire with "unexpected byte 0x27" until `is_symbol_continue` admitted `'`.)
    #[test]
    fn primed_keywords_and_symbols_round_trip() {
        for src in [":wut'", ":probe/echo'", "x'", "mem-store'", ":a/b'c'"] {
            let v = crate::parse_owned(src).expect("primed body must parse");
            assert_eq!(write(&v), src, "round-trip mismatch for {src:?}");
        }
    }

    #[test]
    fn collections() {
        let v = Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(write(&v), "[1 2 3]");

        let l = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(write(&l), "(1 2)");

        let s = Value::Set(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(write(&s), "#{1 2}");

        let m = Value::Map(vec![
            (Value::Keyword(Keyword::new("a")), Value::Integer(1)),
            (Value::Keyword(Keyword::new("b")), Value::Integer(2)),
        ]);
        assert_eq!(write(&m), "{:a 1 :b 2}");
    }

    #[test]
    fn tagged() {
        let v = Value::Tagged(
            Tag::ns("myapp", "Person"),
            Box::new(Value::Map(vec![(
                Value::Keyword(Keyword::new("name")),
                Value::String("Fred".into()),
            )])),
        );
        assert_eq!(write(&v), r#"#myapp/Person {:name "Fred"}"#);
    }
}
