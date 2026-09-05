//! arc 255 "the walls must not be muted" / BRIEF-STONE-the-edn-doc-row-is-imposed — the
//! ```edn fence reader.
//!
//! ⛔ THE CENTRAL CONSTRAINT (see the BRIEF, verbatim): DO NOT WRITE A THIRD DECODER. This
//! module carries NO doc-contract knowledge — no notion of "required field", no enum-variant
//! validation, no shape-per-key rule. It does exactly two things:
//!
//!   1. Find the fenced ```edn ... ``` block in the joined `///` text (`extract_edn_fence`).
//!   2. Parse it with `wat_edn::parse`, validate the top-level tag is `#wat.doc/Row` or
//!      `#wat.doc/Alias` (a proc-macro cannot consult the runtime type registry to resolve the
//!      tag itself — see `DESIGN-the-tagged-edn-doc-row.md` — so the tag NAME is checked here,
//!      by string, at expand time), and generically transcode the body `wat_edn::Value` into
//!      the `WatAST` shape `wat_doc::from_metadata` already accepts (`parse_edn_doc_row`).
//!
//! Every required-field / enum-variant / per-key shape check happens exactly once, inside
//! `wat_doc::from_metadata` — this module hands it a `WatAST::Map` and from_metadata decides
//! whether that map is a valid doc row, exactly as it already does for a wat-side `defn`'s
//! metadata map. `edn_value_to_watast` is a pure, generic, recursive value-shape transcoder:
//! EDN `Map`/`Vector`/`List`/`Set` become the identically-shaped `WatAST` collection, EDN
//! `String`/`Integer`/`Bool`/`Char`/… become the matching `WatAST` literal, and an EDN
//! `Keyword`/bare `Symbol` becomes a `WatAST::Keyword`/`WatAST::Symbol` with the SAME
//! ns/name → `::`-FQDN reversal `edn::write` already performs the other way (measured in the
//! SCORE: `:wat::core::foldl` <-> `:wat.core/foldl`, losslessly).
//!
//! ⚠ **This conversion is NOT total over `wat_edn::Value`** (the BRIEF names this as the one
//! open question this stone must answer, not assume). `Value::Tagged` (a nested tag has no raw
//! `WatAST` node — a tag resolves to a registered record only at eval time), `Value::Inst`,
//! `Value::Uuid`, `Value::BigDec` (no `WatAST` literal spelling exists for any of the three),
//! and a NAMESPACED `Value::Symbol` (`WatAST::Symbol` carries a bare `Identifier` only) all
//! have no `WatAST` spelling. `edn_value_to_watast` returns `Err` rather than panicking or
//! guessing, and the caller turns that into `DocError::EdnValueNotRepresentable` — a real
//! error, never a silent drop, per the BRIEF's STOP-1. None of the five doc-row keys this
//! stone's converted row uses (`:doc`/`:added`/the five axes/`:args`/`:ret`/`:examples`/`:see`/
//! `:alias`) needs any of the four unrepresentable shapes, so this stone does not hit STOP-1 —
//! but the gap is real and is reported, not worked around.

use wat_reader::{Identifier, Span, WatAST};

/// Find a fenced ```edn ... ``` block inside the joined `///` text and return its inner text
/// (the lines between the two fence markers, `dedent`ed and joined with `\n`). `None` when no
/// such fence is present anywhere in the block — the caller falls back to the `@`-directive
/// text grammar, completely unchanged (STOP-2: both forms, or neither; this function makes no
/// changes at all to how a fenceless doc block is read). Also `None` for an unterminated fence
/// (no closing ` ``` ` line) — treated as "no fence found" rather than guessed at.
pub(crate) fn extract_edn_fence(raw_doc: &str) -> Option<String> {
    let mut lines = raw_doc.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "```edn" {
            let mut collected: Vec<&str> = Vec::new();
            for l in lines.by_ref() {
                if l.trim() == "```" {
                    return Some(dedent(&collected));
                }
                collected.push(l);
            }
            return None;
        }
    }
    None
}

/// Strip the common leading-whitespace indent shared by every non-blank line of the fence,
/// so the ```edn block may be indented for readability in the Rust source (a multi-line `:doc`
/// string's continuation lines aligned under its sibling keys, Clojure-docstring style)
/// WITHOUT that indentation becoming literal characters inside the string value —
/// `wat_edn`'s string lexer admits a raw newline inside `"..."` literally, byte for byte
/// (see the module doc), so an indented continuation line would otherwise inject real leading
/// whitespace into the parsed prose and break STOP-3's byte-identical requirement. Mirrors
/// `sniff_doc`'s own single-leading-space strip per `///` line: same purpose (keep the SOURCE
/// readable without polluting the DATA), a measured amount instead of a fixed one — the
/// fence's own least-indented line sets it, exactly like Python's `textwrap.dedent`.
fn dedent(lines: &[&str]) -> String {
    let common = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| if l.len() >= common { &l[common..] } else { "" })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse `edn_src`, validate it is a single `#wat.doc/Row {...}` or `#wat.doc/Alias {...}`
/// tagged map, and convert the body into the `WatAST::Map` shape `wat_doc::from_metadata`
/// accepts. Returns the tag's bare name (`"Row"` / `"Alias"`) and the body as BOTH a `WatAST`
/// (for `from_metadata`) and the original `wat_edn::Value` (so the caller can additionally
/// recover literal example source text — see `example_texts_from_edn_body` — from the SAME
/// parse, not a second one).
pub(crate) fn parse_edn_doc_row(
    edn_src: &str,
) -> Result<(&'static str, WatAST, wat_edn::Value<'_>), wat_doc::DocError> {
    let value = wat_edn::parse(edn_src)
        .map_err(|e| wat_doc::DocError::EdnMalformed { why: e.to_string() })?;
    let (tag_name, body) = match value {
        wat_edn::Value::Tagged(tag, body) => {
            let tag_name: &'static str = match (tag.namespace(), tag.name()) {
                ("wat.doc", "Row") => "Row",
                ("wat.doc", "Alias") => "Alias",
                _ => {
                    return Err(wat_doc::DocError::EdnUnknownTag {
                        got: format!("#{}/{}", tag.namespace(), tag.name()),
                    })
                }
            };
            (tag_name, *body)
        }
        other => {
            return Err(wat_doc::DocError::EdnUnknownTag { got: describe_edn_shape(&other) });
        }
    };
    if !matches!(body, wat_edn::Value::Map(_)) {
        return Err(wat_doc::DocError::EdnUnknownTag {
            got: format!("#wat.doc/{} body is a {}, not a map", tag_name, describe_edn_shape(&body)),
        });
    }
    let map_ast = edn_value_to_watast(&body)
        .map_err(|why| wat_doc::DocError::EdnValueNotRepresentable { why })?;
    Ok((tag_name, map_ast, body))
}

/// A short shape name for an `EdnUnknownTag` diagnostic — never the full value (which may be
/// large), just enough to say what was found instead of a tag.
fn describe_edn_shape(v: &wat_edn::Value<'_>) -> String {
    match v {
        wat_edn::Value::Nil => "nil".into(),
        wat_edn::Value::Bool(_) => "a bool".into(),
        wat_edn::Value::Integer(_) | wat_edn::Value::BigInt(_) => "an integer".into(),
        wat_edn::Value::Float(_) | wat_edn::Value::BigDec(_) => "a float".into(),
        wat_edn::Value::Rational(_) => "a rational".into(),
        wat_edn::Value::String(_) => "a string".into(),
        wat_edn::Value::Char(_) => "a char".into(),
        wat_edn::Value::Symbol(_) => "a symbol".into(),
        wat_edn::Value::Keyword(_) => "a keyword".into(),
        wat_edn::Value::List(_) => "a list".into(),
        wat_edn::Value::Vector(_) => "a vector".into(),
        wat_edn::Value::Map(_) => "a map".into(),
        wat_edn::Value::Set(_) => "a set".into(),
        wat_edn::Value::Tagged(t, _) => format!("a #{}/{} tagged value", t.namespace(), t.name()),
        wat_edn::Value::Inst(_) => "an inst".into(),
        wat_edn::Value::Uuid(_) => "a uuid".into(),
    }
}

/// A synthetic point-span for a `WatAST` node built from EDN data rather than parsed wat
/// source — there is no source location to carry. Mirrors `rust_caller_span!`'s role for
/// Rust-side call sites: a label, not a lie about provenance.
fn synthetic_span() -> Span {
    Span::new(std::sync::Arc::new("<edn-doc-row>".to_string()), 0, 0)
}

/// The reverse of the ns/name transform `edn::write` already performs on a wat FQDN
/// (`:wat::core::foldl` -> `:wat.core/foldl`, measured lossless in the SCORE): every `.` in
/// the namespace becomes `::`, then `::` + the bare name, with the leading `:` every
/// `WatAST::Keyword` carries. `ns: None` is a bare (unqualified) keyword, e.g. `:doc`.
fn fqdn_of(ns: Option<&str>, name: &str) -> String {
    match ns {
        Some(ns) => format!(":{}::{}", ns.replace('.', "::"), name),
        None => format!(":{}", name),
    }
}

/// Generic, structural `wat_edn::Value -> WatAST` transcoder. See the module doc for what
/// this deliberately does NOT do (no doc-contract knowledge) and which four shapes have no
/// `WatAST` spelling at all.
fn edn_value_to_watast(v: &wat_edn::Value) -> Result<WatAST, String> {
    let span = synthetic_span();
    Ok(match v {
        wat_edn::Value::Nil => WatAST::NilLit(span),
        wat_edn::Value::Bool(b) => WatAST::BoolLit(*b, span),
        wat_edn::Value::Integer(i) => WatAST::IntLit(*i, span),
        wat_edn::Value::BigInt(b) => WatAST::BigIntLit((**b).clone(), span),
        wat_edn::Value::Float(f) => WatAST::FloatLit(*f, span),
        wat_edn::Value::Rational(r) => WatAST::RationalLit((**r).clone(), span),
        wat_edn::Value::Char(c) => WatAST::CharLit(*c, span),
        wat_edn::Value::String(s) => WatAST::StringLit(s.to_string(), span),
        wat_edn::Value::Keyword(k) => WatAST::Keyword(fqdn_of(k.namespace(), k.name()), span),
        wat_edn::Value::Symbol(s) => match s.namespace() {
            None => WatAST::Symbol(Identifier::bare(s.name().to_string()), span),
            Some(ns) => {
                return Err(format!(
                    "namespaced symbol `{}/{}` has no WatAST spelling (WatAST::Symbol carries a bare Identifier only)",
                    ns, s.name()
                ))
            }
        },
        wat_edn::Value::List(items) => WatAST::List(
            items.iter().map(edn_value_to_watast).collect::<Result<Vec<_>, _>>()?,
            span,
        ),
        wat_edn::Value::Vector(items) => WatAST::Vector(
            items.iter().map(edn_value_to_watast).collect::<Result<Vec<_>, _>>()?,
            span,
        ),
        wat_edn::Value::Set(items) => WatAST::Set(
            items.iter().map(edn_value_to_watast).collect::<Result<Vec<_>, _>>()?,
            span,
        ),
        wat_edn::Value::Map(pairs) => {
            let mut out = Vec::with_capacity(pairs.len());
            for (k, val) in pairs {
                out.push((edn_value_to_watast(k)?, edn_value_to_watast(val)?));
            }
            WatAST::Map(out, span)
        }
        wat_edn::Value::BigDec(_) => {
            return Err("BigDecimal has no WatAST literal spelling".to_string())
        }
        wat_edn::Value::Tagged(tag, _) => {
            return Err(format!(
                "nested tagged value #{}/{} has no WatAST spelling (a tag resolves to a \
                 registered record only at eval time, never as a raw AST node)",
                tag.namespace(),
                tag.name()
            ))
        }
        wat_edn::Value::Inst(_) => {
            return Err("Inst (timestamp) has no WatAST literal spelling".to_string())
        }
        wat_edn::Value::Uuid(_) => return Err("Uuid has no WatAST literal spelling".to_string()),
    })
}

/// Render an EDN value back as wat SOURCE TEXT — used ONLY to recover literal
/// `&'static str` text for `ExampleSubmission` (the codegen struct `render-doc`'s Examples
/// section reads). This is a second RENDERING of the one `wat_edn::parse` call `emit` already
/// made (see `example_texts_from_edn_body`), not a second parse of the fence and not a
/// doc-contract decoder — it has no notion of which keys are required or what an axis value
/// means, it just prints data back out as wat surface syntax.
fn print_edn_as_wat_source(v: &wat_edn::Value) -> String {
    match v {
        wat_edn::Value::Nil => "nil".to_string(),
        wat_edn::Value::Bool(b) => b.to_string(),
        wat_edn::Value::Integer(i) => i.to_string(),
        wat_edn::Value::BigInt(b) => format!("{b}N"),
        wat_edn::Value::Float(f) => f.to_string(),
        wat_edn::Value::BigDec(d) => d.to_string(),
        wat_edn::Value::Rational(r) => r.to_string(),
        wat_edn::Value::Char(c) => format!("\\{c}"),
        wat_edn::Value::String(s) => format!("\"{}\"", escape_wat_string(s)),
        wat_edn::Value::Symbol(s) => match s.namespace() {
            Some(ns) => format!("{}/{}", ns, s.name()),
            None => s.name().to_string(),
        },
        wat_edn::Value::Keyword(k) => fqdn_of(k.namespace(), k.name()),
        wat_edn::Value::List(items) => format!(
            "({})",
            items.iter().map(print_edn_as_wat_source).collect::<Vec<_>>().join(" ")
        ),
        wat_edn::Value::Vector(items) => format!(
            "[{}]",
            items.iter().map(print_edn_as_wat_source).collect::<Vec<_>>().join(" ")
        ),
        wat_edn::Value::Set(items) => format!(
            "#{{{}}}",
            items.iter().map(print_edn_as_wat_source).collect::<Vec<_>>().join(" ")
        ),
        wat_edn::Value::Map(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .flat_map(|(k, v)| [print_edn_as_wat_source(k), print_edn_as_wat_source(v)])
                .collect();
            format!("{{{}}}", inner.join(" "))
        }
        wat_edn::Value::Tagged(tag, body) => {
            format!("#{}/{} {}", tag.namespace(), tag.name(), print_edn_as_wat_source(body))
        }
        wat_edn::Value::Inst(dt) => dt.to_rfc3339(),
        wat_edn::Value::Uuid(u) => u.to_string(),
    }
}

fn escape_wat_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Recover `(expr_text, expected_text)` pairs from a fence body's `:examples` entries — the
/// literal text `ExampleSubmission` needs. `body` is the exact `wat_edn::Value` `emit` already
/// parsed and fed to `edn_value_to_watast`; this walks it a second time only to PRINT it, not
/// to re-derive or re-validate its shape (that already happened inside `from_metadata` by the
/// time a caller trusts this function's output — see the call site in `wat_intrinsic.rs`).
/// Every metadata-map example is `run: true` (mirrors `from_metadata`'s own `:examples` rule:
/// there is no metadata-map spelling yet for `@example-norun`'s optional-expected shape), so
/// `expected_text` is always `Some`.
pub(crate) fn example_texts_from_edn_body(body: &wat_edn::Value<'_>) -> Vec<(String, Option<String>)> {
    let pairs = match body {
        wat_edn::Value::Map(pairs) => pairs,
        _ => return Vec::new(),
    };
    let examples = pairs.iter().find_map(|(k, v)| match k {
        wat_edn::Value::Keyword(kw) if kw.namespace().is_none() && kw.name() == "examples" => {
            Some(v)
        }
        _ => None,
    });
    let items = match examples {
        Some(wat_edn::Value::Vector(items)) => items,
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|entry| match entry {
            wat_edn::Value::Vector(fields) if fields.len() == 2 => Some((
                print_edn_as_wat_source(&fields[0]),
                Some(print_edn_as_wat_source(&fields[1])),
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_fence_is_none() {
        assert_eq!(extract_edn_fence("plain prose\n@added 1.0.0\n"), None);
    }

    #[test]
    fn finds_a_terminated_fence() {
        let raw = "```edn\n#wat.doc/Row {:added \"1.0.0\"}\n```\n";
        assert_eq!(
            extract_edn_fence(raw).as_deref(),
            Some("#wat.doc/Row {:added \"1.0.0\"}")
        );
    }

    #[test]
    fn unterminated_fence_is_none() {
        let raw = "```edn\n#wat.doc/Row {:added \"1.0.0\"}\n";
        assert_eq!(extract_edn_fence(raw), None);
    }

    /// The indentation-vs-byte-identity tension the builder flagged mid-stone: a
    /// Clojure-docstring-style indented fence (every line, including a multi-line string's
    /// continuation lines, aligned under `#wat.doc/Row {`) must dedent down to the SAME text a
    /// flush-left fence would parse to — the common margin is stripped, never a per-line guess.
    #[test]
    fn indented_fence_dedents_to_the_flush_left_reading() {
        let raw = "```edn\n  #wat.doc/Row {\n    :doc \"line one\n  line two\"\n    :added \"1.0.0\"\n  }\n```\n";
        let got = extract_edn_fence(raw).expect("fence found");
        assert_eq!(
            got,
            "#wat.doc/Row {\n  :doc \"line one\nline two\"\n  :added \"1.0.0\"\n}"
        );
    }

    #[test]
    fn tag_resolves_row_and_body_becomes_a_watast_map() {
        let (tag, ast, _body) =
            parse_edn_doc_row(r#"#wat.doc/Row {:added "1.0.0"}"#).expect("parses");
        assert_eq!(tag, "Row");
        match ast {
            WatAST::Map(pairs, _) => {
                assert_eq!(pairs.len(), 1);
                match &pairs[0] {
                    (WatAST::Keyword(k, _), WatAST::StringLit(s, _)) => {
                        assert_eq!(k, ":added");
                        assert_eq!(s, "1.0.0");
                    }
                    other => panic!("unexpected pair shape: {other:?}"),
                }
            }
            other => panic!("expected a Map, got {other:?}"),
        }
    }

    #[test]
    fn tag_resolves_alias() {
        let (tag, _, _) = parse_edn_doc_row(r#"#wat.doc/Alias {:alias :wat.core/foldl}"#)
            .expect("parses");
        assert_eq!(tag, "Alias");
    }

    #[test]
    fn bad_tag_is_an_error_naming_it() {
        let err = parse_edn_doc_row(r#"#wat.doc/Bogus {:added "1.0.0"}"#).unwrap_err();
        assert_eq!(err, wat_doc::DocError::EdnUnknownTag { got: "#wat.doc/Bogus".to_string() });
    }

    #[test]
    fn untagged_value_is_an_error() {
        let err = parse_edn_doc_row(r#"{:added "1.0.0"}"#).unwrap_err();
        assert_eq!(err, wat_doc::DocError::EdnUnknownTag { got: "a map".to_string() });
    }

    #[test]
    fn malformed_edn_is_an_error() {
        let err = parse_edn_doc_row("#wat.doc/Row {:added").unwrap_err();
        assert!(matches!(err, wat_doc::DocError::EdnMalformed { .. }));
    }

    #[test]
    fn fqdn_keyword_axis_value_round_trips() {
        let (_, ast, _) =
            parse_edn_doc_row(r#"#wat.doc/Row {:purity :wat.runtime.Purity/Pure}"#).expect("parses");
        match ast {
            WatAST::Map(pairs, _) => match &pairs[0].1 {
                WatAST::Keyword(k, _) => assert_eq!(k, ":wat::runtime::Purity::Pure"),
                other => panic!("unexpected: {other:?}"),
            },
            other => panic!("expected a Map, got {other:?}"),
        }
    }

    #[test]
    fn example_texts_are_recovered_from_the_body() {
        let (_, _, body) = parse_edn_doc_row(
            r#"#wat.doc/Row {:examples [[(:wat.core/char "x") (:wat.core/char "x")]]}"#,
        )
        .expect("parses");
        let texts = example_texts_from_edn_body(&body);
        assert_eq!(
            texts,
            vec![(
                "(:wat::core::char \"x\")".to_string(),
                Some("(:wat::core::char \"x\")".to_string())
            )]
        );
    }

    #[test]
    fn nested_tagged_value_is_not_representable() {
        let err =
            parse_edn_doc_row(r#"#wat.doc/Row {:doc #some.other/Tag {:x 1}}"#).unwrap_err();
        assert!(matches!(err, wat_doc::DocError::EdnValueNotRepresentable { .. }));
    }
}
