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
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl std::error::Error for ResolveError {}

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────

impl crate::to_edn::WatError for ResolveError {
    /// Concise COLLECTION summary — a count, NOT the concatenated multi-line
    /// render of every reference (each `UnresolvedReference` carries its own
    /// path / context / span structurally under `:unresolved`).
    fn message(&self) -> String {
        match self {
            ResolveError::UnresolvedReferences(list) => {
                let n = list.len();
                format!("{} unresolved reference{}", n, if n == 1 { "" } else { "s" })
            }
        }
    }
    /// `ResolveError` is a collection of per-reference failures; no single
    /// primary span exists at the top level. Individual references carry
    /// their own spans inside `variant()`.
    fn location(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Nil
    }
    fn causes(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Vector(vec![])
    }
    /// `variant()` returns the existing `to_edn()` form — `#wat.kernel/UnresolvedReferences
    /// {:unresolved […]}` — which carries no top-level `:span` key. Its items
    /// are `UnresolvedReference` SUB-VALUES (not one of the 11 `WatError`
    /// families), so per the strike's scope they stay `to_edn` and keep their
    /// per-reference `:span`.
    fn variant(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::ToEdn;
        self.to_edn()
    }
}

impl crate::to_edn::ToEdn for ResolveError {
    /// `#wat.resolve/UnresolvedReferences {:unresolved [#wat.resolve/UnresolvedReference {…} …]}`
    /// — each failed reference is a navigable tagged value (path, context,
    /// span), not a line in a prose blob. Stone B: `span.to_edn()` emits the
    /// derive-generated typed `#wat.core/Span` record.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::{edn_kw, edn_str};
        use wat_edn::{OwnedValue, Tag};

        match self {
            ResolveError::UnresolvedReferences(list) => {
                let refs: Vec<OwnedValue> = list
                    .iter()
                    .map(|r| {
                        let fields = vec![
                            (edn_kw("path"), edn_str(&r.path)),
                            (edn_kw("context"), edn_str(r.context)),
                            (edn_kw("span"), r.span.to_edn()),
                        ];
                        OwnedValue::Tagged(Tag::ns(crate::error_ns::RESOLVE, "UnresolvedReference"), Box::new(OwnedValue::Map(fields)))
                    })
                    .collect();
                OwnedValue::Tagged(
                    Tag::ns(crate::error_ns::RESOLVE, "UnresolvedReferences"),
                    Box::new(OwnedValue::Map(vec![(edn_kw("unresolved"), OwnedValue::Vector(refs))])),
                )
            }
        }
    }
}
