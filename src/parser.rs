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

// ─── Excursus 003 sweep S3 (+ strike B2 item 4) — the parse + lex gates ──────
//
// G-list, G-strict (per the brief's per-strike gate list;
// docs/excursus/2026/09/003-the-little-wat-findings/
// BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md), the same
// shape S1/S2 used. Placed here (not `wat-reader`): both `WatError`
// (composing the floor form) and `decode_trusted_wire`/`TypeEnv` live in the
// `wat` crate.
//
// Strike B2 item 4 RETIRED `g_lex_never_produces_a_tag` — the MEASUREMENT S3
// built its header on (`LexErrorKind` never reached the wire as its own tag,
// only as a flattened `Display` string under `:wat::parse::Lex.cause`) — and
// replaced it with `LexErrorKind`'s own G-list/G-strict pair below
// (`g_list_lex_declaration_is_the_list` /
// `g_strict_lex_wire_is_dotted_and_decodes_typed`): the absence that gate
// proved is exactly what this strike cures (`crates/wat-reader/src/
// lexer.rs`'s `LexError`/`LexErrorKind` are now `#[derive(ToEdn)]`,
// `qualified` dot-joining every `LexErrorKind` tag).
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
    /// `cause` field now types `:wat::lex::LexError` (excursus 003 strike B2,
    /// item 4 — see `g_strict_lex_wire_is_dotted_and_decodes_typed` below for
    /// the dedicated proof that the NESTED `LexErrorKind` tag itself is
    /// dotted and typed, not merely that this outer decode succeeds).
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

    /// One instance of every `LexErrorKind` variant (10, measured against
    /// `crates/wat-reader/src/lexer.rs`), paired with its Rust variant name —
    /// the same name the dotted wire tag now carries
    /// (`#wat.lex/LexErrorKind.<Name>`).
    fn all_lex_variants() -> Vec<(&'static str, LexErrorKind)> {
        vec![
            ("UnexpectedChar", LexErrorKind::UnexpectedChar('x')),
            ("UnterminatedString", LexErrorKind::UnterminatedString),
            ("UnknownEscape", LexErrorKind::UnknownEscape('q')),
            ("InvalidNumber", LexErrorKind::InvalidNumber("1.2.3".into())),
            ("UnclosedBracketInKeyword", LexErrorKind::UnclosedBracketInKeyword),
            ("CommaInKeywordBody", LexErrorKind::CommaInKeywordBody),
            ("CommaInSymbolBody", LexErrorKind::CommaInSymbolBody),
            ("AngleTypeHeadInName", LexErrorKind::AngleTypeHeadInName),
            ("InvalidChar", LexErrorKind::InvalidChar("empty char literal".into())),
            ("ControlCharacterInSource", LexErrorKind::ControlCharacterInSource { codepoint: 7 }),
        ]
    }

    /// Scan `wat/lex-errors.wat`'s OWN source for the `(:wat::core::defenum
    /// :wat::lex::LexErrorKind ...)` form's variant keywords (each a bare
    /// `:Name`, optionally followed by a field `Vector` — skipped, not a
    /// variant of its own).
    fn declared_lex_kind_names() -> BTreeSet<String> {
        let src = include_str!("../wat/lex-errors.wat");
        let forms = wat_reader::parse_all_with_file(src, "wat/lex-errors.wat")
            .expect("wat/lex-errors.wat must parse");
        for form in &forms {
            let wat_reader::WatAST::List(items, _) = form else { continue };
            let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
            if head.as_str() != ":wat::core::defenum" {
                continue;
            }
            let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
            if name.as_str() != ":wat::lex::LexErrorKind" {
                continue;
            }
            let mut names = BTreeSet::new();
            let mut i = 3; // skip head keyword, type name, purity marker
            while i < items.len() {
                if let wat_reader::WatAST::Keyword(kw, _) = &items[i] {
                    if let Some(bare) = kw.as_str().strip_prefix(':') {
                        names.insert(bare.to_string());
                    }
                    i += 1;
                    if matches!(items.get(i), Some(wat_reader::WatAST::Vector(_, _))) {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            return names;
        }
        panic!("wat/lex-errors.wat must declare (:wat::core::defenum :wat::lex::LexErrorKind ...)");
    }

    /// G-list — the declaration is the list, for the lex taxonomy. Mirrors
    /// `g_list_declaration_is_the_list` above, for `LexErrorKind`.
    ///
    /// Mutation (recorded in the strike report): add a stray `:Bogus` variant
    /// to `wat/lex-errors.wat`'s `defenum` — RED (declared has an extra name
    /// `produced` never names).
    #[test]
    fn g_list_lex_declaration_is_the_list() {
        let produced: BTreeSet<String> = all_lex_variants().iter().map(|(name, _)| name.to_string()).collect();
        let declared = declared_lex_kind_names();
        assert_eq!(
            produced, declared,
            "declared :wat::lex::LexErrorKind variants in wat/lex-errors.wat must equal the \
             LexErrorKind tags produced via ParseErrorKind::Lex.cause"
        );
    }

    /// G-strict, PLUS the dotted-tag proof — excursus 003 strike B2, item 4.
    /// CLOSES `g_lex_never_produces_a_tag`'s retired absence: every
    /// `LexErrorKind` variant now produces a REAL dotted tag
    /// (`#wat.lex/LexErrorKind.<Name>`, nested inside `ParseErrorKind::
    /// Lex.cause`'s own `#wat.lex/LexError {:position :kind}`), and decodes,
    /// through the typed decoder, AS THE ENUM — not a generic untyped map,
    /// and not the flattened `Display` string the retired gate proved was
    /// the ENTIRE wire form before this strike.
    ///
    /// Mutation (recorded in the strike report): drop `qualified` from
    /// `LexErrorKind`'s derive attribute (`crates/wat-reader/src/lexer.rs`)
    /// — every variant's tag reverts to flat `#wat.lex/<Variant>`; RED on the
    /// dotted-tag-name assertion below for all 10 (the derive has no
    /// per-variant granularity, unlike the hand-written sum types).
    #[test]
    fn g_strict_lex_wire_is_dotted_and_decodes_typed() {
        let types = TypeEnv::with_builtins();
        for (variant_name, kind) in all_lex_variants() {
            let lex_err = LexError { position: 3, kind };
            let err = ParseError { span: s(), kind: ParseErrorKind::Lex(lex_err) };
            let wire = err.error_edn();

            let wat_edn::OwnedValue::Tagged(_, body) = &wire else {
                panic!("{variant_name}: expected a Tagged wire value, got {wire:?}");
            };
            let wat_edn::OwnedValue::Map(fields) = body.as_ref() else {
                panic!("{variant_name}: expected a Map body, got {body:?}");
            };
            let cause = fields
                .iter()
                .find(|(k, _)| matches!(k, wat_edn::OwnedValue::Keyword(kw) if kw.name() == "cause"))
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("{variant_name}: wire has no :cause field"));
            let wat_edn::OwnedValue::Tagged(lex_err_tag, lex_err_body) = cause else {
                panic!("{variant_name}: :cause must be a Tagged value, got {cause:?}");
            };
            assert_eq!(lex_err_tag.namespace(), "wat.lex", "{variant_name}: :cause tag namespace must be wat.lex");
            assert_eq!(lex_err_tag.name(), "LexError", "{variant_name}: :cause tag must be #wat.lex/LexError");
            let wat_edn::OwnedValue::Map(lex_err_fields) = lex_err_body.as_ref() else {
                panic!("{variant_name}: :cause body must be a Map, got {lex_err_body:?}");
            };
            let kind_v = lex_err_fields
                .iter()
                .find(|(k, _)| matches!(k, wat_edn::OwnedValue::Keyword(kw) if kw.name() == "kind"))
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("{variant_name}: LexError wire has no :kind field"));
            let wat_edn::OwnedValue::Tagged(kind_tag, _) = kind_v else {
                panic!("{variant_name}: :kind must be a Tagged value, got {kind_v:?}");
            };
            assert_eq!(kind_tag.namespace(), "wat.lex", "{variant_name}: :kind tag namespace must be wat.lex");
            assert_eq!(
                kind_tag.name(),
                format!("LexErrorKind.{variant_name}"),
                "{variant_name}: :kind wire tag must be dotted #wat.lex/LexErrorKind.{variant_name}, \
                 not the flat #wat.lex/{variant_name}"
            );

            let decoded = decode_trusted_wire(&wat_edn::write(&wire), Some(&types), None);
            assert!(
                decoded.is_ok(),
                "{variant_name}: decode_trusted_wire(error_edn()) must succeed as a typed record; got {decoded:?}"
            );
        }
    }
}
