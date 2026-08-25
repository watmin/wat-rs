//! Arc 296 Strike 2b — `ToEdn` + `WatError` impls for [`CheckError`] and [`CheckErrors`].
//!
//! The hand-written `check_error_to_edn` match body is replaced by
//! `#[derive(ToEdn)]` on [`CheckErrorKind`] (see `src/check/error.rs`).
//! The outer Pattern-A struct calls `splice_span(self.kind.to_edn(), &self.span)`
//! to append `:span` uniformly (D1: primary span key normalized across all variants;
//! secondary spans keep their domain keys via `#[to_edn(key="...")]` field attrs).
//!
//! ## Tag convention
//!
//! `#wat.kernel/<VariantName>` — the variant name from `CheckErrorKind` is
//! the tag discriminator. The outer struct's span is included as `:span` when
//! it is not `crate::rust_caller_span!()`.
//!
//! ## Field naming
//!
//! Single-word field names keep their name (`:callee`, `:expected`, `:got`).
//! Multi-word snake_case field names from the Rust struct are translated to
//! kebab-case (`:thread-binding`, `:process-identifier`). This mirrors the
//! EDN idiom used throughout `runtime_error_edn.rs` and `macros/error_edn.rs`.

use wat_edn::{Keyword, OwnedValue, Tag};

use super::error::{CheckError, CheckErrors};

// ─── ToEdn + WatError impls ──────────────────────────────────────────────────

impl crate::edn::contract::ToEdn for CheckError {
    /// Pattern A: derive on CheckErrorKind generates the variant body;
    /// `:span` appended via `span.to_edn()` (Stone B).
    fn to_edn(&self) -> OwnedValue {
        use crate::edn::contract::edn_kw;
        let kind_val = self.kind.to_edn();
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span.to_edn()));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => other,
        }
    }
}

impl crate::edn::contract::WatError for CheckError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (no `file:line` prefix, no multi-line hint/remedy sections — those live
    /// in `:location` and the structured variant fields).
    fn message(&self) -> String {
        crate::edn::contract::first_line(self.kind.to_string())
    }
    fn location(&self) -> OwnedValue {
        crate::edn::contract::location_from_span(&self.span)
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        use crate::edn::contract::ToEdn;
        crate::edn::contract::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::edn::contract::ToEdn for CheckErrors {
    /// `#wat.kernel/CheckErrors {:errors [#wat.kernel/<Variant> {…} …]}` —
    /// each `CheckError` in the collection is a navigable tagged value, not a
    /// line in a `:detail` prose blob. This is the structured form the
    /// process-boundary IPC path and `--check-output` consumers read.
    fn to_edn(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.to_edn()).collect();
        tagged(
            "CheckErrors",
            OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]),
        )
    }
}

impl crate::edn::contract::WatError for CheckErrors {
    /// Concise COLLECTION summary — a count, NOT the concatenated multi-line
    /// render of every item. Each item carries its own single-line `:message`
    /// inside the recursively-floored `:errors` array, so re-rendering them
    /// here would double-encode the exact content the floor already holds.
    fn message(&self) -> String {
        let n = self.0.len();
        format!("{} type-check error{}", n, if n == 1 { "" } else { "s" })
    }
    /// `CheckErrors` is a collection; no single primary span exists at this
    /// level. Individual `CheckError` items carry their own `:location`.
    fn location(&self) -> OwnedValue {
        OwnedValue::Nil
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    /// Arc 296 strike 2 — RECURSIVE floor: each `CheckError` in `:errors` is
    /// embedded via its `WatError::error_edn()` (floor form: single-line
    /// `:message`, `:location` never `:span`), NOT its raw `to_edn()`. The
    /// collection envelope itself carries no top-level `:span`.
    fn variant(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.error_edn()).collect();
        tagged(
            "CheckErrors",
            OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]),
        )
    }
}

// ─── Low-level builders (mirrors runtime_error_edn.rs) ───────────────────────

fn tagged(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::CHECK, variant), Box::new(body))
}

fn kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}
