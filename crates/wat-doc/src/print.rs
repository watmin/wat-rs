//! `#wat.doc/Row` emitter — the inverse of [`crate::from_metadata`], and of
//! `wat-macros`' `edn_doc::dedent`.
//!
//! A named emitter, not `wat-edn`'s `write-pretty`. `write-pretty` escapes
//! newlines inside strings (measured); the docstring here is a LITERAL
//! multi-line EDN string whose continuation lines sit at column 0 — the
//! map's own margin — so `dedent` of an indented fence restores the same
//! bytes. Values are EDN ns/name keywords (`:wat.core/foldl`), never wat
//! FQDNs (`:wat::core::foldl`); `::` is a lexer error in EDN.

use std::fmt::Write;

use wat_reader::WatAST;

use crate::{Deprecation, DocArg, DocComment, DocExample, DocYields};

/// Print `doc` as a `#wat.doc/Row { … }` block.
///
/// Continuation lines of `:doc` carry the map's margin (column 0 of this
/// unfenced emission). Wrapping the result in a ```edn fence and indenting
/// every line by the same amount is then the exact inverse of `dedent`.
pub fn print(doc: &DocComment) -> String {
    let mut out = String::new();
    out.push_str("#wat.doc/Row {\n");
    emit_docstring(&mut out, &doc.prose);
    emit_kv(&mut out, ":added", &edn_quoted(&doc.added));
    if doc.alias.is_none() {
        emit_kv(&mut out, ":purity", &axis_edn(crate::Purity::WAT_TYPE_PATH, doc.purity.as_str()));
        emit_kv(
            &mut out,
            ":determinism",
            &axis_edn(crate::Determinism::WAT_TYPE_PATH, doc.determinism.as_str()),
        );
        emit_kv(
            &mut out,
            ":totality",
            &axis_edn(crate::Totality::WAT_TYPE_PATH, doc.totality.as_str()),
        );
        emit_kv(
            &mut out,
            ":expand-time",
            &axis_edn(crate::ExpandTime::WAT_TYPE_PATH, doc.expand_time.as_str()),
        );
        emit_kv(
            &mut out,
            ":category",
            &axis_edn(crate::Category::WAT_TYPE_PATH, doc.category.as_str()),
        );
    }
    if !doc.args.is_empty() {
        emit_kv(&mut out, ":args", &print_args(&doc.args));
    }
    emit_kv(&mut out, ":ret", &print_ret(&doc.ret_type, &doc.ret));
    emit_kv(&mut out, ":examples", &print_examples(&doc.examples));
    if !doc.yields.is_empty() {
        emit_kv(&mut out, ":yields", &print_yields(&doc.yields));
    }
    if !doc.see.is_empty() {
        emit_kv(&mut out, ":see", &print_see(&doc.see));
    }
    if let Some(d) = &doc.deprecated {
        emit_kv(&mut out, ":deprecated", &print_deprecated(d));
    }
    if let Some(a) = &doc.alias {
        emit_kv(&mut out, ":alias", &wat_fqdn_to_edn_keyword(a));
    }
    out.push('}');
    out
}

fn emit_kv(out: &mut String, key: &str, value: &str) {
    out.push_str("  ");
    out.push_str(key);
    out.push(' ');
    out.push_str(value);
    out.push('\n');
}

/// `:doc` as a literal multi-line string. Continuation lines are flush-left
/// (column 0) so `wat_edn::parse` of this unfenced block does not inject the
/// map's inner indent into the prose — and so indenting the whole block for
/// a ```edn fence is `dedent`'s inverse.
fn emit_docstring(out: &mut String, prose: &str) {
    out.push_str("  :doc ");
    push_edn_string(out, prose);
    out.push('\n');
}

fn edn_quoted(s: &str) -> String {
    let mut out = String::new();
    push_edn_string(&mut out, s);
    out
}

/// EDN string with `"` and `\` escaped, newlines left LITERAL (not `\n`).
fn push_edn_string(out: &mut String, s: &str) {
    out.push('"');
    for (i, line) in s.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for c in line.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                _ => out.push(c),
            }
        }
    }
    out.push('"');
}

fn axis_edn(wat_type_path: &str, variant: &str) -> String {
    wat_fqdn_to_edn_keyword(&format!("{wat_type_path}::{variant}"))
}

/// Forward of `edn/render.rs`'s `wat_keyword_to_clojure_symbol`, as an EDN
/// keyword. `:wat::core::foldl` → `:wat.core/foldl`; a `Type/method` leaf
/// folds the type into the namespace (`:wat::holon::Hologram/make` →
/// `:wat.holon.Hologram/make`). Bare keywords (`:doc`) stay bare.
pub(crate) fn wat_fqdn_to_edn_keyword(kw: &str) -> String {
    let body = match kw.strip_prefix(':') {
        Some(b) => b,
        None => return format!(":{kw}"),
    };
    if !body.contains("::") || body.ends_with("::") {
        return format!(":{body}");
    }
    let final_seg = wat_reader::identifier::leaf(body);
    let mut ns_parts: Vec<&str> = wat_reader::identifier::path(body).split("::").collect();
    let name: &str =
        if final_seg.contains('/') && !wat_reader::identifier::receiver(final_seg).is_empty() {
            ns_parts.push(wat_reader::identifier::receiver(final_seg));
            wat_reader::identifier::method(final_seg)
        } else {
            final_seg
        };
    format!(":{}/{}", ns_parts.join("."), name)
}

fn print_type_token(ty: &str) -> String {
    if ty.starts_with(':') && !ty.contains([' ', '\t', '(', '[']) {
        return wat_fqdn_to_edn_keyword(ty);
    }
    match wat_reader::parse_one_with_file(ty, "<wat-doc print type>") {
        Ok(ast) => watast_to_edn(&ast),
        Err(_) => edn_quoted(ty),
    }
}

fn print_args(args: &[DocArg]) -> String {
    let mut out = String::from("[");
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push('[');
        if a.is_rest {
            out.push_str(&a.name);
            out.push_str("...");
        } else {
            out.push_str(&a.name);
        }
        out.push(' ');
        out.push_str(&print_type_token(&a.ty));
        out.push(' ');
        push_edn_string(&mut out, &a.desc);
        out.push(']');
    }
    out.push(']');
    out
}

fn print_ret(ty: &str, desc: &str) -> String {
    let mut out = String::from("[");
    out.push_str(&print_type_token(ty));
    out.push(' ');
    push_edn_string(&mut out, desc);
    out.push(']');
    out
}

fn print_examples(examples: &[DocExample]) -> String {
    let mut out = String::from("[");
    for (i, ex) in examples.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push('[');
        out.push_str(&watast_to_edn(&ex.expr));
        if ex.run {
            out.push(' ');
            match &ex.expected {
                Some(e) => out.push_str(&watast_to_edn(e)),
                None => out.push_str("nil"),
            }
        }
        out.push(']');
    }
    out.push(']');
    out
}

fn print_yields(yields: &[DocYields]) -> String {
    let mut out = String::from("[");
    for (i, y) in yields.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push('[');
        out.push_str(&y.arg);
        out.push(' ');
        push_edn_string(&mut out, &y.desc);
        out.push(']');
    }
    out.push(']');
    out
}

fn print_see(see: &[String]) -> String {
    let mut out = String::from("[");
    for (i, s) in see.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&wat_fqdn_to_edn_keyword(s));
    }
    out.push(']');
    out
}

fn print_deprecated(d: &Deprecation) -> String {
    let mut out = String::from("[");
    push_edn_string(&mut out, &d.since);
    out.push(' ');
    push_edn_string(&mut out, &d.use_instead);
    out.push(']');
    out
}

/// Render a `WatAST` as EDN surface syntax (ns/name keywords, not wat FQDNs).
fn watast_to_edn(ast: &WatAST) -> String {
    let mut out = String::new();
    push_watast_edn(ast, &mut out);
    out
}

fn push_watast_edn(ast: &WatAST, out: &mut String) {
    match ast {
        WatAST::NilLit(_) => out.push_str("nil"),
        WatAST::BoolLit(b, _) => out.push_str(if *b { "true" } else { "false" }),
        WatAST::IntLit(n, _) => {
            let _ = write!(out, "{n}");
        }
        WatAST::FloatLit(f, _) => {
            let _ = write!(out, "{f}");
        }
        WatAST::BigIntLit(n, _) => {
            let _ = write!(out, "{n}N");
        }
        WatAST::RationalLit(r, _) => {
            let _ = write!(out, "{}/{}", r.numer(), r.denom());
        }
        WatAST::CharLit(c, _) => {
            out.push('\\');
            out.push(*c);
        }
        WatAST::StringLit(s, _) => push_edn_string(out, s),
        WatAST::Keyword(k, _) => out.push_str(&wat_fqdn_to_edn_keyword(k)),
        WatAST::Symbol(id, _) => out.push_str(id.as_str()),
        WatAST::List(items, _) => push_seq(items, '(', ')', out),
        WatAST::Vector(items, _) => push_seq(items, '[', ']', out),
        WatAST::Set(items, _) => {
            out.push('#');
            push_seq(items, '{', '}', out);
        }
        WatAST::Map(pairs, _) => {
            out.push('{');
            for (i, (k, v)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                push_watast_edn(k, out);
                out.push(' ');
                push_watast_edn(v, out);
            }
            out.push('}');
        }
    }
}

fn push_seq(items: &[WatAST], open: char, close: char, out: &mut String) {
    out.push(open);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        push_watast_edn(item, out);
    }
    out.push(close);
}

/// Render a `WatAST` as wat source (FQDN keywords). Used by `from_metadata`
/// to stringify a compound type token after the EDN→WatAST transcoder.
pub(crate) fn watast_to_wat_source(ast: &WatAST) -> String {
    let mut out = String::new();
    push_watast_wat(ast, &mut out);
    out
}

fn push_watast_wat(ast: &WatAST, out: &mut String) {
    match ast {
        WatAST::NilLit(_) => out.push_str("nil"),
        WatAST::BoolLit(b, _) => out.push_str(if *b { "true" } else { "false" }),
        WatAST::IntLit(n, _) => {
            let _ = write!(out, "{n}");
        }
        WatAST::FloatLit(f, _) => {
            let _ = write!(out, "{f}");
        }
        WatAST::BigIntLit(n, _) => {
            let _ = write!(out, "{n}N");
        }
        WatAST::RationalLit(r, _) => {
            let _ = write!(out, "{}/{}", r.numer(), r.denom());
        }
        WatAST::CharLit(c, _) => {
            out.push('\\');
            out.push(*c);
        }
        WatAST::StringLit(s, _) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        WatAST::Keyword(k, _) => out.push_str(k),
        WatAST::Symbol(id, _) => out.push_str(id.as_str()),
        WatAST::List(items, _) => push_seq_wat(items, '(', ')', out),
        WatAST::Vector(items, _) => push_seq_wat(items, '[', ']', out),
        WatAST::Set(items, _) => {
            out.push('#');
            push_seq_wat(items, '{', '}', out);
        }
        WatAST::Map(pairs, _) => {
            out.push('{');
            for (i, (k, v)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                push_watast_wat(k, out);
                out.push(' ');
                push_watast_wat(v, out);
            }
            out.push('}');
        }
    }
}

fn push_seq_wat(items: &[WatAST], open: char, close: char, out: &mut String) {
    out.push(open);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        push_watast_wat(item, out);
    }
    out.push(close);
}

#[cfg(test)]
mod print_tests {
    use super::*;
    use crate::parse;

    const CHAR_SHAPE: &str = concat!(
        "`(:wat::core::char s)` → the single `:wat::core::char` in the length-1 String `s`.\n\n",
        "@added 1.0.0\n",
        "@Purity Pure\n",
        "@Determinism Deterministic\n",
        "@Totality Unreviewed\n",
        "@ExpandTime Unreviewed\n",
        "@Category Transform\n",
        "@arg s :wat::core::String a length-1 BMP string\n",
        "@ret :wat::core::char the single character in `s`\n",
        "@example (:wat::core::char \"x\") #=> (:wat::core::char \"x\")",
    );

    fn assert_printed(expected: &str, printed: &str) {
        assert_eq!(printed, expected.trim_end());
    }

    #[test]
    fn print_of_char_is_the_row_shape() {
        let doc = parse(CHAR_SHAPE).expect("char-shaped @-form parses");
        assert_printed(include_str!("print_tests__char_shape.edn"), &print(&doc));
    }

    #[test]
    fn docstring_continuation_lines_are_flush_left() {
        let raw = "line one\nstill going\n\nparagraph two\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::nil n\n@example (f) #=> nil";
        let doc = parse(raw).expect("parses");
        assert_printed(include_str!("print_tests__flush_left_docstring.edn"), &print(&doc));
    }

    #[test]
    fn type_method_keyword_folds_the_type_into_the_namespace() {
        assert_eq!(
            wat_fqdn_to_edn_keyword(":wat::holon::Hologram/make"),
            ":wat.holon.Hologram/make"
        );
        assert_eq!(wat_fqdn_to_edn_keyword(":wat::core::foldl"), ":wat.core/foldl");
        assert_eq!(
            wat_fqdn_to_edn_keyword(":wat::runtime::Purity::Pure"),
            ":wat.runtime.Purity/Pure"
        );
        assert_eq!(wat_fqdn_to_edn_keyword(":added"), ":added");
    }

    #[test]
    fn rest_arg_prints_the_ellipsis() {
        let raw = "Count them.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Reflection\n@arg xs… :wat::core::Value the args to count\n@ret :wat::core::i64 n\n@example (f) #=> 0";
        let doc = parse(raw).expect("parses");
        assert_printed(include_str!("print_tests__rest_arg.edn"), &print(&doc));
    }

    #[test]
    fn norun_example_is_a_length_one_vector() {
        let raw = "Effect.\n\n@added 1.0.0\n@Purity Effectful\n@Determinism Nondeterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Io\n@ret :wat::core::nil n\n@example-norun (f x)";
        let doc = parse(raw).expect("parses");
        assert_printed(include_str!("print_tests__norun.edn"), &print(&doc));
    }

    #[test]
    fn deprecated_is_emitted_when_present() {
        let raw = "Old.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::nil n\n@example (f) #=> nil\n@deprecated 1.2.0 use :wat::core::other";
        let doc = parse(raw).expect("parses");
        assert_printed(include_str!("print_tests__deprecated.edn"), &print(&doc));
    }
}
