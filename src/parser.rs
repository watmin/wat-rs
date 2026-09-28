//! S-expression parser — re-exported from `wat-reader`. See `wat_reader::parser` for docs.
//!
//! The `parse_one!` and `parse_all!` macros are re-declared here so call sites
//! in the `wat` crate can continue using `crate::parse_one!()` unchanged.

pub use wat_reader::parser::*;

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────
//
// `ParseError` lives in `wat-reader`; `ToEdn` is now `wat-edn`'s trait.
// `impl ToEdn for ParseError` moved to `wat-reader/src/parser.rs` (orphan
// rule: both trait and type are now foreign to `wat`; the impl must live
// in the crate that owns the type).

impl crate::edn::contract::WatError for ParseError {
    /// Concise single-line headline: the span-free kind Display (no `file:line`
    /// prefix — that lives in `:location`).
    fn message(&self) -> String {
        crate::edn::contract::first_line(self.kind.to_string())
    }
    fn location(&self) -> crate::span::Span {
        crate::edn::contract::location_from_span(&self.span)
    }
    fn causes(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::ToEdn;
        crate::edn::contract::strip_span_from_tagged(self.to_edn())
    }
}

/// Parse one form, auto-capturing the call-site Rust source location.
/// Re-declared in this crate so `crate::parse_one!()` resolves inside
/// `wat`'s own source. Delegates to `parse_one_with_file` (re-exported
/// from `wat-reader`).
#[macro_export]
macro_rules! parse_one {
    ($src:expr $(,)?) => {
        $crate::parser::parse_one_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

/// Parse all forms, auto-capturing the call-site Rust source location.
/// Re-declared in this crate so `crate::parse_all!()` resolves inside
/// `wat`'s own source. Delegates to `parse_all_with_file` (re-exported
/// from `wat-reader`).
#[macro_export]
macro_rules! parse_all {
    ($src:expr $(,)?) => {
        $crate::parser::parse_all_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

// ─── Excursus 003 sweep S3 — the parse (+ measured lex) taxonomy's gates ─────
//
// G-list, G-strict (per the brief's per-strike gate list;
// docs/excursus/2026/09/003-the-little-wat-findings/
// BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md), the same
// shape S1/S2 used, plus `g_lex_never_produces_a_tag` — the MEASUREMENT this
// strike's header (`wat/parse-errors.wat`) is built on: `LexErrorKind` never
// reaches the wire as its own tag, only as a flattened `Display` string
// under `:wat::parse::Lex.cause`. Placed here (not `wat-reader`): both
// `WatError` (composing the floor form) and `decode_trusted_wire` /
// `TypeEnv` live in the `wat` crate.
//
// G-mirror (no golden moved) is a `git diff --stat -- '*.edn'` check, not a
// Rust assertion — stated in the strike report, not here.
#[cfg(test)]
mod excursus_003_s3_gates {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use super::{ParseError, ParseErrorKind};
    use crate::edn::contract::WatError;
    use crate::edn::render::decode_trusted_wire;
    use crate::lexer::{LexError, LexErrorKind};
    use crate::span::Span;
    use crate::types::TypeEnv;

    fn s() -> Span {
        Span::new(Arc::new("test.wat".to_string()), 1, 0)
    }

    /// One instance of every `ParseErrorKind` variant (11, measured against
    /// `crates/wat-reader/src/parser.rs:37`), paired with its Rust variant
    /// name — the same name `error_edn()` tags it with on the wire
    /// (`#wat.parse/<Name>`).
    fn all_variants() -> Vec<(&'static str, ParseErrorKind)> {
        vec![
            ("Lex", ParseErrorKind::Lex(LexError { position: 3, kind: LexErrorKind::UnterminatedString })),
            ("UnexpectedRParen", ParseErrorKind::UnexpectedRParen),
            ("UnclosedParen", ParseErrorKind::UnclosedParen),
            ("UnexpectedRBracket", ParseErrorKind::UnexpectedRBracket),
            ("UnclosedBracket", ParseErrorKind::UnclosedBracket),
            ("UnexpectedRBrace", ParseErrorKind::UnexpectedRBrace),
            ("UnclosedBrace", ParseErrorKind::UnclosedBrace),
            ("MalformedBraceLiteral", ParseErrorKind::MalformedBraceLiteral { reason: "odd number of forms".into() }),
            ("TrailingContent", ParseErrorKind::TrailingContent),
            ("Empty", ParseErrorKind::Empty),
            ("ForgedBinderNamespace", ParseErrorKind::ForgedBinderNamespace { spelling: "$bound/x".into() }),
        ]
    }

    /// Scan `wat/parse-errors.wat`'s OWN source for every top-level
    /// `(:wat::core::defrecord :wat::parse::<Name> ...)` form.
    fn declared_parse_kind_names() -> BTreeSet<String> {
        let src = include_str!("../wat/parse-errors.wat");
        let forms = wat_reader::parse_all_with_file(src, "wat/parse-errors.wat")
            .expect("wat/parse-errors.wat must parse");
        let mut names = BTreeSet::new();
        for form in &forms {
            let wat_reader::WatAST::List(items, _) = form else { continue };
            let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
            if head.as_str() != ":wat::core::defrecord" {
                continue;
            }
            let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
            let Some(bare) = name.as_str().strip_prefix(":wat::parse::") else { continue };
            names.insert(bare.to_string());
        }
        names
    }

    /// G-list — the declaration is the list. The set of tags `error_edn()`
    /// produces for `all_variants()` must equal the set of `defrecord
    /// :wat::parse::*` names in `wat/parse-errors.wat`.
    ///
    /// Mutation (recorded in the strike report): add a stray
    /// `(:wat::core::defrecord :wat::parse::Bogus [])` to
    /// `wat/parse-errors.wat` — RED (declared has an extra name `produced`
    /// never names).
    #[test]
    fn g_list_declaration_is_the_list() {
        let produced: BTreeSet<String> = all_variants().iter().map(|(name, _)| name.to_string()).collect();
        let declared = declared_parse_kind_names();
        assert_eq!(
            produced, declared,
            "declared `:wat::parse::<Kind>` records in wat/parse-errors.wat must equal the \
             ParseErrorKind tags error_edn() produces"
        );
    }

    /// G-strict — every declared kind decodes typed, `Lex` included: its
    /// `cause` field types `:wat::core::String` (a plain string, matching
    /// what is actually on the wire — see `g_lex_never_produces_a_tag`), so
    /// it needs no further taxonomy declared to decode fully typed.
    ///
    /// Also closes the brief's two S2-pending kinds: `LoadErrorKind::Parse`
    /// and `StdlibErrorKind::ParseFailed` both nest a `ParseError` typed
    /// `:wat::core::Error`, all-or-nothing strict decode, so THIS gate
    /// (proving every `ParseErrorKind` variant decodes typed on its own)
    /// is what makes those two flip from foreign to typed — asserted
    /// directly in `src/load/loader.rs` / `src/load/stdlib.rs`'s own gates.
    ///
    /// Mutation (recorded in the strike report): comment out one
    /// `wat_record_from!` line in `src/types.rs` (e.g. `:wat::parse::
    /// UnexpectedRParen`) — RED, and the assertion message names
    /// `UnexpectedRParen`.
    #[test]
    fn g_strict_every_declared_kind_decodes_typed() {
        let types = TypeEnv::with_builtins();
        for (name, kind) in all_variants() {
            let err = ParseError { span: s(), kind };
            let wire = wat_edn::write(&err.error_edn());
            let decoded = decode_trusted_wire(&wire, Some(&types), None);
            assert!(
                decoded.is_ok(),
                "{name}: decode_trusted_wire(error_edn()) must succeed as a typed record; got {decoded:?}"
            );
        }
    }

    /// MEASUREMENT — the brief's explicit instruction before declaring
    /// anything for `lex`: does `LexErrorKind` reach the wire as its own tag?
    ///
    /// Driven, not assumed: every one of `LexErrorKind`'s 10 variants
    /// (`crates/wat-reader/src/lexer.rs:198`) is wrapped in
    /// `ParseErrorKind::Lex` and serialized. The wire text must NEVER
    /// contain a `wat.lex` namespace tag, and must contain the `LexError`'s
    /// OWN `Display` text verbatim — proving the lex failure rides as an
    /// opaque flattened string, not a navigable structured value. This is
    /// why `wat/rete-errors.wat`-style declarations for `:wat::lex::*` do
    /// not exist: there is no produced tag to mirror (see
    /// `wat/parse-errors.wat`'s header).
    ///
    /// Mutation (recorded in the strike report): change
    /// `crates/wat-reader/src/parser.rs`'s `ParseErrorKind::Lex(e) => ...`
    /// arm from `OwnedValue::String(Cow::Owned(e.to_string()))` to a bare
    /// literal (e.g. drop the real message) — RED on the `wire.contains`
    /// assertion, naming the missing text.
    #[test]
    fn g_lex_never_produces_a_tag() {
        let cases: Vec<LexErrorKind> = vec![
            LexErrorKind::UnexpectedChar('x'),
            LexErrorKind::UnterminatedString,
            LexErrorKind::UnknownEscape('q'),
            LexErrorKind::InvalidNumber("1.2.3".into()),
            LexErrorKind::UnclosedBracketInKeyword,
            LexErrorKind::CommaInKeywordBody,
            LexErrorKind::CommaInSymbolBody,
            LexErrorKind::AngleTypeHeadInName,
            LexErrorKind::InvalidChar("empty char literal".into()),
            LexErrorKind::ControlCharacterInSource { codepoint: 7 },
        ];
        assert_eq!(
            cases.len(),
            10,
            "LexErrorKind must have exactly 10 variants — re-measure crates/wat-reader/src/lexer.rs:198"
        );
        // EDN string escaping (`wat_edn::writer::write_string`) doubles a
        // literal backslash (and escapes a literal quote) inside a string
        // body — `LexErrorKind::UnknownEscape('q')`'s Display text contains
        // one, so a naive `wire.contains(&display)` would fail on valid,
        // correctly-escaped EDN. Mirror the same escaping before comparing.
        fn edn_escape(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for c in s.chars() {
                if c == '\\' || c == '"' {
                    out.push('\\');
                }
                out.push(c);
            }
            out
        }

        let types = TypeEnv::with_builtins();
        for kind in cases {
            let lex_err = LexError { position: 3, kind };
            let display = lex_err.to_string();
            let escaped_display = edn_escape(&display);
            // Anchored on the `:cause "…"` KEY specifically — `:message` also
            // embeds the same lex Display text (`ParseErrorKind::Lex`'s own
            // Display wraps it: "lex error: {e}"), so a bare `wire.contains`
            // would pass even if `:cause` itself were replaced with
            // something else entirely. This is the field PURE DECLARATION's
            // `wat::parse::Lex.cause <- :wat::core::String` actually claims.
            let expected_cause_fragment = format!(":cause \"{escaped_display}\"");
            let err = ParseError { span: s(), kind: ParseErrorKind::Lex(lex_err) };
            let wire = wat_edn::write(&err.error_edn());
            assert!(
                !wire.contains("wat.lex"), // rune:lint(loose-assert) — a targeted ABSENCE over the whole wire text (not a deterministic value being pinned): the wire's exact bytes already vary per LexErrorKind variant driven in this loop, and the measurement is "no wat.lex namespace anywhere", not a single golden shape.
                "a LexErrorKind variant must never produce its own `#wat.lex/...` tag on today's \
                 wire — found one in: {wire}"
            );
            assert!(
                wire.contains(&expected_cause_fragment),
                "the lex error's own Display text ({display:?}) must ride flattened under the \
                 `:cause` key itself (expected to find {expected_cause_fragment:?}) — not found \
                 in: {wire}"
            );
            let decoded = decode_trusted_wire(&wire, Some(&types), None);
            assert!(
                decoded.is_ok(),
                "Lex must decode fully typed (`cause` is a plain String field, no nested tag to \
                 resolve); got {decoded:?}"
            );
        }
    }
}
