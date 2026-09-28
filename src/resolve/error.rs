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

// ─── Excursus 003 sweep S2 — the ResolveError taxonomy's declaration gates ───
//
// G-list, G-strict (per the brief's per-strike gate list;
// docs/excursus/2026/09/003-the-little-wat-findings/
// BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md), the same
// shape S1 used in `src/check/error_edn.rs::excursus_003_s1_gates`. Only ONE
// `ResolveError` variant exists (`UnresolvedReferences`) plus its nested
// `UnresolvedReference` sub-value — both driven here.
//
// G-mirror (no golden moved) is a `git diff --stat -- '*.edn'` check, not a
// Rust assertion — stated in the strike report, not here.
#[cfg(test)]
mod excursus_003_s2_gates {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use super::{ResolveError, UnresolvedReference};
    use crate::edn::contract::WatError;
    use crate::edn::render::decode_trusted_wire;
    use crate::span::Span;
    use crate::types::TypeEnv;

    fn s() -> Span {
        Span::new(Arc::new("test.wat".to_string()), 1, 0)
    }

    /// Scan `wat/resolve-errors.wat`'s OWN source for every top-level
    /// `(:wat::core::defrecord :wat::resolve::<Name> ...)` form.
    fn declared_resolve_kind_names() -> BTreeSet<String> {
        let src = include_str!("../../wat/resolve-errors.wat");
        let forms = wat_reader::parse_all_with_file(src, "wat/resolve-errors.wat")
            .expect("wat/resolve-errors.wat must parse");
        let mut names = BTreeSet::new();
        for form in &forms {
            let wat_reader::WatAST::List(items, _) = form else { continue };
            let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
            if head.as_str() != ":wat::core::defrecord" {
                continue;
            }
            let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
            let Some(bare) = name.as_str().strip_prefix(":wat::resolve::") else { continue };
            names.insert(bare.to_string());
        }
        names
    }

    /// G-list — the declaration is the list. `ResolveError` produces exactly
    /// ONE top-level tag (`UnresolvedReferences`); its embedded
    /// `UnresolvedReference` items are a SECOND real produced tag on the wire
    /// (inside `:causes`), also declared. The declared set in
    /// `wat/resolve-errors.wat` must equal both.
    ///
    /// Mutation (recorded in the strike report): add a stray
    /// `(:wat::core::defrecord :wat::resolve::Bogus [])` to
    /// `wat/resolve-errors.wat` — RED (declared has an extra name the
    /// produced set never names).
    #[test]
    fn g_list_declaration_is_the_list() {
        let produced: BTreeSet<String> =
            ["UnresolvedReferences", "UnresolvedReference"].iter().map(|s| s.to_string()).collect();
        let declared = declared_resolve_kind_names();
        assert_eq!(
            produced, declared,
            "declared `:wat::resolve::<Kind>` records in wat/resolve-errors.wat must equal the \
             ResolveError/UnresolvedReference tags error_edn() produces"
        );
    }

    /// G-strict — both the `ResolveError` aggregate and its nested
    /// `UnresolvedReference` items decode typed end to end (the strict decode
    /// is all-or-nothing, so this also proves the embedded items decode typed).
    ///
    /// Mutation (recorded in the strike report): comment out
    /// `wat_record_from!(env, "wat/resolve-errors.wat",
    /// ":wat::resolve::UnresolvedReference")` in `src/types.rs` — RED, naming
    /// the aggregate's decode failure (the nested tag is now unresolved).
    #[test]
    fn g_strict_resolve_error_decodes_typed() {
        let types = TypeEnv::with_builtins();
        let err = ResolveError::UnresolvedReferences(vec![
            UnresolvedReference { path: ":user::ghost".into(), context: "call head", span: s() },
            UnresolvedReference { path: ":user::ghost2".into(), context: "macro call (not expanded)", span: s() },
        ]);
        let wire = wat_edn::write(&err.error_edn());
        let decoded = decode_trusted_wire(&wire, Some(&types), None);
        assert!(decoded.is_ok(), "ResolveError::UnresolvedReferences must decode typed; got {decoded:?}");
    }
}
