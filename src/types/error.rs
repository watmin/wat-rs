//! vigilatum: 2026-06-01T02:47:26Z — vigilia 9-spell L1+L2=0
//!
//! Type-declaration error types — Pattern A (Stone 243.3) home.
//!
//! [`TypeError`] carries the source span at the outer struct; variant data
//! lives in [`TypeErrorKind`]. Every constructor demands the span —
//! silent omission is uncompilable.

use crate::span::{span_prefix, Span};
use std::fmt;

/// Type-declaration errors. Pattern A (Stone 243.3): span at the outer
/// struct level; variant data in `TypeErrorKind`. Every constructor demands
/// the span — silent omission is uncompilable.
#[derive(Debug)]
pub struct TypeError {
    pub span: Span,
    pub kind: TypeErrorKind,
}

/// Variant data for [`TypeError`]. Spans live in the outer struct; variants
/// carry ONLY data unique to each failure kind.
///
/// Arc 296 Strike 3a: `#[derive(ToEdn)]` generates `impl crate::to_edn::ToEdn
/// for TypeErrorKind`. All fields use the default derive (`.to_edn()`); no
/// nested error causes — `remedies: Vec<Remedy>` serializes via the blanket
/// `impl<T: ToEdn> ToEdn for Vec<T>`, identical to the deleted `remedies_to_edn`.
#[derive(Debug, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::TYPE)]
pub enum TypeErrorKind {
    /// Arc 138 slice 2 — names the OFFENDING decl's name keyword
    /// (the second declaration that collides). The first registration
    /// is already in the registry; the diagnostic points at the new one
    /// the user is trying to add.
    DuplicateType { name: String },
    /// Arc 138 slice 2 — names the offending name keyword carrying
    /// the reserved prefix.
    ReservedPrefix { name: String },
    /// Arc 138 slice 2 — names the whole malformed decl form
    /// (`(:wat::core::struct ...)` outer span).
    MalformedDecl { head: String, reason: String },
    /// Arc 138 slice 2 — names the bad name keyword.
    MalformedName { raw: String, reason: String },
    /// Arc 138 slice 2 — names the offending field item (the
    /// `(name :Type)` form or whatever stand-in landed in its place).
    MalformedField { reason: String },
    /// Arc 130 follow-up — surface enum / variant / span / remedies at
    /// type-registration time. The shape was previously a bare
    /// `reason: String`, which gave consumers no location data and no
    /// structured remedy when sonnet (or a human) wrote a unit variant
    /// as a bare symbol (`PutAck`) instead of the canonical keyword
    /// (`:PutAck`). Stone 241.10 upgrades from `hint: Option<String>`
    /// to `remedies: Vec<Remedy>` — structured ranked candidates per
    /// the substrate-errors-as-values doctrine (arc 233).
    MalformedVariant {
        enum_name: String,
        offending: String,
        reason: String,
        /// Ranked structured remediation candidates. Empty vec = no
        /// remedy offered. Per `feedback_no_semantic_abuse_of_option`:
        /// `Vec<Remedy>` not `Option<Vec<Remedy>>` — empty IS absence.
        remedies: Vec<crate::remedy::Remedy>,
    },
    /// Arc 138 slice 2 — names the bad type keyword (the
    /// outermost type expression that failed to parse).
    MalformedTypeExpr { raw: String, reason: String },
    /// User source wrote `:Any` (as a bare path or parametric head).
    /// 058-030 forbids the escape hatch; every apparent use has a
    /// principled alternative (`:wat::holon::HolonAST`, parametric T, or a named
    /// enum).
    ///
    /// Arc 138 slice 2 — names the keyword carrying the `:Any`
    /// (the outermost type expression).
    AnyBanned { raw: String },
    /// A typealias's expansion, traced through the currently-registered
    /// aliases, reaches the alias's own name. Detected at registration
    /// time so the wat refuses to start rather than looping at
    /// unification later. Example:
    /// `(typealias :A :B) (typealias :B :A)` — the second registration
    /// fires this error because walking `:B`'s expression reaches `:A`
    /// which already expands to `:B`.
    ///
    /// Arc 138 slice 2 — names the alias decl that closes the
    /// cycle (the new decl whose registration was refused).
    CyclicAlias { name: String },
    /// A parametric typealias was referenced with the wrong number of
    /// type arguments. Example: `(typealias :Pair<A,B> :(A,B))` used as
    /// `:Pair<i64>` — declared 2 params, supplied 1.
    ///
    /// Arc 138 slice 2 — names the call site (where the alias
    /// is referenced with the wrong arity).
    AliasArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    /// Arc 115 slice 2 — a type argument inside a compound (`<>`,
    /// `()`, `fn(...)`, fn return after `->`) carried a leading
    /// `:` it shouldn't. The colon prefix is the wat keyword
    /// marker and lives at the OUTERMOST type position only. Inside
    /// compounds, args are bare Rust symbols.
    ///
    /// Examples (all illegal):
    /// - `:Vec<:String>` — drop the inner `:` → `:Vec<String>`
    /// - `:Result<:Option<i64>,:wat::kernel::ThreadDiedError>` →
    ///   `:Result<Option<i64>,wat::kernel::ThreadDiedError>`
    /// - `:fn(:i64)->:bool` → `:fn(i64)->bool`
    /// - `:(:String,:i64)` → `:(String,i64)`
    ///
    /// Arc 138 slice 2 — names the outermost type keyword
    /// (the keyword whose inner argument carries the illegal colon).
    InnerColonInCompoundArg {
        raw: String,
        offending: String,
    },

    // ─── Stone 237.1 — typeunion declaration errors ─────────────────────────

    /// A typeunion's member graph, traced through the currently-registered
    /// typeunions, closes a cycle. Detected at registration time so
    /// unification cannot loop at use. Example:
    /// `(typeunion :A [:i64 :B]) (typeunion :B [:f64 :A])` — the second
    /// registration fires this error (`:B`'s members reach `:A` which
    /// is already a union containing `:B`).
    CyclicUnion { name: String },

    /// A typeunion was declared with zero members. Use case is unclear;
    /// mirrors the `:Any`-ban rationale for rejecting vacuously-typed
    /// positions early.
    EmptyUnion { name: String },

    /// A typeunion was declared with exactly one member. A single-member
    /// union is identical in effect to a typealias; the diagnostic
    /// recommends `(:wat::core::typealias ...)` instead (one canonical
    /// path per `feedback_wat_llm_first_design`).
    SingleMemberUnion { name: String },

    /// A typeunion's member list contained a shape that typeunion does
    /// not accept. Only `Path`, `Parametric`, and `Tuple` members are
    /// sound; `Fn` (weird dispatch semantics) and `Var` (synthetic;
    /// never appears in user-written declarations) are rejected.
    InvalidUnionMember {
        union_name: String,
        member_form: String,
        reason: String,
    },

    // ─── Stone S-A — typesub (is-a hierarchy) errors ───────────────────────

    /// A `register_subtype(child, parent, span)` call would close a cycle
    /// in the typesub hierarchy — `parent` is already a transitive subtype
    /// of `child`. Refused at registration time so `is_subtype` cannot loop.
    /// The span is the caller-supplied declaration span, not a baked-in unknown.
    CyclicSubtype { child: String, parent: String },

    // ─── Arc 293.W — containment rule ──────────────────────────────────────

    /// A portable aggregate (Record | HolonRecord) declared a field whose type
    /// is non-portable (e.g. a Struct). Such a field cannot be reconstructed
    /// from EDN bytes on the far side of a comms boundary, so a portable
    /// container holding one could never cross — it must not be representable.
    ///
    /// Detected in the post-registration validation pass (after all types are
    /// registered) so forward-references resolve before the check runs.
    ImpureFieldInPureAggregate {
        aggregate: String,
        field: String,
        field_ty: String,
    },

    /// Arc 293.W.2b — the enum counterpart of the containment rule. A `Pure`
    /// enum (`:wat::enum::Pure`) declares that its values hold only pure data (fully
    /// EDN-reconstructable anywhere) — so every variant field must itself be pure.
    /// An impure variant field (a struct, a channel handle) could never be
    /// reconstructed on the far side, so a `Pure` enum holding one could never
    /// cross — it must not be representable. (An `Impure` enum is unrestricted;
    /// declare the enum `:wat::enum::Impure` if it must hold a live resource.)
    ///
    /// Detected in the same post-registration validation pass as the aggregate rule.
    ImpureVariantFieldInPureEnum {
        enum_name: String,
        variant: String,
        field: String,
        field_ty: String,
    },
}


impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeErrorKind::DuplicateType { name } => {
                write!(f, "duplicate type declaration: {}", name)
            }
            TypeErrorKind::ReservedPrefix { name } => write!(
                f,
                "type name {} uses a reserved prefix ({}); user types must use their own prefix",
                name,
                crate::resolve::reserved_prefix_list()
            ),
            TypeErrorKind::MalformedDecl { head, reason } => {
                write!(f, "malformed {} declaration: {}", head, reason)
            }
            TypeErrorKind::MalformedName { raw, reason } => {
                write!(f, "malformed type name {:?}: {}", raw, reason)
            }
            TypeErrorKind::MalformedField { reason } => {
                write!(f, "malformed field: {}", reason)
            }
            TypeErrorKind::MalformedVariant {
                enum_name,
                offending,
                reason,
                remedies,
            } => {
                write!(
                    f,
                    "malformed enum variant in '{}': '{}' — {}",
                    enum_name, offending, reason
                )?;
                let section = crate::remedy::render_remedies(remedies);
                if !section.is_empty() {
                    write!(f, "\n{}", section)?;
                }
                Ok(())
            }
            TypeErrorKind::MalformedTypeExpr { raw, reason } => {
                write!(f, "malformed type expression {:?}: {}", raw, reason)
            }
            TypeErrorKind::AnyBanned { raw } => write!(
                f,
                ":Any is not part of the type system (058-030); use :wat::holon::HolonAST for any algebra value, a named enum for closed heterogeneous sets, or parametric T/K/V for generics. Offending expression: {}",
                raw
            ),
            TypeErrorKind::CyclicAlias { name } => write!(
                f,
                "typealias {} forms a cycle through the current alias graph — refused at registration time so unification doesn't loop",
                name
            ),
            TypeErrorKind::AliasArityMismatch { name, expected, got } => write!(
                f,
                "typealias {} declared with {} type parameter(s), used with {}",
                name, expected, got
            ),
            TypeErrorKind::InnerColonInCompoundArg { raw, offending } => write!(
                f,
                "type expression {} contains an illegal leading ':' on the inner argument {}: \
                 inside `<>`, `()`, or `fn(...)`, type arguments are bare Rust symbols. \
                 The colon prefix marks wat keywords and lives at the OUTERMOST type position \
                 only. Drop the leading ':' on the inner: write {} instead.",
                raw,
                offending,
                raw.replacen(&format!(":{}", offending.trim_start_matches(':')), offending.trim_start_matches(':'), 1)
            ),
            TypeErrorKind::CyclicUnion { name } => write!(
                f,
                "typeunion {} forms a cycle through the current union graph — refused at \
                 registration time so unification cannot loop",
                name
            ),
            TypeErrorKind::EmptyUnion { name } => write!(
                f,
                "typeunion {} has no members — use a non-empty member list `[...]` with at \
                 least two type paths",
                name
            ),
            TypeErrorKind::SingleMemberUnion { name } => write!(
                f,
                "typeunion {} has exactly one member — a single-member union is a typealias \
                 in disguise; use (:wat::core::typealias {name} :MemberType) instead",
                name
            ),
            TypeErrorKind::InvalidUnionMember { union_name, member_form, reason } => write!(
                f,
                "typeunion {} contains an invalid member {}: {}. \
                 Members must be Path, Parametric, or Tuple types.",
                union_name,
                member_form,
                reason
            ),
            TypeErrorKind::CyclicSubtype { child, parent } => write!(
                f,
                "register_subtype({child:?}, {parent:?}) would close a cycle in the typesub \
                 hierarchy — {parent:?} is already a transitive subtype of {child:?}; \
                 refused at registration time so `is_subtype` cannot loop"
            ),
            TypeErrorKind::ImpureFieldInPureAggregate { aggregate, field, field_ty } => write!(
                f,
                "containment rule (arc 293.W): pure aggregate {aggregate:?} may only hold \
                 pure fields — field {field:?} has impure (struct) type {field_ty:?}. \
                 A struct cannot be reconstructed from EDN bytes across a comms boundary; \
                 a record or holon holding a struct field could never cross — it must not exist."
            ),
            TypeErrorKind::ImpureVariantFieldInPureEnum { enum_name, variant, field, field_ty } => write!(
                f,
                "containment rule (arc 293.W.2b): :wat::enum::Pure enum {enum_name:?} may only hold \
                 pure variant fields — variant {variant:?} field {field:?} has impure type \
                 {field_ty:?}, which cannot be reconstructed from EDN bytes across an address-space \
                 boundary. Declare the enum :wat::enum::Impure if it must hold a live resource (it then \
                 stays in shared memory and never crosses)."
            ),
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
    }
}

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────

impl crate::to_edn::WatError for TypeError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (no `file:line` prefix, no multi-line remedy sections — those live in
    /// `:location` and the structured variant fields).
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

impl crate::to_edn::ToEdn for TypeError {
    /// `#wat.kernel/<VariantName> {<variant-fields> :span {…}}` — Pattern A:
    /// variant fields are generated by `#[derive(ToEdn)]` on `TypeErrorKind`;
    /// `:span` is spliced in last by `splice_span` (elided when unknown).
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::splice_span;
        splice_span(self.kind.to_edn(), &self.span)
    }
}

impl std::error::Error for TypeError {}
