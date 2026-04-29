//! Single-pass EDN writer. `write_to` appends to an existing buffer
//! (preferred for large outputs); `write` returns a fresh String.

use crate::escapes::{char_to_name, encode_string_escape};
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

/// True if the value is "scalar enough" to inline without breaking.
fn is_scalar(v: &Value) -> bool {
    matches!(
        v,
        Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Char(_)
            | Value::Symbol(_)
            | Value::Keyword(_)
            | Value::Inst(_)
            | Value::Uuid(_)
    )
}

/// True if every element is scalar (so we can inline a small collection).
fn all_scalar(items: &[Value]) -> bool {
    items.iter().all(is_scalar)
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
            } else if all_scalar(items) && items.len() <= 8 {
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
                let inner = level + 1;
                for (i, (k, val)) in entries.iter().enumerate() {
                    if i > 0 {
                        push_indent(out, inner);
                    }
                    write_pretty_indented(k, out, inner);
                    out.push(' ');
                    write_pretty_indented(val, out, inner);
                    if i + 1 < entries.len() {
                        out.push('\n');
                    }
                }
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
        out.push_str(ns);
        out.push('/');
    }
    out.push_str(k.name());
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
    if f.is_nan() {
        out.push_str("#wat-edn.float/nan nil");
        return;
    }
    if f.is_infinite() {
        if f.is_sign_negative() {
            out.push_str("#wat-edn.float/neg-inf nil");
        } else {
            out.push_str("#wat-edn.float/inf nil");
        }
        return;
    }
    // Rust's default formatter elides ".0" for whole floats which would
    // round-trip back as integers. Force a fractional component.
    if f == f.trunc() && f.abs() < 1e16 {
        write!(out, "{:.1}", f).unwrap();
    } else {
        write!(out, "{}", f).unwrap();
    }
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
    if (c as u32) < 0x20 || (c as u32) > 0x7E {
        write!(out, "u{:04X}", c as u32).unwrap();
        return;
    }
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
            out.push_str(", ");
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
        assert_eq!(write(&m), "{:a 1, :b 2}");
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
