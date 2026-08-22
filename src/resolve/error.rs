//! Error types for the name-resolution pass.
//!
//! [`UnresolvedReference`] — one failed call-head reference with source location.
//! [`ResolveError`] — the top-level error collecting all failures.

use crate::span::Span;
use std::fmt;

/// Arc 109 (a-type-reference-must-resolve) — which HALF of the resolve pass produced an
/// [`UnresolvedReference`]: the original call-head walk (arc 251 and earlier), or the new
/// declared-type-position sweep. Consulted ONLY by `freeze.rs`'s resolve/check precedence —
/// see the doc there. Not serialized to EDN (`ResolveError::to_edn` omits it deliberately, so
/// every existing golden keeps its shape byte-for-byte); it is a Rust-side discriminant, not a
/// wire fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    /// A call-position keyword head, a `:rust::*` coverage gap, or a namespaced symbol ref —
    /// the resolve pass's original subject.
    CallHead,
    /// A declared type position (param, return, field, variant payload, alias RHS, surface
    /// member) naming something that is neither a registered type nor a bound type variable.
    Type,
}

/// One unresolved reference, with context about where it appeared.
/// Stone 243.7e: each reference carries its source span so the collection
/// is location-complete without an outer span on [`ResolveError`].
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedReference {
    /// The keyword path that didn't resolve.
    pub path: String,
    /// Human-friendly context: a short phrase like "call head" or
    /// "macro call (not expanded)", or — for a type reference — the specific declared slot
    /// (`"type in the signature of :user::f, parameter #1"`). Arc 109 widened this from
    /// `&'static str` to `String`: a type-reference context names the enclosing declaration
    /// and slot, which cannot be known at compile time.
    pub context: String,
    /// Source location of the offending keyword reference. `crate::rust_caller_span!()`
    /// when the site genuinely has no recoverable location.
    pub span: Span,
    /// Arc 109 — which half of the pass produced this finding. See [`ReferenceKind`].
    pub kind: ReferenceKind,
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
                            (edn_kw("context"), edn_str(&r.context)),
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
