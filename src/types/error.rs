//! vigilatum: 2026-06-01T02:47:26Z — vigilia 9-spell L1+L2=0
//!
//! Type-declaration error types — Pattern A (Stone 243.3) home.
//!
//! [`TypeError`] carries the source span at the outer struct; variant data
//! lives in [`TypeErrorKind`]. Every constructor demands the span —
//! silent omission is uncompilable.

use crate::span::Span;
use std::fmt;

/// Type-declaration errors. Pattern A (Stone 243.3): span at the outer
/// struct level; variant data in `TypeErrorKind`. Every constructor demands
/// the span — silent omission is uncompilable.
pub struct TypeError {
    span: Span,
    /// Boxed (arc 109 stone C, mirroring `RuntimeError`'s B2). Inline, this
    /// field made `TypeError` 152 bytes; boxed, it is 56 (48 span + 8
    /// pointer), so its width no longer tracks `TypeErrorKind`'s widest
    /// variant. Private — reached only through `new` / `kind` / `into_kind`
    /// — so the box is invisible to callers, the same contract as
    /// `RuntimeError` (`src/value/signal.rs`).
    kind: Box<TypeErrorKind>,
}

impl TypeError {
    /// The ONE door for construction.
    pub fn new(span: Span, kind: TypeErrorKind) -> Self {
        Self { span, kind: Box::new(kind) }
    }
    /// The ONE door for reading the kind.
    pub fn kind(&self) -> &TypeErrorKind {
        &self.kind
    }
    /// The ONE door for taking the kind by value.
    pub fn into_kind(self) -> TypeErrorKind {
        *self.kind
    }
    /// Span stays inline — it is not what this stone boxes.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// Arc 296 stone I — the taxonomy conversion `resolve::register`'s `?` performs at every
/// type-registration call site. `Rejection::verdict` is never `Insert`/`NoOp` (see its
/// doc), so those two arms are unreachable by construction.
impl From<crate::resolve::Rejection> for TypeError {
    fn from(r: crate::resolve::Rejection) -> Self {
        use crate::resolve::Registration;
        let kind = match r.verdict {
            Registration::Duplicate => TypeErrorKind::DuplicateType { name: r.name },
            Registration::Reserved => TypeErrorKind::ReservedPrefix { name: r.name },
            Registration::Unnamespaced => TypeErrorKind::UnnamespacedName { name: r.name },
            Registration::DottedName => TypeErrorKind::DottedName { name: r.name },
            Registration::Insert | Registration::NoOp => {
                unreachable!("resolve::register never rejects with Insert/NoOp")
            }
        };
        TypeError::new(r.span, kind)
    }
}

/// Variant data for [`TypeError`]. Spans live in the outer struct; variants
/// carry ONLY data unique to each failure kind.
///
/// Arc 296 Strike 3a: `#[derive(ToEdn)]` generates `impl crate::edn::contract::ToEdn
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
    /// A top-level type name reached a registration gate with no namespace.
    /// Only fn arguments and `let` bindings may be bare — those are lexical
    /// and never reach a gate. Held against `Privilege::Stdlib` too; there
    /// is no privilege escape from the namespacing wall.
    UnnamespacedName { name: String },
    /// Arc 296 stone H-1 — the type name (the segment after the last `::`) contains a
    /// `.`. Held against `Privilege::Stdlib` too; there is no privilege escape from the
    /// dot wall, same as `UnnamespacedName`. Reserved because a dotted NAME is the wire
    /// discriminator for a tagged-enum variant (`#ns/Enum.Variant`) — a record whose own
    /// name contained a dot could forge that tag.
    DottedName { name: String },
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

    /// BRIEF-construction-inside-a-fn.md, gap (b) — a `HolonRecord` aggregate's OWN
    /// declared field count exceeds the encoding budget (`floor(sqrt(dim_count))`,
    /// `bundle_capacity_verdict`, runtime.rs) at the dimension the program is frozen at.
    /// Both `field_count` (a property of the TYPE's declaration) and `budget` (a property
    /// of the frozen `EncodingCtx`) are freeze-time constants for a given program — not a
    /// per-call-site or per-instance quantity — so every construction of this type would
    /// fail identically at runtime (`build_holon_hologram`); refused at freeze instead so
    /// the failure is reported once, at start, naming the type, rather than at the first
    /// `:then`/`aggregate-new` call that happens to reach it.
    HolonRecordCapacityExceeded {
        aggregate: String,
        field_count: usize,
        budget: usize,
    },

    /// Arc 109 (DESIGN-STONE-a-param-spec-must-be-consumed) — a type declaration's param-spec
    /// named a type parameter that no member type (field, variant, newtype inner, alias body,
    /// union member, or surface field/method) reaches. This is a WALL, not a soundness fix — an
    /// unused param still discriminates types (nominal tagging, `PhantomData`'s use case); the
    /// declaration is rejected for READABILITY: a reader cannot tell a deliberate tag from a
    /// leftover edit unless every param is written into the shape somewhere. Consumption walks
    /// nested type expressions (`crate::declare::typevar::collect_free_type_vars_in`), so
    /// `[x <- (Vector <- [T])]` counts as consuming `T` — only a param absent from EVERY
    /// reachable type expression, at any depth, fires this.
    UnconsumedTypeParam {
        /// The declaration's own name (`TypeDef::name()`), e.g. `:user::R`.
        decl: String,
        /// The offending param's bare name (no leading colon), e.g. `"O"`.
        param: String,
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
            TypeErrorKind::UnnamespacedName { name } => write!(
                f,
                "top-level name '{}' is not namespaced — only fn arguments and let-bindings \
                 may be bare; give it a namespace, e.g. ':my::{}'",
                name,
                name.trim_start_matches(':')
            ),
            TypeErrorKind::DottedName { name } => write!(
                f,
                "type name '{}' contains a '.' in its name segment — reserved: a dot in a \
                 tag's NAME half means \"this is an enum variant\" (`#ns/Enum.Variant`), so a \
                 record name may not contain one, or it could forge that tag; rename without \
                 the dot",
                name
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
                ":Any is not part of the type system (058-030); use :wat::WatAST for any wat form, :wat::holon::HolonAST ONLY for a VSA/HDC algebra value, a named enum for closed heterogeneous sets, or parametric T/K/V for generics. Offending expression: {}",
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
            TypeErrorKind::HolonRecordCapacityExceeded { aggregate, field_count, budget } => write!(
                f,
                "holon record {aggregate:?} declares {field_count} fields, exceeding the encoding \
                 budget of {budget} (floor(sqrt(dim_count)) at this program's configured \
                 dimension) — every construction of this type would fail this same capacity check \
                 at runtime; reduce the field count or raise the encoding dimension \
                 (:wat::config::set-dim-count!)."
            ),
            TypeErrorKind::UnconsumedTypeParam { decl, param } => write!(
                f,
                "type parameter \"{param}\" in {decl}'s param-spec is declared but never used — \
                 every parameter in a type declaration's param-spec must be consumed by a field, \
                 variant, or body type (direct, e.g. `x <- {param}`, or nested, e.g. \
                 `x <- (Vector <- [{param}])`) — an unused parameter still discriminates types, \
                 but that discrimination must be written, not inferred. Remove \"{param}\" from \
                 {decl}'s param-spec, or use it."
            ),
        }
    }
}

impl fmt::Debug for TypeError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────

impl crate::edn::contract::WatError for TypeError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (no `file:line` prefix, no multi-line remedy sections — those live in
    /// `:location` and the structured variant fields).
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

impl crate::edn::contract::ToEdn for TypeError {
    /// Pattern A: derive on TypeErrorKind generates the variant body;
    /// `:span` appended via `span.to_edn()` (Stone B).
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::edn_kw;
        use wat_edn::OwnedValue;
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

impl std::error::Error for TypeError {}
