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
//! EDN idiom used throughout `edn/error.rs` and `macros/error_edn.rs`.

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
    fn location(&self) -> crate::span::Span {
        crate::edn::contract::location_from_span(&self.span)
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
    ///
    /// Excursus 003 strike B1: reverting step 3c's move — the aggregate's
    /// items are its MEMBERS, not its causes (F3), so this raw form and the
    /// floor form ([`crate::edn::contract::WatError::variant`] below) both
    /// carry them under `:errors`, the aggregate's own declared field.
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
    /// inside the `:errors` array, so re-rendering them here would
    /// double-encode the exact content that array already holds.
    fn message(&self) -> String {
        let n = self.0.len();
        format!("{} type-check error{}", n, if n == 1 { "" } else { "s" })
    }
    /// `CheckErrors` is a collection with no single primary span of its own;
    /// its location is its FIRST item's location — the aggregate happened
    /// wherever its first error is. Every production call site guards
    /// `is_empty()` before constructing a `CheckErrors` (`src/check.rs`,
    /// `src/freeze/env.rs`), so a zero-item aggregate is not reachable here;
    /// `.expect` names that invariant rather than silently fabricating a span.
    fn location(&self) -> crate::span::Span {
        self.0
            .first()
            .expect("CheckErrors must not be empty — every construction site guards is_empty()")
            .location()
    }
    /// Excursus 003 strike B1: the items are the aggregate's OWN `:errors`
    /// field now (reverting step 3c's move onto the shared `causes` floor
    /// slot) — each item already satisfies the floor (`CheckError: WatError`),
    /// so it is embedded via its own `error_edn()`.
    fn variant(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.error_edn()).collect();
        tagged("CheckErrors", OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]))
    }
}

// ─── Low-level builders (mirrors edn/error.rs) ───────────────────────

fn tagged(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::CHECK, variant), Box::new(body))
}

fn kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

// ─── Excursus 003 sweep S1 — the check taxonomy's declaration gates ──────────
//
// G-list, G-strict (per the brief's per-strike gate list;
// docs/excursus/2026/09/003-the-little-wat-findings/
// BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md). Placed as an
// internal `#[cfg(test)]` module (not `tests/diagnostics/`, unlike step 3a's
// `probe_excursus003_step3a_wat_records.rs`): `decode_trusted_wire`
// (`src/edn/render.rs`) is `pub(crate)`, and `CheckErrorKind::MalformedForm`'s
// `remedies` field needs `crate::remedy::Remedy` — a `pub(crate) mod remedy`
// type unreachable from an external integration-test crate. An in-tree unit
// test has both; `TypeMismatch`'s driven instance uses a genuine
// retirement-table `callee` (`:wat::core::struct`) so its `remedies` field
// exercises a REAL, non-empty `#wat.kernel/Remedy` on the wire without ever
// naming the `Remedy` type directly.
//
// G-mirror (no golden moved) is a `git diff --stat -- '*.edn'` check, not a
// Rust assertion — stated in the strike report, not here.
#[cfg(test)]
mod excursus_003_s1_gates {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use super::super::error::{CheckError, CheckErrorKind, CheckErrors, EnsureFnInvalidReason};
    use crate::edn::contract::WatError;
    use crate::edn::render::decode_trusted_wire;
    use crate::span::Span;
    use crate::types::TypeEnv;

    fn s() -> Span {
        Span::new(Arc::new("test.wat".to_string()), 1, 0)
    }

    /// One instance of every `CheckErrorKind` variant (34, measured against
    /// `src/check/error.rs:87`), paired with its Rust variant name — the same
    /// name `error_edn()` tags it with on the wire (`#wat.check/<Name>`).
    /// Excursus 003 strike B1, item 6: `NoMatchingClauseAtCallSite` is now a
    /// declared record too (`wat/check-errors.wat`) — every variant here now
    /// decodes typed; see `g_strict_every_declared_kind_decodes_typed` below.
    fn all_variants() -> Vec<(&'static str, CheckErrorKind)> {
        vec![
            ("ArityMismatch", CheckErrorKind::ArityMismatch { callee: ":user::f".into(), expected: 2, got: 1 }),
            // Arc 241.8 retirement-table hit — exercises a REAL, non-empty
            // `remedies` (a genuine `#wat.kernel/Remedy`) via `type_error_remedies_via`,
            // with no direct `Remedy` construction needed.
            ("TypeMismatch", CheckErrorKind::TypeMismatch {
                callee: ":wat::core::struct".into(), param: "x".into(), expected: "i64".into(), got: "String".into(),
            }),
            ("ReturnTypeMismatch", CheckErrorKind::ReturnTypeMismatch {
                function: ":user::f".into(), expected: "i64".into(), got: "String".into(), remedies: vec![],
            }),
            ("UnknownCallee", CheckErrorKind::UnknownCallee { callee: ":user::ghost".into() }),
            ("MalformedForm", CheckErrorKind::MalformedForm { head: "if".into(), reason: "missing branch".into(), remedies: vec![] }),
            ("CommCallOutOfPosition", CheckErrorKind::CommCallOutOfPosition { callee: ":wat::kernel::send".into() }),
            ("ProcessJoinBeforeOutputDrain", CheckErrorKind::ProcessJoinBeforeOutputDrain {
                process_identifier: "p".into(), output_accessor: ":wat::kernel::Process/stdout".into(), output_accessor_span: s(),
            }),
            ("ProcessJoinHoldsStdinSender", CheckErrorKind::ProcessJoinHoldsStdinSender {
                process_identifier: "p".into(), stdin_sender_span: s(),
            }),
            ("BareLegacyPrimitive", CheckErrorKind::BareLegacyPrimitive { primitive: ":i64".into(), fqdn: ":wat::core::i64".into() }),
            ("BareLegacyUnitType", CheckErrorKind::BareLegacyUnitType),
            ("BareLegacyUnitValue", CheckErrorKind::BareLegacyUnitValue),
            ("BareLegacyUnitName", CheckErrorKind::BareLegacyUnitName),
            ("BareLegacyLetStar", CheckErrorKind::BareLegacyLetStar),
            ("BareLegacyLambda", CheckErrorKind::BareLegacyLambda),
            ("BareLegacyLowercaseFn", CheckErrorKind::BareLegacyLowercaseFn),
            ("BareLegacyContainerHead", CheckErrorKind::BareLegacyContainerHead { head: ":vector".into(), fqdn: ":wat::core::Vector".into() }),
            ("BareLegacyStreamPath", CheckErrorKind::BareLegacyStreamPath { old: ":wat::std::stream::map".into(), new: ":wat::stream::map".into() }),
            ("BareLegacyLruCacheServicePath", CheckErrorKind::BareLegacyLruCacheServicePath {
                old: ":wat::lru::CacheService::get".into(), new: ":wat::lru::get".into(),
            }),
            ("BareLegacyKernelQueuePath", CheckErrorKind::BareLegacyKernelQueuePath {
                old: ":wat::kernel::QueueSender".into(), new: ":wat::kernel::Sender".into(),
            }),
            ("DefRedefForbidden", CheckErrorKind::DefRedefForbidden { name: ":user::x".into(), original_def_span: s() }),
            ("DefRedefTypeChange", CheckErrorKind::DefRedefTypeChange {
                name: ":user::x".into(), prior_type: "i64".into(), new_type: "String".into(), original_def_span: s(),
            }),
            ("UnnamespacedName", CheckErrorKind::UnnamespacedName { name: "x".into() }),
            ("ReservedPrefix", CheckErrorKind::ReservedPrefix { name: ":wat::x".into() }),
            ("DottedName", CheckErrorKind::DottedName { name: ":user::x.y".into() }),
            ("DuplicateScheme", CheckErrorKind::DuplicateScheme { name: ":user::x".into() }),
            ("BareLegacyMainSignature", CheckErrorKind::BareLegacyMainSignature),
            ("BareLegacyConsolePath", CheckErrorKind::BareLegacyConsolePath { path: ":wat::console::log".into() }),
            ("DefRestrictedCallerNotAllowed", CheckErrorKind::DefRestrictedCallerNotAllowed {
                callee: ":wat::x".into(), enclosing_fn: ":user::f".into(), prefixes: vec![":user::".into()],
            }),
            // The ONE STOP-listed kind — see wat/check-errors.wat's header. No
            // record declared; must stay foreign (asserted below).
            ("NoMatchingClauseAtCallSite", CheckErrorKind::NoMatchingClauseAtCallSite {
                name: ":user::clause".into(), called_arity: 1, called_arg_types: vec!["i64".into()],
                attempted_clauses: vec![(1, vec!["String".into()])],
            }),
            ("AmbiguousClauseReturnAtCallSite", CheckErrorKind::AmbiguousClauseReturnAtCallSite {
                name: ":user::clause".into(), called_arg_types: vec!["i64".into()], candidate_returns: vec!["String".into(), "bool".into()],
            }),
            ("GuardExprNotBoolean", CheckErrorKind::GuardExprNotBoolean {
                defclause_name: ":user::clause".into(), clause_index: 0, got_type: "i64".into(),
            }),
            ("EnsureFnInvalid", CheckErrorKind::EnsureFnInvalid {
                defclause_name: ":user::clause".into(), clause_index: 0, reason: EnsureFnInvalidReason::ArityNotOne { got: 2 },
            }),
            ("HygieneScopeDivergence", CheckErrorKind::HygieneScopeDivergence {
                name: ":user::x".into(), ref_key: "x#1".into(), binder_key: "x#2".into(),
            }),
            ("PublicOpInAlarm", CheckErrorKind::PublicOpInAlarm { variant: ":probe::Op::Bump".into(), op_type: ":probe::Op".into() }),
        ]
    }

    /// Excursus 003 strike B1, item 6: the last G-list exception retired.
    /// `NoMatchingClauseAtCallSite` is now a declared record (its
    /// `attempted-clauses` attempts are now tagged `AttemptedClause`s, not a
    /// bare map) — kept as an empty list, not deleted, so a FUTURE STOP has
    /// somewhere to go without re-inventing this scaffold.
    const G_LIST_STOP_EXCEPTIONS: &[&str] = &[];

    /// Scan `wat/check-errors.wat`'s OWN source for every top-level
    /// `(:wat::core::defrecord :wat::check::<Name> ...)` form, excluding the
    /// `CheckErrors` aggregate (not a `CheckErrorKind` variant).
    fn declared_check_kind_names() -> BTreeSet<String> {
        let src = include_str!("../../wat/check-errors.wat");
        let forms = wat_reader::parse_all_with_file(src, "wat/check-errors.wat")
            .expect("wat/check-errors.wat must parse");
        let mut names = BTreeSet::new();
        for form in &forms {
            let wat_reader::WatAST::List(items, _) = form else { continue };
            let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
            if head.as_str() != ":wat::core::defrecord" {
                continue;
            }
            let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
            let Some(bare) = name.as_str().strip_prefix(":wat::check::") else { continue };
            // `CheckErrors` (the aggregate) and `AttemptedClause` (excursus 003 strike B1,
            // item 6's sub-value — `NoMatchingClauseAtCallSite.attempted-clauses`' element
            // type) are declared records in this file but are NOT `CheckErrorKind` variants,
            // so `all_variants()` never produces either tag.
            if bare == "CheckErrors" || bare == "AttemptedClause" {
                continue;
            }
            names.insert(bare.to_string());
        }
        names
    }

    /// G-list — the declaration is the list. The set of tags `error_edn()`
    /// produces for `all_variants()` must equal the set of `defrecord
    /// :wat::check::*` names in `wat/check-errors.wat`, modulo the one ruled
    /// STOP exception (which must stay UNDECLARED, not merely absent from
    /// `all_variants()`).
    ///
    /// Mutation (recorded in the strike report, not re-encoded here): add a
    /// stray `(:wat::core::defrecord :wat::check::Bogus [])` to
    /// `wat/check-errors.wat` — RED (declared has an extra name `produced`
    /// never names).
    #[test]
    fn g_list_declaration_is_the_list() {
        let produced: BTreeSet<String> = all_variants().iter().map(|(name, _)| name.to_string()).collect();
        let declared = declared_check_kind_names();
        let mut expected = produced;
        for ex in G_LIST_STOP_EXCEPTIONS {
            assert!(expected.remove(*ex), "{ex}: expected in the produced set (all_variants) — is it still a real CheckErrorKind variant?");
        }
        assert_eq!(
            expected, declared,
            "declared `:wat::check::<Kind>` records in wat/check-errors.wat must equal the \
             CheckErrorKind tags error_edn() produces, minus the ruled STOP exception(s)"
        );
        for ex in G_LIST_STOP_EXCEPTIONS {
            assert!(
                !declared.contains(*ex),
                "{ex} is a ruled STOP exception (an untagged map on the wire) and must stay undeclared — \
                 declaring it now would mean the STOP no longer applies and this constant is stale"
            );
        }
    }

    /// G-strict — every declared kind decodes typed. For every `CheckErrorKind`
    /// variant, `decode_trusted_wire(error_edn())` must succeed (strict decode
    /// is all-or-nothing: `edn_to_value_caps`'s `foreign` flag is `false` on
    /// this path, so ANY unresolved nested tag anywhere in the tree fails the
    /// WHOLE decode — success here is already proof of full, deep typing, not
    /// just the outer tag). Excursus 003 strike B1's GB4: `G_LIST_STOP_EXCEPTIONS`
    /// is now empty, so EVERY variant — `NoMatchingClauseAtCallSite` included —
    /// must decode `Ok`. This is the "foreign count is 0" gate: mutate by
    /// untagging `AttemptedClause` again (`src/check.rs::clause_attempts_to_edn`)
    /// and this test goes RED on `NoMatchingClauseAtCallSite`.
    ///
    /// Mutation (recorded in the strike report): comment out one
    /// `wat_record_from!` line in `src/types.rs` (e.g. `:wat::check::
    /// ArityMismatch`) — RED, and the assertion message names `ArityMismatch`.
    #[test]
    fn g_strict_every_declared_kind_decodes_typed() {
        let types = TypeEnv::with_builtins();
        for (name, kind) in all_variants() {
            let err = CheckError { span: s(), kind };
            let wire = wat_edn::write(&err.error_edn());
            let decoded = decode_trusted_wire(&wire, Some(&types), None);
            if G_LIST_STOP_EXCEPTIONS.contains(&name) {
                assert!(
                    decoded.is_err(),
                    "{name}: STOP-listed kind was expected to remain foreign (no declared record), \
                     but decode_trusted_wire succeeded — {decoded:?}"
                );
            } else {
                assert!(
                    decoded.is_ok(),
                    "{name}: decode_trusted_wire(error_edn()) must succeed as a typed record; got {decoded:?}"
                );
            }
        }
    }

    /// G-strict, the `CheckErrors` aggregate — `#wat.check/CheckErrors {...}`
    /// with two nested, fully-floored `CheckError` items in `:causes` must
    /// decode typed end to end (the same all-or-nothing strict decode as
    /// above, so this also proves the nested items decode typed).
    #[test]
    fn g_strict_check_errors_aggregate_decodes_typed() {
        let types = TypeEnv::with_builtins();
        let errs = CheckErrors(vec![
            CheckError { span: s(), kind: CheckErrorKind::ArityMismatch { callee: ":user::f".into(), expected: 2, got: 1 } },
            CheckError { span: s(), kind: CheckErrorKind::UnknownCallee { callee: ":user::ghost".into() } },
        ]);
        let wire = wat_edn::write(&errs.error_edn());
        let decoded = decode_trusted_wire(&wire, Some(&types), None);
        assert!(decoded.is_ok(), "CheckErrors aggregate must decode typed; got {decoded:?}");
    }
}
