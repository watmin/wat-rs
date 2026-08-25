//! Arc 300 Stone D — the gate under `\c`'s literal lane.
//!
//! **The thesis, and it must be something that RUNS.** Before this stone the reader
//! desugared `\a` into `(:wat::core::char/of "a")` at parse time, so wat's own
//! `read-string` handed every wat program — every wat-fix codemod, `wat/lint.wat`,
//! `wat/grep.wat` — a *function call* where the user had written a *character*.
//! Arc 300's law is `VNVS LECTOR NE DIVIDANTVR`; that desugar was its counterexample.
//!
//! A prose claim or a hand-run probe would rot. `a_char_literal_parses_to_char_lit_not_a_call`
//! cannot: restoring the desugar and re-running (the non-vacuity control, 2026-08-25) turns it
//! RED and prints the wreck — `Keyword(":wat::core::char/of", Span { col: 1, end: col 3 })`,
//! a nineteen-character name claiming a two-column span. The other two stayed green under that
//! control and each says so in its own doc comment; only the first is the regression gate.
//!
//! Deliberately at the PARSER level, not the runtime level. `tests/value/wat_arc220_char.rs`
//! already covers evaluation, equality and BMP enforcement — and covered them happily
//! for months while the AST was wrong, because a desugared call *evaluates identically*.
//! Only the AST's shape can tell the two worlds apart.

// rune:lint(no-inlined-wat) — this gate asserts the SOURCE-TEXT -> AST relation itself, which is
// the one claim a fixture cannot carry: `call_beside_value` returns a runtime Value and
// `startup_beside` a loaded world, and neither exposes the parse tree. Routing `"\\a"` through a
// .wat file would also have to make it a legal top-level program, which a bare char literal is not.
// The lint's own guidance names this case ("e.g. a parser/reader test"); the strings here are
// inputs to `parse_all_with_file`, never a program to run.

use wat_reader::ast::WatAST;
use wat_reader::parser::parse_all_with_file;

fn parse_one(src: &str) -> WatAST {
    let forms = parse_all_with_file(src, "<gate-char-literal>")
        .unwrap_or_else(|e| panic!("{src:?} must parse: {e:?}"));
    assert_eq!(forms.len(), 1, "{src:?} must produce exactly one top-level form");
    forms.into_iter().next().expect("one form")
}

/// THE THESIS — `\a` is a literal node, not a call.
///
/// Named and unicode forms are resolved by the LEXER before the parser sees them,
/// so all four must land on the same variant. `A` is included because it is the
/// form furthest from a bare `\a` and the likeliest to be routed differently.
#[test]
fn a_char_literal_parses_to_char_lit_not_a_call() {
    for (src, expected) in [("\\a", 'a'), ("\\newline", '\n'), ("\\space", ' '), ("\\u0041", 'A')] {
        match parse_one(src) {
            WatAST::CharLit(c, _) => assert_eq!(
                c, expected,
                "{src:?} must carry {expected:?}"
            ),
            other => panic!(
                "{src:?} must parse to WatAST::CharLit — the scalar-literal lane every other \
                 literal is in (arc 244 NilLit, arc 300 B RationalLit / C1 BigIntLit). \
                 Got {other:?}. A List here means the parse-time desugar to \
                 `(:wat::core::char/of …)` is back, and wat's reader is lying to every \
                 wat program that reads wat."
            ),
        }
    }
}

/// The literal's span covers exactly its own text.
///
/// ⚠ **This test does NOT guard against the desugar's return, and saying so is the point.**
/// Measured by restoring the desugar and re-running (2026-08-25): it stayed GREEN, because
/// it reads the TOP-LEVEL form's span — which under the desugar is the `List`'s span, and the
/// `List`'s span really is the literal's own 2 columns. The phantom lived one level down, on
/// the synthesized `Keyword` child. `a_char_literal_parses_to_char_lit_not_a_call` is the
/// discriminator; that one went RED, naming the exact wreck.
///
/// It is kept because it asserts a true and load-bearing property — a codemod splices its
/// replacement into a span, so a span that is not its node's own text corrupts source. That
/// is what happened at 50 sites, one of them a test asserting two *different* chars unequal,
/// which would have been silently INVERTED rather than broken.
/// See `255/NOTE-a-name-the-reader-manufactured-has-no-text-to-rewrite.md`.
#[test]
fn a_char_literals_span_covers_exactly_its_own_text() {
    for src in ["\\a", "\\newline", "\\space", "\\u0041"] {
        let form = parse_one(src);
        let span = form.span();
        let end = span.end.as_ref().unwrap_or_else(|| {
            panic!("{src:?}: a span read from SOURCE always has an end; None marks a Rust call site")
        });
        assert_eq!(span.line, end.line, "{src:?}: a char literal cannot straddle a line");
        assert_eq!(
            (end.col - span.col) as usize,
            src.chars().count(),
            "{src:?}: the span must cover exactly the literal's own text — no wider, no narrower"
        );
    }
}

/// The VERB survives — a companion property, not a regression gate (it stayed green under the
/// desugar control, as it should: an explicitly written call is a `List` in both worlds).
///
/// `(:wat::core::char "x")` is a real runtime String→char
/// conversion with its own error surface (length-1, BMP-only); this stone changed what
/// the READER emits, and deliberately did not retire the verb. It still parses as an
/// ordinary call, and the head keyword's span is its own written text.
#[test]
fn the_char_of_verb_still_parses_as_an_ordinary_call() {
    const HEAD: &str = ":wat::core::char";
    match parse_one("(:wat::core::char \"x\")") {
        WatAST::List(items, _) => {
            match items.first().expect("a head") {
                WatAST::Keyword(k, span) => {
                    assert_eq!(k.as_str(), HEAD);
                    let end = span.end.as_ref().expect("a written head has an end");
                    assert_eq!(
                        (end.col - span.col) as usize,
                        HEAD.chars().count(),
                        "an explicitly WRITTEN head spans its own text — unlike the \
                         synthesized head this stone removed"
                    );
                }
                other => panic!("head must be the keyword {HEAD}, got {other:?}"),
            }
        }
        other => panic!("an explicit char/of call must stay a List, got {other:?}"),
    }
}
