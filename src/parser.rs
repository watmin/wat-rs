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
    fn location(&self) -> wat_edn::OwnedValue {
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
