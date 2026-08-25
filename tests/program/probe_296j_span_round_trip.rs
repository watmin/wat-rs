//! Stone J (arc 296) — gate item 2: the program-transport round trip
//! preserves every node's real `Span`, not just its semantic structure.
//!
//! `WatAST::eq` is structural-transparent on `Span` (see `wat_reader::span`
//! module docs — two nodes with the same shape but different spans compare
//! equal), so an ordinary `assert_eq!(forms, decoded)` proves NOTHING about
//! span survival even though the round trip already exercises spans. This
//! probe walks both trees in lockstep and compares `Span` fields directly.
//!
//! See `docs/arc/2026/06/296-diagnostics-fully-edn/BRIEF-296-J-the-forms-carry-their-spans.md`.

use wat::span::{Pos, Span};
use wat::edn::bridge::{edn_to_program, program_to_edn};
use wat::WatAST;

/// A real multi-line, multi-form program so nodes carry genuine parser spans
/// (file/line/col AND `end`, per `wat_reader::span`'s module docs) — not
/// synthesized ones.
const SRC: &str = r#"
(:wat::core::defn :myapp/add [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::core.i64/+ x y))

(:wat::core::defn :user::main [] -> :wat::core::nil
  nil)
"#;

/// Recursively assert every node's `Span` (file, line, col, end) is IDENTICAL
/// between `orig` and `decoded` — not merely that the trees are structurally
/// equal (`WatAST::eq` cannot see this; see module docs above).
fn assert_spans_match(orig: &WatAST, decoded: &WatAST, path: &str) {
    let (os, ds): (&Span, &Span) = (orig.span(), decoded.span());
    assert_eq!(
        os.file.as_str(),
        ds.file.as_str(),
        "{path}: span.file mismatch (orig {os}, decoded {ds})"
    );
    assert_eq!(os.line, ds.line, "{path}: span.line mismatch (orig {os}, decoded {ds})");
    assert_eq!(os.col, ds.col, "{path}: span.col mismatch (orig {os}, decoded {ds})");
    let end = |p: &Span| -> Option<(i64, i64)> { p.end.as_ref().map(|Pos { line, col }| (*line, *col)) };
    assert_eq!(
        end(os),
        end(ds),
        "{path}: span.end mismatch (orig {os}, decoded {ds})"
    );

    match (orig, decoded) {
        (WatAST::List(a, _), WatAST::List(b, _))
        | (WatAST::Vector(a, _), WatAST::Vector(b, _))
        | (WatAST::Set(a, _), WatAST::Set(b, _)) => {
            assert_eq!(a.len(), b.len(), "{path}: child count mismatch");
            for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
                assert_spans_match(x, y, &format!("{path}[{i}]"));
            }
        }
        (WatAST::Map(a, _), WatAST::Map(b, _)) => {
            assert_eq!(a.len(), b.len(), "{path}: map pair count mismatch");
            for (i, ((ka, va), (kb, vb))) in a.iter().zip(b.iter()).enumerate() {
                assert_spans_match(ka, kb, &format!("{path}{{{i}}}.key"));
                assert_spans_match(va, vb, &format!("{path}{{{i}}}.val"));
            }
        }
        _ => {}
    }
}

/// GATE — round trip, encode direction: `program_to_edn` then `edn_to_program`
/// reproduces every node's span exactly, for a real parsed program (covers
/// `Some(end)` spans at every depth: literals, keywords, symbols, and the
/// enclosing `List`/`Vector` forms).
#[test]
fn span_survives_program_to_edn_round_trip() {
    let forms = wat::parser::parse_all_with_file(SRC, "probe_296j_span_round_trip.wat")
        .expect("sample program must parse");
    assert!(forms.len() >= 2, "sample program should parse to >= 2 top-level forms");

    let frame = program_to_edn(&forms);
    let decoded = edn_to_program(&frame).expect("edn_to_program must decode the frame it was given");

    assert_eq!(decoded.len(), forms.len(), "form count must survive the round trip");
    for (i, (orig, dec)) in forms.iter().zip(decoded.iter()).enumerate() {
        assert_spans_match(orig, dec, &format!("form[{i}]"));
    }
}

/// GATE — round trip also preserves a point-span (`end: None`) faithfully:
/// a form built programmatically via `WatAST::keyword` (no parser range,
/// `rust_caller_span!()`'s own file/line/col) must decode back to that exact
/// point-span, not acquire a fabricated `end`.
#[test]
fn point_span_with_no_end_survives_round_trip() {
    let synthetic = WatAST::keyword(":user::synthetic-marker");
    assert!(
        synthetic.span().end.is_none(),
        "WatAST::keyword's rust_caller_span!() must be a point-span with no end"
    );

    let frame = program_to_edn(std::slice::from_ref(&synthetic));
    let decoded = edn_to_program(&frame).expect("edn_to_program must decode a single synthetic form");
    assert_eq!(decoded.len(), 1);
    assert_spans_match(&synthetic, &decoded[0], "synthetic");
}
