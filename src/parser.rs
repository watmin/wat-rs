//! S-expression parser — re-exported from `wat-reader`. See `wat_reader::parser` for docs.
//!
//! The `parse_one!` and `parse_all!` macros are re-declared here so call sites
//! in the `wat` crate can continue using `crate::parse_one!()` unchanged.

pub use wat_reader::parser::*;

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────
//
// `ParseError` lives in the `wat-reader` crate; `ToEdn` is `wat`'s trait.
// The orphan rule permits a LOCAL trait impl for a FOREIGN type, so this
// impl lives in the `wat` crate (here, the wat-side parser module) rather
// than in `wat-reader` (which knows nothing of `ToEdn`).

impl crate::to_edn::WatError for ParseError {
    /// Concise single-line headline: the span-free kind Display (no `file:line`
    /// prefix — that lives in `:location`).
    fn message(&self) -> String {
        crate::to_edn::first_line(self.kind.to_string())
    }
    fn location(&self) -> wat_edn::OwnedValue {
        crate::to_edn::location_from_span(&self.span)
    }
    fn causes(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::ToEdn;
        crate::to_edn::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::to_edn::ToEdn for ParseError {
    /// `#wat.kernel/<VariantName> {:span {…} <variant fields>}` — Pattern A:
    /// span at the outer struct. The `Lex` variant nests the underlying
    /// `LexError` message as `:cause` (a foreign leaf error carrying only a
    /// human message); every other variant is fully structured.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::{edn_kw, edn_str, push_span_field};
        use wat_edn::{OwnedValue, Tag};

        let span = &self.span;
        let (variant, mut fields): (&str, Vec<(OwnedValue, OwnedValue)>) = match &self.kind {
            ParseErrorKind::Lex(e) => {
                ("Lex", vec![(edn_kw("cause"), edn_str(&e.to_string()))])
            }
            ParseErrorKind::UnexpectedRParen => ("UnexpectedRParen", vec![]),
            ParseErrorKind::UnclosedParen => ("UnclosedParen", vec![]),
            ParseErrorKind::UnexpectedRBracket => ("UnexpectedRBracket", vec![]),
            ParseErrorKind::UnclosedBracket => ("UnclosedBracket", vec![]),
            ParseErrorKind::UnexpectedRBrace => ("UnexpectedRBrace", vec![]),
            ParseErrorKind::UnclosedBrace => ("UnclosedBrace", vec![]),
            ParseErrorKind::MalformedBraceLiteral { reason } => (
                "MalformedBraceLiteral",
                vec![(edn_kw("reason"), edn_str(reason))],
            ),
            ParseErrorKind::TrailingContent => ("TrailingContent", vec![]),
            ParseErrorKind::Empty => ("Empty", vec![]),
        };
        push_span_field(&mut fields, "span", span);
        OwnedValue::Tagged(Tag::ns(crate::error_ns::PARSE, variant), Box::new(OwnedValue::Map(fields)))
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
