//! Error types for the name-resolution pass.
//!
//! [`UnresolvedReference`] — one failed call-head reference with source location.
//! [`ResolveError`] — the top-level error collecting all failures.

use crate::span::Span;
use std::fmt;

/// One unresolved reference, with context about where it appeared.
/// Stone 243.7e: each reference carries its source span so the collection
/// is location-complete without an outer span on [`ResolveError`].
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedReference {
    /// The keyword path that didn't resolve.
    pub path: String,
    /// Human-friendly context: a short phrase like "call head" or
    /// "macro call (not expanded)".
    pub context: &'static str,
    /// Source location of the offending keyword reference. `crate::rust_caller_span!()`
    /// when the site genuinely has no recoverable location.
    pub span: Span,
}

/// Name-resolution errors.
pub enum ResolveError {
    /// One or more references don't resolve. `unresolved` carries ALL
    /// failures so the user can fix them in a single pass.
    UnresolvedReferences(Vec<UnresolvedReference>),
}

impl fmt::Debug for ResolveError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl std::error::Error for ResolveError {}

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────

impl crate::edn::contract::WatError for ResolveError {
    /// Concise COLLECTION summary — a count, NOT the concatenated multi-line
    /// render of every reference (each `UnresolvedReference` carries its own
    /// path / context / location structurally under `:causes`).
    fn message(&self) -> String {
        match self {
            ResolveError::UnresolvedReferences(list) => {
                let n = list.len();
                format!("{} unresolved reference{}", n, if n == 1 { "" } else { "s" })
            }
        }
    }
    /// `ResolveError` is a collection of per-reference failures; its location
    /// is its FIRST item's location — the aggregate happened wherever its
    /// first unresolved reference is. Every production call site
    /// (`src/resolve/walk.rs`, `src/resolve/normalize.rs`) guards
    /// `is_empty()` before constructing one, so a zero-item aggregate is not
    /// reachable here.
    fn location(&self) -> crate::span::Span {
        match self {
            ResolveError::UnresolvedReferences(list) => list
                .first()
                .expect(
                    "ResolveError::UnresolvedReferences must not be empty — every \
                     construction site guards is_empty()",
                )
                .location(),
        }
    }
    /// Excursus 003 step 3c: the items ARE the causes. `UnresolvedReference`
    /// is a sub-value, not one of the 11 `WatError` families, but it GAINS
    /// the floor here (`impl WatError for UnresolvedReference` below) so its
    /// `path`/`context` sit beside a real `:message`/`:location`/`:causes`
    /// instead of the retired bespoke `:unresolved` key.
    fn causes(&self) -> wat_edn::OwnedValue {
        match self {
            ResolveError::UnresolvedReferences(list) => {
                wat_edn::OwnedValue::Vector(list.iter().map(|r| r.error_edn()).collect())
            }
        }
    }
    /// The collection envelope carries no variant-specific fields beyond the
    /// floor.
    fn variant(&self) -> wat_edn::OwnedValue {
        use wat_edn::{OwnedValue, Tag};
        OwnedValue::Tagged(
            Tag::ns(crate::error_ns::RESOLVE, "UnresolvedReferences"),
            Box::new(OwnedValue::Map(vec![])),
        )
    }
}

impl crate::edn::contract::ToEdn for ResolveError {
    /// `#wat.resolve/UnresolvedReferences {:causes [#wat.resolve/UnresolvedReference {…} …]}`
    /// — each failed reference is a navigable tagged value (path, context,
    /// span), not a line in a prose blob. Excursus 003 step 3c: the bespoke
    /// `:unresolved` key retires in favour of `:causes`, matching
    /// `CheckErrors`/`ReteCheckErrors`.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::edn_kw;
        use wat_edn::{OwnedValue, Tag};

        match self {
            ResolveError::UnresolvedReferences(list) => {
                let refs: Vec<OwnedValue> = list.iter().map(|r| r.to_edn()).collect();
                OwnedValue::Tagged(
                    Tag::ns(crate::error_ns::RESOLVE, "UnresolvedReferences"),
                    Box::new(OwnedValue::Map(vec![(edn_kw("causes"), OwnedValue::Vector(refs))])),
                )
            }
        }
    }
}

impl crate::edn::contract::ToEdn for UnresolvedReference {
    /// `#wat.resolve/UnresolvedReference {:path :context :span}` — Stone B:
    /// `span.to_edn()` emits the derive-generated typed `#wat.core/Span`
    /// record.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::{edn_kw, edn_str};
        use wat_edn::{OwnedValue, Tag};
        let fields = vec![
            (edn_kw("path"), edn_str(&self.path)),
            (edn_kw("context"), edn_str(self.context)),
            (edn_kw("span"), self.span.to_edn()),
        ];
        OwnedValue::Tagged(Tag::ns(crate::error_ns::RESOLVE, "UnresolvedReference"), Box::new(OwnedValue::Map(fields)))
    }
}

impl crate::edn::contract::WatError for UnresolvedReference {
    /// Excursus 003 step 3c item 2 — `UnresolvedReference` "gains the
    /// floor": it is a sub-value embedded inside `ResolveError::causes()`,
    /// but that embedding is via `error_edn()`, so it needs its own
    /// `message`/`location`/`causes`/`variant`.
    fn message(&self) -> String {
        format!("unresolved reference `{}` ({})", self.path, self.context)
    }
    fn location(&self) -> crate::span::Span {
        self.span.clone()
    }
    fn causes(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Vector(vec![])
    }
    /// `:path`/`:context`, span stripped (it is now `:location`).
    fn variant(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::ToEdn;
        crate::edn::contract::strip_span_from_tagged(self.to_edn())
    }
}
