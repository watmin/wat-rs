//! Type declarations + the type environment.
//!
//! Four declaration forms per 058-030, each with a distinct head keyword:
//!
//! - `(:wat::core::struct :name (field :Type) ...)` — product type.
//! - `(:wat::core::enum :name :unit-variant (tagged-variant (field :Type)) ...)` —
//!   coproduct type.
//! - `(:wat::core::newtype :name :Inner)` — nominal wrapper.
//! - `(:wat::core::typealias :name :Expr)` — structural alias (same type,
//!   alternative name).
//!
//! Parametric polymorphism (058-030 Q1 resolved YES): the name keyword is followed by a
//! sibling `:- [T U V]` binder declaring type parameters (arc 109 ③ retired the old
//! `<T,U,V>` name-suffix spelling — it is refused outright). Example:
//! `:my::Wrapper :- [T]` declares a type with one type variable `T`.
//!
//! # What this slice does
//!
//! - Classifies each declaration form at startup.
//! - Extracts the name, type parameters, and structural shape (field
//!   name/type pairs, enum variants).
//! - Parses type expressions (`:f64`, `(:Vec :- [T])`, `:fn(T,U)->R`,
//!   `:my::ns::MyType`) into structured [`TypeExpr`] values.
//! - Stores the result in a [`TypeEnv`], keyed by the bare declaration
//!   name (no `:- [T]` binder in the key — parametric types are registered once;
//!   call-site instantiation is [`crate::check`]'s concern).
//! - Rejects duplicate declarations and reserved-prefix names. The
//!   authoritative prefix list is
//!   [`crate::resolve::RESERVED_PREFIXES`].
//!
//! # Scope notes
//!
//! The name-resolution pass resolves call heads; field-position type
//! references are validated at use site, not at registration time.
//! Code generation for Rust-backed compiled binaries is outside wat-rs
//! scope by design — the substrate compiles to its own runtime.

pub mod error;
pub use error::{TypeError, TypeErrorKind};
pub(crate) mod defstruct;
pub(crate) use defstruct::parse_defstruct;
pub(crate) mod surface;
pub(crate) use surface::parse_defsurface;

use crate::ast::WatAST;
use crate::span::Span;
use std::collections::HashMap;
use wat_macros::wat_special_form_impl;

/// Arc 215 stone 1 — type-placeholder path for HM-style inference.
///
/// Appears in type-arg slots of parametric constructor calls to signal
/// "infer this type from the values." Used by:
///
/// - Explicit verb-form with inference: `(:wat::core::HashMap :- [:wat::core::keyword
///   :wat::type::Infer] :k v)` — K is explicit, V is inferred; detected by
///   `infer_hashmap_constructor`, routes to `fresh.fresh()`.
/// - The `#{...}` set-literal verb-form twin is `infer_hashset_constructor`.
///
/// `{...}` map literals and `#{...}` set literals themselves (Arc 257 slice 1)
/// parse to native `WatAST::Map` / `WatAST::Set` nodes — NOT a desugar to a
/// `(:wat::core::HashMap ...)` / `(:wat::core::HashSet ...)` constructor call
/// — and their own `infer_map_literal` / `infer_set_literal` skip the leading
/// type-keyword sentinel slots entirely, starting from fresh type variables
/// directly; `:wat::type::Infer` never appears on that path.
///
/// `parse_type_expr(":wat::type::Infer")` returns
/// `Ok(TypeExpr::Path(":wat::type::Infer"))` — no special registration
/// needed. The constructors in `check.rs` match on this sentinel path
/// and route to `fresh.fresh()` for the inference variable.
///
/// Analogous to Rust's `_` in type position and Haskell's `_` wildcard.
/// NOT a valid user-level type (callers cannot unify against it directly;
/// it dissolves into a concrete type during constructor inference).
pub const INFER_TYPE_PATH: &str = ":wat::type::Infer";

/// A type expression — the shape that appears after `:` in a keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// A bare type path: `:f64`, `:wat::holon::HolonAST`, `:my::ns::Candle`. Lexically-
    /// scoped type variables (`:T`, `:K`, `:V`) also appear as `Path`
    /// when parsed — the type checker distinguishes them via the
    /// enclosing scheme's / declaration's `type_params`.
    ///
    /// `:Any` is banned — the type universe is closed per 058-030's
    /// rejection of the escape hatch. `parse_type_expr` refuses it at
    /// the parse layer.
    Path(String),
    /// `(:wat::core::Vector :- [T])`, `(:wat::core::HashMap :- [K V])`, `(:my::ns::Container :- [wat::holon::HolonAST f64])`.
    Parametric {
        head: String,
        args: Vec<TypeExpr>,
    },
    /// `:fn(T,U)->R`. Function type — arguments and return.
    Fn {
        args: Vec<TypeExpr>,
        ret: Box<TypeExpr>,
    },
    /// Fresh unification variable — synthetic, NEVER produced by
    /// parsing. The checker generates these during scheme
    /// instantiation (one per `type_params` entry per call site) and
    /// substitutes them away when unification succeeds. The integer
    /// is a monotonically-increasing id allocated by the checker's
    /// `InferCtx`.
    Var(u64),
    /// A tuple type — `:(T,U)`, `:(i64,String,bool)`. The empty
    /// tuple `:()` is the unit type (0-tuple). A single-element
    /// keyword like `:(T)` is grouping (flattened to `T`), not a
    /// 1-tuple; write `:(T,)` with a trailing comma for the 1-tuple.
    /// Semantics and written syntax match Rust's tuple types exactly.
    Tuple(Vec<TypeExpr>),
}

/// The one place the bare-parametric-head invariant is written down.
/// `"wat::core::Vector"` → `":wat::core::Vector"`. Idempotent: input that already
/// carries the colon is returned unchanged.
///
/// `TypeExpr::Parametric.head` is stored WITHOUT a leading colon — deliberately,
/// so its two parse paths (`(Head :- [args])` and `(Ctor arg…)`) produce a byte-identical
/// string for unification (see the parsing notes near `parse_type_inner` and
/// `parse_fn_body`). `TypeExpr::Path` carries its colon. Do NOT "fix" this asymmetry
/// at storage — that breaks unification. Normalize at the read site, through this
/// function, instead.
pub(crate) fn parametric_head_fqdn(head: &str) -> String {
    if head.starts_with(':') {
        head.to_string()
    } else {
        format!(":{head}")
    }
}

/// STONE-defservice-emits-the-binder (arc 109) — the ONE renderer for a parametric type
/// reference's surviving spelling, `check::format_type`'s companion the way
/// `parametric_head_fqdn` is `TypeExpr::Parametric.head`'s. `head` is already prefixed
/// however the caller's position needs (colon or bare); `args` are the already-rendered
/// element strings for the binder vector, in that SAME caller's convention (so nested
/// Parametric args recurse through the identical renderer and stay self-consistent).
///
/// `args` EMPTY -> `head` UNCHANGED, never `head<>` — `(Head :- [])` and `Head` are one
/// thing at every position (the builder's rule, STONE row 3, now true at reference position
/// too because this is what stops minting the distinct `Head<>` identity).
/// `args` non-empty -> `(head :- [a b …])` — the surviving form, copy-pasteable into source.
pub(crate) fn render_binder_ref(head: &str, args: &[String]) -> String {
    if args.is_empty() {
        head.to_string()
    } else {
        format!("({head} :- [{}])", args.join(" "))
    }
}

/// STONE-close-the-last-two-channels (arc 109) — the ONE renderer for a function type's
/// surviving spelling, the `TypeExpr::Fn` analogue of [`render_binder_ref`] above. The old
/// `:wat::core::Fn(A,B)->C` FQDN carries a comma inside a keyword body — refused since the
/// comma strike, so for two or more arguments it could not be read back AT ALL. The reader
/// gained a second surface for this reason (`parse_fn_type_bracket`, arc 251.4c): a bracket
/// `[arg… :-> ret]`, unambiguous because each element is its own form in the vector rather
/// than text packed inside one keyword.
///
/// `args`/`ret` are already-rendered strings in the caller's convention (colon-having at a
/// top position, colon-stripped when the caller is itself building a nested/inner spelling)
/// — same contract as `render_binder_ref`'s `args`, so a `Fn` nested inside a binder's `[...]`
/// stays self-consistent with its siblings.
pub(crate) fn render_fn_type_ref(args: &[String], ret: &str) -> String {
    if args.is_empty() {
        format!("[:-> {ret}]")
    } else {
        format!("[{} :-> {ret}]", args.join(" "))
    }
}

impl TypeExpr {
    /// FQDN of this type's head — colon-prefixed, type args stripped.
    /// `None` for variants with no nameable head (Tuple, Fn, …).
    ///
    /// One implementation, two doors: the `Parametric` arm calls
    /// [`parametric_head_fqdn`] rather than re-hand-rolling the colon-prepend.
    pub(crate) fn base_fqdn(&self) -> Option<String> {
        match self {
            TypeExpr::Path(p) => Some(p.clone()),
            TypeExpr::Parametric { head, .. } => Some(parametric_head_fqdn(head)),
            TypeExpr::Fn { .. } | TypeExpr::Var(_) | TypeExpr::Tuple(_) => None,
        }
    }
}

/// Arc 203 — per-struct access-control restrictions, populated by
/// `(:wat::core::struct-restricted ...)` declarations.
///
/// `ctor_whitelist` governs `Name/new`; `field_restrictions` maps each
/// restricted field name to its allowed-caller-prefix whitelist. Fields
/// absent from `field_restrictions` are public (no `:restricted-to` entry
/// in `SymbolTable.binding_metadata` — no restriction means any caller
/// allowed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructRestrictions {
    /// Allowed-caller prefixes for the auto-synthesized `Name/new` constructor.
    pub ctor_whitelist: Vec<String>,
    /// Per-field whitelists. Only restricted fields appear here;
    /// public fields are absent (no whitelist = no restriction).
    pub field_restrictions: HashMap<String, Vec<String>>,
}

/// Arc 293.2b — `Nature` is the one categorical axis for aggregates: the EDN capability trit.
/// Three variants:
///   Struct      = named product type (stays in process, never crosses the wire)
///   Record      = base record (`:wat::core::Record` hierarchy, wire-portable)
///   HolonRecord = holonic record (`:wat::holon::Record` hierarchy, wire-portable + holon_form)
/// Arc 293 S3-Nature-2 — a fourth variant, `Peer`, joins the axis but sits OFF the aggregate
/// contravariant ladder: a `:nature :Peer` surface requires an EXACT match (a dialed
/// `(Peer' :- [S::Op S::Reply])`), not a floor. See `rank()` for why it carries the sentinel `i8::MIN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    Struct,
    Record,
    HolonRecord,
    Peer,
}

impl Nature {
    /// The purity wall: `Struct` permits impurity (holds resources); `Record`/`HolonRecord` guarantee purity.
    /// Arc 293.W.2b — the purity predicate (was renamed from the symptom-name to the cause-name).
    /// Arc 293 S3-Nature-2 — `Peer` joins `Struct` on the impure side: a peer holds a live channel
    /// (crosses no comms; only its address does — the circuit / 293.W `:ephemeral`-only rule).
    pub fn is_pure(&self) -> bool { !matches!(self, Nature::Struct | Nature::Peer) }

    /// Arc 293 K1a — the capability-ladder rank (the balanced trit). A required `:nature` on a surface
    /// is a FLOOR, not an exact kind: a candidate satisfies it iff `candidate.rank() >= required.rank()`.
    /// So `:nature :Struct` (-1) accepts struct+record+holon, `:nature :Record` (0) accepts record+holon,
    /// `:nature :HolonRecord` (+1) accepts holon only — the contravariant ladder of AGGREGATE-MODEL § principle 6.
    /// Arc 293 S3-Nature-2 — `Peer => i8::MIN` is an OFF-LADDER sentinel: `:Peer` does not participate
    /// in the aggregate rank floor at all — the exact-match branch in `nature_floor_ok` handles `:Peer`
    /// surfaces directly, and `MIN` merely guarantees a peer candidate can never clear an *aggregate*
    /// surface's rank floor (`MIN >= any aggregate rank` is false). This value is never the deciding
    /// one for a `:Peer` surface — that path branches before rank is ever consulted.
    pub fn rank(&self) -> i8 {
        match self {
            Nature::Struct => -1,
            Nature::Record => 0,
            Nature::HolonRecord => 1,
            Nature::Peer => i8::MIN,
        }
    }

    /// Arc 293 inheritance annihilation — the nature-root keyword for subtype edge registration.
    /// Every parsed aggregate registers `:Name <: root_keyword()`.
    pub fn root_keyword(&self) -> &'static str {
        match self {
            Nature::Struct => ":wat::core::Struct",
            Nature::Record => ":wat::core::Record",
            Nature::HolonRecord => ":wat::holon::Record",
            Nature::Peer => ":wat::kernel::Peer",
        }
    }

    /// Strict inverse of `root_keyword` — the single canonical keyword→nature map.
    /// Called by both the surface `:nature` parser and `parse_aggregate`.
    /// Returns `None` for anything that is not a nature-root symbol.
    pub fn from_root_keyword(kw: &str) -> Option<Nature> {
        match kw {
            ":wat::core::Struct"  => Some(Nature::Struct),
            ":wat::core::Record"        => Some(Nature::Record),
            ":wat::holon::Record" => Some(Nature::HolonRecord),
            ":wat::kernel::Peer" => Some(Nature::Peer),
            _                     => None,
        }
    }
}

/// Arc 293.W.2b — `Purity` is the enum's purity axis, the sum-type counterpart of the aggregate's
/// `Nature`. An enum has no *backing* (a sum is not "made of" a struct/record), so it declares
/// PURITY directly — whether its values hold only pure data or may hold live resources:
///   Pure   = values hold nothing but data (scalars, records, sums of data; fully EDN-reconstructable
///            anywhere); they serialize to EDN and cross an address-space boundary (process / remote).
///   Impure = values may hold live resources (Sender/socket/closure); bound to their locus; can
///            drift between threads (shared memory, by reference) but never serialize, never leave.
/// Declared (not derived) — mandatory `:wat::enum::Pure` | `:wat::enum::Impure` on `defenum`;
/// the enum-containment pass enforces that a `Pure` enum holds only pure variant fields,
/// exactly as the W.1 pass enforces it for a pure aggregate's fields. See `293/DESIGN-293.W § 293.W.2b`.
/// (⊘ Supersedes `Mobility { Portable, Anchored }` / `:wat::enum::Portable|Anchored` — the movement-frame.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Purity {
    Pure,
    Impure,
}

impl Purity {
    /// The purity wall for enums: a `Pure` enum's values cross address spaces; an `Impure` enum's never do.
    /// Read by `is_pure_type`'s enum arm (mirrors `Nature::is_pure`).
    pub fn is_pure(&self) -> bool { matches!(self, Purity::Pure) }

    /// The single canonical marker-keyword → purity map, for `parse_defenum`'s mandatory marker.
    /// Returns `None` for anything that is not one of the two `:wat::enum::*` markers.
    pub fn from_marker_keyword(kw: &str) -> Option<Purity> {
        match kw {
            ":wat::enum::Pure"   => Some(Purity::Pure),
            ":wat::enum::Impure" => Some(Purity::Impure),
            _                    => None,
        }
    }
}

/// Arc 293 inheritance annihilation — unified product-type declaration (replaces the annihilated
/// `StructDef` + `RecordDef`). Carries `nature: Nature` as the sole categorical position. The
/// `parent` field is DELETED: the nature IS the position; every parsed aggregate registers its
/// subtype edge via `nature.root_keyword()`. Non-nature-root parents are rejected at parse time.
///
/// Produced by:
///   - `parse_defstruct` → `nature: Struct, restrictions: Some/None`
///   - `parse_recordtype` → `nature: Record | HolonRecord, restrictions: None`
///   - `register_builtin_types` → `nature: Struct` for all builtins
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateDef {
    pub name: String,
    pub type_params: Vec<String>,         // structs use; records leave empty
    pub fields: Vec<(String, TypeExpr)>,  // always-typed (D2)
    pub nature: Nature,
    /// Arc 203 — access-control restrictions. Struct-only; always `None` for records.
    pub restrictions: Option<StructRestrictions>,
}

impl AggregateDef {
    /// Iterator over field names in declaration order.
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(|(n, _)| n.as_str())
    }
    /// The field names as a shareable `Arc`, for [`crate::value::AggregateValue::names`] (arc 296 G).
    ///
    /// The door every REGISTRY-HOLDING construction site uses to hand the declaration's names to
    /// the value it is building, so the value never has to be re-named by a lookup later — the
    /// lookup that used to fail four ways and answer `:field-N` to all of them.
    ///
    /// Statically-typed sites with no registry in scope use a `wat_field_names_from!` const
    /// instead; both roads lead to the same `.wat` declaration, and neither passes through a
    /// human typing a field name into Rust.
    ///
    /// **Deliberately NOT a cached field on `AggregateDef`.** A stored `names` alongside `fields`
    /// would be a second copy of one truth inside one struct, free to drift from it — the exact
    /// shape this arc is removing one layer up. `fields` stays the single source; the small
    /// allocation is paid at construction. If it shows up in a profile, that is the moment to
    /// optimise it — with a number, not a guess.
    pub fn names_arc(&self) -> std::sync::Arc<Vec<String>> {
        std::sync::Arc::new(self.fields.iter().map(|(n, _)| n.clone()).collect())
    }
    /// Iterator over field types in declaration order.
    pub fn field_types(&self) -> impl Iterator<Item = &TypeExpr> {
        self.fields.iter().map(|(_, t)| t)
    }
}

/// Enum declaration — coproduct type. Variants are either unit
/// (payload-free) or tagged (with named typed fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    /// Arc 293.W.2b — the enum's purity, declared via the mandatory `:wat::enum::*` marker
    /// on `defenum` (the sum-type counterpart of `AggregateDef.nature`).
    pub purity: Purity,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariant {
    Unit(String),
    Tagged {
        name: String,
        fields: Vec<(String, TypeExpr)>,
    },
}

impl EnumDef {
    /// Field names of a tagged variant, declaration order — the enum mirror of
    /// [`AggregateDef::names_arc`] (arc 296 G′).
    ///
    /// The door every REGISTRY-HOLDING construction site uses to hand a variant's declared
    /// names to the [`crate::value::EnumValue`] it is building, so no render site ever
    /// re-derives them by walking `self.variants` (or worse, guessing `field-N`).
    ///
    /// Returns `None` when the variant is absent from this def, or is a `Unit` variant —
    /// the caller RAISES on `None`; it does not fabricate a positional fallback.
    pub fn variant_names_arc(&self, variant: &str) -> Option<std::sync::Arc<Vec<String>>> {
        self.variants.iter().find_map(|v| match v {
            EnumVariant::Tagged { name, fields } if name == variant => Some(std::sync::Arc::new(
                fields.iter().map(|(n, _)| n.clone()).collect(),
            )),
            _ => None,
        })
    }
}

/// Newtype declaration — nominal wrapper distinct from its inner type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewtypeDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub inner: TypeExpr,
}

/// Typealias — structural alias for an existing type expression.
/// `:A` and its expansion are THE SAME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub expr: TypeExpr,
}

/// Typeunion — named bounded set of types. Stone 237.1.
///
/// `(:wat::core::typeunion :Name [:T1 :T2 ...])` declares a named
/// grouping of two or more types. Unification resolves the union to
/// whichever member matches. Members must be `Path`, `Parametric`, or
/// `Tuple`; `Fn` and `Var` are rejected at registration time.
///
/// `type_params` is reserved for future parametric typeunions
/// (e.g. `typeunion :Result :- [T E]`); arc 237 ships non-parametric only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<TypeExpr>,
}

/// Arc 293.4a — a member of a `defsurface` declaration: either a required named
/// field (row-polymorphic width subtyping) or a required named method (structural
/// "methods are accessors": the satisfier backs it with a `defn :T/<name>`).
///
/// Types-local: does NOT depend on `value`.
/// The `Method` variant carries the full `ArgSpec` from `parse_argspec_triples`
/// (not a flattened `Vec<TypeExpr>`) so the binder names are preserved and the
/// structural check can compare per-position with the candidate `defn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceMember {
    /// A field requirement: type T must have a field `name` with a type assignable to `ty`.
    Field { name: String, ty: TypeExpr },
    /// A method requirement: type T must expose a `defn :T/<name>` with an assignable sig.
    /// `args` is the ArgSpec from parsing `[self …]`; `fixed_params` may be empty when the
    /// surface member uses a bare `[self]` with no type annotation. `ret` is the required
    /// return type. The satisfier checks `defn :<T>/name` ret ← mret (+ arg types per-position
    /// when `args.fixed_params` is non-empty).
    Method {
        name: String,
        args: crate::argspec::ArgSpec,
        ret: TypeExpr,
        type_params: Vec<String>,
        /// Arc 278 #16 Stone 16.0 — per-operation request-byte budget, parsed from the
        /// OPTIONAL `:max-request-bytes N` key in the kwargs options map that may follow
        /// `-> :RetType` on a `:features` op (options are order-independent `:keyword value`
        /// pairs; a later stone adds `:max-page-bytes` to that same map). When the op omits
        /// the key, this still defaults to `edn::render::DEFAULT_MAX_FRAME_BYTES` (512 KiB) cast
        /// to `i64` at PARSE time (non-serviceable surfaces — `:Struct`/`:Record`/
        /// `:HolonRecord` — legitimately ride this default forever; their methods are
        /// in-thread accessors, not wire ops). See `max_request_bytes_explicit` below for
        /// whether the value in this field was actually written by the source.
        max_request_bytes: i64,
        /// Arc 278 #16 Stone 16.3 — true iff the source explicitly wrote `:max-request-bytes`
        /// on this op (false = it rode the silent parse-time default above). Consulted ONLY
        /// by `synthesize_surface_protocol`'s mandatory-budget lock: a `:nature :Peer'`
        /// surface's op omitting `:max-request-bytes` is a LOCATED compile error — a
        /// serviceable op must explicitly speak its wire cap, never ride the silent default
        /// (the whole point of Stone 16.2's per-op enforcement codegen). Non-serviceable
        /// surfaces never consult this field, so their methods staying implicit is fine.
        max_request_bytes_explicit: bool,
    },
}

/// Surface declaration — structural interface (arc 293.3-core).
///
/// `(:wat::core::defsurface :Name [member <- :T ...])` declares a named
/// structural surface. A struct (or record, future arc) satisfies a surface
/// by having every member with a field-type assignable to the member's type
/// (row-polymorphic width subtyping — extra fields are fine). No `:satisfies`,
/// no `:parent`, no declaration at the use site.
///
/// Arc 293.4a — `members` now carries `SurfaceMember` variants (Field or Method).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<SurfaceMember>,
    /// Arc 293 R3 — optional categorical nature bound. `None` → pure-structural (today's behavior).
    /// `Some(h)` → the aggregate's `nature` must equal `h` (enforced in `assignable`).
    pub nature: Option<Nature>,
}

/// One of the declaration variants (arc 293.2b: Struct+Record merged → Aggregate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    /// Arc 293.2b — unified product-type declaration (struct or record, discriminated by kind).
    Aggregate(AggregateDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
    /// Stone 237.1 — named bounded set of types for bounded-existential
    /// unification. See [`UnionDef`].
    Union(UnionDef),
    /// Arc 293.3-core — structural surface for row-polymorphic width subtyping.
    Surface(SurfaceDef),
}

impl TypeDef {
    pub fn name(&self) -> &str {
        match self {
            // Arc 293.2b — Struct + Record collapsed into Aggregate.
            TypeDef::Aggregate(a) => &a.name,
            TypeDef::Enum(e) => &e.name,
            TypeDef::Newtype(n) => &n.name,
            TypeDef::Alias(a) => &a.name,
            TypeDef::Union(u) => &u.name,
            // Arc 293.3-core
            TypeDef::Surface(s) => &s.name,
        }
    }
}

/// Keyword-path ↦ `TypeDef` registry.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    types: HashMap<String, TypeDef>,
    /// Stone 255-builtin-registry — names that have MEMBERSHIP but no STRUCTURE: primitives
    /// (`:wat::core::i64`), built-in parametric container heads (`:wat::core::Vector`), and
    /// opaque capability/handle types (`:wat::kernel::Peer`, `:rust::crossbeam_channel::Sender`)
    /// — Rust structs exposed to wat with no `TypeDef` to hold, so registering them as an
    /// `Aggregate`/`Alias`/etc. would fabricate a structure that does not exist (rejected
    /// options A/B in `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-*`).
    /// `contains` consults both `types` and this set; `get` deliberately does NOT — a builtin
    /// leaf has membership, not structure, and `TypeEnv::get` must keep answering `None` for
    /// these names so that asymmetry stays a queryable fact of the door rather than a
    /// fabricated `TypeDef`. Populated once, in `register_builtin_types`.
    builtin_names: std::collections::HashSet<String>,
    /// Stone S-A — the `typesub` child→parent edge registry.
    /// Maps a child FQDN (e.g. `":wat::holon::Record"`) to the list of its direct
    /// parent FQDNs (e.g. `[":wat::core::Record"]`). Populated by `register_subtype`;
    /// walked (transitively) by `is_subtype`. Distinct from `typeunion` membership:
    /// this is the Clojure `derive`/`isa?` axis — an open directional is-a hierarchy.
    subtype_edges: HashMap<String, Vec<String>>,
    /// Arc 170 — the ORIGINAL source decl form for each user (non-reserved)
    /// type, retained verbatim at registration time. Freeze ships these
    /// across a process fork instead of reconstructing via `type_def_to_ast`
    /// (a hand-written reconstruction that DRIFTS as the grammar evolves —
    /// e.g. `defsurface` reconstruction cannot recover the `:messages` block).
    /// Only user-namespace decls are captured; stdlib types (`:wat::*`) are
    /// re-registered in the child via `with_builtins` and never shipped, and
    /// synthesized `derived` defs (backing records / `::Op` / `::Reply`) have
    /// no user form and fall back to reconstruction.
    source_forms: HashMap<String, WatAST>,
}

// Privilege (user vs stdlib) is the shared `crate::resolve::Privilege` — the ONE bit
// every registration path threads (was the local `RegistrationPrivilege`, collapsed in
// the reserved-prefix-one-gate arc).

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a `TypeEnv` seeded with wat-rs's own built-in type
    /// declarations. This is the **self-trust** path: wat-rs is the
    /// layer that DEFINES what lives under `:wat::*` prefixes, so it
    /// calls [`Self::register_builtin`] directly — the reserved-prefix
    /// check exists to protect wat PROGRAMS from accidentally claiming
    /// those paths, not to protect wat-rs from itself. User source
    /// continues to flow through [`Self::register`] where the gate
    /// still applies.
    ///
    /// Current builtins:
    /// - `:wat::holon::CapacityExceeded` — the error type populated
    ///   in the `Err` slot of a `:Result` returned by
    ///   `:wat::holon::Bundle` under `:error` mode when a frame
    ///   exceeds Kanerva's capacity. Carries `(cost :i64)` and
    ///   `(budget :i64)` in declaration order.
    pub fn with_builtins() -> Self {
        let mut env = Self::default();
        register_builtin_types(&mut env);
        env
    }

    /// Answers MEMBERSHIP: is `name` a real type name, structured or not?
    /// Consults both stores — `types` (has a `TypeDef`) and `builtin_names`
    /// (membership without structure; see the field doc). `get` intentionally
    /// does NOT gain the same `||`: a builtin leaf's whole point is that it
    /// has no structure to return.
    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.builtin_names.contains(name)
    }

    /// Answers STRUCTURE. Deliberately unchanged by the builtin-leaf population
    /// (stone 255-builtin-registry) — a primitive/container/opaque type has
    /// membership (`contains` → true) but no `TypeDef` to return, so this stays
    /// `None` for those names. See `builtin_names`'s field doc.
    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    /// Register a name that has membership but no structure — a primitive, a
    /// built-in parametric container head, or an opaque capability/handle type.
    /// Stone 255-builtin-registry, storage option C (see the DESIGN's
    /// CORRECTION): the door (`contains`) learns the name; `get` stays `None`
    /// for it. Not `pub`: only `register_builtin_types` seeds these, mirroring
    /// `register_builtin`'s privilege.
    fn register_builtin_leaf(&mut self, name: impl Into<String>) {
        let name = name.into();
        debug_assert!(
            !self.types.contains_key(&name),
            "builtin leaf {name} already registered as a structured TypeDef"
        );
        debug_assert!(
            !self.builtin_names.contains(&name),
            "builtin leaf {name} registered twice"
        );
        self.builtin_names.insert(name);
    }

    /// Arc 170 — the retained ORIGINAL source decl form for user type `name`,
    /// if one was captured at registration (only non-reserved user types are).
    /// Freeze/closure-extract prefers this over `type_def_to_ast` reconstruction.
    pub fn source_form(&self, name: &str) -> Option<&WatAST> {
        self.source_forms.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &TypeDef)> {
        self.types.iter()
    }

    /// Build a map from every unit-variant keyword path (`:enum::Variant`) to its
    /// enum type. Allocates a fresh map; the checker calls this once at CheckEnv
    /// construction to seed value-position unit-variant resolution.
    pub fn build_unit_variant_map(&self) -> HashMap<String, TypeExpr> {
        let mut out = HashMap::new();
        for (name, def) in self.iter() {
            if let TypeDef::Enum(e) = def {
                for variant in &e.variants {
                    if let EnumVariant::Unit(variant_name) = variant {
                        out.insert(
                            format!("{}::{}", name, variant_name),
                            TypeExpr::Path(name.clone()),
                        );
                    }
                }
            }
        }
        out
    }

    pub fn register(&mut self, def: TypeDef) -> Result<(), TypeError> {
        // arc 138: no span — public surface preserved; external callers
        // (lib re-export, test helpers) bind a TypeDef without a source
        // form. Spanned routing uses `register_with_span` from
        // `register_types`, which threads the form's decl span.
        self.register_with_span(def, crate::rust_caller_span!())
    }

    /// Arc 138 slice 2 — span-carrying variant. The decl's name keyword
    /// span surfaces through `ReservedPrefix` / `DuplicateType` /
    /// `CyclicAlias` errors so consumers (humans + agents) navigate to
    /// the offending decl.
    pub fn register_with_span(&mut self, def: TypeDef, span: Span) -> Result<(), TypeError> {
        self.register_validated(def, span, crate::resolve::Privilege::User)
    }

    /// Register a TRUSTED stdlib type declaration. Bypasses the
    /// reserved-prefix gate because stdlib wat files live under
    /// `:wat::std::*` by design — same privilege that
    /// [`crate::macros::MacroRegistry::register_stdlib`] grants
    /// stdlib defmacros. User source still goes through
    /// [`Self::register`] where the prefix check catches
    /// mis-namespaced user declarations.
    ///
    /// Duplicates and cyclic aliases are still rejected; arc 054's
    /// idempotency rule applies — byte-equivalent re-registration is
    /// a no-op.
    pub fn register_stdlib(&mut self, def: TypeDef) -> Result<(), TypeError> {
        // arc 138: no span — public surface preserved; matches the
        // user-facing `register()` shape. Real source forms route via
        // `register_stdlib_with_span` from `register_stdlib_types`.
        self.register_stdlib_with_span(def, crate::rust_caller_span!())
    }

    /// Arc 138 slice 2 — span-carrying variant of [`Self::register_stdlib`].
    pub fn register_stdlib_with_span(
        &mut self,
        def: TypeDef,
        span: Span,
    ) -> Result<(), TypeError> {
        self.register_validated(def, span, crate::resolve::Privilege::Stdlib)
    }

    /// Shared guard chain for [`register_with_span`] and
    /// [`register_stdlib_with_span`]. The `privilege` parameter
    /// distinguishes the stdlib path (which bypasses the reserved-prefix
    /// check because stdlib types ARE in the reserved namespace).
    fn register_validated(
        &mut self,
        def: TypeDef,
        span: Span,
        privilege: crate::resolve::Privilege,
    ) -> Result<(), TypeError> {
        let name = def.name().to_string();
        // The ONE gate (resolve::registration). Equivalence is `==` (a byte-equivalent
        // re-declaration is a no-op — Arc 054, e.g. an in-crate shim delivered both via
        // wat_sources() and on-disk, OR a forked child re-baking a stdlib form it holds).
        let existing = match self.types.get(&name) {
            None => crate::resolve::Existing::Absent,
            Some(e) if e == &def => crate::resolve::Existing::Equivalent,
            Some(_) => crate::resolve::Existing::Divergent,
        };
        crate::resolve::register(&name, privilege, existing, &span, || -> Result<(), TypeError> {
            // Reject cyclic aliases BEFORE insertion so `expand_alias` can
            // assume every alias in the registry is non-cyclic.
            if let TypeDef::Alias(alias) = &def {
                check_alias_no_cycle(&name, &alias.expr, self, &span)?;
            }
            // Stone 237.1 — reject typeunions with invalid members or cycles.
            if let TypeDef::Union(union) = &def {
                validate_union_members(&name, &union.members, &span)?;
                check_union_no_cycle(&name, &union.members, self, &span)?;
            }
            // Arc 293 inheritance annihilation — wire subtype edge derived from nature.
            // parse_aggregate rejected any non-nature-root parent, so root_keyword() always
            // names a registered builtin. No ":wat::core::Value" skip needed: every parsed
            // aggregate registers :Name <: nature.root_keyword().
            if let TypeDef::Aggregate(agg) = &def {
                let root = agg.nature.root_keyword();
                self.types.insert(name.clone(), def);
                return self.register_subtype(&name, root, span.clone());
            }
            // Arc 278 the string-wrap annihilation — a `:nature :wat::core::Record` surface
            // IS a subtype of `:wat::core::Record`, exactly like a concrete Record aggregate:
            // every value satisfying the surface is a record, so `:wat::core::Error <:
            // :wat::core::Record`. Without this edge a record accessor (param `:wat::core::Record`)
            // rejects a surface-typed value — e.g. `(:wat::core::Fault/message
            // (:wat::kernel::Failure/error f))`, where `Failure/error` yields `:wat::core::Error`.
            // Restricted to Nature::Record ONLY: a `:nature :HolonRecord` surface must NOT gain a
            // `<: :wat::holon::Record` edge — that would let `is_subtype` short-circuit the holon
            // NATURE LADDER (a non-holon foreign type could then satisfy a holon-floor surface;
            // see probe_arc293_holder_ladder_foreign). Struct/Peer surfaces are unaffected.
            if let TypeDef::Surface(surf) = &def {
                if surf.nature == Some(Nature::Record) {
                    let root = Nature::Record.root_keyword();
                    if name != root {
                        self.types.insert(name.clone(), def);
                        return self.register_subtype(&name, root, span.clone());
                    }
                }
            }
            self.types.insert(name.clone(), def);
            Ok(())
        })?;
        Ok(())
    }

    /// Privileged internal registration — bypasses the reserved-prefix
    /// gate so wat-rs itself can seed `:wat::*` type declarations via
    /// [`Self::with_builtins`]. Not exposed as `pub`: consumer crates
    /// use `register` (or their own `#[wat_dispatch]`-generated shims
    /// under `:rust::*`).
    fn register_builtin(&mut self, def: TypeDef) {
        let name = def.name().to_string();
        debug_assert!(
            !self.types.contains_key(&name),
            "built-in type {} registered twice",
            name
        );
        // Arc 293.W.2b — register the nature-root subtype edge for Aggregate builtins,
        // mirroring what `register` does for user-defined aggregates (types.rs:525-532).
        // Without this edge, builtin Record types (e.g. :wat::kernel::Failure after its
        // Nature::Struct → Nature::Record flip) have no entry in `subtype_edges`, so
        // `is_subtype(":wat::kernel::Failure", ":wat::core::Record")` returns false and
        // the accessor param-type check (accessor param = :wat::core::Record for monomorphic
        // Record types) rejects callers passing the concrete type.
        // Guard: skip the edge when the aggregate IS the root (e.g. :wat::core::Struct
        // registering :wat::core::Struct <: :wat::core::Struct is a reflexive cycle).
        // Builtins are registered acyclically (root comes first), so unwrap is safe.
        if let TypeDef::Aggregate(agg) = &def {
            let root = agg.nature.root_keyword();
            let child = name.clone();
            self.types.insert(name, def);
            if child != root {
                self.register_subtype(&child, root, crate::rust_caller_span!())
                    .expect("builtin aggregate subtype edge must not cycle");
            }
            return;
        }
        self.types.insert(name, def);
    }

    // ─── Stone S-A — typesub (is-a hierarchy) ──────────────────────────────

    /// Register a child→parent is-a edge in the `typesub` hierarchy.
    ///
    /// Cycle-rejection: if adding `child → parent` would close a cycle
    /// (i.e. `parent` is already a transitive subtype of `child` through the
    /// current registry), the registration is rejected with `TypeError::CyclicSubtype`.
    /// This mirrors `check_union_no_cycle` for the typeunion relation.
    ///
    /// Edges from unregistered names are allowed: the hierarchy is orthogonal to
    /// the `TypeDef` registry — a tag can derive regardless of whether it has a
    /// `TypeDef` entry. This mirrors Clojure's hierarchy being independent of what
    /// the tags ARE.
    pub fn register_subtype(&mut self, child: &str, parent: &str, span: Span) -> Result<(), TypeError> {
        // Cycle check: if parent is already transitively is-a child, adding this
        // edge closes a cycle.
        if is_subtype(parent, child, self) {
            return Err(TypeError::new(
                span,
                TypeErrorKind::CyclicSubtype {
                    child: child.to_string(),
                    parent: parent.to_string(),
                },
            ));
        }
        self.subtype_edges
            .entry(child.to_string())
            .or_default()
            .push(parent.to_string());
        Ok(())
    }

    /// Return the direct parent FQDNs of `name` in the `typesub` hierarchy.
    /// Returns `None` if `name` has no registered parent edges.
    /// Internal helper consumed by [`is_subtype`].
    fn subtype_parents(&self, name: &str) -> Option<&[String]> {
        self.subtype_edges.get(name).map(|v| v.as_slice())
    }
}

/// Heads to try for extend-type edges of a Handle-like parametric: bare `Handle`,
/// `(Handle :- [:T])`, `(Handle :- [:Xt])`. STONE-defservice-emits-the-binder — these three
/// strings are matched EXACT-string against `register_subtype`'s stored child key
/// (`extend-type`'s target arg, rendered through `check::format_type` — types.rs's
/// `:wat::core::extend-type` arm), so this guess MUST stay byte-identical to what
/// `format_type` now emits for `Parametric { head, args: [Path(":T"|":Xt")] }` — the LEADING
/// COLON is load-bearing: `parse_type_node`'s `WatAST::Symbol` arm prepends `:` to any
/// namespace-less symbol before storing it as a `TypeExpr::Path` (a bare `T` binder symbol
/// parses to `Path(":T")`, never `Path("T")`), so `format_type`'s Path arm — which returns
/// the stored string unchanged — renders it WITH the colon. Measured: guessing `"T"` (no
/// colon) here left `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` (and two
/// siblings) unable to find `(Handle :- [Wire])`'s registered `(Handle :- [:T]) <:
/// (TypedCapability :- […])` edge — `every_wat_scripts_file_loads` caught it.
pub(crate) fn transport_satisfier_heads(head: &str) -> Vec<String> {
    let fq = parametric_head_fqdn(head);
    vec![
        fq.clone(),
        render_binder_ref(&fq, &[":T".to_string()]),
        render_binder_ref(&fq, &[":Xt".to_string()]),
    ]
}

/// Extract a RENDERED type string's base — the head before any parametric suffix.
/// `check::format_type` has one surviving parametric spelling, `(Head :- [args])`
/// (STONE-defservice-emits-the-binder); a non-parenthesized `s` has no suffix to strip.
/// `family_extends`'s own base-extraction, below, is the ONE consumer that compares against a
/// `check::format_type`-rendered string rather than a literal declared name, so it is the one
/// taught the new form.
///
/// STONE reap-the-angle-machinery (arc 109) — this used to fall back to
/// `crate::runtime::split_type_params_pub` for the legacy `Head<args>` spelling.
/// `format_type` never renders that spelling any more (every `TypeExpr` arm emits either the
/// `(Head :- [args])` form caught by the branch below, or a plain `<`-free string), and every
/// `family_extends` caller passes a `sup`/`sub` that is itself always `<`-free (a
/// `TypeExpr::Path` or `parametric_head_fqdn` output) — so a non-parenthesized `s` here was
/// already bare; the strip was a no-op.
fn base_of_rendered_type(s: &str) -> &str {
    if let Some(rest) = s.strip_prefix('(') {
        if let Some(sp) = rest.find(' ') {
            return &rest[..sp];
        }
    }
    s
}

/// Does `sub`'s FAMILY extend `sup`'s family — existence only, arguments ignored?
///
/// The question the old deleted helper was asking with a `<`-suffixed string-prefix match:
/// "is ANY instantiation of this surface reachable from this type?" Asking it by string
/// prefix meant the code claimed a relation it never checked. This asks it directly: walk
/// the `extend-type` edges from each of `sub`'s guessed keys ([`transport_satisfier_heads`]),
/// and at each parent compare its BASE name (via [`base_of_rendered_type`], just above)
/// against `sup`'s base name — the same base-extraction door `TypeExpr::base_fqdn` uses
/// elsewhere, not a second hand-rolled extraction.
///
/// NOT a substitute for [`is_subtype`], which answers the EXACT question and whose exact-string
/// compare is load-bearing for `assignable`'s transport fast path.
pub(crate) fn family_extends(sub: &str, sup: &str, env: &TypeEnv) -> bool {
    let sup_base = base_of_rendered_type(sup);
    for key in transport_satisfier_heads(sub) {
        if is_subtype(&key, sup, env) {
            return true;
        }
        let mut visited = std::collections::HashSet::new();
        let mut stack: Vec<String> = env
            .subtype_parents(&key)
            .map(|p| p.to_vec())
            .unwrap_or_default();
        while let Some(p) = stack.pop() {
            if base_of_rendered_type(&p) == sup_base {
                return true;
            }
            if visited.insert(p.clone()) {
                if let Some(parents) = env.subtype_parents(&p) {
                    stack.extend(parents.iter().cloned());
                }
            }
        }
    }
    false
}

/// Seeds a fresh [`TypeEnv`] with wat-rs's own `:wat::*` declarations.
/// Called exactly once, from [`TypeEnv::with_builtins`]. New builtins
/// land here as the algebra grows; each entry documents why the
/// declaration is `:wat::*`-scoped.
fn register_builtin_types(env: &mut TypeEnv) {
    // Arc 293 decl-a — :wat::core::Struct: the nature-root for all struct types.
    //
    // Registered FIRST so every subsequent parsed struct finds it in the registry.
    // Zero-field opaque root: user structs register :Name <: :wat::core::Struct via
    // nature.root_keyword(). Value-top is an implicit rule in `is_subtype` — no
    // lattice edge registered (analogous to :wat::core::Record). The type system synthesizes
    // `:wat::core::is-Struct?` via `register_type_predicates`.
    env.register_builtin(TypeDef::Aggregate(AggregateDef {
        nature: Nature::Struct,
        name: ":wat::core::Struct".into(),
        type_params: vec![],
        fields: vec![],
        restrictions: None,
    }));

    // :wat::holon::CapacityExceeded — populated in the Err slot of
    // :wat::holon::Bundle's :Result return when a frame's
    // constituent count exceeds `floor(sqrt(dims))` (Kanerva's capacity
    // budget). The two fields are honest: cost is what the Bundle was
    // asked to hold; budget is what the substrate could hold. Both
    // i64 because wat integer literals are i64.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defstruct :wat::holon::CapacityExceeded …)`
    // in `wat/holon.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/holon.wat", ":wat::holon::CapacityExceeded");

    // :wat::holon::BundleResult — arc 032. Typealias for the
    // canonical Result shape Bundle (and every downstream caller
    // that threads through Bundle) returns. 44 characters wide
    // collapsed to one named type. Non-parametric: Bundle's Ok
    // arm is always HolonAST; CapacityExceeded is the algebra's
    // only capacity-failure shape.
    //
    //   typealias :wat::holon::BundleResult
    //     = (:Result :- [wat::holon::HolonAST wat::holon::CapacityExceeded])
    //
    // Callers can write either form; alias resolution unifies them
    // as the same type at the checker layer.
    env.register_builtin(TypeDef::Alias(AliasDef {
        name: ":wat::holon::BundleResult".into(),
        type_params: vec![],
        expr: TypeExpr::Parametric {
            head: "wat::core::Result".into(),
            args: vec![
                TypeExpr::Path(":wat::holon::HolonAST".into()),
                TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
            ],
        },
    }));

    // :wat::holon::Holons — arc 033. Typealias for the ubiquitous
    // "list of holons" shape that Bundle takes as input and that
    // every encode-*-facts vocab function returns. 35+ lab
    // occurrences plus 12 in wat-rs before the rename. Named via
    // /gaze — structurally honest, epistemically neutral, plural
    // of the element type. Content-agnostic: the type holds facts
    // (ground-truth measurements), claims (predictions), or
    // anything else a caller bundles; the alias makes no truth
    // assertion.
    //
    //   typealias :wat::holon::Holons = (:Vec :- [wat::holon::HolonAST])
    //
    // Callers can write either form; alias resolution unifies them.
    env.register_builtin(TypeDef::Alias(AliasDef {
        name: ":wat::holon::Holons".into(),
        type_params: vec![],
        expr: TypeExpr::Parametric {
            head: "wat::core::Vector".into(),
            args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
        },
    }));

    // :wat::core::EvalError — populated in the Err slot of a :Result
    // returned by the eval-family forms (:wat::eval-ast! /
    // eval-edn! / eval-digest! / eval-signed!) when dynamic evaluation
    // fails. Carries a `kind` discriminator (short machine-readable
    // variant name) and a `message` diagnostic (human-readable detail).
    //
    // `kind` values emitted by the dispatchers:
    //   "verification-failed"   — digest or signature check failed
    //   "parse-failed"          — EDN source couldn't be parsed
    //   "mutation-form-refused" — AST contained define/defmacro/struct/
    //                             enum/newtype/typealias/load! which
    //                             constrained eval refuses (FOUNDATION
    //                             line 663 invariant)
    //   "unknown-function"      — AST referenced a function not in the
    //                             frozen symbol table
    //   "type-mismatch"         — arg types at a call site didn't match
    //   "arity-mismatch"        — wrong number of args at a call site
    //   "channel-disconnected"  — send to a dropped receiver inside
    //                             eval'd code
    //   "runtime-error"         — any other RuntimeError surfaced by
    //                             the inner eval, with the variant's
    //                             Display as the message
    //
    // Two auto-generated accessors land alongside:
    //   :wat::core::EvalError/kind    — :fn(:EvalError) -> :String
    //   :wat::core::EvalError/message — :fn(:EvalError) -> :String
    // Plus the constructor :wat::core::EvalError/new for cases where
    // user code wants to synthesize one (rare — normally produced by
    // the runtime).
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defstruct :wat::core::EvalError …)`
    // in `wat/core.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/core.wat", ":wat::core::EvalError");

    // :wat::core::Bytes — substrate-general byte buffer. Alias for
    // (:Vec :- [u8]). Per arc 062 + /gaze: the universal name "Bytes" wins
    // across adjacent ecosystems (Rust's bytes::Bytes, Python's
    // bytes, Go's []byte, Haskell's ByteString). Lives in :wat::core::*
    // because byte buffers are substrate-general — they predate every
    // current and future consumer (vector serde via arc 061, future
    // crypto/IO/hashing/network arcs). The alias resolves structurally;
    // both `:wat::core::Bytes` and `(:Vec :- [u8])` work at call sites.
    //
    //   typealias :wat::core::Bytes = (:Vec :- [u8])
    env.register_builtin(TypeDef::Alias(AliasDef {
        name: ":wat::core::Bytes".into(),
        type_params: vec![],
        expr: TypeExpr::Parametric {
            head: "wat::core::Vector".into(),
            args: vec![TypeExpr::Path(":wat::core::u8".into())],
        },
    }));

    // :wat::core::nil — arc 153. Renamed from `:wat::core::unit`
    // (which arc 109 slice 1d minted). Same type-theoretic role as
    // Rust's `()`: singleton type, one inhabitant, "no meaningful
    // return value." The name `nil` ships the marker effect the
    // user wants without collapsing wat's existing
    // `(Option :- [T])::None` / `Some(t)` discipline (per arc 153
    // DESIGN — `nil` ≠ `None` ≠ `false` ≠ empty-list).
    //
    //   typealias :wat::core::nil = :()
    //
    // The bare empty-tuple type spelling `:()` continues to fire
    // `BareLegacyUnitType` per arc 109 slice 1d (steering toward
    // `:wat::core::nil`). The empty-tuple LITERAL VALUE `()` at
    // value position is a list literal and stays untouched; the
    // `:wat::core::nil` keyword is also accepted at value position
    // (additive recognition; both spellings evaluate to the nil
    // singleton).
    //
    // Note: the retired `:wat::core::unit` typealias was removed in
    // arc 153 slice 2 closure per substrate-as-teacher § "Retire
    // the hint when its window closes." All in-tree consumers
    // migrated during sweep 1b; out-of-tree callers spelling
    // `:wat::core::unit` now produce a TypeMismatch resolving the
    // unknown FQDN against `:()`.
    env.register_builtin(TypeDef::Alias(AliasDef {
        name: ":wat::core::nil".into(),
        type_params: vec![],
        expr: TypeExpr::Tuple(vec![]),
    }));

    // Arc 163 slice 3e — the typealiases for Option / Result /
    // HashMap / HashSet / Vector are RETIRED. They were originally
    // minted (arc 109 slices 1e + 1f) as transitional bridges
    // between source FQDN (`:wat::core::Option<T>`) and substrate-
    // internal bare-head storage (`Parametric { head: "Option", ... }`).
    //
    // Slice 3e closed that bridge by promoting substrate-internal
    // storage to FQDN: the head now reads `"wat::core::Option"`
    // directly. The aliases became identity (alias `:wat::core::Option`
    // mapped to `Parametric { head: "wat::core::Option", ... }`),
    // which created an `expand_alias` self-reference loop.
    //
    // The aliases are now redundant: source FQDN flows through
    // `parse_type_inner` unchanged to the FQDN head; bare forms
    // are rejected by the BareLegacyContainerHead walker. No
    // alias resolution is needed because no transformation is
    // needed. Constructors / dispatch / type-checking match the
    // FQDN head string directly.
    //
    // Constructor verbs (`:wat::core::Vector`, `:wat::core::HashMap`,
    // `:wat::core::HashSet`) are still recognized by the runtime
    // dispatcher (`collection/eval.rs eval_vector_ctor`, etc.) and the
    // type-checker (`check.rs infer_*_constructor`). Pattern 2
    // poison still surfaces friendly redirects for legacy spellings
    // (`:wat::core::vec`, `:Option<T>` etc.) at type-check time.

    // :wat::eval::StepResult — populated in the Ok slot of the :Result
    // returned by :wat::eval-step! (arc 068). Two variants distinguish
    // "one rewrite happened, here's the next form" from "this is the
    // terminal value." Both arms carry a payload — the next form as
    // wat::WatAST, the terminal value as wat::holon::HolonAST. The
    // consumer drives the loop, feeding StepNext.form back in until
    // StepTerminal arrives.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::eval::StepResult".into(),
        type_params: vec![],
        purity: Purity::Impure, // in-locus eval-step control; carries WatAST forms — Impure (never crosses)
        variants: vec![
            EnumVariant::Tagged {
                name: "StepNext".into(),
                fields: vec![("form".into(), TypeExpr::Path(":wat::WatAST".into()))],
            },
            EnumVariant::Tagged {
                name: "StepTerminal".into(),
                fields: vec![(
                    "value".into(),
                    TypeExpr::Path(":wat::holon::HolonAST".into()),
                )],
            },
            // Arc 070 — distinguishes "input was already a value; no
            // work happened" from "this step reduced a redex." Fires
            // on holon-value-shape WatASTs (`to-watast(holon)` round-
            // trips like Bundle's bare-list lift, holon-constructor
            // forms with all-canonical args, primitive literals).
            // Walkers and tracers care about chain-length 0 vs ≥ 1.
            EnumVariant::Tagged {
                name: "AlreadyTerminal".into(),
                fields: vec![(
                    "value".into(),
                    TypeExpr::Path(":wat::holon::HolonAST".into()),
                )],
            },
        ],
    }));

    // Arc 070 — (:wat::eval::WalkStep :- [A]) — what the visitor passed to
    // :wat::eval::walk returns. Two variants:
    //
    //   Continue(acc')        — keep walking; acc' is the new
    //                           accumulator. If the current
    //                           step-result was StepNext, walk
    //                           recurses on the next form. If it
    //                           was StepTerminal/AlreadyTerminal,
    //                           walk returns (terminal, acc').
    //   Skip(terminal, acc')  — caller has its own answer for this
    //                           coordinate (cache hit, etc.).
    //                           Walk stops here and returns
    //                           (terminal, acc').
    //
    // Generic over A so the consumer's accumulator can be any
    // type — cache, trace, counter, tier, etc.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::eval::WalkStep".into(),
        type_params: vec!["A".into()],
        purity: Purity::Impure, // in-locus walk control — Impure (never crosses)
        variants: vec![
            EnumVariant::Tagged {
                name: "Continue".into(),
                fields: vec![("acc".into(), TypeExpr::Path("A".into()))],
            },
            EnumVariant::Tagged {
                name: "Skip".into(),
                fields: vec![
                    (
                        "terminal".into(),
                        TypeExpr::Path(":wat::holon::HolonAST".into()),
                    ),
                    ("acc".into(), TypeExpr::Path("A".into())),
                ],
            },
        ],
    }));

    // Arc 170 — :wat::core::ReadOutcome — what `:wat::core::read-string` returns.
    //
    // read-string RAISED on malformed source, and wat has no try/catch by design, so a caller had
    // no way to survive bad input. Proven live and it is why this exists: an arrow key at the REPL
    // sends ESC (0x1B), the lexer rejects the control byte, and the raise unwound THROUGH the loop
    // and killed the session. A parse failure is a VALUE the reader faces
    // (`DESIGN-no-hidden-failures.md`).
    //
    //   :Forms     [forms] — the top-level forms, as data (the old return, unchanged)
    //   :Malformed [cause] — the text did not parse
    //
    // CONVERTED IN PLACE, no total-sibling verb. That is what this substrate did every previous
    // time — RecvOutcome / SendOutcome / CloseOutcome each REPLACED the raiser rather than standing
    // beside it (below/above in this file); TrySendOutcome is the lone near-twin and its own comment
    // says it exists because `try-send` is a genuinely different OP, not a totality variant. Two
    // ways to parse would be the synonym anti-pattern (`docs/ITERATION-PATTERNS.md` — "Synonyms are
    // LLM-hostile"). Callers for whom a parse failure is fatal (wat/fix.wat, deporder.wat, lint.wat
    // — tools parsing files they own) still die, but VISIBLY, in an arm they wrote.
    //
    // Pure, and honestly so: `:wat::WatAST` holds no fd and no peer (a form is a tree of keywords
    // and literals), and `:wat::core::Error` is Record-natured (`wat/core.wat`). Marking it Impure
    // would bar it from pure aggregates and the wire for no reason — SendOutcome's own argument.
    //
    // The cause is `:wat::core::Error`, the structural surface, NOT a parse-specific enum: lifting
    // the parser's ten variants (`Lex`, `UnclosedParen`, `UnclosedBrace`, …) into every caller's
    // exhaustive match would hand them arms nobody will branch on — the same dead-arm defect
    // `ReadFrameOutcome` refuses below. The discrimination lives in the navigable causes tree, where
    // `#wat.parse/Lex` keeps its own tag. `ParseError` already impls `WatError` (`src/parser.rs`),
    // so `error_edn()` composes the message/location/causes floor for free.
    // ★★ 2026-08-05 — `Option` and `Result` ARE ENUMS, and the type env did not know it.
    //
    // FOUND by the builder while `:wat::rete::core::enum::=`'s gate refused `:wat::core::Option<?0>`:
    // *"Option /is an enum/ right?... its mismanaged this whole time?... Result probably too?"* Yes,
    // and yes. TWELVE builtin sum types are registered here as `TypeDef::Enum` — several minted this
    // very arc (`RecvOutcome`, `SendOutcome`, `TrySendOutcome`, `CloseOutcome`) — while the two most
    // fundamental sum types in the language had NO `TypeDef` of any kind.
    //
    // WHY, traced rather than guessed: arc 109 slices 1e/1f minted them as TYPEALIASES (bridges from
    // source FQDN to the then-bare `Parametric { head: "Option" }` storage). Arc 163 slice 3e RETIRED
    // those aliases — correctly, they had become identity aliases causing an `expand_alias`
    // self-reference loop (see the note above). Nobody ever registered the ENUM that should have
    // replaced them. They fell through the gap between "alias retired" and "enum registered": a
    // traceable omission, not a design decision.
    //
    // NOT BROKEN — BYPASSED, which is worse in a specific way. `match` works, because the checker
    // carries a PARALLEL hardcoded mechanism (`MatchShape::Option`/`::Result`, plus bare `"Some"`
    // string matches). So nothing visibly failed. But anything asking the type env the GENERIC
    // question *"is this an enum?"* answered NO for the two enums every wat program touches.
    // Already on disk: `variant_typo_remedies` (`check.rs:1836`) matches `Some(TypeDef::Enum(e))`
    // and returns `vec![]` otherwise — so Option/Result typos got NO remediation, silently, in a
    // substrate whose stated doctrine (R29 `RVINA ERVDIT`) is that the checker educates the caller.
    //
    // PURITY — a decision, not a transcription. `EnumDef.purity` gates two real things: whether the
    // type may cross the wire / live in `:durable` (`is_pure_type`, `check.rs:12914`), and whether
    // variant fields must themselves be pure (`validate_aggregate_containment`, `:12977`).
    // `Pure` matches the `SendOutcome`/`CloseOutcome`/`Signal` siblings and is what the corpus
    // already assumes — `(Option :- [String])` in `:durable` is everywhere, and `Impure` would put a wall
    // through the middle of it. `RecvOutcome`'s `Impure` is about `O` being a live PEER OUTPUT, not
    // about parametricity: `(WalkStep :- [A])` is the parametric-and-registered precedent.
    //
    // Field names are INTERNAL, not API: the wire form is positional — measured,
    // `(:wat::core::Some 42)` prints `#wat.core.Option/Some [42]` — so no observable shape moves.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::core::Option".into(),
        type_params: vec!["T".into()],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Some".into(),
                fields: vec![("value".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Unit("None".into()),
        ],
    }));
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::core::Result".into(),
        type_params: vec!["T".into(), "E".into()],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Tagged {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Path("E".into()))],
            },
        ],
    }));

    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::core::ReadOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Forms".into(),
                fields: vec![("forms".into(), TypeExpr::Path(":wat::WatAST".into()))],
            },
            EnumVariant::Tagged {
                name: "Malformed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::core::Error".into()))],
            },
        ],
    }));

    // Arc 278 Stone 1 (`wat --mcp`) — (:wat::edn::ReadJsonOutcome :- [T]) — what
    // `:wat::edn::read-json` returns.
    //
    // `:wat::edn::read-json`'s input arrives from a REMOTE, UNTRUSTED harness over stdio (the
    // MCP JSON-RPC transport), so a malformed byte must not be able to raise: exactly the failure
    // `:wat::core::read-string` was converted to fix above — one bad byte unwinding a raise
    // THROUGH the loop and killing the session. Mirrors `:wat::core::ReadOutcome` variant-for-
    // variant, including WHY the cause is the structural `:wat::core::Error` and not a
    // JSON-specific enum: `wat_edn::JsonError`'s variants in every caller's exhaustive match would
    // be arms nobody branches on. Discrimination lives in the navigable causes tree.
    //
    // PARAMETRIC over `T` — CORRECTED from a first pass that declared `:Value`'s payload as the
    // bare `:wat::core::Value` (the universal top, arc 278 R7). That was wrong: UP is free, DOWN is
    // CHECKED, so a `:wat::core::Value` payload can be PRODUCED but never CONSUMED — no accessor
    // (`HashMap/get`, a Struct/Record field, …) type-checks against an opaque `Value` receiver, by
    // design (measured: `HashMap/get` on a decoded JSON object refused with "expected
    // HashMap<?,?>; got :wat::core::Value"). `T` is generic exactly as `(ReadlnOutcome :- [T])`'s `T` is
    // (immediately above) and for the same reason — the payload's type is the CALLER's, driven by
    // the annotated binding, not ours to fix in advance.
    //
    //   :Value     [value <- T] — the decoded value, at the caller's chosen type
    //   :Malformed [cause]      — the JSON text did not parse (or did not decode)
    //
    // Pure, and for `ReadOutcome`'s stated reason: the payload holds no fd and no peer (T here is
    // ordinary decoded data — a String/HashMap/record — never a live resource, unlike
    // `(ReadlnOutcome :- [T])`'s T which can be), and `:wat::core::Error` is Record-natured. Marking it
    // Impure would bar it from pure aggregates and the wire for nothing.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::edn::ReadJsonOutcome".into(),
        type_params: vec!["T".into()],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Value".into(),
                fields: vec![("value".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Tagged {
                name: "Malformed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::core::Error".into()))],
            },
        ],
    }));

    // `:wat::edn::ReadForeignOutcome<T>` — what `:wat::edn::read-foreign` returns.
    // Twin of `ReadJsonOutcome<T>`: the verb's input is an untrusted String (a journal
    // Log/message from another universe, a scratch payload), so a malformed byte must
    // not raise. Same two variants, same parametric T (the caller's binding pins it;
    // a ForeignRecord consumer unifies T with `:wat::edn::ForeignRecord`).
    //
    //   :Value     [value <- T] — the decoded value (ForeignRecord / ForeignVariant / typed)
    //   :Malformed [cause]      — the EDN text did not parse, or did not decode
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::edn::ReadForeignOutcome".into(),
        type_params: vec!["T".into()],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Value".into(),
                fields: vec![("value".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Tagged {
                name: "Malformed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::core::Error".into()))],
            },
        ],
    }));

    // Arc 170 — :wat::kernel::ReadFrameOutcome — what `:wat::kernel::read-frame` returns.
    //
    // A FRAME, not a line, and the name is load-bearing: `read_framed_edn`
    // (`src/edn/render.rs`) accumulates physical lines until the buffer forms a complete
    // EDN value. `:wat::io::IOReader/read-line` (`src/io.rs`) is the genuinely-one-line
    // verb; that name is already spent with that meaning, so reusing it here would both
    // lie and collide.
    //
    // MEASURED, because the obvious inference from "it accumulates" is WRONG for wat
    // source and I shipped that inference before running it: `next_complete_frame`
    // continues ONLY on `EdnFrameStatus::Incomplete`, and terminates the frame on
    // `Complete` *or* `Malformed` (`src/edn/render.rs`). Wat source is never valid EDN —
    // `:wat::core::defn` is precisely the "keyword begins with ::" that fails — so a
    // partial wat form scans as Malformed, not Incomplete, and the frame ENDS at the
    // first newline. Consequence: for wat input a frame is exactly one physical line, and
    // multi-line forms are NOT supported. Accumulation is real, but only for input that
    // is well-formed-EDN-so-far (an unclosed `{`).
    //
    // Three variants:
    //   :Frame [text] — the raw frame text, UNDECODED. `readln` reads the same bytes and
    //                   then EDN-decodes them, which is right for a wire and wrong for a
    //                   human: a REPL user types `(:wat::core::+ 1 1)`, which is wat
    //                   source, not an EDN literal, and decoding it fails on the `::`.
    //   :Eof []       — the clean stop, as a VALUE. The StdIn service has always returned
    //                   a matchable `::Eof` (`stdio.wat`, "NOT a panic that kills
    //                   the serve loop"); `stdio-read` then raised on it to preserve the
    //                   old fd-0 behavior for the 72 `readln` callers. That bank is what
    //                   made a REPL loop unable to stop cleanly. This verb spends it.
    //   :Stopped []   — a process-wide stop was requested while `stdio-read-frame`
    //                   (`stdio.wat`) was blocked waiting on the StdIn service.
    //                   NOT an `Eof` (the peer didn't close) and NOT an error — its own
    //                   outcome, matching `StdIn::ReadFrameResponse::Stopped` one layer
    //                   below. Named `Stopped`, not `Shutdown`: wat already has a word
    //                   for this fact — `(:wat::kernel::stopped?)` — and nothing is
    //                   shutting down here, a stop was merely requested. Ruled by an
    //                   intueri cast (2026-07-28, the "Stopped, not Shutdown" brief).
    //
    // It does NOT return the service's own `StdIn::ReadFrameResponse`. That enum carries
    // `RequestTooLarge`/`RequestMalformed` because `defservice` MANDATES those on every
    // serviceable op-Response — but this op's request is one i64 the kernel itself
    // builds, so neither can fire. Handing a caller an exhaustive match with two dead
    // arms is the same defect as any unreachable arm, and it would couple `:user::` code
    // to the stdin service's wire contract. A `:TooLong` arm is deliberately absent for
    // the same reason: it is only earned once `FramedRead::TooLarge` becomes a reply
    // instead of a raise inside `IOReader/read-frame`.
    //
    // Impure — it is I/O — and a sibling of the caller-facing `*Outcome` family
    // (RecvOutcome / SendOutcome / ConnectOutcome), so a reader already knows the shape.
    // Named by an intueri cast (2026-07-28), which also caught the frame-vs-line lie.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::ReadFrameOutcome".into(),
        type_params: vec![],
        purity: Purity::Impure, // an I/O outcome
        variants: vec![
            EnumVariant::Tagged {
                name: "Frame".into(),
                // `text`, not `line` — it may span several physical lines.
                fields: vec![("text".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
            EnumVariant::Unit("Eof".into()),
            // Arc 170 stdin-joins-the-lock-step — a process-wide stop was requested
            // while `stdio-read-frame` (`stdio.wat`) was blocked waiting on
            // the StdIn service. NOT an `Eof` (the peer didn't close) and NOT an
            // error — its own outcome, matching `StdIn::ReadFrameResponse::Stopped`
            // one layer below. Named by the arc-170 intueri cast, 2026-07-28: wat
            // already has `(:wat::kernel::stopped?)` for this fact, so `Shutdown`
            // (a second word for the same thing) was the synonym anti-pattern.
            EnumVariant::Unit("Stopped".into()),
        ],
    }));

    // Arc 170 closure #24 — (:wat::kernel::ReadlnOutcome :- [T]) — what `readln` returns.
    //
    // The THIRD outcome at this seam, and deliberately not either of the other two.
    // `IOReader::ReadFrameOutcome` and `kernel::ReadFrameOutcome` both carry RAW TEXT;
    // `readln` sits one level above them and hands back a DECODED value whose type flows
    // from the consumer (the arc-258/R54 `-> :T` annihilation: "readln reads what the
    // self-describing EDN wire says; the decoded value's type flows from the consumer").
    // So its payload cannot be `String` — it is the caller's `T`.
    //
    // WHY IT EXISTS. `readln` was the last IPC verb still RAISING. Every other one got its
    // outcome wall this arc (recv'/send'/close'/accept'/connect'), because a raise in a
    // language with no try/catch UNWINDS PAST THE READER — R53's `VERBO MEO CAPTVS`. The
    // wat `stdio-read` collapsed `Eof` and `Stopped` into `assertion-failed!` and said so
    // in its own comment: "the matchable ::Eof variant is BANKED, not yet exposed to the
    // 72 readln callers … there is no caller-facing value form for 'raise' to hand a stop
    // through". This is that value form; the bank is spent.
    //
    // `T` is generic exactly as `(RecvOutcome :- [O])`'s `O` is, and for the same reason —
    // the payload's type is the consumer's, not ours. That precedent is what makes this
    // mechanism already proven rather than newly invented.
    //
    // Impure for `(RecvOutcome :- [O])`'s reason exactly: `T` may itself be a live resource.
    //
    // ⚠ `Datum` is PROVISIONAL — arc 170 closure #26 casts intueri over this whole
    // surface (`StdIn::ReadFrameResponse` / `read-frame` / `:Frame`) and this variant name
    // rides with it. Named `Datum` and not `Value` to avoid colliding with
    // `:wat::core::Value`, the universal top; not `Line`, which is taken one layer down
    // for the raw text and would re-tell the frame-vs-line lie the 2026-07-28 cast caught.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::ReadlnOutcome".into(),
        type_params: vec!["T".into()],
        purity: Purity::Impure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Datum".into(),
                fields: vec![("v".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Unit("Eof".into()),
            EnumVariant::Unit("Stopped".into()),
        ],
    }));

    // Arc 170 stdin-joins-the-lock-step — :wat::io::IOReader::ReadFrameOutcome — what
    // `:wat::io::IOReader/read-frame` returns.
    //
    // The raw-IOReader-level sibling of `:wat::kernel::ReadFrameOutcome` above: this
    // one is what the verb hands back DIRECTLY (see `eval_ioreader_read_frame`,
    // `src/io.rs`); `:wat::kernel::ReadFrameOutcome` is the higher, caller-facing
    // outcome the StdIn *service* (`stdin-svc` in `stdio.wat`) builds from its
    // own `StdIn::ReadFrameResponse` reply. Two different enums at two different
    // layers, deliberately — the brief's "rooms 4 and 6" are not the same room.
    //
    // Owner-qualified (`IOReader::`), not bare `:wat::io::ReadFrameOutcome`: ruled by
    // the same arc-170 intueri cast, because that bare name was structurally identical
    // (same variants, same purity, same field name) to `:wat::kernel::ReadFrameOutcome`
    // above — the only hand-written duplicate base name in the wat type vocabulary.
    // `:wat::kernel::ReadFrameOutcome` keeps the short name (its verb is
    // `:wat::kernel::read-frame`, so verb and outcome agree, and it's the surface wat
    // programmers meet); this plumbing-layer one is owner-qualified instead, the same
    // shape as `:wat::kernel::StdIn::ReadFrameResponse`. A throwaway four-segment probe
    // (register_builtin + construct/match from a `.wat` fixture) proved the mechanism
    // resolves cleanly before this name shipped.
    //
    // Was `(Option :- [String])` before this arc: `Frame(Some(text))` / `Eof(None)`. A
    // process-wide stop request is neither — `(Option :- [String])` had no third state to
    // carry it, so this dedicated enum replaces it. See `eval_ioreader_read_frame`'s
    // doc comment (`src/io.rs`) for the poll that produces `Stopped`.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::io::IOReader::ReadFrameOutcome".into(),
        type_params: vec![],
        purity: Purity::Impure, // an I/O outcome
        variants: vec![
            EnumVariant::Tagged {
                name: "Frame".into(),
                fields: vec![("text".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
            EnumVariant::Unit("Eof".into()),
            // "A stop was requested; nothing is wrong with the stream." Named
            // `Stopped` (not `Shutdown`) by the same arc-170 intueri cast as the
            // sibling above — see that comment for the full rationale.
            EnumVariant::Unit("Stopped".into()),
        ],
    }));

    // Arc 170 — (:wat::eval::FormOutcome :- [T]) — what `:wat::eval-with-defs!` returns:
    // the outcome of handing ONE form to a world built from a definition set.
    //
    // Why FOUR variants and not a Result. A caller submitting a line (a REPL's `E`
    // phase is the motivating consumer) can receive four genuinely different answers,
    // and the substrate CANNOT let them infer which from an error:
    //
    //   :Declared    — the form was a declaration; the world took it. Nullary on
    //                  purpose — the caller already holds `defs` and `form`, so the
    //                  `conj` is theirs; returning a grown set would mint a second
    //                  authority for one fact. (Shape precedent: SendOutcome::Sent.)
    //   :Evaluated   — the form was an expression; here is its value.
    //   :CheckFailed — the form did not survive the freeze (parse / macro-expand /
    //                  type-check). A STATIC failure: nothing ran. The cause is the
    //                  freeze error's OWN `error_edn()` floor record — a navigable
    //                  tagged value (`#wat.check/CheckErrors {…}`,
    //                  `#wat.resolve/UnresolvedReferences {…}`, …), NEVER that tree
    //                  flattened into a String. `:wat::core::Error` is a structural
    //                  surface (message/location/causes, wat/core.wat:1816) and
    //                  `error_edn()`'s whole contract is to place exactly those three
    //                  floor keys ahead of the variant fields — so every freeze error
    //                  satisfies it as-is, with its causes chain intact and walkable.
    //   :Raised      — the form type-checked, ran, and unwound. A DYNAMIC failure.
    //                  (A form that RETURNS an Err value is `:Evaluated`, not `:Raised`.)
    //
    // The static/dynamic split is not stylistic — collapsing both into one `cause`
    // slot is the overloaded-bucket Ruling A forbids (DESIGN-service-io-budgets.md),
    // and the two carriers are genuinely different types: every `EvalError.kind` is a
    // dynamic-eval kind (see the EvalError doc above — "unknown-function",
    // "type-mismatch", "runtime-error"), none of which can describe a freeze
    // rejection. `StartupError` is what the freeze itself returns (freeze.rs) — it is
    // reused rather than duplicated. Its single `message` field is thin for a REPL
    // (which wants the location); growing it is that type's own follow-up, exactly
    // the "extensible … if a real consumer surfaces" its comment invites.
    //
    // `:Declared` cannot be inferred from a refusal, which is why it is a first-class
    // answer: MEASURED (wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat),
    // `defn` and `defrecord` fail eval with `unknown-function` — byte-identical to a
    // TYPO — because both are macros with no runtime verb to find.
    //
    // Impure, for (RecvOutcome :- [O])'s reason exactly: the caller's live `Environment` is
    // threaded through unchanged so impure bindings survive, so `T` may be a live
    // resource, and a Pure marking would lie the moment it is. `T` is phantom in three
    // of the four variants — the precedent is `(:wat::service::Outcome :- [S R O])`, whose
    // `O` is phantom for Reply/Stop/NoReply.
    //
    // Named by an intueri cast (2026-07-28), which also ruled the verb
    // `:wat::eval-with-defs!` and this `:wat::eval::` home — the namespace the eval
    // TYPES already use (StepResult, WalkStep above); `:wat::core::` would have been
    // drift, and a bare `Outcome` would read ambiguously beside
    // `:wat::service::Outcome` in the defservice handler that is its first consumer.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::eval::FormOutcome".into(),
        type_params: vec!["T".into()],
        purity: Purity::Impure, // T may be a live resource — see above
        variants: vec![
            EnumVariant::Unit("Declared".into()),
            EnumVariant::Tagged {
                name: "Evaluated".into(),
                fields: vec![("value".into(), TypeExpr::Path("T".into()))],
            },
            EnumVariant::Tagged {
                name: "CheckFailed".into(),
                fields: vec![(
                    "cause".into(),
                    TypeExpr::Path(":wat::core::Error".into()),
                )],
            },
            EnumVariant::Tagged {
                name: "Raised".into(),
                fields: vec![(
                    "cause".into(),
                    TypeExpr::Path(":wat::core::EvalError".into()),
                )],
            },
        ],
    }));

    // :wat::kernel::LociDiedError — the ONE loci-agnostic death report
    // (arc 278 the IPC de-prime, DESIGN-loci-died-error.md). Annihilates the
    // two near-twin `ThreadDiedError` / `ProcessDiedError` enums: a service /
    // bracket-worker never knows its own locus (thread · process · uds ·
    // localhost tcp · remote mTLS · whatever comes), so its death is measured
    // as ONE enum every peer exhaustively handles (explicit-exception-paths,
    // verbosity-is-the-shield). The variant set is the UNION of the two dead
    // enums, generalized loci-agnostic — the variant names *how* a peer died;
    // the locus rides as data:
    //
    //   Panic(message, failure)  — peer raised/panicked; catch_unwind captured
    //                              the payload as `message`, `failure` is
    //                              `:Some(...)` when the panic carried an
    //                              arc-016/064 AssertionPayload, `:None`
    //                              for a plain `panic!()`.
    //   RuntimeError(message)    — a type/arity/etc. error surfaced at run.
    //   Disconnected             — the wire dropped (was ChannelDisconnected).
    //   Stopped                  — a stop was requested mid-recv, any locus (arc 170 intueri
    //                              cast: wat's word for this fact, not Rust's "shutdown").
    //   StartupError(message)    — the locus didn't come up (fork/exec fail,
    //                              or a remote ECONNREFUSED).
    //   EntryFormFailure(message)— the peer program's entry form was malformed.
    //   MainSignature(message)   — the peer's :user::main had a bad signature.
    //   BadReturn(message)       — the peer returned a value that won't cross
    //                              the wire.
    //
    // Purity::Pure — a death report crosses back to the owner as EDN data; its
    // payload is String / (Option :- [Failure]) (no live resource), unlike
    // (RecvOutcome :- [O]) which is Impure only because O may be live.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::LociDiedError".into(),
        type_params: vec![],
        purity: Purity::Pure, // a death report — Pure (crosses back to the owner as EDN data)
        variants: vec![
            EnumVariant::Tagged {
                name: "Panic".into(),
                fields: vec![
                    ("message".into(), TypeExpr::Path(":wat::core::String".into())),
                    (
                        "failure".into(),
                        TypeExpr::Parametric {
                            head: "wat::core::Option".into(),
                            args: vec![TypeExpr::Path(":wat::kernel::Failure".into())],
                        },
                    ),
                ],
            },
            EnumVariant::Tagged {
                name: "RuntimeError".into(),
                fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
            // Reconciled: the two dead enums' ChannelDisconnected → Disconnected
            // (the wire dropped — loci-agnostic; "channel" was thread-tier vocab).
            EnumVariant::Unit("Disconnected".into()),
            // arc 170 Slice A — a stop was requested during recv, any locus. Renamed
            // Shutdown -> Stopped by the arc-170 intueri cast (RULING A): the wat-visible
            // layer says "stopped", never "shutdown" — nothing is shutting down when this
            // fires, a stop was merely requested. Rust's own vocabulary (`trigger_shutdown`,
            // `RecvError::Shutdown`, …) is UNCHANGED; only this wat-visible variant moves.
            EnumVariant::Unit("Stopped".into()),
            // arc 170 slice 1i — structured exit variants for all peer death
            // paths. extract-panics / the recv' Lost decoder use the TypeEnv to
            // reconstruct these from EDN on round-trip; they must be registered
            // here so edn_to_value can find them.
            // Arc 278 "errors first-class EDN" (stone 1) — `StartupError`'s cause is
            // the structured `:wat::core::Error` floor record (`error_edn()`), NOT a
            // `to_wire_edn` String (the double-encoded mask this stone kills). The
            // child emits `#wat.kernel.LociDiedError/StartupError [#wat.runtime/<V> {…}]`
            // (see `verbs.rs::startup_error_chain_edn`); the owner STRICT-decodes the
            // cause to a typed record. `LociDiedError/message` is a DERIVED accessor
            // reading `error.message` (see `eval_died_error_message`).
            EnumVariant::Tagged {
                name: "StartupError".into(),
                fields: vec![("error".into(), TypeExpr::Path(":wat::core::Error".into()))],
            },
            EnumVariant::Tagged {
                name: "EntryFormFailure".into(),
                fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
            EnumVariant::Tagged {
                name: "MainSignature".into(),
                fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
            EnumVariant::Tagged {
                name: "BadReturn".into(),
                fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
            },
        ],
    }));

    // :wat::kernel::Location — a point in a source file. Populated by
    // `:wat::kernel::run-sandboxed` when a panic carries a PanicInfo
    // location, and by future assertion primitives whose failure-payload
    // needs to cite file:line:col.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Location …)`
    // in `wat/core.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it. Change the field list in the `.wat` and this registration follows —
    // there is no second copy to drift, and `include_str!` makes rustc rebuild when it moves.
    //
    // This is the first row converted, and it is the PROOF for the other twelve: if the emitted
    // registration were not identical to the literal it replaced, the corpus's own re-declaration
    // would stop hitting arc 054's `Existing::Equivalent` arm and the stdlib would fail to load.
    ::wat_source_derive::wat_record_from!(env, "wat/core.wat", ":wat::kernel::Location");

    // :wat::kernel::Frame — one entry on the wat call stack, captured by
    // `(:wat::kernel::call-site)` (from the runtime `FrameInfo` trampoline
    // stack) or by `(:wat::kernel::macro-call-site)` (from the expand-time
    // macro-invocation stack). Every field is ALWAYS KNOWN — the older
    // all-`Option` shape (justified by a never-built Rust-backtrace→Frame
    // path where symbol resolution could fail per-frame) was a lie: every
    // LIVE construction has a real file/line span and a real symbol (a named
    // fn's path, the `<anonymous>` marker for an anon fn, or the macro name
    // for a macro-call-site). Arc 109 — concrete, non-`Option` fields.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Frame …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::Frame");

    // :wat::core::Span — the leaf source location an error's `:location` floor
    // key carries (arc 278 "errors first-class EDN"). `Span` write-side is the
    // `#[derive(ToEdn)]` in `wat-reader` (`#wat.core/Span {:file :line :col :end}`)
    // but that derive is WRITE-ONLY (no `EdnSchema` submit) — so `edn_to_value`
    // STRICT could not reconstruct a `:location` back to a typed record; it hit
    // `UnknownTag`. Hand-register the decode schema here (the `:wat::kernel::Location`
    // exemplar above), so a `:wat::core::Error` floor record round-trips fully:
    // `:message` (String), `:location` (this Span), `:causes` ((Vector :- [Error])).
    // `:end` is `(Option :- [:wat::core::Pos])` (Pos is registered via the EdnSchema
    // drain below); `None` for the `rust_caller_span!()` point-spans.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::core::Span …)`
    // in `wat/core.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/core.wat", ":wat::core::Span");

    // :wat::kernel::Failure — structured panic / assertion payload
    // populated when a sandboxed `:user::main` fails. Slice 2b fills
    // the carried error / frames from `catch_unwind`; slice 3's
    // `:wat::test::assert-*` primitives additionally populate actual /
    // expected when the panic payload carries an AssertionPayload.
    // Arc 293.W.2b — Failure is pure EDN data (all fields are pure scalars/records); flipped
    // Struct → Record. Location and Frame also flipped to Record (pure data, no live resources).
    // This is the 2616-cascade root: ThreadDiedError/ProcessDiedError (Pure enums) carry
    // `failure: (Option :- [Failure])` — containment passes once Failure is a Record.
    //
    // Arc 278 the string-wrap annihilation — Failure carries the raised
    // `:wat::core::Error` STRUCTURALLY in a MANDATORY `error` field (four-questions Fork B).
    // The old stored `message` / `location` fields are REMOVED: `Failure/message` and
    // `Failure/location` are now DERIVED accessors reading `error.message` / `error.location`
    // (storing them alongside `error` fails Simple+Honest — duplication that can drift). The
    // `error` field is pure: `:wat::core::Error` is a `:nature :wat::core::Record` surface
    // (core.wat), and `is_pure_type` reads a surface's declared nature — post-load containment
    // (`validate_aggregate_containment`, freeze/env.rs) sees Error registered and passes.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Failure …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::Failure");

    // :wat::kernel::AssertionFailure — arc 278 (DESIGN-loci-died-error.md): the
    // registered record that the panic-hook `#wat.kernel/AssertionFailure {…}`
    // envelope writer now routes through (via the derived `ToEdn`), replacing
    // the hand-built Map with the wrong field shapes. `:frames` is a
    // `(Vector :- [Frame])` (was the ad-hoc `{:callee,:at}` map); `:location` is an
    // `(Option :- [Location])` (was a bare `Span`); `:upstream-chain` is a
    // `(Vector :- [LociDiedError])` (was heterogeneous Thread|Process). Every field
    // type (Frame, Location, Failure, LociDiedError) is registered above/below
    // — the record is EDN all the way down.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::AssertionFailure …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::AssertionFailure");

    // :wat::kernel::StopAccepted — arc 170 "stopping is a protocol" Phase 2. The shutdown worker's
    // one notice, emitted exactly once on STDOUT (via the primed StdOut service, never a raw fd-1
    // write or eprintln — eprintln is wat's PANIC channel and a graceful stop is not a death) BEFORE
    // it asks any held service to stop. `services` names exactly the process-lifetime services being
    // asked (its held stdio Handles that were still live at the moment of the ask — an already-gone
    // Handle is silently omitted, never listed). Pure — crosses no live resource, pure EDN data,
    // rendering as `#wat.kernel/StopAccepted {:services [...]}`.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopAccepted …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::StopAccepted");

    // :wat::kernel::StopFailure — one service's failed stop, inside a `StopFailed`. `cause` carries
    // the STRUCTURED `:wat::core::Error` the failure already is (see `runtime.rs`'s
    // `fault_from_runtime_error`, which builds it as a `:wat::core::Fault` — the canonical minimal
    // record that structurally satisfies the `:wat::core::Error` surface, `wat/core.wat`) — never a
    // stringly message, never a bespoke `StopFailureCause` enum. Registered BEFORE `StopFailed` (which
    // holds `(Vector :- [StopFailure])`), matching the Frame/Location-before-AssertionFailure ordering above.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopFailure …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::StopFailure");

    // :wat::kernel::StopFailed — arc 170 "stopping is a protocol", the builder's silent-drop-annihilation
    // ruling. The shutdown worker no longer discards an ask's (or the `StopAccepted` announce's) error —
    // every failure on the stop path is collected into this record and, once `:user::main` returns,
    // reported LOUDLY: emitted as registered EDN on STDERR (the dying-declaration channel — a graceful
    // stop that failed is no longer graceful) immediately before a non-zero exit
    // (`src/distribution/mod.rs`, beside the existing `emit_structured_exit` call). An empty collection
    // means nothing changes — exit as it always did.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopFailed …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::StopFailed");

    // (:wat::kernel::RecvOutcome :- [O]) — the matchable outcome of a point-to-point
    // peer read (`recv'`). Arc 278 the recv'-outcome wall (DESIGN-recv-outcome-wall.md):
    // recv' RETURNED O and RAISED on close/crash — a raise unwinds past the reader
    // (mute). This makes a reason-free failure UNREPRESENTABLE — a peer read yields a
    // matchable enum with exactly three shapes, mirroring the reason-bearing
    // `:wat::spawn::ServiceEvent` that select'/poll' already return:
    //   :Message [msg <- O]        — a real message (the happy path).
    //   :Closed  []                — a GENUINE clean EOF; the ONLY reason-free terminal.
    //   :Lost    [cause <- Failure] — abnormal loss; UNCONSTRUCTIBLE without a structured
    //                                cause. The cause is the first-class `:wat::kernel::Failure`
    //                                carrier (never a flat String — builder-ruled: wat is EDN
    //                                everywhere), the SAME structured carrier ServiceEvent::Lost
    //                                / Reply::Failed use (built via `message_only_failure`).
    // Impure like ServiceEvent (an I/O outcome). Registered as a builtin (peer with Failure /
    // (WalkStep :- [A])) so the checker knows it from type-env init — recv' is used INSIDE the stdlib
    // (spawn.wat) before any wat defenum would load; a builtin is load-order-robust and, per the
    // design's own note, Impure is the honest fixed purity (a Pure marking would lie the moment O
    // is a live resource). O carries the peer's output element type ((WalkStep :- [A]) is the parametric
    // precedent).
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::RecvOutcome".into(),
        type_params: vec!["O".into()],
        purity: Purity::Impure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Message".into(),
                fields: vec![("msg".into(), TypeExpr::Path("O".into()))],
            },
            EnumVariant::Unit("Closed".into()),
            // Arc 278 #73 — a stop was requested while this read was parked. NOTHING
            // DIED and NOTHING CLOSED: the peer is ALIVE and the channel is OPEN.
            //
            // Before this variant the fact had no honest home. It was produced (the
            // substrate has always known), then reported as `Lost[LociDiedError::Stopped]`
            // — a carrier whose very type name says "died" — so a caller matched a death
            // and had to open the death report to learn nothing had died. `Closed` was
            // the other candidate and is worse: it asserts a clean EOF that did not
            // happen (the false "peer closed" a months-long sigterm flake was made of).
            //
            // UNIT, carrying no cause: four precedents (`types.rs` Stopped variants) and
            // there is nothing to report. The substrate was asked to stop. That is the
            // whole fact — a cause here would be inventing a reason for "you asked me to".
            EnumVariant::Unit("Stopped".into()),
            EnumVariant::Tagged {
                name: "Lost".into(),
                // Arc 278 the LociDiedError stone — the Lost cause is now the
                // loci-agnostic `:wat::kernel::LociDiedError` (was the flat
                // `Failure`). Every peer exhaustively handles every death
                // regardless of its locus. The death CHAIN is a container-level
                // Vector; Lost holds the single head (the immediate peer death).
                fields: vec![(
                    "cause".into(),
                    TypeExpr::Path(":wat::kernel::LociDiedError".into()),
                )],
            },
        ],
    }));

    // (:wat::stream::NextOutcome :- [T]) — Arc 118.11a (stone A of two, "mint next +
    // NextOutcome", DESIGN-STONE-118.11a). The matchable outcome of
    // `:wat::stream::next`, the single-force pull primitive that replaces the
    // three-force `empty?`/`first`/`rest` walk protocol (measured: 15 user-code
    // calls for 5 elements without the memo; the memo patches the count but pins
    // the whole realized chain in memory, +297 B/element). `next` forces exactly
    // one cell (`crate::stream::realize`, WHNF) and returns both halves in one
    // shot — nothing to dedupe, so no cache is needed:
    //   :Item      [value <- T, rest <- (Stream :- [T])] — the forced head + the
    //                                                 undrained tail, together.
    //   :Exhausted []                              — the named end.
    // Parametric in T exactly as `(RecvOutcome :- [O])` above (the copied exemplar) —
    // and for the identical reason: the nullary `Exhausted` variant is the
    // documented hazard in `check.rs` (the un-parametrized-nullary-variant
    // unify failure) unless the enum itself carries `type_params`. IMPURE like
    // `(RecvOutcome :- [O])`, not `SendOutcome`/`CloseOutcome`: T is a caller-supplied
    // element type that MAY be a live resource (a `Stream` of open peers, say),
    // so a blanket `Pure` marking would lie about what crosses. This stone is
    // purely additive — no existing verb moves, no call site migrates onto `next` yet.
    // (Stone 118.B3 has since DELETED the `forced: OnceLock` memo this comment used to say was
    // untouched; the migration it anticipated happened in 118.B2/B2b. Both are done.)
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::stream::NextOutcome".into(),
        type_params: vec!["T".into()],
        purity: Purity::Impure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Item".into(),
                fields: vec![
                    ("value".into(), TypeExpr::Path("T".into())),
                    (
                        "rest".into(),
                        TypeExpr::Parametric {
                            head: "wat::stream::Stream".into(),
                            args: vec![TypeExpr::Path("T".into())],
                        },
                    ),
                ],
            },
            EnumVariant::Unit("Exhausted".into()),
        ],
    }));

    // :wat::kernel::SendOutcome — Arc 278 the send'-outcome wall (Phase 1,
    // DESIGN-send-outcome-wall.md): the send-side twin of (RecvOutcome :- [O]) above.
    // send' RAISED reason-free MalformedForms on a gone peer ("peer already
    // closed" / "channel disconnected") — the last raise-that-masks. This makes
    // a send failure a matchable value instead, mirroring RecvOutcome exactly
    // except NON-parametric — send' carries no received payload, so no <O>:
    //   :Sent   []                — delivered (the happy path).
    //   :Closed []                — peer already cleanly closed (use-after-close;
    //                                was the "peer already closed" raise).
    //   :Lost   [cause <- LociDiedError] — disconnected mid-send; UNCONSTRUCTIBLE without
    //                                a structured cause. Arc 278 BRIEF-send-carries-its-cause
    //                                (#70): widened from the flat `Failure` to the SAME
    //                                loci-agnostic `LociDiedError` recv' already carries —
    //                                send' CAN distinguish a stop-woke-a-blocked-write
    //                                (`Stopped`) from a genuine peer loss (`Disconnected`);
    //                                it was simply discarding the distinction. Was the
    //                                "channel disconnected" raise.
    // PURE — unlike RecvOutcome. (RecvOutcome :- [O]) is Impure ONLY because of its payload
    // `O` (the received message may be a live resource — a socket/file handle). SendOutcome
    // is NON-parametric and holds only pure data: two nullary variants + `Lost[cause <-
    // LociDiedError]`, and LociDiedError is Purity::Pure (a death report — crosses back to
    // the owner as EDN data). A SendOutcome is fully EDN-reconstructable / wire-crossable;
    // marking it Impure would LIE (claim its values are locus-bound when they are not).
    // Registered as a builtin for the same load-order reason as RecvOutcome — send' is used
    // inside the stdlib before any wat defenum would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::SendOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Sent".into()),
            EnumVariant::Unit("Closed".into()),
            // Arc 278 #73 — the send-side twin of `RecvOutcome::Stopped` (see above for
            // the full argument). Landed in the SAME pass, deliberately: a half-fixed
            // pair is precisely how this arc got here — recv' was walled at R53 and the
            // send side went unwalled for months (R57 `IGNORANTIAM DELEMVS`).
            //
            // `send'` has always been able to tell a stop from a peer loss —
            // `SendError::Shutdown` is a distinct variant (`comms/mod.rs:919`, built to
            // mirror `RecvError::Shutdown`) — and folded it into `Lost` anyway.
            EnumVariant::Unit("Stopped".into()),
            EnumVariant::Tagged {
                name: "Lost".into(),
                fields: vec![(
                    "cause".into(),
                    TypeExpr::Path(":wat::kernel::LociDiedError".into()),
                )],
            },
        ],
    }));

    // :wat::kernel::TrySendOutcome — Arc 278 the send'-outcome wall Phase 3a
    // (BRIEF-send-wall-3a-try-send-outcome.md): `try-send'`'s OWN outcome type,
    // sibling to SendOutcome, NOT a reuse. `try-send'` is NON-BLOCKING, so it has
    // an outcome `send'` structurally cannot: WouldBlock (a live peer just not
    // draining — the channel-full / deadlock-guard case, `service.wat:1163`).
    // Four-questions ruled: adding WouldBlock to SendOutcome FAILS (`send'`
    // never returns it — Obvious/Simple/Honest all fail); mapping WouldBlock to
    // Lost FAILS Honest ("alive but not draining" is not "gone"). So try-send'
    // gets its own type:
    //   :Sent       []                — delivered (the happy path).
    //   :WouldBlock []                — channel full / receiver not draining
    //                                    (crossbeam TrySendError::Full /
    //                                    process-tier EWOULDBLOCK) — try-send' ONLY.
    //   :Closed     []                — peer already cleanly closed (cell None).
    //   :Lost       [cause <- LociDiedError] — receiver dropped mid-send (crossbeam
    //                                    TrySendError::Disconnected / a genuine
    //                                    process-tier write failure). Arc 278
    //                                    BRIEF-send-carries-its-cause (#70): widened
    //                                    symmetric with SendOutcome::Lost above.
    // PURE for the same reason SendOutcome is (see above) — non-parametric, only
    // pure data (three nullary variants + a pure `LociDiedError` record).
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::TrySendOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Sent".into()),
            EnumVariant::Unit("WouldBlock".into()),
            EnumVariant::Unit("Closed".into()),
            EnumVariant::Tagged {
                name: "Lost".into(),
                fields: vec![(
                    "cause".into(),
                    TypeExpr::Path(":wat::kernel::LociDiedError".into()),
                )],
            },
        ],
    }));

    // :wat::kernel::CloseOutcome — Arc 278 peer-lifecycle Strike 2 (the close'
    // OUTCOME WALL, BRIEF-close-outcome-wall.md). `close'` (:wat::kernel::-restricted
    // teardown intrinsic) used to RAISE on its *handleable* failures (thread-join-
    // panic, process-signaled, process-wait-fail, process-stopped); per the
    // peer-lifecycle LAW those become a matchable outcome, only the must-never-happen
    // raises (double-close, close'-on-a-timer, arity/type) stay raises. Shape B (RULED):
    //   :Closed   [exit <- (Option :- [i64])] — clean close. None = thread (no OS exit code);
    //                                     Some(code) = process exit status. Loci-agnostic
    //                                     (R32): the exit rides in an Option, not two variants.
    //   :Signaled [signal <- i64]       — process TERMINATED by a signal (was the
    //                                     "killed by signal N" raise). Signaled means
    //                                     *terminated*, never merely stopped.
    //   :Failed   [cause <- Failure]    — join-panic / wait-fail / stopped-not-terminated;
    //                                     the abnormal-close carrier (structured Failure).
    // PURE — like SendOutcome, unlike (RecvOutcome :- [O]). Non-parametric; the peer is
    // CONSUMED (close' takes the Option, leaving None), so no value here holds a live
    // resource. It carries only pure data: an (Option :- [i64]), an i64, and a Nature::Record
    // Failure — fully EDN-reconstructable / wire-crossable. Marking it Impure would LIE.
    // Registered as a builtin for the same load-order reason as SendOutcome — close' is a
    // kernel intrinsic used before any wat defenum would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::CloseOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Closed".into(),
                fields: vec![(
                    "exit".into(),
                    TypeExpr::Parametric {
                        head: "wat::core::Option".into(),
                        args: vec![TypeExpr::Path(":wat::core::i64".into())],
                    },
                )],
            },
            EnumVariant::Tagged {
                name: "Signaled".into(),
                fields: vec![("signal".into(), TypeExpr::Path(":wat::core::i64".into()))],
            },
            EnumVariant::Tagged {
                name: "Failed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
        ],
    }));

    // :wat::kernel::Signal — Arc 278 process-signal-owner-to-child stone
    // (DESIGN-STONE-process-signal-owner-to-child.md § "The shape";
    // BRIEF-process-signal-p2-mint.md). A CLOSED SET (R27: a closed set is an
    // enum; the name holds the value) — a bare i64 signal number would be the
    // string-key mistake with a different hat. Six variants, three tiers,
    // deliberately NOT uniform in what they cause:
    //
    //   tier   variant     POSIX     who observes, and how
    //   flag   User1       SIGUSR1   the CHILD, and it keeps running — (sigusr1?) reads true
    //   flag   User2       SIGUSR2   the CHILD, and it keeps running — (sigusr2?) reads true
    //   flag   Hangup      SIGHUP    the CHILD, and it keeps running — (sighup?) reads true
    //   stop   Interrupt   SIGINT    the CHILD, and it chooses when to stop — (stopped?) reads true
    //   stop   Terminate   SIGTERM   the CHILD, and it chooses when to stop — (stopped?) reads true
    //   kill   Kill        SIGKILL   the OWNER — the child observes nothing and stops mid-instruction
    //
    // THIS TABLE IS THE ENUM'S DOC COMMENT, not commentary alongside it — it is
    // the only honest home for two facts about the SET that no single variant
    // name can carry:
    //   1. `Interrupt` and `Terminate` share ONE landing (both reach
    //      `substrate_on_stop_signal`; the child cannot tell them apart). Named
    //      independently anyway (RULED 2026-08-03): the shared landing is a
    //      HANDLER decision, not an identity claim about the signals — a
    //      non-wat observer (`strace`, `ps`) still sees the difference even
    //      though wat's own handler does not, and collapsing them to one `Stop`
    //      variant would forfeit the ability to say which one went out.
    //   2. `Kill` has NO child-side observable at all — SIGKILL is uncatchable
    //      (a POSIX guarantee, not a substrate choice; `handle.rs`: "SIGKILL is
    //      unignorable"). The round trip still closes, on the OWNER side, via
    //      `CloseOutcome::Signaled[signal <- i64]`.
    //
    // WHY the send side (this enum) is closed while the receive side
    // (`CloseOutcome::Signaled`'s bare i64) is open: we choose what is
    // SENDABLE — a closed, finite set we author — but we do not control what
    // KILLS you — any process on the box can send any signal. One concept,
    // two honest shapes for two different directions of control, not an
    // inconsistency to unify.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::Signal".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("User1".into()),
            EnumVariant::Unit("User2".into()),
            EnumVariant::Unit("Hangup".into()),
            EnumVariant::Unit("Interrupt".into()),
            EnumVariant::Unit("Terminate".into()),
            EnumVariant::Unit("Kill".into()),
        ],
    }));

    // :wat::kernel::SignalOutcome — the matchable outcome of
    // `(:wat::kernel::signal proc sig)` (BRIEF-process-signal-p2-mint.md).
    // Non-parametric — the peer is BORROWED, not consumed (unlike close', a
    // process may be signalled any number of times before it is closed), and
    // no variant holds a live resource. Same MUST_USE_TYPES slot as
    // CloseOutcome/SendOutcome (see check.rs `MUST_USE_TYPES`): a dropped
    // outcome is a compile error, closing both discard doors.
    //
    // `Delivered`, not `Sent` — `Sent` names the OWNER's action and is silent
    // on arrival, which is the entire reason this type exists.
    //
    // ⚠ STOP-2 — `Gone` (ESRCH) was NOT minted. Measured by a dedicated probe
    // (own run, 2026-08-03, `Pidfd::send_signal` against a child that had
    // exited but was deliberately left un-reaped): sending a signal to that
    // pidfd returns `Ok(())`, not ESRCH — delivery to a zombie is a silent
    // no-op, not an error. ESRCH appeared ONLY after the pidfd had already
    // been reaped — and in this substrate nothing reaps a `Process` peer's
    // pidfd except `close` (`eval_peer_close_prime`, `src/kernel/resource.rs`), which CONSUMES the peer
    // (`Option::take`). So the only way to reach an already-reaped pidfd
    // through this verb is to call it on an already-closed peer, and that path
    // is intercepted before the syscall (the same "peer already closed" guard
    // close' itself uses) — a live `signal` call can never observe ESRCH. Two
    // arms and a raise, per the stone's own named fallback for this outcome.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::SignalOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Delivered".into()),
            EnumVariant::Tagged {
                name: "Failed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
        ],
    }));

    // :wat::edn::Validation — Arc 278 the REQUEST-MALFORMED wall (Stone 1,
    // DESIGN-request-malformed-input-sanitization.md). The outcome of
    // `(:wat::edn::validate <value> :DeclaredType)` — the DEEP shape check at a
    // trust boundary. `:wat::core::conforms?` cannot answer this question: for an
    // Aggregate it is a NOMINAL identity check only (runtime.rs `conforms_check`,
    // the `TypeDef::Aggregate` arm → `concrete_type_name_matches`) — it never
    // recurses into a record's FIELDS, so `#dos.Bag/PutRequest {:items [1 2 3]}`
    // against `items <- (Vector :- [String])` passes it. `edn_to_typed_value`
    // (edn/render.rs) IS the deep walker (per-field, per-element, with the offending
    // path); `validate` is its thin wat-facing wrapper.
    //   :Valid   []                       — the value matches the declared shape.
    //   :Invalid [path expected got]      — it does not, at `path` (segments, e.g.
    //                                       ["items" "[0]"]), where the declaration
    //                                       says `expected` and the wire carried `got`.
    // `path` is STRUCTURED ((Vector :- [String]) — segments the caller can index/walk);
    // `expected`/`got` are STRINGS, ruled by the four questions (see
    // DESIGN-request-malformed-input-sanitization.md): `expected` is
    // `check::format_type`'s rendering (the ONE authoritative type renderer), and
    // `got` is the EDN SHAPE that arrived ("Integer", "Vector", "Map") — an
    // untyped wire value has NO declared type, so structuring `got` as a type form
    // would FABRICATE information. An asymmetric pair (structured expected, string
    // got) would imply a comparison the substrate cannot make. This is a 400-class
    // diagnostic, not data anyone computes on.
    // PURE — three Strings/String-vectors and a nullary variant; fully
    // EDN-reconstructable. Registered as a builtin because the defservice-generated
    // serve loop matches on it, before any wat defenum would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::edn::Validation".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Valid".into()),
            EnumVariant::Tagged {
                name: "Invalid".into(),
                fields: vec![
                    (
                        "path".into(),
                        TypeExpr::Parametric {
                            head: "wat::core::Vector".into(),
                            args: vec![TypeExpr::Path(":wat::core::String".into())],
                        },
                    ),
                    ("expected".into(), TypeExpr::Path(":wat::core::String".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::String".into())),
                ],
            },
        ],
    }));

    // (:wat::kernel::AcceptOutcome :- [R S]) — Arc 278 peer-lifecycle Strike 3 (the accept'
    // OUTCOME WALL, BRIEF-accept-outcome-wall.md). `accept'` used to RETURN a bare
    // `(Peer' :- [R S])` and RAISE on its *handleable* failures (rendezvous dropped/shutdown,
    // decode error, `select` error, `peer_cred` read fail). Per the peer-lifecycle LAW
    // (2026-07-23) — "we deliver an enum for code to handle exceptions with; raise is
    // uncatchable on purpose, a thing that must never happen" — those become a matchable
    // outcome; only the must-never-happen raises (arity, listener-type-mismatch, and the
    // in-process malformed-connect-request substrate bug) stay raises. Shape (RULED):
    //   :Accepted [peer <- (Peer' :- [R S])]  — an AUTHORIZED peer connected (the happy path).
    //   :Closed   []                    — the listener's rendezvous shut down / address
    //                                     dropped (clean; no peer). The reason-free terminal.
    //   :Failed   [cause <- Failure]    — a decode / select / peer_cred / socket-wrap io
    //                                     error; the structured-cause carrier (never a flat
    //                                     String — built via `message_only_failure`).
    // `Rejected` is CUT: the security gate BOUNCES a stranger INTERNALLY (process tier:
    // drop + re-poll; thread tier: no gate — the crossbeam handle IS the grant), so no
    // tier returns a security-reject to the caller — a `Rejected` variant would never be
    // constructed (fails Honest).
    // Impure + PARAMETRIC, mirroring (RecvOutcome :- [O]): `Accepted` holds a live `Peer'` (a
    // socket/channel handle), so a Pure marking would lie the moment the peer is a live
    // resource. R,S carry the peer's wire element types (the parametric precedent).
    // Registered as a builtin for the same load-order reason as RecvOutcome — accept' is a
    // kernel verb usable inside the stdlib before any wat defenum would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::AcceptOutcome".into(),
        type_params: vec!["R".into(), "S".into()],
        purity: Purity::Impure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Accepted".into(),
                fields: vec![(
                    "peer".into(),
                    TypeExpr::Parametric {
                        head: "wat::kernel::Peer".into(),
                        args: vec![
                            TypeExpr::Path("R".into()),
                            TypeExpr::Path("S".into()),
                        ],
                    },
                )],
            },
            EnumVariant::Unit("Closed".into()),
            EnumVariant::Tagged {
                name: "Failed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
        ],
    }));

    // (:wat::kernel::ConnectOutcome :- [S R]) — Arc 278 peer-lifecycle Strike 4 (the connect'
    // OUTCOME WALL, BRIEF-connect-outcome-wall.md — the LAST peer-lifecycle wall). The
    // exact TWIN of `(AcceptOutcome :- [R S])` above. `connect'` used to RETURN a bare
    // `(Peer' :- [S R])` and RAISE on its *handleable* failures (ECONNREFUSED / no listener /
    // rendezvous gone, the `OnlyThisPeer` identity reject, `peer_cred` read fail,
    // socket-wrap io error). Per the peer-lifecycle LAW (2026-07-23) — "we deliver an enum
    // for code to handle exceptions with; raise is uncatchable on purpose, a thing that
    // must never happen" — those become a matchable outcome; only the must-never-happen
    // raises (arity, address-type-mismatch, and the in-process malformed-address substrate
    // bug — see below) stay raises. Shape (RULED):
    //   :Connected [peer <- (Peer' :- [S R])]  — dialed + admitted (the happy path).
    //   :Refused   [cause <- Failure]    — ECONNREFUSED / no listener / rendezvous gone;
    //                                      RETRYABLE transport (the server may come up).
    //   :Rejected  [cause <- Failure]    — the `OnlyThisPeer` identity check failed (the
    //                                      answerer's pid/euid != the address minter's);
    //                                      NOT retryable (wrong process, not a transport
    //                                      blip). FIRES here (unlike accept', where the
    //                                      gate bounces internally) — the client dials once
    //                                      and a server-identity mismatch is caller-visible.
    //   :Failed    [cause <- Failure]    — a `peer_cred` read / socket-wrap io error; the
    //                                      structured-cause carrier (never a flat String —
    //                                      built via `message_only_failure`).
    // Note the arg order `<S,R>` — connect's return is `Peer'<S,R>` (send-type first), the
    // MIRROR of accept's `Peer'<R,S>`. The must-never-happen raises stay raises: arity,
    // address-type-mismatch, and the in-process malformed-abstract-name (`from_abstract_name`
    // on `SocketAddress::name`) — the name is either kernel-minted (autobind, 5 random
    // bytes) or a wire-received `SocketAddressWire` already fully validated at decode
    // (non-empty, <=107-byte abstract-UDS limit, bytes 0..=255 — `capability::registry`),
    // so a malformed name at connect time is an in-process substrate bug, not adversarial
    // wire data (STOP-3, grounded).
    // Impure + PARAMETRIC, mirroring (AcceptOutcome :- [R S])/(RecvOutcome :- [O]): `Connected` holds a
    // live `Peer'` (a socket/channel handle), so a Pure marking would lie the moment the
    // peer is a live resource. S,R carry the peer's wire element types. Registered as a
    // builtin for the same load-order reason as AcceptOutcome — connect' is a kernel verb
    // usable inside the stdlib before any wat defenum would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::ConnectOutcome".into(),
        type_params: vec!["S".into(), "R".into()],
        purity: Purity::Impure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Connected".into(),
                fields: vec![(
                    "peer".into(),
                    TypeExpr::Parametric {
                        head: "wat::kernel::Peer".into(),
                        args: vec![
                            TypeExpr::Path("S".into()),
                            TypeExpr::Path("R".into()),
                        ],
                    },
                )],
            },
            EnumVariant::Tagged {
                name: "Refused".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
            EnumVariant::Tagged {
                name: "Rejected".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
            EnumVariant::Tagged {
                name: "Failed".into(),
                fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()))],
            },
        ],
    }));

    // :wat::holon::VectorDecodeOutcome — Arc 278 the dimension-heresy strike
    // (BRIEF-dimension-heresy-screams.md). `:wat::holon::bytes-vector` used to
    // return a bare `(:Option :- [wat::holon::Vector])`, collapsing FOUR structurally
    // distinct wire-decode failures (short header, wrong data length, foreign
    // encoding dimension, reserved 0b11 cell pattern) into one reason-free
    // `:None`. Per the builder's ruling on this strike — "the entire check is
    // 'are these two dims the same vec length?' — that's it. This is trivially
    // measured and is not deserving of a crash but an expressive enum to be
    // handled" — each failure becomes its own named variant, not a lumped
    // `Malformed[reason, at]`: the failure space is a CLOSED set already
    // explicitly branched in the decoder's own source (unlike
    // `RequestMalformed`'s open-ended String, which is honest precisely
    // because ITS space is open-ended). The tell: `at` is meaningful only for
    // `InvalidCell` — a shared field honest for one member and vacuous for the
    // rest is the evidence a lumped shape would be wrong here.
    //   :Decoded           [vector <- Vector]     — the happy path.
    //   :DimensionMismatch [expected <- i64  got <- i64] — the wire header's
    //                        dim disagrees with this program's constant
    //                        `dim-count` (`config::collect_entry_file`).
    //                        Neither vector is "foreign" in the combine sense
    //                        below — this one DOES cross a wire, so the
    //                        disagreement is against the ambient program's d.
    //   :TruncatedHeader   [got <- i64]            — fewer than 4 header bytes;
    //                        no `expected` field — the 4-byte minimum is a
    //                        protocol constant, not a per-call datum, and the
    //                        actual (short) length is the one thing a log wants.
    //   :LengthMismatch    [expected <- i64  got <- i64] — header dim parsed
    //                        fine, but the data bytes don't match `ceil(dim/4)`.
    //   :InvalidCell       [at <- i64]             — a 2-bit cell decoded to
    //                        the reserved `0b11` pattern at cell index `at`.
    // PURE — `:wat::holon::Vector` is fully EDN-reconstructable ternary cell
    // data (the very reason `vector-bytes`/`bytes-vector` exist to serialize
    // it), and every other field is a bare `i64`. Registered as a builtin
    // (peer with the other outcome walls) for load-order robustness, though
    // `bytes-vector` itself has zero wat-corpus callers today.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::holon::VectorDecodeOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Decoded".into(),
                fields: vec![("vector".into(), TypeExpr::Path(":wat::holon::Vector".into()))],
            },
            EnumVariant::Tagged {
                name: "DimensionMismatch".into(),
                fields: vec![
                    ("expected".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::i64".into())),
                ],
            },
            EnumVariant::Tagged {
                name: "TruncatedHeader".into(),
                fields: vec![("got".into(), TypeExpr::Path(":wat::core::i64".into()))],
            },
            EnumVariant::Tagged {
                name: "LengthMismatch".into(),
                fields: vec![
                    ("expected".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::i64".into())),
                ],
            },
            EnumVariant::Tagged {
                name: "InvalidCell".into(),
                fields: vec![("at".into(), TypeExpr::Path(":wat::core::i64".into()))],
            },
        ],
    }));

    // :wat::holon::CombineOutcome — Arc 278 the dimension-heresy strike, part
    // 2. `vector-bind` / `vector-bundle` / `vector-blend` each RAISED a
    // `TypeMismatch` on differing Vector dimensions; per the same ruling as
    // `VectorDecodeOutcome` above, a differing `d` is cheap to detect and
    // meaningful to recover from, so it becomes a matchable value instead.
    // ONE shared enum for all three verbs, not three per-verb siblings —
    // unlike `RecvOutcome`/`SendOutcome`/`TrySendOutcome` (whose split is
    // earned because their outcome SHAPES genuinely differ), bind/bundle/blend
    // have an IDENTICAL outcome space: both reduce to `[expected, got]`.
    //   :Combined          [vector <- Vector]                — the happy path
    //                        (bind's XOR-compose / bundle's superposition /
    //                        blend's weighted linear combination — three
    //                        verbs, one shape of success).
    //   :DimensionMismatch [expected <- i64  got <- i64]      — the operands
    //                        disagree. Deliberately the SAME variant name as
    //                        `VectorDecodeOutcome::DimensionMismatch` — one
    //                        fact reached by two routes. NOT `ForeignDimension`:
    //                        here neither vector is foreign (both are ordinary
    //                        in-program values that simply disagree), unlike
    //                        the wire-decode case above where one honestly did
    //                        cross a boundary.
    // PURE, for the same reason `VectorDecodeOutcome` is: a bare `Vector` +
    // two `i64`s, all EDN-reconstructable.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::holon::CombineOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Combined".into(),
                fields: vec![("vector".into(), TypeExpr::Path(":wat::holon::Vector".into()))],
            },
            EnumVariant::Tagged {
                name: "DimensionMismatch".into(),
                fields: vec![
                    ("expected".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::i64".into())),
                ],
            },
        ],
    }));

    // :wat::holon::DegenerateSide — Arc 278 the cosine outcome wall
    // (BRIEF-cosine-outcome-wall.md, DESIGN-STONE-where-admits-only-rete-ops.md
    // "THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT" + its AMENDED
    // 2026-08-03 block). Diagnostic payload for `CosineOutcome::Degenerate`
    // below — WHICH operand had a zero-magnitude vector (the case cosine
    // cannot honestly answer, since a direction is undefined for a
    // zero-magnitude vector). Three-valued rather than two bools deliberately
    // (orchestrator's amendment to the ward's original cast): a pair of bools
    // makes `(false, false)` — a `Degenerate` that is not degenerate —
    // representable, in a substrate whose standing doctrine is the wrong
    // state has no form. `Target`/`Reference` are the implementation's own
    // operand names (mirroring `pair_values_to_vectors`'s `target`/`reference`
    // callers use), not invented ones.
    // PURE — three nullary variants, no fields at all.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::holon::DegenerateSide".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Target".into()),
            EnumVariant::Unit("Reference".into()),
            EnumVariant::Unit("Both".into()),
        ],
    }));

    // :wat::holon::CosineOutcome — Arc 278 the cosine outcome wall. `cosine`
    // had two domain holes, both dishonest: a dimension mismatch raised
    // `TypeMismatch` (uncatchable, unwinds past the reader), and a
    // zero-magnitude operand returned a guarded `0.0` — which in cosine's
    // own codomain MEANS "orthogonal, unrelated", a fabricated answer that
    // sails through `(f64::> ... 0.9)` as a confident no-match (probe
    // `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`: genuine
    // unrelatedness reads `-0.0086`, the sentinel reads exactly `0.0` — the
    // two are indistinguishable to a caller without this wall). Per the
    // design stone's ruled law (a MEASUREMENT may not absorb its own
    // undefined case), both holes become named variants a caller faces:
    //   :Similarity        [similarity <- f64]        — the happy path, the
    //                        raw cosine, clamped to [-1, 1].
    //   :Degenerate        [side <- DegenerateSide]    — one operand (or
    //                        both) is a zero-magnitude vector, so a
    //                        direction — and therefore a cosine — is
    //                        undefined. ONE variant carrying which side,
    //                        not three variants proliferated: the caller
    //                        acts identically regardless of which side was
    //                        degenerate, and the side is a diagnostic, not a
    //                        behavioral fork — exactly the role
    //                        `DimensionMismatch`'s fields already play below.
    //   :DimensionMismatch [expected <- i64  got <- i64] — the two operands
    //                        disagree in dimension; was the `pair_values_to_vectors`
    //                        `TypeMismatch` raise, now a domain fact.
    // PURE — non-parametric, holding only pure data: an f64, a `DegenerateSide`
    // (itself pure), and two i64s. Fully EDN-reconstructable / wire-crossable;
    // marking it Impure would lie. Registered as a builtin, peer with the
    // other outcome walls in this family (`CombineOutcome`, `VectorDecodeOutcome`).
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::holon::CosineOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Similarity".into(),
                fields: vec![("similarity".into(), TypeExpr::Path(":wat::core::f64".into()))],
            },
            EnumVariant::Tagged {
                name: "Degenerate".into(),
                fields: vec![(
                    "side".into(),
                    TypeExpr::Path(":wat::holon::DegenerateSide".into()),
                )],
            },
            EnumVariant::Tagged {
                name: "DimensionMismatch".into(),
                fields: vec![
                    ("expected".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::i64".into())),
                ],
            },
        ],
    }));

    // :wat::holon::DotOutcome — Arc 278 the cosine outcome wall's sibling for
    // `dot`. TWO enums, not one shared with `CosineOutcome` — `dot` performs
    // no division (`Similarity::dot` sums `i8 × i8` products, bounded by
    // `d × 127²`, so reaching ±Inf needs `d ≈ 10³⁰⁴` — closed, not merely
    // unlikely), so a zero-magnitude operand yields an HONEST `0.0`: a zero
    // vector really does dot to zero. A shared enum would hand `dot` a
    // `Degenerate` arm it can never construct — the `TrySendOutcome`-from-
    // `SendOutcome` precedent: split earned by a genuine, structural
    // difference in outcome space, not a naming convenience.
    //   :Computed          [product <- f64]           — the happy path.
    //   :DimensionMismatch [expected <- i64  got <- i64] — same fact,
    //                        same shape as `CosineOutcome::DimensionMismatch`
    //                        (one fact reached by two routes through the
    //                        shared `pair_values_to_vectors` guard).
    // PURE, for the same reason `CosineOutcome` is.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::holon::DotOutcome".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Tagged {
                name: "Computed".into(),
                fields: vec![("product".into(), TypeExpr::Path(":wat::core::f64".into()))],
            },
            EnumVariant::Tagged {
                name: "DimensionMismatch".into(),
                fields: vec![
                    ("expected".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ("got".into(), TypeExpr::Path(":wat::core::i64".into())),
                ],
            },
        ],
    }));

    // :wat::kernel::RunResult — the matchable outcome of running a program:
    // `:wat::kernel::run-sandboxed`, `:wat::test::run-thread` /
    // `run-hermetic'`, and (via the `:wat::test::TestResult` alias) every
    // `deftest`. Arc 278 the vacuous-gate wall (BRIEF-vacuous-deftest-gate-wall.md),
    // the third of the outcome walls after RecvOutcome (R53) and SendOutcome (R57).
    //
    // It WAS a Nature::Struct with one field, `failure <- (Option :- [Failure])`
    // (arc 278 wave 2d dropped the stdout/stderr capture buffers, leaving that
    // single slot). That shape is what let a caller look away: a pass and a
    // failure wore the SAME type, distinguished only by an `Option` slot nobody
    // was forced to read. The Rust gate idiom `call_beside_value(..).is_ok()` therefore
    // certified a fired assertion as a pass — proven by mutating a live gate's
    // `(assert-eq n 1)` to `n 4242` and watching the test still PASS.
    //
    // As an enum, a reason-free failure is UNREPRESENTABLE — exactly two shapes,
    // and `match` forces the reader to face both:
    //   :Passed []                    — the run completed with no failure.
    //   :Failed [failure <- Failure]  — UNCONSTRUCTIBLE without a structured cause;
    //                                    the first-class `:wat::kernel::Failure`
    //                                    carrier (never a flat String — wat is EDN
    //                                    everywhere), the same carrier RecvOutcome::Lost
    //                                    / SendOutcome::Lost / Reply::Failed use.
    //
    // PURE, for the same reason SendOutcome is: non-parametric, holding only pure
    // data (a nullary variant + a `Failure`, which is Nature::Record / pure EDN,
    // arc 293.W.2b). A RunResult is fully EDN-reconstructable and wire-crossable;
    // marking it Impure would lie. Registered as a builtin (like its two sibling
    // outcome walls) because `run-thread'` constructs it inside the stdlib, before
    // any wat `defenum` would load.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::RunResult".into(),
        type_params: vec![],
        purity: Purity::Pure,
        variants: vec![
            EnumVariant::Unit("Passed".into()),
            EnumVariant::Tagged {
                name: "Failed".into(),
                fields: vec![(
                    "failure".into(),
                    TypeExpr::Path(":wat::kernel::Failure".into()),
                )],
            },
        ],
    }));

    // :wat::kernel::ForkedChild RETIRED 2026-04-30 (arc 112).
    // The struct collapsed into (:wat::kernel::Process :- [I O]) — both
    // spawn-process and spawn-program' now return the unified Process
    // shape. The wait mechanism lives inside ProgramHandle's
    // InThread / Forked enum variant; the ChildHandle is no longer
    // wat-visible. Pre-arc-112 fixtures used:
    //   (child :wat::kernel::ForkedChild<I,O>) (spawn-process forms)
    //   (handle :wat::kernel::ChildHandle)     (ForkedChild/handle child)
    //   (exit  :i64)                           (wait-child handle)
    // Migration:
    //   (proc  (:wat::kernel::Process :- [I O]))     (spawn-process forms)
    //   (rcv   (:Result :- [:() :ProcessDiedError])) (Process/join-result proc)

    // :wat::kernel::StartupError — error variant of the Result
    // returned by `:wat::kernel::spawn-program` / `-ast` (arc 105a).
    // Captured when freeze (parse + type-check + config + macro)
    // or `:user::main` signature validation fails. Single field
    // for now (the diagnostic message); extensible to kind /
    // location if a real consumer surfaces.
    //
    // Auto-generated `StartupError/new` + `StartupError/message`
    // accessor land in the symbol table at freeze time via
    // register_struct_methods.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defstruct :wat::kernel::StartupError …)`
    // in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
    // source of truth; Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::StartupError");

    // :wat::holon::CoincidentExplanation — arc 069 diagnostic record
    // returned by `:wat::holon::coincident-explain`. Bundles the raw
    // cosine, the current coincident floor, the dim where comparison
    // happened, the sigma feeding the floor, the same boolean
    // `coincident?` would have returned, and the smallest sigma at
    // which the pair would coincide. Lets a consumer see *why* a
    // coincidence judgement landed where it did instead of guessing.
    //
    // Auto-generated `CoincidentExplanation/new` + per-field accessors
    // land in the symbol table at freeze time via register_struct_methods.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defstruct :wat::holon::CoincidentExplanation …)`
    // in `wat/holon.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/holon.wat", ":wat::holon::CoincidentExplanation");

    // :wat::holon::Match — the result of `:wat::holon::Hologram/find`. A Hologram
    // matches by SIMILARITY, so the key `find` hands back is not necessarily the
    // probe that was passed in — it is whatever stored key coincided above the
    // filter's floor. `Match` carries that asymmetry in its name (the way
    // `match.group()` in a regex API is never assumed to equal the pattern);
    // `get` answers "what value did my probe reach?" and discards the matched
    // key, while `find` exists precisely so a caller (e.g. `HolographicLru::get`
    // bumping recency) can name the key that actually matched and act on it.
    // Fields stay `key` / `value`, matching `:wat::cache::Entry`'s precedent —
    // the type name carries the semantics, so `matched-key` here would be
    // redundant. Auto-generated `Match/key` / `Match/value` accessors land in
    // the symbol table at freeze time via register_struct_methods.
    // ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
    // is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::holon::Match …)`
    // in `wat/holon.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
    // Rust consumes it.
    ::wat_source_derive::wat_record_from!(env, "wat/holon.wat", ":wat::holon::Match");

    // :wat::core::Record — Arc 234 Stone 234.1.5. Opaque umbrella type for the
    // wat-record hologram (Value::wat__holon__Record). Pascal-Case namespace per
    // the `::`/`/` semantic-split doctrine: the namespace IS the umbrella
    // type; `::` verbs operate at the type tier (Record::of, Record::def,
    // Record::is?); `/` methods operate on instances (Record/field-at,
    // Record/to-map). Registered as opaque zero-field struct so the TypeEnv
    // contains the path and `env.types().get(":wat::core::Record")` resolves
    // cleanly. Per-class types (`:myapp::Voltage` as `:wat::core::Record` aliases)
    // ship in Stone 234.2b when the defrecord macro lands.
    // ⛔ ARC 296 — nature CORRECTED from `Struct` to `Record` (2026-08-15).
    //
    // This umbrella was registered `nature: Nature::Struct` — which, read against the holder
    // trit this project's own AGGREGATE-MODEL states (`Struct(−1)` impure-capable · `Record(0)`
    // pure, edn-repr, crosses · `HolonRecord(+1)` pure + VSA), said **"a record may hold impure
    // values"**: the exact inverse of what a record IS. The builder's verdict: *"this is
    // outrageous heresy."*
    //
    // The cause was legible in the old comment — it wanted an OPAQUE placeholder and reached for
    // `Struct` as the generic choice. `opaque` and `Struct` are not synonyms, and conflating them
    // cost three separate defects:
    //   1. `is_pure_type` grew a hardcoded short-circuit to undo it, whose own comment named the
    //      symptom ("would return a FALSE POSITIVE impure verdict") — patched at the CONSUMER
    //      instead of the declaration. That patch is DELETED with this fix.
    //   2. The patch was never given to the sibling `:wat::holon::Record`, so no pure aggregate
    //      could hold a field typed "any holon record" while "any record" was fine.
    //   3. A SPURIOUS LATTICE EDGE: `register_builtin` derives each type's subtype edge from
    //      `nature.root_keyword()`, so a Struct-natured Record umbrella emitted
    //      `:wat::core::Record <: :wat::core::Struct` — making EVERY record in wat a subtype of
    //      Struct, a claim nobody declared. With the nature correct, `child == root` and the
    //      guard skips: no edge, and a record is no longer assignable to a `:Struct` slot.
    //
    // Zero fields is not a reason to pick a holder — a record with no fields is legal (builder,
    // 2026-08-15). A root's nature is what it IS, not what is convenient to register.
    env.register_builtin(TypeDef::Aggregate(AggregateDef { nature: Nature::Record,
        name: ":wat::core::Record".into(),
        type_params: vec![],
        fields: vec![],
        restrictions: None,
    }));

    // Stone S-A — `:wat::holon::Record` opaque umbrella type + typesub root edge.
    //
    // `:wat::holon::Record` is the "holonic record" flavor — a core record that additionally
    // keeps a hologram IN PARITY with its data (builder, 2026-08-15: *"holonic records are
    // transmitted as edn and their holograms are rebuilt on consumption"*). The `typesub` edge
    // seeds the built-in is-a root: `:wat::holon::Record` is-a `:wat::core::Record`.
    //
    // ⛔ ARC 296 — nature CORRECTED from `Struct` to `HolonRecord` (2026-08-15), for the same
    // reason as its sibling above. This one was the sharper defect of the pair: it never got the
    // `is_pure_type` short-circuit that hid the sibling's, so the flavor that is MOST certainly
    // pure data — a record plus a hologram — was the one the type system called impure, and a
    // field typed `:wat::holon::Record` was REJECTED from every pure aggregate. Measured before
    // the fix: `[r <- :wat::core::Record]` clean, `[r <- :wat::holon::Record]` →
    // `ImpureFieldInPureAggregate`. It survived because nothing exercised the twin.
    //
    // NOTE: `register_type_predicates` synthesizes `:wat::holon::is-Record?` for this type
    // (same as `:wat::core::Record` gets `:wat::is-Record?`). This is correct — it is a type.
    // See SCORE-STONE-S-A § Honest deltas.
    env.register_builtin(TypeDef::Aggregate(AggregateDef { nature: Nature::HolonRecord,
        name: ":wat::holon::Record".into(),
        type_params: vec![],
        fields: vec![],
        restrictions: None,
    }));
    // Seed the built-in typesub root: `:wat::holon::Record` is-a `:wat::core::Record`.
    // Cannot cycle (fresh registry with no edges yet); `expect` is correct here.
    // built-in root hierarchy seed — no source form exists; unreachable cycle path (two distinct roots).
    env.register_subtype(":wat::holon::Record", ":wat::core::Record", crate::rust_caller_span!())
        .expect("built-in typesub root cannot cycle");

    // Arc 278 "errors first-class EDN" (stone 1) — register the `RuntimeError`
    // enum's variants as `:wat::core::Error`-satisfying decode records so a
    // startup / peer death carrying a `#wat.runtime/<Variant> {…}` cause
    // STRICT-decodes back to a TYPED record (not a string-wrapped blob, not an
    // `UnknownTag`). See `register_runtime_error_variants`.
    register_runtime_error_variants(env);

    // Arc 296 stone D — drain `inventory::iter::<::wat_edn::EdnSchema>()`.
    //
    // Any Rust type annotated with `#[derive(Edn)]` emits an
    // `::inventory::submit!(::wat_edn::EdnSchema { … })` at link time.  Here
    // we iterate those entries and call `register_builtin` for each, making
    // the type readable by `reconstruct_record` in `edn/render.rs` without any
    // hand-written registration.
    //
    // Ordering: this runs at `TypeEnv::with_builtins()` time — before stdlib
    // and user types land — which is correct because these are builtin
    // substrate types (e.g. `:wat::core::Pos`), not user-defined types.
    //
    // rune:sequi(ambient-context) — `inventory::iter` is link-time static
    // state; the same idiom as the `RestrictionEntry` drain in `freeze/env.rs`.
    for schema in inventory::iter::<::wat_edn::EdnSchema>() {
        // Convert "wat.core" + "Pos" → ":wat::core::Pos"
        let name = format!(
            ":{}::{}",
            schema.tag_ns.replace('.', "::"),
            schema.tag_name,
        );
        let fields: Vec<(String, TypeExpr)> = schema
            .fields
            .iter()
            .map(|(edn_key, wat_path)| {
                ((*edn_key).to_string(), TypeExpr::Path((*wat_path).to_string()))
            })
            .collect();
        env.register_builtin(TypeDef::Aggregate(AggregateDef {
            nature: Nature::Record,
            name,
            type_params: vec![],
            fields,
            restrictions: None,
        }));
    }

    // ── Stone 255-builtin-registry — names with MEMBERSHIP but no STRUCTURE ──
    //
    // `TypeEnv::contains` (through `SymbolTable::registrations`, THE DOOR at
    // `src/value/symbol_table.rs:244`) has never answered for the scalar
    // primitives, the built-in parametric container heads, or the opaque
    // capability/handle types — only for the 36 aggregate error/outcome
    // records registered above. This is the door's `Type` facet becoming
    // honest for the rest of its own population. Storage is option C (see
    // the DESIGN's CORRECTION): `register_builtin_leaf` adds the name to
    // `builtin_names` only — `get` stays `None`, because a primitive/opaque
    // genuinely has no `TypeDef` to fabricate one for.

    // Groups 1 & 2 — DERIVED from the checker's own consts, never
    // transcribed, so the registry cannot drift from `check.rs`'s source of
    // truth. `BARE_CONTAINER_HEADS`'s FQDN column carries NO leading colon
    // (it follows `TypeExpr::Parametric.head`'s convention) — add one back
    // so every registered name is colon-prefixed like the rest of the
    // registry (EXPECTATIONS' named trap-door: a naive iteration registers
    // `wat::core::Vector` and row 2 half-passes on the container).
    for (_bare, fqdn) in crate::check::BARE_PRIMITIVES {
        env.register_builtin_leaf(*fqdn);
    }
    for (_bare, fqdn) in crate::check::BARE_CONTAINER_HEADS {
        env.register_builtin_leaf(format!(":{fqdn}"));
    }

    // Group 3 — opaque capability/handle types and scalar/AST-leaf sentinels
    // with no const to derive from: Rust structs (or Rust-checker literals)
    // exposed to wat with no `TypeDef`, a token rather than a structure.
    // Evidence for this list originates from a rider's convergence on branch
    // `arc109-type-refs-parked` (`src/resolve/type_refs.rs`'s
    // `known_builtin_leaf_types`), but every name below was RE-VERIFIED
    // against this tree's own corpus (`grep -rn <name> --include=*.wat .`,
    // excluding `target/`) before being registered — each citation is one
    // real occurrence, not the full count. `:wat::core::Never` is the one
    // name from that evidence list that did NOT clear this bar (see the
    // rider's report) and is deliberately NOT registered here.
    for name in [
        // scalars — check.rs::infer literal construction (RationalLit,
        // BigIntLit, keyword); e.g. `wat/core.wat:90` `[x <- :wat::core::bigint] ->
        // :wat::core::bigint`, `wat/core.wat:118` (`rational`),
        // `wat-scripts/scratch-pad/probe-timer-as-peer.wat:44` `-> :wat::core::keyword`.
        ":wat::core::bigint",
        ":wat::core::rational",
        ":wat::core::keyword",
        // AST leaves — `wat-tests/holon/Reject.wat:31` (`HolonAST` param+return),
        // `tests/resolve/probe_arc251_decl_migrator.wat:4` `[kw <- :wat::WatAST] -> :wat::WatAST`.
        ":wat::holon::HolonAST",
        ":wat::WatAST",
        // sentinels — `:wat::core::Value` is the universal top, genuinely used
        // as a declared type: `tests/types/probe_arc278_value_universal_top_widen.wat:4`
        // `(:wat::core::defrecord :my::Box [slot <- :wat::core::Value])`,
        // `tests/collection/probe_map_container.wat:68` `-> (:wat::core::Option :wat::core::Value)`.
        // `:wat::core::Never` is EXCLUDED — see the header note above.
        ":wat::core::Value",
        // container — no bare-legacy pairing to derive from; `wat/seq.wat:240`
        // `coll <- (:wat::core::List :- [T])`, `wat-scripts/scratch-pad/probe-seqable-to-stream-native-check.wat:17`
        // `-> (:wat::core::List :- [:wat::core::i64])`.
        ":wat::core::List",
        // opaques — `wat/telemetry.wat:77` `uuid <- :wat::core::Uuid`;
        // `wat/cache.wat:273` `[hologram <- :wat::holon::Hologram`;
        // `tests/collection/vector_first_class.wat:19` `vec <- :wat::holon::Vector`
        // (the algebra Vector, distinct from the container `:wat::core::Vector`
        // derived from `BARE_CONTAINER_HEADS` above);
        // `tests/rete/probe_arc278_6a_purity.wat:10` `-> :wat::io::IOReader`;
        // `tests/program/wat_arc170_program_contracts_t1_legacy_3arg.wat:4`
        // `stdout <- :wat::io::IOWriter`.
        ":wat::core::Uuid",
        ":wat::holon::Hologram",
        ":wat::holon::Vector",
        ":wat::io::IOReader",
        ":wat::io::IOWriter",
        // kernel opaques — `tests/kernel/probe_arc278_close_outcome_wall.wat:19`
        // `-> (:wat::kernel::Process :- [:wat::core::i64 :wat::core::i64])`;
        // `wat-tests/test.wat:77` `self <- (:wat::kernel::ThreadSelfPeer :- […])`
        // (also covers `Thread`, the family it self-identifies as);
        // `wat-tests/service-parametric-messages.wat:116`
        // `a <- (:wat::kernel::Address :- […])`;
        // `wat-scripts/scratch-pad/probe-timer-as-peer.wat:28`
        // `l <- (:wat::kernel::Listener :- [:wat::core::keyword :wat::core::nil])`;
        // `wat-tests/service-parametric-messages.wat:117`
        // `-> (:wat::kernel::Peer :- […])`.
        ":wat::kernel::Process",
        ":wat::kernel::Thread",
        ":wat::kernel::Address",
        ":wat::kernel::Listener",
        ":wat::kernel::Peer",
        ":wat::kernel::ThreadSelfPeer",
        // stream — `wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat:34`
        // `-> (:wat::stream::Stream :- [U])`.
        ":wat::stream::Stream",
        // time — `wat/service.wat:56` `after <- :wat::time::Duration` (field
        // type in the stdlib itself, not just a probe);
        // `wat-scripts/scratch-pad/probe-derive-chain-split.wat:98`
        // `t0 <- :wat::time::Instant`.
        ":wat::time::Duration",
        ":wat::time::Instant",
        // rust-backed — the RHS of `:wat::kernel::Sender :- [T]` /
        // `:wat::kernel::Receiver :- [T]`'s own typealiases in the stdlib:
        // `wat/kernel/channel.wat:43` `(:rust::crossbeam_channel::Sender :- [T]))`,
        // `wat/kernel/channel.wat:46` `(:rust::crossbeam_channel::Receiver :- [T]))`.
        ":rust::crossbeam_channel::Sender",
        ":rust::crossbeam_channel::Receiver",
    ] {
        env.register_builtin_leaf(name);
    }
}

/// Arc 278 "errors first-class EDN" (stone 1) — register the `RuntimeError`
/// enum's variants as `:wat::core::Error`-satisfying decode RECORDS.
///
/// Each `RuntimeError` emits `#wat.runtime/<Variant> {…}` via `#[derive(ToEdn)]`
/// (`RuntimeErrorKind`, `signal.rs`) composed with the `WatError::error_edn()`
/// floor (`:message` / `:location` / `:causes` + the variant's own coordinate
/// fields). That derive is WRITE-ONLY (no `EdnSchema` submit) — so STRICT
/// `edn_to_value` hit `UnknownTag` and the cause was string-wrapped. Here we
/// hand-register the DECODE schema for each variant so a startup / peer death
/// cause round-trips to a typed record (`reconstruct_record`, `edn/render.rs`).
///
/// **Why a hand table and not the `#[derive(Edn)]` flip (the STOP):** flipping
/// `RuntimeErrorKind` `ToEdn → Edn` runs the schema generator over ALL 32
/// variants, which hits the `derive`'s STOP-2 scalar-only wall on the hairy
/// field types (`Box<ValueSnapshot>` / `&'static str` / `Span` / `Vec<_>` /
/// `Option<_>` / nested `HashError` / `MacroError`) AND would require the derive
/// to compose the floor keys — a substrate change bigger than this stone. Per
/// DESIGN-errors-first-class-edn.md's STOP clause, the derive-enhancement is
/// split into its own stone; this bounded loop registers RuntimeError for the
/// proof, WITHOUT a lossy uniform `[message location causes]` shortcut (each
/// variant keeps its coordinate fields).
///
/// **Scope:** the 25 variants whose coordinate fields are fully
/// scalar-decodable (String / i64 / (Option :- [String]) / (Vector :- [String]) / Span) are
/// registered here. The 7 variants that carry a nested value-snapshot / typed
/// sub-error (`NotCallable`, `TypeMismatch`, `BadCondition`,
/// `EvalVerificationFailed`, `MacroExpansionFailed`, `NoMatchingClause`,
/// `PostconditionFailed`) are DEFERRED to the stone that registers their nested
/// sub-value types (`ValueSnapshot` / `HashError` / `MacroError` /
/// `ClauseAttempt`); their outer record cannot fully decode until then.
///
/// Every `RuntimeError`'s `:location` is a real `Span` (`error_edn()` splices
/// `self.span`; never nil) — so no nil-location B-leaf fix is owed here.
fn register_runtime_error_variants(env: &mut TypeEnv) {
    // The `:wat::core::Error` floor keys, prepended to every variant record.
    // `:message` String, `:location` Span (registered above), `:causes`
    // (Vector :- [Error]). A variant whose own field is literally named `message`
    // (`AssertionFailed`, `MacroAbort`) has it stripped by `error_edn()`'s
    // floor-dedup, so it is NOT re-declared as a coordinate field.
    let s = |p: &str| TypeExpr::Path(p.to_string());
    let string = || s(":wat::core::String");
    let i64t = || s(":wat::core::i64");
    let span = || s(":wat::core::Span");
    let opt_string = || TypeExpr::Parametric {
        head: "wat::core::Option".into(),
        args: vec![TypeExpr::Path(":wat::core::String".into())],
    };
    let vec_string = || TypeExpr::Parametric {
        head: "wat::core::Vector".into(),
        args: vec![TypeExpr::Path(":wat::core::String".into())],
    };
    let floor = || -> Vec<(String, TypeExpr)> {
        vec![
            ("message".into(), TypeExpr::Path(":wat::core::String".into())),
            ("location".into(), TypeExpr::Path(":wat::core::Span".into())),
            (
                "causes".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":wat::core::Error".into())],
                },
            ),
        ]
    };

    // (variant tag name, coordinate fields) — EDN keys are the kebab-cased
    // field idents / `#[to_edn(key = …)]` overrides from `signal.rs`.
    let variants: Vec<(&str, Vec<(String, TypeExpr)>)> = vec![
        ("UnboundSymbol", vec![("name".into(), string())]),
        ("UnknownFunction", vec![("path".into(), string())]), // ← the cache-probe gate
        (
            "ArityMismatch",
            vec![("op".into(), string()), ("expected".into(), i64t()), ("got".into(), i64t())],
        ),
        ("MalformedForm", vec![("head".into(), string()), ("reason".into(), string())]),
        ("ParamShadowsBuiltin", vec![("name".into(), string())]),
        ("DivisionByZero", vec![]),
        (
            "IntegerOverflow",
            vec![("op".into(), string()), ("a".into(), i64t()), ("b".into(), i64t())],
        ),
        ("DuplicateDefine", vec![("name".into(), string())]),
        ("ReservedPrefix", vec![("prefix".into(), string())]),
        ("UnnamespacedName", vec![("name".into(), string())]),
        ("DottedName", vec![("name".into(), string())]),
        ("DeclarationInExpressionPosition", vec![("head".into(), string())]),
        ("EvalForbidsMutationForm", vec![("head".into(), string())]),
        ("UserMainMissing", vec![]),
        ("ChannelDisconnected", vec![("op".into(), string())]),
        ("NoEncodingCtx", vec![("op".into(), string())]),
        ("NoSourceLoader", vec![("op".into(), string())]),
        ("NoMacroRegistry", vec![("op".into(), string())]),
        ("PatternMatchFailed", vec![("value-type".into(), string())]),
        ("EffectfulInStep", vec![("op".into(), string())]),
        ("NoStepRule", vec![("op".into(), string())]),
        // `message` collides with the floor → floor-only + these two.
        (
            "AssertionFailed",
            vec![("actual".into(), opt_string()), ("expected".into(), opt_string())],
        ),
        (
            "SandboxScopeLeak",
            vec![("offending-name".into(), string()), ("outer-define-span".into(), span())],
        ),
        ("ServiceNotRunning", vec![("op".into(), string())]),
        (
            "EdnCoerceMismatch",
            vec![
                ("op".into(), string()),
                ("expected".into(), string()),
                ("got".into(), string()),
                // `edn_path_segments` writes `:path` as (Vector :- [String]) segments.
                ("path".into(), vec_string()),
            ],
        ),
        (
            "UnknownField",
            vec![
                ("record-class".into(), string()),
                ("field".into(), string()),
                ("available".into(), vec_string()),
            ],
        ),
        // `message` collides with the floor → floor-only (no extra coordinate).
        ("MacroAbort", vec![]),
    ];

    for (variant, coords) in variants {
        let mut fields = floor();
        fields.extend(coords);
        env.register_builtin(TypeDef::Aggregate(AggregateDef {
            nature: Nature::Record,
            name: format!(":wat::runtime::{}", variant),
            type_params: vec![],
            fields,
            restrictions: None,
        }));
    }
}

/// Arc 293 K3 — derive the THREE backing aggregates from a surface declaration.
///
/// For every `SurfaceDef` with a `nature`, emits THREE companion `TypeDef::Aggregate` values:
///   - `<surface-name>$core-record`   (nature = Record)
///   - `<surface-name>$holon-record`  (nature = HolonRecord)
///
/// Both share the same fields (surface's `Field` members only; methods excluded).
/// The surface's own `:nature` governs satisfaction (who may pass `[x <- :S]` slots);
/// these two backing types are always emitted regardless of the surface's nature so
/// callers may project at either pure tier via `to-record` / `:wat::holon::to-record`.
/// RETIRED 293 K3-revise: `$struct` (Struct-nature backing) — projection is ONE-WAY UP;
/// `$struct` is the impure tier; you already have the struct in locus. See retirement.rs.
///
/// Returns an empty Vec for surfaces without a `:nature` (abstract surfaces; defensive skip).
/// Each flows through `register_aggregate_methods` (step 6.8a in freeze/env.rs) to
/// auto-generate the ctor and per-field accessors — exactly as a hand-written aggregate would.
fn derive_surface_backing_records(surface: &SurfaceDef) -> Vec<TypeDef> {
    if surface.nature.is_none() {
        return vec![];
    }
    let fields: Vec<(String, TypeExpr)> = surface
        .members
        .iter()
        .filter_map(|m| match m {
            SurfaceMember::Field { name, ty } => Some((name.clone(), ty.clone())),
            SurfaceMember::Method { .. } => None,
        })
        .collect();
    // RETIRED 293 K3-revise: $struct (Struct-nature) removed — see retirement.rs.
    // A surface emits the PAIR: $core-record (portable EDN) + $holon-record (EDN + VSA).
    vec![
        TypeDef::Aggregate(AggregateDef {
            name: format!("{}$core-record", surface.name),
            type_params: vec![],
            fields: fields.clone(),
            nature: Nature::Record,
            restrictions: None,
        }),
        TypeDef::Aggregate(AggregateDef {
            name: format!("{}$holon-record", surface.name),
            type_params: vec![],
            fields,
            nature: Nature::HolonRecord,
            restrictions: None,
        }),
    ]
}

/// Arc 293 S1 — synthesize a surface's wire-protocol enums (`<S>::Op` / `<S>::Reply`).
///
/// When a `defsurface` carries METHOD members whose request AND response sigs are all
/// **pure** (`is_pure_type` — EDN-crossable), the surface names a serviceable wire
/// protocol: it emits two `Pure` enums, one variant per method —
/// `<S>::Op::<Method> [req <- <request>]` and `<S>::Reply::<Method> [resp <- <response>]`.
/// These shared enums are what every `:satisfies` service speaks and every `:calls`
/// client dials (later 293 stones), so they are built **structurally IDENTICAL** to
/// what a hand-written `defenum` registers (downstream cannot tell they were synthesized).
///
/// The purity gate is DERIVED, not a marker: a surface is loci-agnostic by nature, so it
/// is always dialable *unless* its sigs can't cross. Impure sigs (a method holding a live
/// `Peer'`/`Connection`) → 293.W already rejects such an enum → the surface is in-thread-only
/// and we synthesize NOTHING for it (silent, correct; the surface still registers + works
/// for extend-type / width-subtyping).
///
/// Returns `vec![]` (synthesize nothing) when:
///   - the surface has **no** method members (a pure data surface — not a service);
///   - any method lacks a request arg (`args[1]`, the payload after `self`);
///   - any method's request or response type is **impure** (STOP-IMPURE);
///   - `<S>::Op` or `<S>::Reply` **already exists** (a user hand-declared protocol wins —
///     never overwrite; STOP-COLLISION).
///
/// Mirrors `derive_surface_backing_records`: called inline in `register_types_impl` using
/// the SAME `register` closure, so the synthesized enums inherit the surface's registration
/// privilege (stdlib surface → `register_stdlib`, user surface → `register`).
///
/// Purity is judged against `env`, which — by source order — already holds the request/
/// response records declared before the surface. (A request/response record declared AFTER
/// the surface is an unknown path to `is_pure_type` → treated as pure-by-convention; if it
/// is in fact impure, the post-registration containment pass `validate_aggregate_containment`
/// catches the synthesized `Pure` enum's impure field, exactly as it backstops the backing
/// records above.)
#[wat_special_form_impl(":wat::core::defsurface", role = declare)]
fn synthesize_surface_protocol(
    surface: &SurfaceDef,
    env: &TypeEnv,
    acronyms: &HashMap<String, Vec<String>>,
    decl_span: &Span,
) -> Result<Vec<TypeDef>, TypeError> {
    // Arc 278 S4c — the surface's namespace-scoped acronym set. Keyed by the surface's own
    // name (with leading colon), EXACTLY as `defservice :impls` keys its lookup on
    // `proto-str`/`surface-kw` (the satisfied surface's keyword string) — see
    // `wat/service.wat` `kebab->pascal-in <surface-kw> <op>` at :805/:1041. No registry
    // entry for this surface → empty set → plain kebab→pascal (the prior behavior).
    let ns_acronyms: &[String] = acronyms
        .get(&surface.name)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut op_variants: Vec<EnumVariant> = Vec::new();
    let mut reply_variants: Vec<EnumVariant> = Vec::new();
    let mut saw_method = false;

    // Arc 278 #16 Stone 16.1c — the required shape of the ruling-A `RequestTooLarge` variant:
    // exactly `[bytes <- :wat::core::i64  cap <- :wat::core::i64]` (matched against each
    // serviceable op-Response enum below). `:wat::core::i64` is how the parser represents an
    // i64 field type (see `TypeExpr::Path(":wat::core::i64")` throughout, e.g. wat/query.wat).
    const RTL_VARIANT: &str = "RequestTooLarge";
    let rtl_fields: Vec<(String, TypeExpr)> = vec![
        ("bytes".to_string(), TypeExpr::Path(":wat::core::i64".into())),
        ("cap".to_string(), TypeExpr::Path(":wat::core::i64".into())),
    ];
    // Arc 278 Stone 2 (ANNIHILATE the knob) — the SHAPE sibling of the size variant, and now
    // under the identical lock. Stone 1 built the request-shape guard and defaulted it OFF
    // behind an `:all | :none` opt-in clause; no service opted in, so the denial of
    // service (a wrong-typed body under a correct tag kills the service for EVERY client)
    // stayed live across the whole corpus. Builder ruling: a knob whose off-position is "crash
    // on malformed input" is a non-option surfaced as a choice. The clause is deleted and
    // `wat/service.wat` now generates the guard UNCONDITIONALLY for every op of every service
    // — which makes the refusal variant MANDATORY here, exactly as `RequestTooLarge` is.
    //
    // WHY DECLARED AND NOT AUTO-INJECTED: an exception path the author never wrote is a
    // surprise in the caller's exhaustive match. Every failure kind is a VISIBLE,
    // author-declared, checker-forced named variant — the verbosity IS the shield.
    //
    // Shape: `[path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String
    //          got <- :wat::core::String]`. `path` is STRUCTURED (the segments `["items" "[0]"]`
    // a caller indexes and walks — real data); `expected`/`got` are rendered Strings because
    // `got` is NOT a type and cannot be made one: the value arrived off an untyped wire with no
    // declaration, so its honest datum is its EDN shape ("Integer", "Vector", "Map"). Ruled at
    // Stone 1, four questions, 4×YES / 4×NO (DESIGN-request-malformed-input-sanitization.md).
    //
    // Arc 109 ③ — angle brackets are ILLEGAL for types, so `parse_type_expr` on a
    // `Vector<T>`-spelled string constant no longer parses (it screams, correctly — this
    // constant WAS exactly the class of hardcoded angle-string this stone hunts). Hand-
    // assemble the `TypeExpr` directly instead of parsing the canonical spelling: this is a
    // Rust-level literal, not a `.wat` source form, so there is no reference-FORM keyword
    // text (`(:wat::core::Vector :- [...])`) to parse in the first place — direct
    // `TypeExpr::Parametric` construction IS the canonical spelling at this layer.
    const RM_VARIANT: &str = "RequestMalformed";
    // User-facing EXAMPLE text only (the format!()s further down, showing a remedy) — the
    // `TypeExpr` itself is hand-assembled below, not parsed from this string.
    const RM_PATH_TY: &str = "(:wat::core::Vector :- [:wat::core::String])";
    let rm_fields: Vec<(String, TypeExpr)> = vec![
        (
            "path".to_string(),
            TypeExpr::Parametric {
                head: "wat::core::Vector".to_string(),
                args: vec![TypeExpr::Path(":wat::core::String".to_string())],
            },
        ),
        ("expected".to_string(), TypeExpr::Path(":wat::core::String".into())),
        ("got".to_string(), TypeExpr::Path(":wat::core::String".into())),
    ];
    // Ruling A binds SERVICEABLE ops — the wire ops of a service. Only a `:nature
    // :wat::kernel::Peer` surface is a service (its ops' returns ARE `<Op>Response`s
    // that cross the wire and can face a too-large request); a `:Struct`/`:Record`/
    // `:HolonRecord` surface's methods are in-thread accessors whose returns are ordinary
    // values, not Responses, so the RequestTooLarge lock does not apply to them.
    let enforce_rtl_lock = surface.nature == Some(Nature::Peer);

    for member in &surface.members {
        let SurfaceMember::Method { name, args, ret, max_request_bytes_explicit, .. } = member
        else {
            continue; // Field members are data, not operations.
        };
        saw_method = true;

        // Arc 278 #16 Stone 16.3 — MANDATORY `:max-request-bytes` lock. Mirrors 16.1c's
        // RequestTooLarge lock immediately below: same gate (`enforce_rtl_lock` — ONLY a
        // `:nature :Peer'` surface's ops are wire ops), same site (this per-member loop, before
        // any downstream codegen consumes the surface), same shape (a located `MalformedDecl`
        // naming the offending op + surface). A serviceable op that omits the key would
        // otherwise ride the silent `DEFAULT_MAX_FRAME_BYTES` parse-time default (see
        // `SurfaceMember::Method::max_request_bytes` doc) straight into 16.2's per-op
        // enforcement codegen — a silent cap nobody chose. Make the omission a compile error
        // instead: every wire op must explicitly speak its own budget.
        if enforce_rtl_lock && !max_request_bytes_explicit {
            return Err(TypeError::new(
                decl_span.clone(),
                TypeErrorKind::MalformedDecl {
                    head: ":wat::core::defsurface".to_string(),
                    reason: format!(
                        "op `{}` in surface {}: `:max-request-bytes N` is MANDATORY on a \
                         serviceable (`:nature :Peer`) op — a wire op must explicitly declare \
                         its request-byte budget, never ride the silent {}-byte default (arc \
                         278 #16 Stone 16.3). Add `:max-request-bytes <N>` after the op's \
                         `-> :Response` in its `:features` clause.",
                        name,
                        surface.name,
                        crate::edn::render::DEFAULT_MAX_FRAME_BYTES,
                    ),
                },
            ));
        }

        // Arc 278 #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05: "convention is
        // law — enforce it… services are our OOP layer, we make requests to them and get
        // responses back."). Same gate, same site, same shape as the two locks immediately
        // above: a serviceable op's response type name is no longer read (the machinery that
        // read it — `build_op_response_type_constants` and the wat-side EDN-decode branches —
        // is deleted by this same strike); it is REQUIRED to be
        // `<surface-base>::<OpPascal>Response`, checker-forced here so the wat macros can go
        // back to splicing a literal ctor keyword, guaranteed correct by construction.
        //
        // Compare the BASE name only (⛔ never the rendered type — `(GetResponse :- [K V])`
        // CONFORMS): type args are stripped from both sides, and `TypeExpr::Path` carries a
        // leading `:` while `TypeExpr::Parametric`'s `head` does NOT (deliberate — see the note
        // at the now-deleted `build_op_response_type_constants`) — normalize here, at this one
        // read site, exactly as that function did.
        if enforce_rtl_lock {
            let declared_base: String = match ret.base_fqdn() {
                Some(b) => b,
                None => {
                    return Err(TypeError::new(
                        decl_span.clone(),
                        TypeErrorKind::MalformedDecl {
                            head: ":wat::core::defsurface".to_string(),
                            reason: format!(
                                "op `{}` in surface {}: a serviceable op's response type must \
                                 be a nameable type (`<Op>Response`) — declared `{:?}`, which \
                                 has no name to compare against the law (arc 278 #74: an op's \
                                 response type IS `<Op>Response`)",
                                name, surface.name, ret
                            ),
                        },
                    ));
                }
            };
            // `surface.name` is stored WITHOUT its `<...>` suffix by `parse_declared_name`
            // (type params are split off into `surface.type_params` at parse time) — and
            // `<K,V>` is unexpressible at all (arc 109 ③'s wall, `src/types.rs:4688`), so no
            // keyword the reader hands back can carry one to begin with. Used directly, never
            // stripped (arc 109 "reap the twelve" — measured 41,172 calls, 0 type-heads).
            let surface_base = surface.name.as_str();
            let required = format!(
                "{surface_base}::{}Response",
                crate::string::kebab_to_pascal_with_acronyms(name, ns_acronyms),
            );
            if declared_base != required {
                return Err(TypeError::new(
                    decl_span.clone(),
                    TypeErrorKind::MalformedDecl {
                        head: ":wat::core::defsurface".to_string(),
                        reason: format!(
                            "op `{}` in surface {}: response type name is LAW — declared `{}`, \
                             required `{}` (arc 278 #74, builder ruling 2026-08-05: an op's \
                             response type IS `<Op>Response`; rename the declaration to match)",
                            name, surface.name, declared_base, required
                        ),
                    },
                ));
            }
        }

        // Request payload = the arg AFTER `self` (`args[1]`). A method with no request arg
        // carries no wire payload → this surface is not a clean protocol; synthesize nothing.
        let request_ty = match args.fixed_params.get(1) {
            Some((_, ty)) => ty.clone(),
            None => return Ok(vec![]),
        };

        // Arc 278 #74b — `<Op>Request` is LAW, the twin of the `<Op>Response` rule above (same
        // builder ruling, 2026-08-05: "convention is law — enforce it… services are our OOP
        // layer, we make requests to them and get responses back."). Same gate
        // (`enforce_rtl_lock`), same base-name comparison, same both-names diagnostic. Placed
        // AFTER the request-arg bail immediately above (an op with no request arg has no
        // request to name) and unconditionally AFTER the Response check above it — do not
        // reorder: `probe_arc278_repl_durable_forms_response_law.wat.bad` violates BOTH laws
        // and its committed test asserts the Response message verbatim, so Response must fire
        // first.
        if enforce_rtl_lock {
            let declared_base: String = match request_ty.base_fqdn() {
                Some(b) => b,
                None => {
                    return Err(TypeError::new(
                        decl_span.clone(),
                        TypeErrorKind::MalformedDecl {
                            head: ":wat::core::defsurface".to_string(),
                            reason: format!(
                                "op `{}` in surface {}: a serviceable op's request type must \
                                 be a nameable type (`<Op>Request`) — declared `{:?}`, which \
                                 has no name to compare against the law (arc 278 #74b: an op's \
                                 request type IS `<Op>Request`)",
                                name, surface.name, request_ty
                            ),
                        },
                    ));
                }
            };
            // `surface.name` is stored WITHOUT its `<...>` suffix by `parse_declared_name`
            // (type params are split off into `surface.type_params` at parse time) — and
            // `<K,V>` is unexpressible at all (arc 109 ③'s wall, `src/types.rs:4688`), so no
            // keyword the reader hands back can carry one to begin with. Used directly, never
            // stripped (arc 109 "reap the twelve" — measured 41,859 calls, 0 type-heads).
            let surface_base = surface.name.as_str();
            let required = format!(
                "{surface_base}::{}Request",
                crate::string::kebab_to_pascal_with_acronyms(name, ns_acronyms),
            );
            if declared_base != required {
                return Err(TypeError::new(
                    decl_span.clone(),
                    TypeErrorKind::MalformedDecl {
                        head: ":wat::core::defsurface".to_string(),
                        reason: format!(
                            "op `{}` in surface {}: request type name is LAW — declared `{}`, \
                             required `{}` (arc 278 #74b, builder ruling 2026-08-05: an op's \
                             request type IS `<Op>Request`; rename the declaration to match)",
                            name, surface.name, declared_base, required
                        ),
                    },
                ));
            }
        }

        // The purity gate: BOTH request and response must cross (EDN-serializable). Any impure
        // sig → in-thread-only surface → synthesize nothing (293.W would reject an impure enum).
        if !crate::check::is_pure_type(&request_ty, env)
            || !crate::check::is_pure_type(ret, env)
        {
            return Ok(vec![]);
        }

        // Arc 278 #16 Stone 16.1c — LOCK ruling A. Every serviceable op-Response must be an
        // outcome ENUM carrying a well-shaped `RequestTooLarge [bytes <- i64  cap <- i64]`
        // variant: any wire op can face a request that overruns its `:max-request-bytes`
        // budget, and that breach must be a first-class, non-swallowable outcome of the op
        // (records-as-Responses are retired for services). A fleet migration (git 4536eaf6)
        // already brought every Response into conformance — this rule is the CONTRACT LOCK:
        // a future op whose Response is a record, or an enum missing/malforming
        // `RequestTooLarge`, becomes a LOCATED compile error instead of a silent drift.
        //
        // Resolution (STOP-1 cleared): `ret` is the op's `<Op>Response` path. Its type-decl
        // was hoisted OUT of this surface's `:messages` block to the top-level form stream
        // BEFORE the defsurface form (expand_all's `hoist_surface_messages`, src/macros/
        // expand.rs), so it is already registered in `env` by the time this synthesis runs —
        // `env.get(<path>)` resolves it. (This is why the rule can live at synthesize time:
        // enums, unlike the retired record Responses, resolve here.) A Response declared
        // AFTER the surface is not our concern — it is an unknown path with its own
        // unresolved-reference diagnostic downstream, so we only lock what we can resolve.
        // Arc 278 #76 — the lock reaches a PARAMETRIC response too. It used to read
        // `if let TypeExpr::Path(resp_path) = ret`, so a `(GetResponse :- [K V])` (a
        // `TypeExpr::Parametric`) fell to the `_ => {}` arm below and the whole ruling-A
        // SHAPE lock — RequestTooLarge well-shaped, RequestMalformed well-shaped — NEVER
        // RAN ON IT. Proven with a non-vacuity control: a parametric response missing
        // `RequestTooLarge` was ACCEPTED silently while its monomorphic twin was refused,
        // located. Every parametric Response in the corpus (4, one of them stdlib —
        // `wat/cache.wat`) carried both variants by AUTHOR DILIGENCE, not by this lock.
        //
        // A parametric decl REGISTERS under its bare base — `parse_declared_name`
        // (~:4676) refuses any `<` in the name outright (arc 109 ③) and returns an
        // empty params Vec; the type params are parsed separately, from the sibling
        // `Head :- [T …]` binder (`take_declared_binder`, just below it) — so
        // `env.get` needs the base with its colon re-prepended. `Parametric`'s `head` is
        // stored WITHOUT the leading `:` and that is DELIBERATE (both parse paths,
        // `(Head :- [args])` and `(Ctor arg…)`, must yield a byte-identical head for
        // unification), so the normalization belongs HERE, at the read site, never
        // upstream in the parser. This is the same one-line-per-site hand-match that
        // task #75's `TypeExpr` accessor exists to delete across ~137 sites; when that
        // lands, this becomes one of its call sites.
        let resp_lookup: Option<String> = ret.base_fqdn();
        if enforce_rtl_lock { if let Some(resp_path) = resp_lookup.as_ref() {
            match env.get(resp_path) {
                Some(TypeDef::Aggregate(_)) => {
                    return Err(TypeError::new(
                        decl_span.clone(),
                        TypeErrorKind::MalformedVariant {
                            enum_name: resp_path.clone(),
                            offending: RTL_VARIANT.to_string(),
                            reason: format!(
                                "op `{}` in surface {}: `{}` must be an outcome enum carrying \
                                 `{}` (records-as-Responses are retired for services — arc 278 \
                                 ruling A); make it `(:wat::core::defenum {} :wat::enum::Pure \
                                 :Ok [...] :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64] \
                                 :RequestMalformed [path <- {}  expected <- :wat::core::String  \
                                 got <- :wat::core::String])`",
                                name, surface.name, resp_path, RTL_VARIANT, resp_path, RM_PATH_TY
                            ),
                            remedies: vec![],
                        },
                    ));
                }
                Some(TypeDef::Enum(EnumDef { variants, .. })) => {
                    let well_shaped = variants.iter().any(|v| {
                        matches!(v,
                            EnumVariant::Tagged { name: vn, fields }
                                if vn == RTL_VARIANT && *fields == rtl_fields)
                    });
                    if !well_shaped {
                        return Err(TypeError::new(
                            decl_span.clone(),
                            TypeErrorKind::MalformedVariant {
                                enum_name: resp_path.clone(),
                                offending: RTL_VARIANT.to_string(),
                                reason: format!(
                                    "op `{}` in surface {}: `{}` must carry \
                                     `:RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]` \
                                     (arc 278 ruling A — every serviceable op-Response is an outcome \
                                     enum that can face a too-large request)",
                                    name, surface.name, resp_path
                                ),
                                remedies: vec![],
                            },
                        ));
                    }
                    // Arc 278 Stone 2 — the SHAPE half, same lock, same site, same standing.
                    // `wat/service.wat` generates the request-shape guard unconditionally into
                    // every op's dispatch arm; on a violation it replies with THIS variant and
                    // keeps serving. Omitting it would make the generated guard reference a
                    // variant that does not exist — so the omission is a located error here,
                    // where the author can see which op and which surface, rather than an
                    // unresolved path inside expanded macro output.
                    let rm_shaped = variants.iter().any(|v| {
                        matches!(v,
                            EnumVariant::Tagged { name: vn, fields }
                                if vn == RM_VARIANT && *fields == rm_fields)
                    });
                    if !rm_shaped {
                        return Err(TypeError::new(
                            decl_span.clone(),
                            TypeErrorKind::MalformedVariant {
                                enum_name: resp_path.clone(),
                                offending: RM_VARIANT.to_string(),
                                reason: format!(
                                    "op `{}` in surface {}: `{}` must carry \
                                     `:RequestMalformed [path <- {}  expected <- :wat::core::String  \
                                     got <- :wat::core::String]` (arc 278 Stone 2 — input \
                                     sanitization is unconditional: every serviceable op-Response \
                                     is an outcome enum that can face a request whose SHAPE is not \
                                     the one the op declared it accepts, and that refusal must be a \
                                     first-class value the caller's exhaustive match faces, never a \
                                     crash that takes the service down for every other client)",
                                    name, surface.name, resp_path, RM_PATH_TY
                                ),
                                remedies: vec![],
                            },
                        ));
                    }
                }
                // Non-Path ret, or a ret that resolves to a Newtype/Alias/Union/Surface, or an
                // as-yet-unregistered path — out of this lock's scope (each has its own
                // diagnostic elsewhere). Only records and enums are Response candidates.
                _ => {}
            }
        } }

        // Variant name = PascalCase(method-name) via the EXISTING kebab→pascal conversion
        // (`put` → `Put`, `scan-index` → `ScanIndex`), threading the surface's namespace
        // acronyms so `create-web-acl` → `CreateWebACL` when `ACL` is declared — the SAME
        // registry `defservice :impls` consults, so the two paths agree on casing.
        let variant = crate::string::kebab_to_pascal_with_acronyms(name, ns_acronyms);
        op_variants.push(EnumVariant::Tagged {
            name: variant.clone(),
            fields: vec![("req".to_string(), request_ty)],
        });
        reply_variants.push(EnumVariant::Tagged {
            name: variant,
            fields: vec![("resp".to_string(), ret.clone())],
        });
    }

    if !saw_method {
        return Ok(vec![]); // pure data surface (fields only) — not a service.
    }

    // Arc 278 no-hidden-failures — the PROTOCOL-TIER failure floor. Each `<Op>Response` is
    // already an outcome enum (:Success | :Transient | :Fatal), so failure is first-class AT
    // THE OP TIER. But a decode failure is a TIER BELOW: the client message never resolves to
    // any op (the whole `Op::<Method>(…)` fails to hydrate), so no `<Op>Response` can carry it.
    // A reserved `Reply::Failed [cause <- :wat::kernel::Failure]` variant lays the missing
    // floor: the serve loop replies it to the originating client and the generated client
    // method surfaces it as an unignorable raise (wat/service.wat) — the service survives, and
    // no caller is ever left blind. This completes the 293 outcome model; it is not a new
    // paradigm.
    //
    // COLLISION — the reserved name `Failed` clashes if the surface declares a method that
    // pascal-cases to `Failed` (an op literally named `failed`). That is a real conflict, not
    // something to silently override: a user's `failed` op and the protocol-tier failure floor
    // cannot both own `Reply::Failed`. Surface it (the surface must rename its op).
    const RESERVED_FAILURE_VARIANT: &str = "Failed";
    if reply_variants.iter().any(|v| match v {
        EnumVariant::Tagged { name, .. } => name == RESERVED_FAILURE_VARIANT,
        EnumVariant::Unit(name) => name == RESERVED_FAILURE_VARIANT,
    }) {
        return Err(TypeError::new(
            decl_span.clone(),
            TypeErrorKind::MalformedVariant {
                enum_name: format!("{}::Reply", surface.name),
                offending: RESERVED_FAILURE_VARIANT.to_string(),
                reason: format!(
                    "surface {} declares an op that maps to the reserved reply variant \
                     `{}`; `Reply::{}` is reserved for the protocol-tier decode-failure floor \
                     (arc 278: a client message that never hydrates to any op is replied as \
                     `Reply::Failed[cause]` and surfaced as a raise). Rename the op.",
                    surface.name, RESERVED_FAILURE_VARIANT, RESERVED_FAILURE_VARIANT
                ),
                remedies: vec![],
            },
        ));
    }
    reply_variants.push(EnumVariant::Tagged {
        name: RESERVED_FAILURE_VARIANT.to_string(),
        fields: vec![(
            "cause".to_string(),
            TypeExpr::Path(":wat::kernel::Failure".into()),
        )],
    });

    // Protocol enums live under the surface's own namespace: `:S::Op` / `:S::Reply`
    // (`surface.name` keeps the leading colon, e.g. `:probe::Kv`).
    let op_name = format!("{}::Op", surface.name);
    let reply_name = format!("{}::Reply", surface.name);

    // STOP-COLLISION — never overwrite a user hand-declared protocol enum. If either name is
    // already registered, yield to the user's declaration (do not synthesize, do not error).
    if env.contains(&op_name) || env.contains(&reply_name) {
        return Ok(vec![]);
    }

    // Arc 278 — the PARAMETRIC PROTOCOL. `Op`/`Reply` inherit the surface's own type params.
    //
    // The variant fields are the surface members' request/response `TypeExpr`s VERBATIM, so a
    // parametric surface's messages (`(GetRequest :- [K V])`) put `K`/`V` in those field positions. Born
    // with `type_params: vec![]`, `K` was UNBOUND in the enum — the surface declared a parametric
    // protocol and the wire silently stripped it. The params bind here, at the one place that knows
    // both the surface's binders and the fields that reference them.
    //
    // THE IDENTITY PROPERTY (the whole floor rides on it): a surface with no type params has
    // `surface.type_params == []`, so this clone is byte-for-byte the old `vec![]` — every one of the
    // nine concrete defservices is untouched. (Verified: `--check --check-output edn` over the whole
    // `.wat` corpus is byte-identical across this change.)
    Ok(vec![
        TypeDef::Enum(EnumDef {
            name: op_name,
            type_params: surface.type_params.clone(),
            purity: Purity::Pure,
            variants: op_variants,
        }),
        TypeDef::Enum(EnumDef {
            name: reply_name,
            type_params: surface.type_params.clone(),
            purity: Purity::Pure,
            variants: reply_variants,
        }),
    ])
}

/// Arc 278 S4c — build the `<S>::surface-forms` carrier: a 0-arg `defn` returning a
/// `(Vector :- [WatAST])` of the peer surface's own forms (here, the whole post-expansion
/// `defsurface` form). `defservice` concats `(<S>::surface-forms)` into its shipped
/// `service-forms` bundle so a forked child re-registers the surface's protocol
/// (its `:messages` records + the synthesized `::Op`/`::Reply`) at a fresh startup.
///
/// Mirrors `wat/service.wat`'s `<fqdn>::service-forms` defn: a 0-arg fn the checker can
/// type at call sites, whose `(:wat::core::forms …)` body yields the forms as `(Vector :- [WatAST])`.
///
/// This is injected AFTER `expand_all` (in `register_types`), so it must be built in the
/// LOW-LEVEL `(:wat::core::def :name (:wat::core::fn [] -> :ret body))` shape that `defn`
/// expands to — `register_defines`/`try_parse_fn_shape_def` consume that directly, whereas the
/// `defn` macro form would never be expanded and would go unregistered.
fn build_surface_forms_carrier(surface_name: &str, surface_form: WatAST, span: Span) -> WatAST {
    use crate::scope::Identifier;
    let carrier_name = format!("{}::surface-forms", surface_name);
    let forms_body = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::forms".into(), span.clone()),
            surface_form,
        ],
        span.clone(),
    );
    // Arc 109 ③ — angle brackets are ILLEGAL for types; the flat
    // `:wat::core::Vector<wat::WatAST>` keyword is RETIRED in favour of the reference FORM
    // `(:wat::core::Vector :- [:wat::WatAST])`, built directly as a `WatAST::List` (this fn
    // constructs raw AST, never source text, so there is no string to re-spell).
    let vec_watast_ty = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::Vector".into(), span.clone()),
            WatAST::Keyword(":-".into(), span.clone()),
            WatAST::Vector(vec![WatAST::Keyword(":wat::WatAST".into(), span.clone())], span.clone()),
        ],
        span.clone(),
    );
    let fn_form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::fn".into(), span.clone()),
            WatAST::Vector(vec![], span.clone()),
            WatAST::Symbol(Identifier::bare("->"), span.clone()),
            vec_watast_ty,
            forms_body,
        ],
        span.clone(),
    );
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::def".into(), span.clone()),
            WatAST::Keyword(carrier_name, span.clone()),
            fn_form,
        ],
        span,
    )
}

/// Arc 278 #16.2 — build one `(:wat::core::def :<S>::<OP>-MAX-REQUEST-BYTES <n>)` `WatAST` per
/// serviceable op on `surface`, carrying each `SurfaceMember::Method`'s parsed
/// `:max-request-bytes` budget (16.0) as a runtime constant the serve-loop codegen
/// (`wat/service.wat`'s `serve-op-arms`) can reference by keyword to build its
/// measure+flag guard (see `build_surface_forms_carrier` just above — same shape, same
/// downstream channel: a runtime `def`, spliced into `rest` alongside the surface carrier).
/// Field members are skipped (no wire budget). `CONST_NAME = "<Surface>::<OP>-MAX-REQUEST-BYTES"`
/// (op name upper-cased), e.g. surface `:probe::Cap1`, op `do-op` → `:probe::Cap1::DO-OP-MAX-REQUEST-BYTES`.
fn build_op_budget_constants(surface: &SurfaceDef, span: &Span) -> Vec<WatAST> {
    surface
        .members
        .iter()
        .filter_map(|member| match member {
            SurfaceMember::Method { name, max_request_bytes, .. } => {
                // `surface.name` already carries the leading `:` sigil (matches every other
                // `WatAST::Keyword` string in this codebase) — do NOT prepend another.
                let const_name =
                    format!("{}::{}-MAX-REQUEST-BYTES", surface.name, name.to_uppercase());
                Some(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::def".into(), span.clone()),
                        WatAST::Keyword(const_name, span.clone()),
                        WatAST::IntLit(*max_request_bytes, span.clone()),
                    ],
                    span.clone(),
                ))
            }
            SurfaceMember::Field { .. } => None,
        })
        .collect()
}

/// Shared loop body for [`register_types`] and [`register_stdlib_types`].
/// Differs only in which `env` registration method is called — passed as
/// `register`. Non-type-decl forms are spliced via `splice` (handles
/// do/let recursion per Arc 170 slice 3 Gap J).
fn register_types_impl(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
    register: &dyn Fn(&mut TypeEnv, TypeDef, Span) -> Result<(), TypeError>,
    splice: &dyn Fn(WatAST, &mut TypeEnv) -> Result<WatAST, TypeError>,
    acronyms: &HashMap<String, Vec<String>>,
) -> Result<Vec<WatAST>, TypeError> {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        match classify_type_decl(&form) {
            Some(head) => {
                // Arc 138 slice 2 — capture decl span BEFORE the form
                // is consumed by `parse_type_decl`. Threaded through
                // every emission site for source-coordinate prefixes.
                let decl_span = form.span().clone();
                // Arc 278 S4c — a peer surface OWNS its `:messages` protocol forms and must SHIP
                // them across a process fork. Clone the raw defsurface form BEFORE it is consumed
                // so we can (a) register each message type-decl and (b) emit the `<S>::surface-forms`
                // carrier (a `(Vector :- [WatAST])` of the surface's own forms) that `defservice` concats
                // into its shipped `service-forms` bundle.
                let surface_form_clone = if head == "defsurface" { Some(form.clone()) } else { None };
                // Arc 170 — retain the ORIGINAL decl form (clone BEFORE `parse_type_decl`
                // consumes it) so freeze can ship it verbatim instead of reconstructing.
                // Generalizes `surface_form_clone` to every decl head. Stored only for
                // non-reserved user names, AFTER a successful registration (below).
                let form_clone = form.clone();
                let def = parse_type_decl(head, form, decl_span.clone(), env)?;
                // Arc 293 K3-revise — when a surface is registered, derive and register the
                // PAIR of backing aggregates (`:S$core-record`, `:S$holon-record`).
                // Field members only; methods are behavior. The `register` closure is re-used
                // so stdlib surfaces (if any) get the same privilege as the surface itself;
                // user surfaces go through the reserved-prefix gate automatically (their
                // `$core-record`/`$holon-record` names are in the same non-reserved namespace).
                // RETIRED 293 K3-revise: `:S$struct` — see retirement.rs.
                //
                // Arc 278 S4c — capture `(surface-name, is-peer)` to emit the `surface-forms`
                // carrier AFTER the surface + its messages register.
                let mut surface_carrier: Option<(String, WatAST)> = None;
                // Arc 278 #16.2 — the per-op `:max-request-bytes` budget constants (one per
                // serviceable method), emitted alongside the surface carrier below.
                let mut op_budget_consts: Vec<WatAST> = Vec::new();
                let derived = if let TypeDef::Surface(ref surf) = def {
                    // Arc 293 K3-revise — the backing record PAIR ($core-record / $holon-record).
                    let mut d = derive_surface_backing_records(surf);
                    // Arc 294 item 9a — a peer surface's `:messages` type-decls (`recordtype`/
                    // `defenum`, post-expansion) now register via the ONE ordinary top-level
                    // path: `expand_all`'s `hoist_surface_messages` (src/macros/expand.rs)
                    // already spliced each `:messages` child to the top-level form stream
                    // BEFORE this defsurface form, registering its kwargs-companion `defmacro`
                    // and letting it register HERE through the ordinary `classify_type_decl`/
                    // `parse_type_decl` arm above — `register_aggregate_methods` (freeze/env.rs
                    // step 6.8a) then mints its ctor + accessors for free. Registering them a
                    // second time here (the retired Way 2) would DuplicateDefine. Still build
                    // the carrier below: the child re-hoists `:messages` identically at its own
                    // fresh `expand_all` (wat/service.wat:1027-1130).
                    if surf.nature == Some(crate::types::Nature::Peer) {
                        if let Some(ref sform) = surface_form_clone {
                            // The carrier ships the whole (post-expansion) defsurface form; the child
                            // re-registers messages + re-synthesizes `::Op`/`::Reply` from it identically.
                            surface_carrier = Some((surf.name.clone(), build_surface_forms_carrier(&surf.name, sform.clone(), decl_span.clone())));
                        }
                        // Arc 278 #16.2 — one `<S>::<OP>-MAX-REQUEST-BYTES` runtime const per
                        // serviceable op, so `serve-op-arms` can reference the budget by keyword.
                        //
                        // Arc 278 #74 — the sibling `<S>::<OP>-RESPONSE-TYPE` runtime const
                        // (DESIGN-STONE-the-client-validates-locally.md) that used to ride this
                        // same channel is RETIRED: the builder ruled the response type's name
                        // into LAW (`<Op>Response`, enforced above by
                        // `synthesize_surface_protocol`'s own check), so `serve-op-arms` and
                        // `op-methods` no longer need to READ it at runtime — they build the
                        // ctor by concatenation again, now guaranteed correct by construction.
                        op_budget_consts = build_op_budget_constants(surf, &decl_span);
                    }
                    // Arc 293 S1 — the wire-protocol enums (`::Op` / `::Reply`) when the method
                    // sigs are pure. Same `register` closure → same privilege as the surface.
                    d.extend(synthesize_surface_protocol(surf, env, acronyms, &decl_span)?);
                    // Arc 278 — the surface-minted op alias (BRIEF-surface-minted-op-alias-
                    // stone.md, scout answer in BRIEF-surface-minted-op-alias-scout.md). Mint
                    // one `TypeDef::Alias` per op with a request arg, named
                    // `<Surface>::<op>/Request` / `<Surface>::<op>/Response`, targeting the
                    // request/response type EXACTLY as `:features` declared it (bare or
                    // parametric). `wat/service.wat` names these aliases instead of guessing a
                    // message's type name by concatenation — its only prior channel, since
                    // `expand_all` runs before `register_types` and the registry is empty at
                    // expand time. This is what retires the message-params lock immediately
                    // below: a message no longer needs to spell the surface's params to be
                    // nameable, because Rust — which DOES hold `:features` at registration time
                    // — mints the uniform alias name and the macro just names it.
                    for member in &surf.members {
                        if let SurfaceMember::Method { name: op_name, args, ret, .. } = member {
                            if let Some((_, request_ty)) = args.fixed_params.get(1) {
                                d.push(TypeDef::Alias(AliasDef {
                                    name: format!("{}::{}/Request", surf.name, op_name),
                                    type_params: surf.type_params.clone(),
                                    expr: request_ty.clone(),
                                }));
                                d.push(TypeDef::Alias(AliasDef {
                                    name: format!("{}::{}/Response", surf.name, op_name),
                                    type_params: surf.type_params.clone(),
                                    expr: ret.clone(),
                                }));
                            }
                        }
                    }
                    d
                } else {
                    vec![]
                };
                // Arc 170 — capture the user name BEFORE `def` is moved into `register`.
                let def_name = def.name().to_string();
                let is_user_type = !crate::resolve::is_reserved_prefix(&def_name);
                register(env, def, decl_span.clone())?;
                // Arc 170 — retain the original source form for user types only. Stdlib
                // (`:wat::*`) is re-registered in the child via `with_builtins`; synthesized
                // `derived` defs below are NOT captured (no user form → reconstruction fallback).
                if is_user_type {
                    env.source_forms.insert(def_name, form_clone);
                }
                for record_def in derived {
                    register(env, record_def, decl_span.clone())?;
                }
                // Arc 278 S4c — the `surface-forms` carrier is a runtime `defn`, not a type decl;
                // it flows downstream (register_defines) exactly like any user/stdlib fn.
                if let Some((_name, carrier)) = surface_carrier {
                    rest.push(carrier);
                }
                // Arc 278 #16.2 — the op budget consts are runtime `def`s too; same channel.
                rest.extend(op_budget_consts);
            }
            None => {
                let spliced = splice(form, env)?;
                rest.push(spliced);
            }
        }
    }
    Ok(rest)
}

/// Walk `forms`, register every type declaration, return the remaining
/// forms in order.
///
/// Arc 170 slice 3 Gap J — extends the top-level walk to recurse into
/// `(:wat::core::do ...)` and `(:wat::core::let ...)` body forms so type
/// declarations nested inside those spliced do/let blocks are registered in
/// the TypeEnv. Mirrors the splice-recursion pattern already used by
/// `preregister_fn_defs_in_do`/`_in_let` in `src/runtime.rs`.
pub fn register_types(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
) -> Result<Vec<WatAST>, TypeError> {
    register_types_with_acronyms(forms, env, &HashMap::new())
}

/// [`register_types`] threading the namespace-scoped acronym registry (from the
/// macro-expansion `SymbolTable`, populated by `preregister_acronyms`). The registry
/// lets a surface's S1 protocol synthesis (`synthesize_surface_protocol`) restore
/// acronym casing on `::Op`/`::Reply` variant names EXACTLY as `defservice :impls`
/// does at expand time — so `:satisfies` and `:impls` never diverge (e.g. both emit
/// `CreateWebACL`, never one `CreateWebAcl`). The production startup path
/// (`freeze::env`) passes `macro_sym.acronym_registry`; callers with no acronyms use
/// the empty-registry [`register_types`] wrapper.
pub fn register_types_with_acronyms(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
    acronyms: &HashMap<String, Vec<String>>,
) -> Result<Vec<WatAST>, TypeError> {
    register_types_impl(
        forms,
        env,
        &|env, def, span| env.register_with_span(def, span),
        &splice_type_decls_user,
        acronyms,
    )
}

/// Stdlib-registration variant of [`register_types`] that bypasses the
/// `:wat::*` reserved-prefix gate. Called by the startup pipeline on
/// the baked stdlib sources so stdlib wat files can declare types
/// (typealiases, structs, enums, newtypes) under `:wat::std::*`.
/// Mirrors [`crate::macros::register_stdlib_defmacros`]'s privileged
/// path.
///
/// Arc 170 slice 3 Gap J — extended to recurse into top-level do/let
/// body forms, mirroring the user-source variant.
pub fn register_stdlib_types(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
) -> Result<Vec<WatAST>, TypeError> {
    // Stdlib surfaces declare no user acronyms (`preregister_acronyms` covers the USER
    // residue only), so the stdlib path registers against an empty registry.
    register_types_impl(
        forms,
        env,
        &|env, def, span| env.register_stdlib_with_span(def, span),
        &splice_type_decls_stdlib,
        &HashMap::new(),
    )
}

/// Arc 170 slice 3 Gap J — recurse into a top-level `do` or `let` form,
/// registering any type declarations found in the body and returning the
/// reconstructed form with type decls stripped.
///
/// Non-do/non-let forms are returned unchanged. For do/let forms, the
/// keyword (and for let, the bindings vector) is preserved; type-decl body
/// children are registered and stripped; remaining body children are kept.
/// Nested do/let forms are handled recursively (do-within-do nesting works
/// naturally via the recursive call).
///
/// Mirrors the splice-recursion pattern in `preregister_fn_defs_in_do`
/// (runtime.rs).
fn splice_type_decls(
    form: WatAST,
    env: &mut TypeEnv,
    register: &dyn Fn(&mut TypeEnv, TypeDef, Span) -> Result<(), TypeError>,
) -> Result<WatAST, TypeError> {
    let (items, span) = match form {
        WatAST::List(items, span) => (items, span),
        other => return Ok(other),
    };
    let head_kw = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(WatAST::List(items, span)),
    };
    match head_kw {
        ":wat::core::do" => {
            let mut new_children = Vec::with_capacity(items.len());
            let mut iter = items.into_iter();
            new_children.push(iter.next().expect("do has keyword"));
            for child in iter {
                match classify_type_decl(&child) {
                    Some(head) => {
                        let decl_span = child.span().clone();
                        let def = parse_type_decl(head, child, decl_span.clone(), env)?;
                        register(env, def, decl_span)?;
                    }
                    None => {
                        new_children.push(splice_type_decls(child, env, register)?);
                    }
                }
            }
            Ok(WatAST::List(new_children, span))
        }
        ":wat::core::let" => {
            let mut new_children = Vec::with_capacity(items.len());
            let mut iter = items.into_iter();
            new_children.push(iter.next().expect("let has keyword"));
            if let Some(bindings) = iter.next() {
                new_children.push(bindings);
            }
            for child in iter {
                match classify_type_decl(&child) {
                    Some(head) => {
                        let decl_span = child.span().clone();
                        let def = parse_type_decl(head, child, decl_span.clone(), env)?;
                        register(env, def, decl_span)?;
                    }
                    None => {
                        new_children.push(splice_type_decls(child, env, register)?);
                    }
                }
            }
            Ok(WatAST::List(new_children, span))
        }
        // Arc 237 follow-on — register the typesub edge Child→Parent from a `derive` form and
        // KEEP the form (downstream passes — infer_list check arm + runtime eval arm — still
        // see it). The form shape is `(:wat::core::derive :Child :Parent)`.
        // Mirrors the extend-type arm immediately below: same register_subtype call, same
        // pre-check point so assignable sees the edge; cycle check surfaces as CyclicSubtype.
        ":wat::core::derive" => {
            let decl_span = span.clone();
            let child = match items.get(1) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => {
                    return Err(TypeError::new(
                        decl_span,
                        TypeErrorKind::MalformedDecl {
                            head: "derive".into(),
                            reason: "expected keyword child type name at position 1".into(),
                        },
                    ))
                }
            };
            let parent = match items.get(2) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => {
                    return Err(TypeError::new(
                        decl_span,
                        TypeErrorKind::MalformedDecl {
                            head: "derive".into(),
                            reason: "expected keyword parent type name at position 2".into(),
                        },
                    ))
                }
            };
            env.register_subtype(&child, &parent, decl_span)?;
            Ok(WatAST::List(items, span))
        }
        // Arc 232.2 — register the subtype edge `T → P` from an `extend-type` form and KEEP the
        // form (do NOT strip it — downstream passes, 232.1 CheckEnv + runtime, still need it).
        // The form shape is `(:wat::core::extend-type :T :P (impl…)…)`.
        ":wat::core::extend-type" => {
            let decl_span = span.clone();
            // Arc 109 identity 2c remainder — the TARGET slot also accepts a parametric-type
            // FORM (`(Head :- [args])`) alongside the bare (non-parametric) Keyword surface —
            // angle brackets can never reach here at all: the lexer refuses `<` inside a
            // keyword token outright (arc 109 "annihilate the angle bracket", wat-reader's
            // lexer.rs), so a parametric target has exactly one live spelling, the FORM. Unlike
            // the SATISFIED-SURFACE arm just below, this one does NOT reduce to `base_fqdn()`:
            // `type_name` feeds `register_subtype`'s CHILD side, which `is_subtype` (below)
            // walks with EXACT-string semantics, and `transport_edge_keys`/
            // `transport_satisfier_heads` (check.rs) guess at the FULL `(Head :- [T])`/`(Head :- [Wire])`
            // spelling verbatim — dropping args here would starve both. `check::format_type`
            // is the substrate's ONE authoritative TypeExpr renderer (types.rs:1987), so
            // re-render through it rather than hand-rolling a second stringifier.
            let type_name = match items.get(1) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                Some(node @ WatAST::List(_, _)) => {
                    crate::check::format_type(&parse_type_node(node)?)
                }
                _ => {
                    return Err(TypeError::new(
                        decl_span,
                        TypeErrorKind::MalformedDecl {
                            head: "extend-type".into(),
                            reason: "expected keyword or type form type name at position 1".into(),
                        },
                    ))
                }
            };
            // Arc 109 stone 1 — the protocol slot also accepts a parametric-type FORM
            // (`(:Proto :- [T])`, ②-iii's eventual spelling) alongside the bare (non-parametric)
            // Keyword surface — the lexer refuses `<` inside a keyword token outright, so a
            // parametric protocol has exactly one live spelling, the FORM. Both the bare Keyword
            // and the FORM go through the SAME two existing doors — `parse_type_node`
            // (already parses `:-`-marked forms into `TypeExpr::Parametric`, no reader change
            // needed) then `TypeExpr::base_fqdn()` — so the lattice's own extraction stays
            // singular; this does not add a second hand-rolled `find('<')`.
            // ⛔ CORRECTED — this arm was added under ruling A-i ("the lattice keys on the BASE
            // NAME") and survived the revert to S2 ("`is_subtype` keeps EXACT-string semantics")
            // because flight 2's brief called it "orthogonal to S2". It was not: it left ONE site
            // keying on the base while `register_subtype` stores VERBATIM, so
            // `(extend-type :A :Proto<S,R>)` registered `":Proto<S,R>"` while
            // `(extend-type :A (:Proto :- [S R]))` registered `":Proto"` — two spellings of one
            // declaration, two different keys, and `is_subtype`'s exact-string query for the full
            // name never found the second. Floor-green only because nothing fed a genuinely
            // parametric protocol through the FORM spelling until `dialable-ty` would have.
            // Renders the FULL name, exactly as the TARGET arm above now does.
            let protocol_name = match items.get(2) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                Some(node @ WatAST::List(_, _)) => {
                    crate::check::format_type(&parse_type_node(node)?)
                }
                _ => {
                    return Err(TypeError::new(
                        decl_span,
                        TypeErrorKind::MalformedDecl {
                            head: "extend-type".into(),
                            reason: "expected keyword or type form protocol name at position 2"
                                .into(),
                        },
                    ))
                }
            };
            env.register_subtype(&type_name, &protocol_name, decl_span)?;
            Ok(WatAST::List(items, span))
        }
        _ => Ok(WatAST::List(items, span)),
    }
}

fn splice_type_decls_user(form: WatAST, env: &mut TypeEnv) -> Result<WatAST, TypeError> {
    splice_type_decls(form, env, &|env, def, span| env.register_with_span(def, span))
}

fn splice_type_decls_stdlib(form: WatAST, env: &mut TypeEnv) -> Result<WatAST, TypeError> {
    splice_type_decls(form, env, &|env, def, span| env.register_stdlib_with_span(def, span))
}

fn classify_type_decl(form: &WatAST) -> Option<&'static str> {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            match k.as_str() {
                // Stone 241.8 — defstruct replaces struct + struct-restricted (HARD CUT).
                ":wat::core::defstruct" => return Some("defstruct"),
                // Arc 293.2-parity — structtype is the low-level primitive defstruct (now a macro) expands to.
                ":wat::core::structtype" => return Some("structtype"),
                // Stone 241.9 — defenum replaces enum (HARD CUT).
                ":wat::core::defenum" => return Some("defenum"),
                ":wat::core::newtype" => return Some("newtype"),
                ":wat::core::typealias" => return Some("typealias"),
                // Stone 237.1 — named bounded set of types.
                ":wat::core::typeunion" => return Some("typeunion"),
                // Stone S-B.1 — record class as a real TypeDef.
                ":wat::core::recordtype" => return Some("recordtype"),
                // Arc 293 decl-a — ONE type-reg primitive; nature derived from parent root.
                ":wat::core::aggregatetype" => return Some("aggregatetype"),
                // Arc 293.3-core — structural surface.
                ":wat::core::defsurface" => return Some("defsurface"),
                _ => {}
            }
        }
    }
    None
}

/// Arc 109 (DESIGN-STONE-a-param-spec-must-be-consumed) — every entry in a `TypeDef`'s
/// `type_params` must be reachable from at least one member `TypeExpr` the def carries.
///
/// Runs ONCE, over the fully-built `TypeDef`, rather than being threaded into each of the
/// seven declarator parsers — all six variants carry `type_params`, so one check here covers
/// every declaration head `parse_type_decl` dispatches to. Empty `type_params` is a no-op
/// (monomorphic declarations, and every non-parametric declaration, are untouched).
///
/// Member-type reachability per variant (see `TypeDef`'s six variants above):
/// - `Aggregate` — every field's type.
/// - `Enum` — every `Tagged` variant's field types (`Unit` variants carry none).
/// - `Newtype` — the inner type.
/// - `Alias` — the body expression.
/// - `Union` — every member type.
/// - `Surface` — every `Field` member's type, AND every `Method` member's fixed-param types,
///   rest-param type (if any), and return type. The design/brief docs shorthand this variant's
///   row as "surface fields", but the corpus's own parametric surfaces ((Seqable :- [T]),
///   (Dialable :- [S R]), (TypedCapability :- [S R]), (Holds :- [T]), (Cache :- [K V]), (Pair :- [A B]), …) declare
///   ZERO plain `Field` members — every one of them consumes its type params exclusively
///   through `:features` method signatures (typically the `self <- (Name :- [T,...])` restatement,
///   sometimes the return type, e.g. `(Holds :- [T])`'s `get [self] -> :T`). A check that read only
///   `Field` members would reject all of them; walking `Method` args + ret is required for the
///   wall to be sound against the surface declarations that already exist.
///
/// Consumption itself walks NESTED type expressions — delegated to
/// `crate::declare::typevar::collect_free_type_vars_in`, which already recurses through `Parametric`,
/// `Fn`, and `Tuple` (stone 251.8a's single door; this reuses it rather than re-walking).
fn check_type_params_consumed(def: &TypeDef, decl_span: &Span) -> Result<(), TypeError> {
    let type_params: &[String] = match def {
        TypeDef::Aggregate(a) => &a.type_params,
        TypeDef::Enum(e) => &e.type_params,
        TypeDef::Newtype(n) => &n.type_params,
        TypeDef::Alias(a) => &a.type_params,
        TypeDef::Union(u) => &u.type_params,
        TypeDef::Surface(s) => &s.type_params,
    };
    if type_params.is_empty() {
        return Ok(());
    }

    let member_types: Vec<TypeExpr> = match def {
        TypeDef::Aggregate(a) => a.fields.iter().map(|(_, t)| t.clone()).collect(),
        TypeDef::Enum(e) => e
            .variants
            .iter()
            .flat_map(|v| match v {
                EnumVariant::Unit(_) => Vec::new(),
                EnumVariant::Tagged { fields, .. } => {
                    fields.iter().map(|(_, t)| t.clone()).collect()
                }
            })
            .collect(),
        TypeDef::Newtype(n) => vec![n.inner.clone()],
        TypeDef::Alias(a) => vec![a.expr.clone()],
        TypeDef::Union(u) => u.members.clone(),
        TypeDef::Surface(s) => s
            .members
            .iter()
            .flat_map(|m| match m {
                SurfaceMember::Field { ty, .. } => vec![ty.clone()],
                SurfaceMember::Method { args, ret, .. } => {
                    let mut v: Vec<TypeExpr> =
                        args.fixed_params.iter().map(|(_, t)| t.clone()).collect();
                    if let Some((_, t)) = &args.rest_param {
                        v.push(t.clone());
                    }
                    v.push(ret.clone());
                    v
                }
            })
            .collect(),
    };

    let consumed = crate::declare::typevar::collect_free_type_vars_in(&member_types);
    for p in type_params {
        if !consumed.contains(p) {
            return Err(TypeError::new(
                decl_span.clone(),
                TypeErrorKind::UnconsumedTypeParam {
                    decl: def.name().to_string(),
                    param: p.clone(),
                },
            ));
        }
    }
    Ok(())
}

fn parse_type_decl(
    head: &str,
    form: WatAST,
    decl_span: Span,
    env: &TypeEnv,
) -> Result<TypeDef, TypeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError::new(
                decl_span,
                TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: "expected list form".into(),
                },
            ))
        }
    };
    let mut iter = items.into_iter();
    let _head_kw = iter.next();
    let def = match head {
        // Stone 241.8 — defstruct replaces struct + struct-restricted (HARD CUT).
        "defstruct" => parse_defstruct(iter.collect(), decl_span.clone(), env),
        // Arc 293.2-parity — structtype: thin alias → parse_aggregate with injected :wat::core::Struct parent.
        "structtype" => parse_structtype(iter.collect(), decl_span.clone(), env),
        // Stone 241.9 — defenum replaces enum (HARD CUT).
        "defenum" => parse_defenum(iter.collect(), decl_span.clone()),
        "newtype" => parse_newtype(iter.collect(), decl_span.clone()),
        "typealias" => parse_typealias(iter.collect(), decl_span.clone()),
        // Stone 237.1 — named bounded set of types.
        "typeunion" => parse_typeunion(iter.collect(), decl_span.clone()),
        // Stone S-B.1 — record class as a real TypeDef; thin alias → parse_aggregate.
        "recordtype" => parse_aggregate(iter.collect(), decl_span.clone(), "recordtype", env),
        // Arc 293 decl-a — ONE type-reg primitive; nature derived from parent root.
        "aggregatetype" => parse_aggregate(iter.collect(), decl_span.clone(), "aggregatetype", env),
        // Arc 293.3-core — structural surface.
        "defsurface" => parse_defsurface(iter.collect(), decl_span.clone()),
        _ => unreachable!(),
    }?;
    // Arc 109 (param-spec-must-be-consumed) — ONE check, here, after every declarator has
    // returned its built TypeDef, rather than seven checks threaded into seven parsers.
    check_type_params_consumed(&def, &decl_span)?;
    Ok(def)
}


/// Stone 241.9 — parse a `(:wat::core::defenum :Name :V1 :V2 [f <- :T ...] ...)` declaration.
///
/// Positional variant grammar with one-token look-ahead (FORM-COLLAPSE verdict D):
///   args[0]      — name keyword (e.g. `:my::ns::Status`)
///   args[1]      — OPTIONAL metadata-map `{...}` (WatAST::List with head
///                  `:wat::core::HashMap`); detected by structural discriminator.
///   args[1..] or args[2..] — positional variants
///
/// Variant discrimination (one-token look-ahead):
///   See `:VariantName` keyword → variant name; peek next item:
///   - Next is keyword (or end-of-args) → UNIT variant; push `EnumVariant::Unit(name)`.
///   - Next is Vector `[...]` → TAGGED variant; consume Vector via `parse_argspec_triples`.
///
/// Metadata keys recognized (under `:variant-metadata`):
///   `:variant-metadata {keyword → metadata-map}`  — per-variant metadata (D5: silent generic storage)
///
/// Empty `{}` metadata-map REJECTED (FORM-COLLAPSE D4 / Stone 241.6 doctrine).
/// Empty variant list REJECTED (≥1 variant required).
/// HARD CUT: no `parse_enum` shim; no `:wat::core::enum` compatibility.
#[wat_special_form_impl(":wat::core::defenum", role = declare)]
fn parse_defenum(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    const HEAD: &str = ":wat::core::defenum";

    // Need at least: name + one variant (2 args minimum).
    if args.len() < 2 {
        return Err(TypeError::new(
            decl_span,
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defenum :Name :V1 ...) with at least one variant; got {} args after head",
                    args.len()
                ),
            },
        ));
    }

    let mut iter = args.into_iter().peekable();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, name_params) = parse_declared_name(HEAD, &name_kw, &decl_span)?;
    let type_params = take_declared_binder(HEAD, name_params, name_kw.span(), &mut iter)?;

    // Slot 1 — MANDATORY purity marker (arc 293.W.2b): the enum DECLARES whether its values are pure
    // (hold only data, fully EDN-reconstructable anywhere) or impure (hold live resources, bound to
    // their locus). One of `:wat::enum::Pure` | `:wat::enum::Impure`, positional, immediately after
    // the name. No default — a default would mask intent (the surface-`:nature`-mandatory rule).
    // Being namespaced, it is unmistakable from the bare Capitalized variant keywords that follow.
    let purity = match iter.next() {
        Some(WatAST::Keyword(k, _)) if Purity::from_marker_keyword(&k).is_some() => {
            Purity::from_marker_keyword(&k).unwrap()
        }
        other => {
            return Err(TypeError::new(
                other.as_ref().map(|n| n.span().clone()).unwrap_or_else(|| decl_span.clone()),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "defenum requires a mandatory purity marker immediately after the name — \
                         one of :wat::enum::Pure (values hold only data; serialize to EDN; cross \
                         address spaces) | :wat::enum::Impure (values may hold live resources; \
                         never cross); got {}",
                        other.map(|n| format!("{:?}", n)).unwrap_or_else(|| "end of form".into()),
                    ),
                },
            ));
        }
    };

    // Collect remaining args for metadata + variants.
    let remaining: Vec<WatAST> = iter.collect();

    // Discriminate: does args[1] look like a metadata-map?
    // Arc 257 slice 1: is_metadata_map() accepts WatAST::Map and legacy HashMap List.
    let is_metadata = remaining.first().map(|n| n.is_metadata_map()).unwrap_or(false);
    let (metadata_node_opt, variant_args): (Option<WatAST>, Vec<WatAST>) = if is_metadata {
        let mut it = remaining.into_iter();
        let meta = it.next().unwrap();
        (Some(meta), it.collect())
    } else {
        (None, remaining)
    };

    // Parse optional metadata-map (D5: silently store; no EnumDef schema extension).
    // We validate the structure but don't extend EnumDef with per-variant metadata.
    if let Some(ref meta_node) = metadata_node_opt {
        // Arc 257 slice 1: use metadata_map_pairs() to handle both Map and legacy List.
        let pairs = meta_node.metadata_map_pairs().ok_or_else(|| TypeError::new(
            meta_node.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "malformed metadata-map (internal structure corrupt)".into(),
            },
        ))?;
        // Empty {} → pairs.len() == 0 → REJECTED (FORM-COLLAPSE D4).
        if pairs.is_empty() {
            return Err(TypeError::new(
                meta_node.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "empty `{}` metadata-map is illegal (use no metadata-map arg for plain defenum)".into(),
                },
            ));
        }
        // Walk key/value pairs — silently accept :variant-metadata + unknown keys (D5).
        for (k_node, _) in &pairs {
            match k_node {
                WatAST::Keyword(_k, _) => {
                    // Key recognized; value already extracted.
                    // :variant-metadata inner keys must be keywords (T5 trap-door).
                    // Silently store for this stone (D5 — no consumer-driven semantic yet).
                    // Unknown keys also silently accepted (D5).
                }
                other => {
                    return Err(TypeError::new(
                        other.span().clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: "metadata-map keys must be keywords".into(),
                        },
                    ));
                }
            }
        }
    }

    // Parse variants: positional with one-token look-ahead.
    // variant_args are the post-metadata args (may be empty if only metadata was given).
    let mut variants: Vec<EnumVariant> = Vec::new();
    let mut vi = 0;
    while vi < variant_args.len() {
        let item = &variant_args[vi];
        match item {
            WatAST::Keyword(k, _) => {
                let variant_name = k.strip_prefix(':').ok_or_else(|| TypeError::new(
                    item.span().clone(),
                    TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: format!("{:?}", k),
                        reason: "defenum variant must be a keyword starting with ':'".to_string(),
                        remedies: vec![],
                    },
                ))?.to_string();

                // One-token look-ahead: peek at the NEXT item.
                let next = variant_args.get(vi + 1);
                match next {
                    // Next is a Vector → TAGGED variant; consume the Vector as argspec.
                    Some(WatAST::Vector(vec_items, vec_span)) => {
                        let argspec = crate::argspec::parse_argspec_triples(
                            vec_items,
                            HEAD,
                            vec_span,
                            crate::argspec::ParseOptions { allow_rest_binder: false },
                        )
                        .map_err(TypeError::from)?;
                        let fields: Vec<(String, crate::types::TypeExpr)> = argspec.fixed_params.into_iter().map(|(id, ty)| (id.as_str().to_owned(), ty)).collect();
                        variants.push(EnumVariant::Tagged { name: variant_name, fields });
                        vi += 2; // consume keyword + vector
                    }
                    // Next is a keyword (or end-of-args) → UNIT variant.
                    _ => {
                        variants.push(EnumVariant::Unit(variant_name));
                        vi += 1; // consume keyword only
                    }
                }
            }
            WatAST::Symbol(ident, _) => {
                // Bare symbol where a keyword is expected: offer "write it as :<name>" remedy.
                let needle = format!(":{}", ident.as_str());
                return Err(TypeError::new(
                    item.span().clone(),
                    TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: ident.as_str().to_owned(),
                        reason: format!(
                            "defenum variant must be a keyword; got bare symbol '{}' — write it as the keyword '{}'",
                            ident.as_str(), needle,
                        ),
                        remedies: vec![],
                    },
                ));
            }
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: format!("{:?}", other),
                        reason: "defenum variant must be a keyword (unit) or keyword followed by Vector (tagged)".to_string(),
                        remedies: vec![],
                    },
                ));
            }
        }
    }

    if variants.is_empty() {
        return Err(TypeError::new(
            decl_span,
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "defenum must have at least one variant".into(),
            },
        ));
    }

    Ok(TypeDef::Enum(EnumDef {
        name,
        type_params,
        purity,
        variants,
    }))
}

#[wat_special_form_impl(":wat::core::newtype", role = declare)]
fn parse_newtype(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // Arc 109 binder strike α — the exact-2 gate can no longer fire on the raw
    // `args.len()` upfront: a binder-bearing form is 4 args (name, `:-`, `[T]`,
    // inner) wide before any of it is consumed. Count the raw len for the
    // diagnostic text (unchanged for the no-binder case), but gate on what
    // remains AFTER the name + binder are peeled off.
    let arg_count = args.len();
    let arity_err = |sp: Span| {
        TypeError::new(
            sp,
            TypeErrorKind::MalformedDecl {
                head: "newtype".into(),
                reason: format!(
                    "expected (:wat::core::newtype :name :InnerType); got {} args",
                    arg_count
                ),
            },
        )
    };
    let mut iter = args.into_iter().peekable();
    let name_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    let (name, name_params) = parse_declared_name("newtype", &name_kw, &decl_span)?;
    let type_params = take_declared_binder("newtype", name_params, name_kw.span(), &mut iter)?;
    let inner_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    if iter.peek().is_some() {
        return Err(arity_err(decl_span));
    }
    // Arc 251.3a — accept Keyword, Symbol (wat.type/X), or List (parametric form).
    let inner = match &inner_kw {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(&inner_kw)?
        }
        other => {
            return Err(TypeError::new(
                decl_span,
                TypeErrorKind::MalformedDecl {
                    head: "newtype".into(),
                    reason: format!(
                        "inner type must be a keyword or type form; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    Ok(TypeDef::Newtype(NewtypeDef {
        name,
        type_params,
        inner,
    }))
}

#[wat_special_form_impl(":wat::core::typealias", role = declare)]
fn parse_typealias(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // Arc 109 binder strike α — see parse_newtype's comment: the exact-2 gate
    // moves from the raw `args.len()` (a binder widens it to 4) to what
    // remains after name + binder are peeled off.
    let arg_count = args.len();
    let arity_err = |sp: Span| {
        TypeError::new(
            sp,
            TypeErrorKind::MalformedDecl {
                head: "typealias".into(),
                reason: format!(
                    "expected (:wat::core::typealias :name :Expr); got {} args",
                    arg_count
                ),
            },
        )
    };
    let mut iter = args.into_iter().peekable();
    let name_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    let (name, name_params) = parse_declared_name("typealias", &name_kw, &decl_span)?;
    let type_params = take_declared_binder("typealias", name_params, name_kw.span(), &mut iter)?;
    let expr_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    if iter.peek().is_some() {
        return Err(arity_err(decl_span));
    }
    // Arc 251.3a — accept Keyword, Symbol (wat.type/X), or List (parametric form).
    let expr = match &expr_kw {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(&expr_kw)?
        }
        other => {
            return Err(TypeError::new(
                decl_span,
                TypeErrorKind::MalformedDecl {
                    head: "typealias".into(),
                    reason: format!(
                        "alias expression must be a keyword or type form; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    Ok(TypeDef::Alias(AliasDef {
        name,
        type_params,
        expr,
    }))
}

/// Stone 237.1 — parse `(:wat::core::typeunion :Name [:T1 :T2 ...])`.
///
/// Two positional slots after the head keyword (consumed by `parse_type_decl`):
///   args[0] — name keyword (e.g. `:my::Numeric`)
///   args[1] — members Vector `[...]` of type-expression keywords
///
/// The Vector literal signals "data/collection" per `feedback_clojure_not_scheme`.
/// Empty Vector → `EmptyUnion`; single-element → `SingleMemberUnion`; member
/// shape validation occurs at registration time (in `validate_union_members`).
fn parse_typeunion(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // Arc 109 binder strike α — see parse_newtype's comment: the exact-2 gate
    // moves from the raw `args.len()` (a binder widens it to 4) to what
    // remains after name + binder are peeled off.
    let arg_count = args.len();
    let arity_err = |sp: Span| {
        TypeError::new(
            sp,
            TypeErrorKind::MalformedDecl {
                head: "typeunion".into(),
                reason: format!(
                    "expected (:wat::core::typeunion :Name [:T1 :T2 ...]); got {} args",
                    arg_count
                ),
            },
        )
    };
    let mut iter = args.into_iter().peekable();
    let name_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    let (name, name_params) = parse_declared_name("typeunion", &name_kw, &decl_span)?;
    let type_params = take_declared_binder("typeunion", name_params, name_kw.span(), &mut iter)?;
    let members_ast = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;
    if iter.peek().is_some() {
        return Err(arity_err(decl_span));
    }
    let member_items = match members_ast {
        WatAST::Vector(items, _) => items,
        other => {
            return Err(TypeError::new(
                decl_span,
                TypeErrorKind::MalformedDecl {
                    head: "typeunion".into(),
                    reason: format!(
                        "member list must be a Vector `[...]`; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    let mut members = Vec::with_capacity(member_items.len());
    for item in member_items {
        let item_span = item.span().clone();
        // Arc 251.3a — accept Keyword, Symbol (wat.type/X), or List (parametric form).
        match &item {
            WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
                members.push(parse_type_node(&item)?);
            }
            other => {
                return Err(TypeError::new(
                    item_span,
                    TypeErrorKind::MalformedDecl {
                        head: "typeunion".into(),
                        reason: format!(
                            "member must be a type keyword or type form; got {}",
                            other.variant_name()
                        ),
                    },
                ))
            }
        }
    }
    Ok(TypeDef::Union(UnionDef {
        name,
        type_params,
        members,
    }))
}


/// Arc 293 decl-a — thin alias for `structtype` dispatch.
///
/// `structtype` args (from `parse_type_decl`): `[name_kw, {meta_node}?, fields_node]` (2 or 3 items).
/// Injects `:wat::core::Struct` as `parent` at position [1] and delegates to `parse_aggregate`.
#[wat_special_form_impl(":wat::core::structtype", role = declare)]
fn parse_structtype(args: Vec<WatAST>, decl_span: Span, env: &TypeEnv) -> Result<TypeDef, TypeError> {
    let mut new_args = Vec::with_capacity(args.len() + 1);
    let mut iter = args.into_iter().peekable();
    // name kw at [0] stays first.
    if let Some(name_kw) = iter.next() {
        new_args.push(name_kw);
    }
    // Arc 109 binder strike α — CARRY a `:- [T…]` binder across the injection.
    // The binder's contract is "immediately after the name"; injecting the parent
    // ahead of it would displace it, and `parse_aggregate` would then swallow the
    // `:-` into its trailing-arity error instead of reading it. Measured before the
    // fix: `(:wat::core::structtype :S :- [T] [f :- T])` and the `defstruct` macro
    // that lowers into this head both died on "expected (:structtype :Name :Parent
    // [fields]); got 5 args" while their `<T>`-spelled twins passed.
    if iter.peek().is_some_and(is_binder_marker) {
        new_args.push(iter.next().unwrap());
        if let Some(vec_node) = iter.next() {
            new_args.push(vec_node);
        }
    }
    // Inject :wat::core::Struct as the parent, AFTER the name (and its binder, if any).
    new_args.push(WatAST::Keyword(":wat::core::Struct".to_string(), crate::rust_caller_span!()));
    // Remaining args (optional metadata + fields).
    new_args.extend(iter);
    parse_aggregate(new_args, decl_span, "structtype", env)
}

/// Arc 293 decl-a — ONE parse fn for ALL aggregate type declarations.
///
/// Three-or-four positional slots after the head keyword (consumed by `parse_type_decl`):
///   args[0]       — name keyword (e.g. `:my::Circle`)
///   args[1]       — parent type keyword (e.g. `:wat::core::Struct`, `:wat::core::Record`)
///   args[2..N-1]  — optional metadata-map `{...}` (WatAST with `is_metadata_map() == true`)
///   args[last]    — field-vector `[field <- :T ...]` (WatAST::Vector)
///
/// nature = `root_nature_of(parent)`:
///   `:wat::core::Struct`    → `Nature::Struct`
///   `:wat::core::Record`          → `Nature::Record`
///   `:wat::holon::Record`   → `Nature::HolonRecord`
///   (non-root record base)  → `Nature::Record`
///
/// Parent validity (parent must be registered before this type) is enforced at registration
/// time in `register_with_span` — identical to the existing `recordtype` check.
///
/// Metadata (restrictions) is optional for ANY nature (GAP-5 capability built here, exposed
/// in decl-b). Field parser is `defstruct::parse_aggregate_fields` (via `parse_argspec_triples`).
///
/// `head` is the caller-supplied surface form name used in error messages ("aggregatetype",
/// "structtype", "recordtype") — preserves existing error text for each alias.
fn parse_aggregate(args: Vec<WatAST>, decl_span: Span, head: &'static str, env: &TypeEnv) -> Result<TypeDef, TypeError> {
    // Arc 109 binder strike α — the 3..=4 gate can no longer fire on the raw
    // `args.len()` upfront: a binder-bearing form widens by 2 (name, `:-`,
    // `[T]`, parent, [meta], fields) before any of it is consumed. Count the
    // raw len for the diagnostic text (unchanged for the no-binder case), but
    // gate on what remains after name + binder + parent are peeled off.
    let arg_count = args.len();
    let arity_err = |sp: Span| {
        TypeError::new(
            sp,
            TypeErrorKind::MalformedDecl {
                head: head.into(),
                reason: format!(
                    "expected (:{} :Name :Parent [fields]) or with optional metadata-map; got {} args",
                    head, arg_count
                ),
            },
        )
    };
    let mut iter = args.into_iter().peekable();
    let name_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;

    let (name, name_params) = parse_declared_name(head, &name_kw, &decl_span)?;
    let type_params = take_declared_binder(head, name_params, name_kw.span(), &mut iter)?;

    let parent_kw = iter.next().ok_or_else(|| arity_err(decl_span.clone()))?;

    let parent = match &parent_kw {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(TypeError::new(
                decl_span,
                TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: format!(
                        "parent must be a type keyword; got {}",
                        other.variant_name()
                    ),
                },
            ));
        }
    };

    // Arc 293 inheritance annihilation — reject any parent that is not a nature-root.
    // Reuse-of-shape is surface-splice (`[~@:Surface own <- :T]`), not nominal inheritance.
    let nature = Nature::from_root_keyword(&parent).ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: head.into(),
            reason: format!(
                "parent '{}' is not a nature-root; inheritance is unsupported — reuse a shape via surface-splice `[~@:Surface \u{2026}]`",
                parent
            ),
        },
    ))?;

    // Discriminate: 1 remaining arg (just fields) vs 2 remaining (metadata + fields).
    if iter.len() == 0 || iter.len() > 2 {
        return Err(arity_err(decl_span));
    }
    let (metadata_node_opt, fields_node) = if iter.len() == 1 {
        (None, iter.next().unwrap())
    } else {
        let meta_node = iter.next().unwrap();
        let fields_node = iter.next().unwrap();
        (Some(meta_node), fields_node)
    };

    // Parse optional metadata-map (struct restrictions; GAP-5 capability available to any nature).
    let (ctor_whitelist, field_restrictions) = if let Some(meta_node) = metadata_node_opt {
        defstruct::parse_defstruct_metadata(meta_node)?
    } else {
        (Vec::new(), std::collections::HashMap::new())
    };

    // Parse field-vector via the ONE canonical field parser (splice-aware — Arc 293).
    let fields = defstruct::parse_aggregate_fields_with_splices(fields_node, head, env)?;

    let restrictions = if ctor_whitelist.is_empty() && field_restrictions.is_empty() {
        None
    } else {
        Some(StructRestrictions { ctor_whitelist, field_restrictions })
    };

    Ok(TypeDef::Aggregate(AggregateDef { name, type_params, fields, nature, restrictions }))
}

// Arc 293 decl-a — `parse_recordtype` ABSORBED into `parse_aggregate` (arc 293 decl-a).
// The dispatch arm "recordtype" calls `parse_aggregate(args, decl_span, "recordtype")` directly.
// Retirement: the old inline groups-of-3 field-parser differed from parse_argspec_triples on
// (a) arrow: inline only accepted "<-"; parse_aggregate_fields also accepts ":-" (arc 251 superset)
// (b) name: inline also accepted Keyword names (stripping ":"); parse_aggregate_fields requires Symbol.
// The new unified path uses parse_argspec_triples (see STOP-FIELD resolution in parse_aggregate_fields).

// Stone 241.9 — `parse_field` DELETED. Its only caller was `parse_enum_variant`,
// which was also deleted (HARD CUT). `parse_defenum` uses `parse_argspec_triples`
// for tagged-variant fields instead of the legacy pair-form parser.

/// Parse a declared type name. Accepts a bare name only:
/// - `:my::ns::MyType` → ("my/ns/MyType", [])
///
/// A name carrying an angle-bracket suffix (`:my::ns::Wrapper<T>`) is REFUSED — arc 109 ③
/// retired that spelling; type parameters now arrive as the sibling `Head :- [T …]` binder
/// just after the name (see `is_binder_marker`/`take_declared_binder` below), never inside
/// the name keyword's own text. The returned `Vec` is therefore always empty here.
///
/// Arc 138 slice 2 — `decl_span` is the whole-decl span used for
/// MalformedDecl errors fired here (when the name slot isn't a
/// keyword); the name keyword's own span is used for MalformedName
/// errors (the bad-name shape itself).
fn parse_declared_name(
    head: &str,
    form: &WatAST,
    decl_span: &Span,
) -> Result<(String, Vec<String>), TypeError> {
    let name_span = form.span().clone();
    let raw = match form {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(TypeError::new(
                decl_span.clone(),
                TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: format!(
                        "name must be a keyword; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    // Strip the colon but keep the rest as the key for TypeEnv.
    let stripped = raw.strip_prefix(':').ok_or_else(|| TypeError::new(
        name_span.clone(),
        TypeErrorKind::MalformedName {
            raw: raw.clone(),
            reason: "keyword must begin with ':'".into(),
        },
    ))?;
    // Arc 109 ③ — angle brackets are ILLEGAL for a declaration's own name.
    // `<T>` used to be sniffed and split into (base, params) here; that
    // spelling is now refused outright. The binder marker
    // (`take_declared_binder`, just below) is the ONE surviving spelling:
    // `Head :- [T …]` as SIBLINGS after the name, no parens.
    if stripped.contains('<') {
        return Err(TypeError::new(
            name_span,
            TypeErrorKind::MalformedName {
                raw: raw.clone(),
                reason: "angle-bracket type parameters are illegal; write `Head :- [T …]` \
                          (siblings after the name, no parens) instead of `Head<T>`"
                    .into(),
            },
        ));
    }
    Ok((raw, Vec::new()))
}

/// Arc 109 binder strike α — is this node the `:-` binder MARKER?
///
/// ONE spelling of that question, for the same reason `Identifier::is_reference`
/// is one spelling of "is this symbol a binder NAME" (stone 251.8a collapsed four
/// hand-rolls of that one into a single door). Three callers: [`take_declared_binder`]
/// peeks it, `parse_structtype` must CARRY it across its synthetic-parent
/// injection so the binder stays adjacent to the name, and `pub(crate)` since
/// arc 109 stone "a type reference is not an expression" — `macros::expand`'s
/// macro-dispatch guard peeks it at index 1 to decline `(Head :- [args])` as a
/// value expression before the head's registered companion macro can fire.
pub(crate) fn is_binder_marker(node: &WatAST) -> bool {
    matches!(node, WatAST::Keyword(k, _) if k == ":-")
}

/// STONE-finish-the-param-spec (arc 109) — the ONE door that peels the
/// `(marker, [types], rest…)` TRIPLE. `is_binder_marker`, just above, answers
/// only *"is this node `:-`"*; every consumer that also needed the type list
/// hand-rolled the `[Keyword, Vector, rest @ ..]` slice pattern itself — nine
/// sites, no two guaranteed to agree. This is the door all nine now call.
///
/// `[:- [T U …] rest…]` → `(Some(&[T,U,…]), rest)`;  no marker → `(None, args)`.
///
/// - No `:-` at `args[0]` → unchanged: `(None, args)`.
/// - `:- []` → `(Some(&[]), rest)`, **never** `(None, _)` — the empty binder is
///   *expressed*, not absent (the builder's rule: `absent ≡ :- [] ≡ :- []`).
/// - `:-` present but NOT followed by a `WatAST::Vector` → the marker is left
///   UNPEELED: `(None, args)`. A malformed binder is a shape for the caller's
///   own diagnostic to name, not for this door to invent a second error path
///   for (mirrors `peel_type_binder`'s existing "leave it for the natural
///   'expected a vector' error" rule).
/// - `:-` as the LAST element (nothing follows the marker itself, so there is
///   no vector to peel) → falls through the same guard: `(None, args)`.
///
/// Returns the raw `WatAST` nodes, never a parsed `TypeExpr` — `check.rs` and
/// `runtime.rs` treat the peeled slice differently downstream (one splices it
/// back into a value stream, one parses each entry as a `TypeExpr`), and a
/// door that pre-committed to one shape would grow a second door for the
/// other.
pub(crate) fn peel_param_spec(args: &[WatAST]) -> (Option<&[WatAST]>, &[WatAST]) {
    match args {
        [WatAST::Keyword(k, _), WatAST::Vector(inner, _), rest @ ..] if k == ":-" => {
            (Some(inner.as_slice()), rest)
        }
        _ => (None, args),
    }
}

/// Arc 109 binder strike α — consume an optional `:- [T …]` binder from the
/// arg stream, immediately after the name. `name_params` is what
/// `parse_declared_name` read from the name's `<…>` spelling; `name_span` is
/// that name keyword's span, used to locate the both-spellings contradiction.
///
/// - No binder present (`iter.peek()` isn't the `:-` keyword) → `name_params`
///   returned unchanged; every existing `<T>`-or-bare form is untouched.
/// - Binder present → consume the `:-` keyword AND the `Vector` that must
///   follow it; the vector's entries become the params.
/// - Both present (`name_params` non-empty AND a binder) → `TypeError`. Two
///   spellings of type-params on one declaration is a contradiction that
///   arises only from a half-applied codemod, never from someone writing it
///   by hand — it must not silently pick one.
/// - Each binder entry must be a bare `Symbol` whose `Identifier::is_reference()`
///   is `false` (no `/`, so `namespace()` reads `$bound`) — the one door
///   251.8a consolidated four hand-rolled checks into. Rejects keyword values,
///   function types, and nested field vectors with one diagnostic.
/// - Returns BARE names — never the `$bound/T`-derived spelling.
///   `identifier.rs:145`'s own doc: the namespace is derived from the
///   spelling today; 251.8b is where derived swaps for stored, and writing
///   the derived form into `type_params` now would pre-encode that artifact.
fn take_declared_binder<I: Iterator<Item = WatAST>>(
    head: &str,
    name_params: Vec<String>,
    name_span: &Span,
    iter: &mut std::iter::Peekable<I>,
) -> Result<Vec<String>, TypeError> {
    let has_binder = iter.peek().is_some_and(is_binder_marker);
    if !has_binder {
        return Ok(name_params);
    }
    let binder_kw = iter.next().unwrap();
    if !name_params.is_empty() {
        return Err(TypeError::new(
            name_span.clone(),
            TypeErrorKind::MalformedDecl {
                head: head.into(),
                reason: format!(
                    "declaration carries BOTH a name-embedded `<...>` type-param spelling \
                     ({:?}) and a `:- [...]` binder — pick one; a declaration with both is a \
                     contradiction, never something to silently resolve",
                    name_params
                ),
            },
        ));
    }
    let vec_node = iter.next().ok_or_else(|| TypeError::new(
        binder_kw.span().clone(),
        TypeErrorKind::MalformedDecl {
            head: head.into(),
            reason: "`:-` binder must be followed by a `[...]` vector of type-parameter names"
                .into(),
        },
    ))?;
    let items = match vec_node {
        WatAST::Vector(items, _) => items,
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: format!(
                        "`:-` binder must be followed by a `[...]` vector of type-parameter \
                         names; got {}",
                        other.variant_name()
                    ),
                },
            ));
        }
    };
    let mut params = Vec::with_capacity(items.len());
    for item in items {
        match &item {
            WatAST::Symbol(id, _) if !id.is_reference() => {
                params.push(id.as_str().to_string());
            }
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: head.into(),
                        reason: format!(
                            "binder entry must be a bare type-parameter name (a Symbol with no \
                             `/`); got {}",
                            other.variant_name()
                        ),
                    },
                ));
            }
        }
    }
    Ok(params)
}

/// Parse a type-expression keyword into a structured [`TypeExpr`].
///
/// Refuses `:Any` at any position (bare path or parametric head) per
/// 058-030's closed-type-universe discipline. Every apparent need for
/// `:Any` has a principled named alternative (`:wat::holon::HolonAST` for algebra
/// values, parametric `T`/`K`/`V` for generics, a named enum for
/// closed heterogeneous sets).
// rune:struere(host-constraint) — public surface preserved for callers
// without a keyword span in scope (arc 138 lineage); crate::rust_caller_span!() is
// the honest placeholder when no source position is available. Span-aware
// callers use parse_type_expr_with_span directly.
pub fn parse_type_expr(kw: &str) -> Result<TypeExpr, TypeError> {
    parse_type_expr_with_span(kw, &crate::rust_caller_span!())
}

/// Arc 109 Stone ②-iii — parse a type from its SOURCE TEXT, whichever spelling it wears.
///
/// [`parse_type_expr`] takes a keyword STRING and requires a leading `:`. The `:-` migration
/// moved every parametric type annotation off the keyword spelling
/// (`:wat::core::Vector<wat::core::String>`) and onto a FORM
/// (`(:wat::core::Vector :- [:wat::core::String])`) — which is not a keyword, has no leading
/// colon, and so became unreadable to any consumer holding only `parse_type_expr`. That is
/// exactly what happened to `wat_source_derive`'s `wat_record_from!`: it reads the `.wat`
/// corpus as the source of truth at Rust-compile time, and five stdlib records went from
/// legible to `field … is not a `name <- :Type` triple` the moment the corpus moved.
///
/// The text is read with the real reader and handed to [`parse_type_node`], so all four
/// spellings — keyword, `wat.type/` symbol, parametric form, `[arg… :-> ret]` bracket — go
/// through the substrate's own parser. There is no second type parser here, and adding a
/// spelling to `parse_type_node` reaches this entry point for free.
pub(crate) fn parse_type_expr_from_source(text: &str) -> Result<TypeExpr, TypeError> {
    let span = crate::rust_caller_span!();
    let node = crate::parse_one_with_file(text, &span.file).map_err(|e| {
        TypeError::new(
            span.clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: text.into(),
                reason: format!("type source text does not read as one wat form: {e:?}"),
            },
        )
    })?;
    parse_type_node(&node)
}

/// Arc 138 slice 2 — span-carrying variant. Consumers with a real
/// keyword span (the type-registration call chain in this file) use
/// this entry point so emitted errors prefix `<file>:<line>:<col>:`.
pub fn parse_type_expr_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError> {
    let stripped = kw.strip_prefix(':').ok_or_else(|| TypeError::new(
        span.clone(),
        TypeErrorKind::MalformedTypeExpr {
            raw: kw.into(),
            reason: "type expression keyword must begin with ':'".into(),
        },
    ))?;
    let expr = parse_type_inner(stripped, kw, true, span)?;
    reject_any(&expr, kw, span)?;
    Ok(expr)
}

/// Arc 109 Stone ②-i-b — span-carrying, NON-canonicalizing sibling of
/// [`parse_type_expr_with_span`]. Byte-identical except `canonicalize=false`:
/// preserves the source spelling — `:wat::core::nil` stays `Path(":wat::core::nil")`
/// instead of collapsing to `Tuple(vec![])` (`parse_type_inner`'s `canonicalize &&
/// raw_path == ":wat::core::nil"` arm below), so the renderer can round-trip what
/// the user actually wrote instead of a type it already lost. Still calls
/// `reject_any` — the `:Any` ban applies on every path, canonicalizing or not.
///
/// Returns `Result`, NEVER `Option` — the caller (the `keyword/to-type-form*`
/// verbs) must surface a genuine parse error to the user. [`parse_type_expr_audit`]
/// is a DIFFERENT existing `canonicalize=false` entry point that swallows errors
/// into `None` for best-effort audit scanning; that silence is why it cannot be
/// reused here.
pub fn parse_type_expr_preserving_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError> {
    let stripped = kw.strip_prefix(':').ok_or_else(|| TypeError::new(
        span.clone(),
        TypeErrorKind::MalformedTypeExpr {
            raw: kw.into(),
            reason: "type expression keyword must begin with ':'".into(),
        },
    ))?;
    let expr = parse_type_inner(stripped, kw, false, span)?;
    reject_any(&expr, kw, span)?;
    Ok(expr)
}

/// Arc 251.3a — dispatch a `WatAST` node in a type-annotation slot.
///
/// Accepts the three node shapes that can appear in a type slot after the
/// dual-read transition begins at 251.3:
///
/// - `WatAST::Keyword(kw, span)` — the existing surface: delegates to
///   `parse_type_expr_with_span`. Covers atomic paths (`:wat::core::i64`, etc.) and
///   `fn(...)->...`/tuple-literal spellings; a parametric reference in Keyword form
///   (`Head<args>`) is refused — arc 109 ③ retired that spelling.
/// - `WatAST::Symbol(ident, span)` — a namespaced symbol `wat.type/X`
///   arriving **pre-normalization** (before `normalize_symbol_refs` has run).
///   Converted to the keyword FQDN (`:wat::type::X`) then parsed; the
///   `wat::type::` → `wat::core::` alias in `parse_type_inner` applies on
///   the canonicalize path, so `wat.type/i64` → `Path(":wat::core::i64")`.
/// - `WatAST::List(_, _)` — a parametric-type FORM `(CTOR arg…)` such as
///   `(wat.type/Vector wat.type/i64)`. Delegates to `parse_type_form`.
///
/// Any other node variant → `TypeError::MalformedTypeExpr` with a
/// descriptive reason.
pub(crate) fn parse_type_node(node: &WatAST) -> Result<TypeExpr, TypeError> {
    match node {
        WatAST::Keyword(kw, span) => parse_type_expr_with_span(kw, span),
        WatAST::Symbol(ident, span) => {
            // Pre-normalization (register_types, step 5, runs before normalize): a
            // `wat.type/X` symbol. Map to its keyword FQDN via the ONE canonical
            // mapping — `ns_to_wat_path`, the same path `normalize_symbol_refs` uses —
            // then parse. (Single source: do NOT reinvent the `a.b/c`→`:a::b::c` rule.)
            let s = ident.as_str();
            let kw = if s.contains('/') {
                crate::edn::render::ns_to_wat_path(ident.receiver(), ident.method())
            } else {
                // Bare symbol without namespace — treat as a keyword by prepending `:`.
                format!(":{}", s)
            };
            parse_type_expr_with_span(&kw, span)
        }
        WatAST::List(_, _) => parse_type_form(node),
        // Arc 251.4c — a `[T… :-> R]` bracket is a function type (core.typed parity).
        WatAST::Vector(items, span) => parse_fn_type_bracket(items, span),
        other => Err(TypeError::new(
            other.span().clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: format!("{:?}", other),
                reason: format!(
                    "type annotation must be a keyword, namespaced symbol, parametric form `(Ctor arg…)`, or function-type bracket `[arg… :-> ret]`; got {}",
                    other.variant_name()
                ),
            },
        )),
    }
}

/// Arc 251.4c — parse a function-type bracket `[arg… :-> ret]` → `TypeExpr::Fn`.
///
/// core.typed's function-type surface. Produces the SAME `TypeExpr::Fn { args, ret }`
/// the keyword form `:wat::core::Fn(args)->ret` yields (`parse_fn_body`), so the two
/// spellings unify. Args and the return type are each parsed via [`parse_type_node`]
/// (so they inherit the keyword / `wat.type/` / parametric-form surfaces). The lone
/// `:->` keyword separates the argument types from the single return type.
fn parse_fn_type_bracket(items: &[WatAST], span: &Span) -> Result<TypeExpr, TypeError> {
    let arrow_pos = items
        .iter()
        .position(|n| matches!(n, WatAST::Keyword(k, _) if k == ":->"));
    let arrow_pos = match arrow_pos {
        Some(p) => p,
        None => {
            return Err(TypeError::new(
                span.clone(),
                TypeErrorKind::MalformedTypeExpr {
                    raw: "[…]".into(),
                    reason: "function-type bracket needs a `:->` arrow: `[arg… :-> ret]`".into(),
                },
            ))
        }
    };
    let ret_nodes = &items[arrow_pos + 1..];
    if ret_nodes.len() != 1 {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: "[…]".into(),
                reason: format!(
                    "function-type bracket needs exactly one return type after `:->`; got {}",
                    ret_nodes.len()
                ),
            },
        ));
    }
    let args = items[..arrow_pos]
        .iter()
        .map(parse_type_node)
        .collect::<Result<Vec<_>, _>>()?;
    let ret = Box::new(parse_type_node(&ret_nodes[0])?);
    let result = TypeExpr::Fn { args, ret };
    // Enforce the :Any ban in fn-type args/ret, mirroring the other parse paths.
    reject_any(&result, "[… :-> …]", span)?;
    Ok(result)
}

/// Arc 251.3a — parse a parametric-type FORM `(CTOR arg…)` → `TypeExpr::Parametric`.
///
/// Produces the SAME `Parametric { head, args }` storage the `<>` keyword surface
/// produces, so the type-checker unification is unchanged. The CTOR head may be:
///
/// - `WatAST::Symbol("wat.type/Vector")` — pre-normalize; converted to `"wat::core::Vector"`.
/// - `WatAST::Keyword(":wat::type::Vector")` — post-normalize; same result.
/// - `WatAST::Keyword(":wat::core::Vector")` — already canonical.
///
/// Each arg is parsed recursively via [`parse_type_node`] (atom → `Path`; nested form → recurse).
///
/// HEAD storage convention (mirrors `parse_type_inner`'s `<>` arm, line ~2340):
/// `raw_head` is the path WITHOUT a leading colon, e.g. `"wat::core::Vector"`.
/// The `wat::type::` → `wat::core::` alias is applied on the canonicalize path
/// to maintain the dual-read invariant through the 251.5 hard-cut.
pub(crate) fn parse_type_form(node: &WatAST) -> Result<TypeExpr, TypeError> {
    let (items, span) = match node {
        WatAST::List(items, span) => (items, span),
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedTypeExpr {
                    raw: format!("{:?}", other),
                    reason: "parse_type_form expects a List node".into(),
                },
            ))
        }
    };
    if items.is_empty() {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: "()".into(),
                reason: "parametric type form must not be empty; expected `(Ctor arg…)`".into(),
            },
        ));
    }
    // Extract the constructor head as a bare path string (no leading colon).
    // Mirrors the <> arm in parse_type_inner which stores `raw_head = s[..lt_index]`
    // (the FQDN before `<`, no colon). We must produce the SAME string for unification.
    let raw_head: String = match &items[0] {
        WatAST::Symbol(ident, _) => {
            // Pre-normalize symbol `wat.type/Vector` → keyword FQDN via the ONE
            // canonical mapping (`ns_to_wat_path`), then strip the leading `:` for the
            // bare head-storage convention. (Single source — no reinvented `.`/`/` rule.)
            let s = ident.as_str();
            if s.contains('/') {
                let kw = crate::edn::render::ns_to_wat_path(ident.receiver(), ident.method());
                kw.strip_prefix(':').unwrap_or(&kw).to_string()
            } else {
                s.to_string()
            }
        }
        WatAST::Keyword(kw, _) => {
            // Post-normalize keyword (`:wat::type::Vector`) or already canonical (`:wat::core::Vector`).
            // Strip the leading `:`.
            kw.strip_prefix(':').unwrap_or(kw).to_string()
        }
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedTypeExpr {
                    raw: format!("{:?}", other),
                    reason: "parametric type form head must be a symbol or keyword".into(),
                },
            ))
        }
    };
    // Arc 251.2 alias: `wat::type::` → `wat::core::` (dual-read, mirrors parse_type_inner ~line 2374).
    let raw_head = match raw_head.strip_prefix("wat::type::") {
        Some(tail) => format!("wat::core::{}", tail),
        None => raw_head,
    };
    // Parse args recursively.
    //
    // Arc 109 step ① originally accepted a bare bracketed type-param group
    // `(Head [type…])` here, and the positional tail `(Head A B)` alongside it.
    // Arc 109 "THE LAST DOORS" stone (door 1, TYPE-ANNOTATION POSITION) retires
    // BOTH: the unmarked bracket and the bare positional tail were the last two
    // heretical spellings of a parametric literal still accepted anywhere in the
    // language. `:- [type…]` — peeled just below — is now the ONLY spelling this
    // door accepts; anything else is a named `MalformedTypeExpr`, not a silent
    // positional parse. This does not collide with the standalone function-type
    // bracket `[A :-> B]` (`parse_type_node`'s `WatAST::Vector` arm, ~line 4383):
    // that arm only fires when a bracket is parsed as a top-level type node on
    // its own, never here, where a bracket is one argument of a parametric head.
    //
    // Arc 109 Stone ②-i-b — the `:-`-marked spelling: `(Head :- [type…])`. `:-`
    // declares "the thing on the left is parameterized by the thing on the right"
    // (the same relation the arg-spec and ret-type arrows already carry); the
    // bracket after it is a type-param list BY DECLARATION, never sniffed.
    //
    // STONE-exactly-one-call-position (arc 109) — SUPERSEDES the prior reasoning
    // here (which read `(Tuple :- [])` as a distinct, legitimate zero-length
    // param-spec, deliberately un-guarded against `!inner.is_empty()`). The
    // builder's rule this stone: absent, `:- []`, and the empty binder are all
    // the SAME thing —
    //
    //     not expressed        →  :- []
    //     expressed and empty  →  :- []
    //     otherwise             →  the binders chosen
    //
    // — so `Parametric{args: []}` must not exist as a distinct value from
    // `Path(head)`. `(Head :- [])` normalises to `Path(head)` below, the same
    // variant the bare `Head` reference already parses to, so the two now unify
    // instead of a type failing to match itself.
    //
    // The builder's second rule: initial values after the bracket
    // (`(Head :- [types] v…)`) make the form a LITERAL, and a literal is not a
    // type — reserved-position violation, not a different production to fall
    // through to. Emit a clean, named error here rather than letting it fall
    // through to the standalone function-type-bracket arm (`parse_fn_type_bracket`),
    // which would misreport it as a malformed `[arg… :-> ret]`.
    let rest_items = &items[1..];
    let (peeled, after_marker) = peel_param_spec(rest_items);
    let via_binder = peeled.is_some();
    let args: Result<Vec<TypeExpr>, TypeError> = match peeled {
        Some(inner) => {
            if !after_marker.is_empty() {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::MalformedTypeExpr {
                        raw: format!("({} :- […] …)", raw_head),
                        reason: "a type declaration cannot carry initial values — \
                                  `(Head :- [types] v…)` is a LITERAL, and a literal is not a \
                                  type. Drop the values here, or move the form out of the type \
                                  position."
                            .into(),
                    },
                ));
            }
            inner.iter().map(parse_type_node).collect()
        }
        None => {
            // Arc 109 "THE LAST DOORS" door 1 — no `:-` marker present. Both
            // heretical spellings this arm used to accept — the unmarked bracket
            // `(Head [type…])` and the bare positional tail `(Head A B …)` — are
            // retired; `:- [type…]` is the one legal param-spec, everywhere.
            return Err(TypeError::new(
                span.clone(),
                TypeErrorKind::MalformedTypeExpr {
                    raw: format!("({} …)", raw_head),
                    reason: "a parametric type must declare its parameters with the `:- [types...]` \
                              binder — `(Head A B …)` (bare positional) and `(Head [A B …])` \
                              (unmarked bracket) are retired spellings. Canonical: \
                              `(Head :- [A B …])`."
                        .into(),
                },
            ));
        }
    };
    let args = args?;
    // Arc 251 — the `Tuple` constructor head produces a TUPLE type, not a generic Parametric:
    // `(wat.type/Tuple A B)` → `TypeExpr::Tuple([A,B])`; the empty `(wat.type/Tuple)` → the
    // 0-tuple. This is the faithful-Clojure spelling of the legacy `:(A,B)` keyword tuple
    // (both produce the SAME `TypeExpr::Tuple`, so they unify identically).
    let result = if raw_head == "wat::core::Tuple" {
        TypeExpr::Tuple(args)
    } else if via_binder && args.is_empty() {
        // STONE-exactly-one-call-position — `(Head :- [])` IS `Head`: the empty
        // binder is the same as absent, so this must be the identical `Path`
        // variant the bare reference parses to, not a `Parametric` with an empty
        // arg list that fails to unify against it.
        TypeExpr::Path(format!(":{}", raw_head))
    } else {
        TypeExpr::Parametric { head: raw_head, args }
    };
    // Re-use reject_any to enforce the :Any ban in parametric/tuple form heads/args.
    reject_any(&result, &format!("({}…)", items[0].variant_name()), span)?;
    Ok(result)
}

/// Arc 109 slice 1c — parse a type expression keyword WITHOUT
/// canonicalizing bare primitives to their internal-form path.
/// Source spelling is preserved in the resulting [`TypeExpr`]:
/// bare `:i64` produces `Path(":i64")`; FQDN `:wat::core::i64`
/// produces `Path(":wat::core::i64")`. The walker that audits for
/// retired bare primitives consumes this faithful structure.
///
/// Returns `None` for non-type keywords (callee paths, value
/// keywords like `:None`) — the parse error is suppressed because
/// the caller is doing best-effort scanning, not unification.
///
/// Use for AUDIT walks only. Type-checker code path stays on
/// `parse_type_expr` to keep the canonical-form invariant intact.
pub fn parse_type_expr_audit(kw: &str) -> Option<TypeExpr> {
    let stripped = kw.strip_prefix(':')?;
    // arc 138: no span — audit path returns Option, never surfaces a
    // TypeError to a consumer; the synthetic span never escapes.
    parse_type_inner(stripped, kw, false, &crate::rust_caller_span!()).ok()
}

/// Walk a parsed [`TypeExpr`] and raise [`TypeError::AnyBanned`] if
/// `:Any` appears anywhere. Protects the type universe's closure.
///
/// Arc 138 slice 2 — `span` is the outermost type-keyword span; the
/// AnyBanned error prefixes `<file>:<line>:<col>:` so the consumer
/// navigates straight to the offending decl/field.
fn reject_any(expr: &TypeExpr, raw: &str, span: &Span) -> Result<(), TypeError> {
    match expr {
        TypeExpr::Path(p) => {
            if p == ":Any" {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::AnyBanned { raw: raw.into() },
                ));
            }
        }
        TypeExpr::Parametric { head, args } => {
            if head == "Any" {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::AnyBanned { raw: raw.into() },
                ));
            }
            for a in args {
                reject_any(a, raw, span)?;
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                reject_any(a, raw, span)?;
            }
            reject_any(ret, raw, span)?;
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                reject_any(e, raw, span)?;
            }
        }
        TypeExpr::Var(_) => {
            // Fresh vars are synthetic; never appear at parse time.
        }
    }
    Ok(())
}

/// Parse the content of a type keyword after the leading ':' has been
/// stripped. `original` is the full keyword string for error reporting.
///
/// Arc 115 slice 2 — reject any leading ':' on `s`. The outermost
/// `parse_type_expr` strips the legitimate leading colon before
/// delegating; any leading colon that survives here means we're
/// parsing an arg from inside a compound (`<>`, `()`, fn args, fn
/// return), where the colon prefix is illegal. Inside compounds,
/// args are bare Rust symbols.
fn parse_type_inner(
    s: &str,
    original: &str,
    canonicalize: bool,
    span: &Span,
) -> Result<TypeExpr, TypeError> {
    if s.starts_with(':') {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::InnerColonInCompoundArg {
                raw: original.into(),
                offending: s.to_string(),
            },
        ));
    }
    // Tuple literal — `(T,U,...)`. Must appear at the start; inner
    // types respect top-level comma splitting.
    if let Some(rest) = s.strip_prefix('(') {
        if !rest.ends_with(')') {
            return Err(TypeError::new(
                span.clone(),
                TypeErrorKind::MalformedTypeExpr {
                    raw: original.into(),
                    reason: "tuple-literal type must close with ')'".into(),
                },
            ));
        }
        let inside = &rest[..rest.len() - 1];
        return parse_tuple_body(inside, original, canonicalize, span);
    }
    // `fn(args)->ret` function type — detect at the start.
    // Arc 155 — `:wat::core::Fn(args)->ret` is the canonical FQDN
    // spelling of the function type (Cap'd type head per the
    // Clojure-faithful capitalization convention; `Fn` = type,
    // `fn` = verb). Both the bare `fn(` prefix and the FQDN
    // `wat::core::Fn(` prefix parse to the same `TypeExpr::Fn`
    // internal representation (canonical-form invariant: the type
    // unifier sees one shape). The `walk_for_legacy_lowercase_fn`
    // walker in `src/check.rs` fires `BareLegacyLowercaseFn` per
    // bare `:fn(...)` site for sweep 1b's mechanical migration.
    if let Some(body) = s.strip_prefix("fn(") {
        return parse_fn_body(body, original, canonicalize, span);
    }
    if let Some(body) = s.strip_prefix("wat::core::Fn(") {
        return parse_fn_body(body, original, canonicalize, span);
    }
    // Arc 109 ③ — angle brackets are ILLEGAL for a parametric type
    // REFERENCE / annotation. `Head<args>` used to be sniffed and split
    // here; that spelling is now refused outright. The ONE surviving
    // reference spelling is the `:-` form parsed by `parse_type_form`:
    // `(Head :- [args])`, in parens.
    if find_top_level_char(s, '<').is_some() {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: original.into(),
                reason: "angle-bracket parametric types are illegal; write \
                          `(Head :- [args])` instead of `Head<args>`"
                    .into(),
            },
        ));
    }
    // Plain path. Arc 109 slice 1a: accept FQDN forms for the
    // built-in primitive types (`:wat::core::i64`, `:wat::core::f64`,
    // `:wat::core::bool`, `:wat::core::String`, `:wat::core::u8`).
    // When `canonicalize` is true (the type-checker path), reduce
    // both bare and FQDN spellings to one internal form so unify
    // sees them as identical. When false (the audit-walker path,
    // arc 109 slice 1c), preserve source spelling so a bare `:i64`
    // stays distinguishable from FQDN `:wat::core::i64` in the
    // resulting Path. Slice 1c retires bare at the parser level
    // once the user-code sweep is complete.
    //
    // Arc 153 (was arc 109 slice 1d): `:wat::core::nil` is the
    // FQDN spelling of the unit/nil type. When canonicalizing,
    // reduce to the internal empty-tuple form so unify sees it as
    // identical to the legacy `:()` spelling and to validators
    // (e.g. user::main return-type check) that compare against
    // `TypeExpr::Tuple(vec![])`. The retired `:wat::core::unit`
    // FQDN spelling was supported during the migration window via
    // `BareLegacyUnitName` walker scaffolding; both the typealias
    // and the walker firing path retired at arc 153 slice 2 per
    // substrate-as-teacher § "Retire the hint when its window
    // closes."
    let raw_path = format!(":{}", s);
    // Arc 251.2 — the `wat.type/` namespace. A scalar type atom written
    // `wat.type/i64` (Symbol) is normalized to the keyword `:wat::type::i64`
    // before it reaches here. On the type-checker path (`canonicalize=true`) it
    // aliases to the internal canonical `:wat::core::<atom>` the checker keys on
    // (literal types + Path comparisons). The INTERNAL canonical deliberately
    // stays `:wat::core::` for the dual-read transition; the flip to `:wat::type::`
    // is deferred to the 251.5 hard-cut (see DESIGN-STONE-251.2.md). The audit
    // walk (`canonicalize=false`) preserves source spelling, and only ATOM paths
    // reach this arm — parametric heads parse via the `<>`/`()` branches above.
    let raw_path = match (canonicalize, raw_path.strip_prefix(":wat::type::")) {
        (true, Some(tail)) => format!(":wat::core::{}", tail),
        _ => raw_path,
    };
    if canonicalize && raw_path == ":wat::core::nil" {
        return Ok(TypeExpr::Tuple(vec![]));
    }
    // Arc 163 slice 3f + 3h — FQDN IS the canonical storage form.
    // Source FQDN flows through unchanged. Source bare-form is
    // rejected by the `BareLegacyPrimitive` walker at check time
    // (slice 3g phase A wired the walker on raw post-expansion
    // forms so define-sig type positions are covered). The
    // canonicalize=true UPGRADE arm (`:i64` → `:wat::core::i64`
    // etc.) retired in slice 3h — raw_path passes through identity.
    Ok(TypeExpr::Path(raw_path))
}

/// Parse the body of a tuple-literal type.
///
/// - Empty body `` → unit (0-tuple): `Tuple(vec![])`.
/// - Single type with no trailing comma: Rust grouping — returns the
///   inner type directly (NOT wrapped in Tuple).
/// - Trailing comma or multiple elements: `Tuple(vec![...])`.
///
/// Matches Rust's tuple-type syntax exactly.
fn parse_tuple_body(
    inside: &str,
    original: &str,
    canonicalize: bool,
    span: &Span,
) -> Result<TypeExpr, TypeError> {
    let trimmed = inside.trim();
    if trimmed.is_empty() {
        return Ok(TypeExpr::Tuple(Vec::new()));
    }
    let has_trailing_comma = trimmed.ends_with(',');
    let effective = if has_trailing_comma {
        trimmed[..trimmed.len() - 1].trim_end()
    } else {
        trimmed
    };
    let elements = parse_type_list(effective, original, canonicalize, span)?;
    if elements.len() == 1 && !has_trailing_comma {
        // `:(T)` is grouping — return the inner type unwrapped.
        return Ok(elements.into_iter().next().unwrap());
    }
    Ok(TypeExpr::Tuple(elements))
}

fn parse_fn_body(
    body: &str,
    original: &str,
    canonicalize: bool,
    span: &Span,
) -> Result<TypeExpr, TypeError> {
    // body is `T,U)->R` — find the matching `)` at depth 0.
    let close = find_matching_close(body, '(', ')').ok_or_else(|| TypeError::new(
        span.clone(),
        TypeErrorKind::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing matching ')'".into(),
        },
    ))?;
    let args_part = &body[..close];
    let tail = &body[close + 1..];
    let ret_part = tail
        .strip_prefix("->")
        .ok_or_else(|| TypeError::new(
            span.clone(),
            TypeErrorKind::MalformedTypeExpr {
                raw: original.into(),
                reason: "fn type missing '->' before return".into(),
            },
        ))?;
    let args = if args_part.trim().is_empty() {
        Vec::new()
    } else {
        parse_type_list(args_part, original, canonicalize, span)?
    };
    let ret = parse_type_inner(ret_part, original, canonicalize, span)?;
    Ok(TypeExpr::Fn {
        args,
        ret: Box::new(ret),
    })
}

/// Parse a comma-separated list of types (respecting nested `<>` and `()`).
///
/// Arc 170 W2 Strike 1a — a bare `>` immediately preceded by `-` is the arrow of a
/// `Fn(...)->T` function type, NOT a closing angle bracket. Before this fix, `depth`
/// blindly decremented on every `>`, so a `Fn(...)->T` embedded as a non-final element
/// of a comma list (a Tuple element, or a generic's Nth type-arg) underflowed depth to
/// -1 — every comma AFTER the arrow then read `depth == 0` as false and never split,
/// silently swallowing the rest of the list into one opaque unparsed `Path`. Mirrors the
/// lexer's own `->`-vs-`<...>` disambiguation (`wat-reader/src/lexer.rs` `lex_keyword`'s
/// `angle_depth` handling) — grounded via `#-then->`, since a genuine angle-bracket close
/// is never preceded by `-` (operator paths like `:wat::core::>=` never reach this parser;
/// they're never split into a type-list). Found via `wat-scripts/probes/arc-170/…`-class
/// probing: a hand-written `Tuple<Fn(i64)->i64,i64,>` return type froze to a malformed
/// 1-element `Tuple(Path("Fn(i64)->i64,i64"))` instead of the correct 2-element Tuple.
///
/// The depth-tracking loop itself now lives in [`split_type_list_top_level`] — extracted so
/// `check.rs`'s call-site type-arg binder splits on the SAME tracker instead of a flat
/// `split(',')` (which tore `Locus/launch<…,State<K,V>,…>` apart). This fn is the parse half.
fn parse_type_list(
    s: &str,
    original: &str,
    canonicalize: bool,
    span: &Span,
) -> Result<Vec<TypeExpr>, TypeError> {
    let mut out = Vec::new();
    let pieces = split_type_list_top_level(s);
    // `split_type_list_top_level` always yields at least one piece, so
    // `split_last` never returns None. Every piece BEFORE the last is
    // parsed unconditionally (an empty one is a malformed list and must
    // surface as such); the trailing piece is skipped when empty, which
    // is what admits `Tuple<A,B,>`'s trailing comma.
    let (tail, init) = pieces.split_last().expect("split yields >= 1 piece");
    for piece in init {
        out.push(parse_type_inner(piece.trim(), original, canonicalize, span)?);
    }
    if !tail.trim().is_empty() {
        out.push(parse_type_inner(tail.trim(), original, canonicalize, span)?);
    }
    Ok(out)
}

/// Split a comma-separated type list on **top-level** commas only —
/// the string-level half of [`parse_type_list`], shared with the
/// call-site type-arg binder in `check.rs`.
///
/// A comma nested inside an inner `<…>` or `(…)` belongs to that inner
/// type, not to this list: `"Op,State<K,V>,Admin<K,V>"` splits into
/// three pieces, not five. `defservice` is the first minter of such a
/// call-head (`Locus/launch<Op,Reply,State<K,V>,Admin<K,V>,Status<K,V>>`);
/// a flat `split(',')` tore `State<K` / `V>` apart and shifted every
/// subsequent type-arg by one.
///
/// The `->` guard is [`parse_type_list`]'s (arc 170 W2 Strike 1a): a `>`
/// preceded by `-` is a `Fn(…)->T` arrow, not a bracket close.
///
/// Pieces are returned unstripped (no `trim`) and INCLUDE empties, so a
/// caller can reproduce `str::split(',')` exactly. On a body with no
/// nesting there is no depth to track and the result is element-for-element
/// identical to `s.split(',').collect()`.
pub(crate) fn split_type_list_top_level(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut prev_char: Option<char> = None;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' if prev_char == Some('-') => {} // Fn(...)->T arrow — not a bracket close.
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        prev_char = Some(c);
    }
    out.push(&s[start..]);
    out
}

/// Find the first occurrence of `c` at bracket-depth 0.
///
/// Checks the match BEFORE adjusting depth so that `c` itself being a
/// bracket (`<` or `(`) is correctly detected at the outermost level —
/// finding `<` in `List<T>` matches position 4, not None.
///
/// Arc 170 W2 Strike 1a — same `->`-arrow-vs-angle-close disambiguation as
/// `parse_type_list` above (twin depth-trackers, same latent bug class); fixed in
/// lockstep even though this fn's sole call site (the `Head<args>` branch of
/// `parse_type_inner`) isn't reachable by the bug today (it returns on the FIRST
/// top-level `<`, always found before any `->` could appear) — leaving one twin
/// fixed and the other not would just relocate the latent bug to this fn's next
/// caller.
fn find_top_level_char(s: &str, c: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut prev_char: Option<char> = None;
    for (i, ch) in s.char_indices() {
        if depth == 0 && ch == c {
            return Some(i);
        }
        match ch {
            '<' | '(' => depth += 1,
            '>' if prev_char == Some('-') => {} // Fn(...)->T arrow — not a bracket close.
            '>' | ')' => depth -= 1,
            _ => {}
        }
        prev_char = Some(ch);
    }
    None
}

/// Given a string that has just consumed an `open` bracket, find the
/// byte index of the matching `close` (accounting for nesting).
fn find_matching_close(s: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1i32; // caller already consumed the opening `open`
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

// ─── Typealias expansion ─────────────────────────────────────────────
//
// 058-030 declares `:wat::core::typealias` as a structural alias:
// `(:Alias :- [K V])` and its expansion are the SAME type. The runtime shape
// below walks alias-headed expressions to their definitions,
// substituting declared type parameters with call-site arguments, until
// a non-alias root is reached. Called from `check::unify` before the
// structural match so unification recognizes an alias and its
// expansion as equivalent.

/// Walk `expr`'s alias chain to its non-alias root. When the head of
/// `expr` names a `TypeDef::Alias` in `env`, substitute the alias's
/// type parameters with the call-site arguments and recurse. Stops
/// when the root is not an alias, when the head is unresolved, or when
/// the alias's arity does not match (the arity mismatch surfaces
/// elsewhere as a type-check error; here we leave the expression as
/// written so the downstream machinery sees the original form).
///
/// Purely-recursive aliases are prevented from looping by the
/// registration-time cycle check in
/// [`check_alias_no_cycle`]; expand_alias does not detect cycles
/// itself — by contract, every alias in `env` has been proven
/// non-cyclic at insertion.
pub fn expand_alias(expr: &TypeExpr, env: &TypeEnv) -> TypeExpr {
    let mut current = expr.clone();
    loop {
        match &current {
            TypeExpr::Path(name) => match env.get(name) {
                Some(TypeDef::Alias(alias)) if alias.type_params.is_empty() => {
                    current = alias.expr.clone();
                }
                _ => return current,
            },
            TypeExpr::Parametric { head, args } => {
                let qualified = parametric_head_fqdn(head);
                match env.get(&qualified) {
                    Some(TypeDef::Alias(alias))
                        if alias.type_params.len() == args.len() =>
                    {
                        let mapping: std::collections::HashMap<String, TypeExpr> = alias
                            .type_params
                            .iter()
                            .cloned()
                            .zip(args.iter().cloned())
                            .collect();
                        current = substitute_type_params(&alias.expr, &mapping);
                    }
                    _ => return current,
                }
            }
            _ => return current,
        }
    }
}

/// Substitute bare-path type-variable references with the caller's
/// supplied type arguments. Type variables appear in declarations as
/// `Path(":T")` (the ':' plus the declared type-param name); the
/// `mapping` is keyed by the param name stripped of the leading colon.
pub fn substitute_type_params(
    expr: &TypeExpr,
    mapping: &std::collections::HashMap<String, TypeExpr>,
) -> TypeExpr {
    match expr {
        TypeExpr::Path(p) => {
            if let Some(stripped) = p.strip_prefix(':') {
                if let Some(replacement) = mapping.get(stripped) {
                    return replacement.clone();
                }
            }
            TypeExpr::Path(p.clone())
        }
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head: head.clone(),
            args: args
                .iter()
                .map(|a| substitute_type_params(a, mapping))
                .collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args
                .iter()
                .map(|a| substitute_type_params(a, mapping))
                .collect(),
            ret: Box::new(substitute_type_params(ret, mapping)),
        },
        TypeExpr::Tuple(elements) => TypeExpr::Tuple(
            elements
                .iter()
                .map(|e| substitute_type_params(e, mapping))
                .collect(),
        ),
        TypeExpr::Var(id) => TypeExpr::Var(*id),
    }
}

/// Starting from the expansion of an alias named `target_name`, verify
/// that the walk never reaches `target_name` itself through other
/// aliases — otherwise registration would produce a cycle that
/// `expand_alias` cannot exit. Called from [`TypeEnv::register`] before
/// the new alias is inserted; the `env` passed is the registry as it
/// stands before this registration.
fn check_alias_no_cycle(
    target_name: &str,
    expr: &TypeExpr,
    env: &TypeEnv,
    span: &Span,
) -> Result<(), TypeError> {
    let mut visiting = std::collections::HashSet::new();
    check_alias_reaches(target_name, expr, env, &mut visiting, span)
}

fn check_alias_reaches(
    target_name: &str,
    expr: &TypeExpr,
    env: &TypeEnv,
    visiting: &mut std::collections::HashSet<String>,
    span: &Span,
) -> Result<(), TypeError> {
    // INVARIANT: every `visiting.insert(name)` is paired with a `visiting.remove(name)`
    // before any `?`-propagation can early-return — the cycle-detection set must not
    // leak names across recursive calls. New `?`-paths must preserve this pairing.
    match expr {
        TypeExpr::Path(name) => {
            if name == target_name {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::CyclicAlias { name: target_name.to_string() },
                ));
            }
            if let Some(TypeDef::Alias(alias)) = env.get(name) {
                if visiting.insert(name.clone()) {
                    check_alias_reaches(target_name, &alias.expr, env, visiting, span)?;
                    visiting.remove(name);
                }
            }
        }
        TypeExpr::Parametric { head, args } => {
            let qualified = parametric_head_fqdn(head);
            if qualified == target_name {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::CyclicAlias { name: target_name.to_string() },
                ));
            }
            if let Some(TypeDef::Alias(alias)) = env.get(&qualified) {
                if visiting.insert(qualified.clone()) {
                    check_alias_reaches(target_name, &alias.expr, env, visiting, span)?;
                    visiting.remove(&qualified);
                }
            }
            for a in args {
                check_alias_reaches(target_name, a, env, visiting, span)?;
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                check_alias_reaches(target_name, a, env, visiting, span)?;
            }
            check_alias_reaches(target_name, ret, env, visiting, span)?;
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                check_alias_reaches(target_name, e, env, visiting, span)?;
            }
        }
        TypeExpr::Var(_) => {}
    }
    Ok(())
}

// ─── Typeunion validation (Stone 237.1) ─────────────────────────────────────

/// Validate that every member of a typeunion declaration has an accepted
/// shape. Called from [`TypeEnv::register_with_span`] before insertion.
///
/// Accepted: `Path`, `Parametric`, `Tuple` — all bounded structural shapes.
/// Rejected: `Fn` (weird dispatch semantics) and `Var` (synthetic; never
/// appears in user-written declarations).
///
/// Also rejects empty member lists (`EmptyUnion`) and single-member lists
/// (`SingleMemberUnion` — recommend typealias).
fn validate_union_members(name: &str, members: &[TypeExpr], span: &Span) -> Result<(), TypeError> {
    if members.is_empty() {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::EmptyUnion { name: name.to_string() },
        ));
    }
    if members.len() == 1 {
        return Err(TypeError::new(
            span.clone(),
            TypeErrorKind::SingleMemberUnion { name: name.to_string() },
        ));
    }
    for member in members {
        match member {
            TypeExpr::Path(_) | TypeExpr::Parametric { .. } | TypeExpr::Tuple(_) => {}
            TypeExpr::Fn { .. } => {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::InvalidUnionMember {
                        union_name: name.to_string(),
                        member_form: format!("{:?}", member),
                        reason: "Fn types are not valid union members (weird dispatch semantics; revisit if a use case surfaces)".to_string(),
                    },
                ));
            }
            TypeExpr::Var(_) => {
                return Err(TypeError::new(
                    span.clone(),
                    TypeErrorKind::InvalidUnionMember {
                        union_name: name.to_string(),
                        member_form: format!("{:?}", member),
                        reason: "Var (synthetic unification variable) is not valid in user-written declarations".to_string(),
                    },
                ));
            }
        }
    }
    Ok(())
}

/// Starting from a typeunion's member list, verify that the walk through
/// registered typeunions never reaches `target_name` itself — otherwise
/// registration would produce a cycle that bounded-existential unification
/// cannot exit.
///
/// Called from [`TypeEnv::register_with_span`] before insertion; the `env`
/// is the registry as it stands BEFORE this union is inserted.
fn check_union_no_cycle(
    target_name: &str,
    members: &[TypeExpr],
    env: &TypeEnv,
    span: &Span,
) -> Result<(), TypeError> {
    let mut visiting = std::collections::HashSet::new();
    for member in members {
        check_union_member_reaches(target_name, member, env, &mut visiting, span)?;
    }
    Ok(())
}

fn check_union_member_reaches(
    target_name: &str,
    expr: &TypeExpr,
    env: &TypeEnv,
    visiting: &mut std::collections::HashSet<String>,
    span: &Span,
) -> Result<(), TypeError> {
    // INVARIANT: every `visiting.insert(name)` is paired with a `visiting.remove(name)`
    // before any `?`-propagation can early-return — the cycle-detection set must not
    // leak names across recursive calls. New `?`-paths must preserve this pairing.
    if let TypeExpr::Path(name) = expr {
        if name == target_name {
            return Err(TypeError::new(
                span.clone(),
                TypeErrorKind::CyclicUnion { name: target_name.to_string() },
            ));
        }
        // Walk through registered typeunions recursively.
        if let Some(TypeDef::Union(union)) = env.get(name) {
            if visiting.insert(name.clone()) {
                for member in &union.members {
                    check_union_member_reaches(target_name, member, env, visiting, span)?;
                }
                visiting.remove(name);
            }
        }
    }
    Ok(())
}

// ─── Stone S-A — typesub is-a hierarchy walk ────────────────────────────────

/// Directional, transitive, reflexive is-a test over the `typesub`
/// child→parent edge-registry on [`TypeEnv`].
///
/// Returns `true` iff `sub` is the same type as `sup` (reflexive) OR
/// there exists a chain of registered edges from `sub` up to `sup`
/// (transitive walk).
///
/// Walks the **new `subtype_edges` registry** — it does NOT call
/// [`collect_union_members`] and has NO connection to `typeunion` membership.
/// The hierarchy is a distinct relation (Clojure's `isa?`/`derive` axis).
///
/// Leaf-safe: a type with no parent edges (`:wat::core::bool`, `:wat::core::i64`, …)
/// returns `false` for any `sup ≠ sub` — the walk is empty.
///
/// Acyclic: edges are registered acyclically (see [`TypeEnv::register_subtype`]);
/// the `visited` guard also bounds the walk defensively.
pub fn is_subtype(sub: &str, sup: &str, env: &TypeEnv) -> bool {
    if sub == sup {
        return true; // reflexive
    }
    // Arc 278 Stone-Value — :wat::core::Value is the universal subtype-top: every type
    // <: Value. UP is free (this rule); DOWN stays checked — for any specific `sup ≠ Value`
    // this rule is skipped, the parents-walk finds no edge, and `assignable`'s (check.rs:13962)
    // fall-through `unify(Value, T)` fails. No registration: Value is recognized as an opaque
    // Path already; a TypeDef::Struct would wrongly synthesize a constructor (Value is
    // un-constructible). Naming the top of the lattice the directional `assignable` already built.
    if sup == ":wat::core::Value" {
        return true;
    }
    // Arc 278 Stone 2 — :wat::core::Never is the universal subtype-BOTTOM: Never <: every type
    // (the exact DUAL of Value's top). DOWN is free (this rule); UP stays checked — nothing is
    // <: Never except Never itself (reflexive, above). Uninhabited: it is the honest send-type of
    // a timer peer (`after` → `(Peer' :- [Never O])`), which never sends, so `send'`-to-a-timer is a
    // compile error (the wrong thing has no form). No registration: like Value, Never is an opaque
    // Path; a TypeDef::Struct would wrongly synthesize a constructor (Never is un-constructible).
    if sub == ":wat::core::Never" {
        return true;
    }
    let mut visited = std::collections::HashSet::new();
    let mut stack: Vec<String> = if let Some(parents) = env.subtype_parents(sub) {
        parents.to_vec()
    } else {
        return false;
    };
    while let Some(p) = stack.pop() {
        if p == sup {
            return true;
        }
        if visited.insert(p.clone()) {
            // Extend with p's own parents (transitive).
            if let Some(parents) = env.subtype_parents(&p) {
                for parent in parents {
                    stack.push(parent.clone());
                }
            }
        }
    }
    false
}

// ─── Typeunion member resolution (Stone 237.1) ───────────────────────────────

/// Collect the full (flattened, transitive) set of concrete member paths
/// reachable from a typeunion. Recursively expands nested typeunions.
/// Aliases are expanded via `expand_alias`. Non-Path, non-Path-via-union
/// members are emitted as-is (Parametric, Tuple).
///
/// Called from `check.rs::unify` to perform bounded-existential member
/// matching. The cycle-check at registration time bounds this walk.
pub fn collect_union_members(union: &UnionDef, env: &TypeEnv) -> Vec<TypeExpr> {
    let mut result = Vec::new();
    let mut visiting = std::collections::HashSet::new();
    for member in &union.members {
        collect_member_recursive(member, env, &mut visiting, &mut result);
    }
    result
}

fn collect_member_recursive(
    expr: &TypeExpr,
    env: &TypeEnv,
    visiting: &mut std::collections::HashSet<String>,
    out: &mut Vec<TypeExpr>,
) {
    // Expand aliases first.
    let expanded = expand_alias(expr, env);
    if let TypeExpr::Path(ref name) = expanded {
        // If the path resolves to a nested typeunion, expand it.
        if let Some(TypeDef::Union(nested)) = env.get(name) {
            if visiting.insert(name.clone()) {
                for member in &nested.members {
                    collect_member_recursive(member, env, visiting, out);
                }
                visiting.remove(name);
                return;
            }
        }
    }
    out.push(expanded);
}

#[cfg(test)]
mod tests {

    /// Arc 109 — THE SECOND WALL, and the one ordinary source can never reach.
    ///
    /// There are two refusals with near-identical opening text. The LEXER's fires on any
    /// `<` preceded by an identifier char, so every angle name a human could WRITE dies
    /// there. `parse_declared_name`'s fires on a declaration name that was never lexed —
    /// a keyword MINTED at expand time and handed straight to the type parser. It is the
    /// backstop for exactly the channel the lexer cannot see, which is why it survived the
    /// purge of every other `<`-inspecting site.
    ///
    /// This control exists because it was briefly proved and then thrown away:
    /// `STONE-reap-the-twelve` demonstrated the wall by constructing a never-lexed
    /// `WatAST::Keyword` in a temporary test, then deleted it to respect a boundary that
    /// said "do not touch this wall". A negative control that CAN be kept MUST be kept —
    /// without it, deleting the wall is a silent green.
    #[test]
    fn a_minted_declaration_name_with_angles_is_refused() {
        let span = crate::span::Span::new(std::sync::Arc::new("<test>".to_string()), 0, 0);
        // Never lexed: built directly, exactly as a macro's `keyword-node` would.
        let minted = WatAST::Keyword(":wat::core::Vector<wat::core::i64>".to_string(), span.clone());
        let err = parse_declared_name(":wat::core::defrecord", &minted, &span)
            .expect_err("a MINTED angle-bracket declaration name must be REFUSED");
        let msg = format!("{err:?}");
        assert!( // rune:lint(loose-assert) — a targeted presence over a large structured diagnostic; the assertion names the REMEDY the wall teaches, which is the whole of its contract.
            msg.contains("write `Head :- [T …]`"),
            "the wall must teach the surviving spelling, not merely refuse; got: {msg}"
        );
    }

    /// The positive twin. A bare minted name is ordinary and must pass — a wall that
    /// refused every minted declaration name would satisfy the test above and take the
    /// stdlib with it.
    #[test]
    fn a_minted_declaration_name_without_angles_is_accepted() {
        let span = crate::span::Span::new(std::sync::Arc::new("<test>".to_string()), 0, 0);
        let minted = WatAST::Keyword(":wat::core::Vector".to_string(), span.clone());
        let (name, params) = parse_declared_name(":wat::core::defrecord", &minted, &span)
            .expect("an ordinary minted declaration name must be ACCEPTED");
        assert_eq!(name, ":wat::core::Vector");
        assert!(params.is_empty(), "no binder was present; got {params:?}");
    }
    use super::*;

    /// Arc 109 one-door stone — `parametric_head_fqdn` is IDEMPOTENT, and that is
    /// load-bearing rather than a nicety.
    ///
    /// This strike DELETED two hand-rolled defensive branches — one in `check.rs`, one
    /// in `runtime.rs` — each written as `if head.starts_with(':') { head.clone() } else
    /// { format!(":{}", head) }` by an author who did not trust the bare-head invariant.
    /// Both now call this function. So if it ever regresses to a blind prepend, those two
    /// sites silently produce `"::wat::…"` — a malformed FQDN that resolves to nothing,
    /// and no other test in the corpus feeds it an already-prefixed head.
    ///
    /// A contract stated only in a doc comment is a convention. This whole stone exists
    /// because the convention rung is where new violations come from — so the contract
    /// gets a check.
    #[test]
    fn parametric_head_fqdn_is_idempotent_and_prepends_exactly_once() {
        // the ordinary case: a bare parametric head gains its colon
        assert_eq!(parametric_head_fqdn("wat::core::Vector"), ":wat::core::Vector");
        // ★ the case the deleted defensive branches existed for: already prefixed,
        //   returned UNCHANGED — never `"::wat::core::Vector"`
        assert_eq!(parametric_head_fqdn(":wat::core::Vector"), ":wat::core::Vector");
        // applying it twice is applying it once
        let once = parametric_head_fqdn("wat::kernel::Peer");
        assert_eq!(parametric_head_fqdn(&once), once);

        // and the two doors agree — `base_fqdn`'s Parametric arm must route through the
        // same implementation, not re-hand-roll the prepend.
        let parametric = TypeExpr::Parametric {
            head: "wat::core::Vector".to_string(),
            args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
        };
        assert_eq!(parametric.base_fqdn().as_deref(), Some(":wat::core::Vector"));
        // a Path already carries its colon and must not gain a second one
        assert_eq!(
            TypeExpr::Path(":wat::core::i64".to_string()).base_fqdn().as_deref(),
            Some(":wat::core::i64"),
        );
        // variants with no nameable head say so rather than fabricating one
        assert_eq!(TypeExpr::Tuple(vec![]).base_fqdn(), None);
    }

    // STONE-finish-the-param-spec (arc 109) — `peel_param_spec`'s four pinned edge
    // cases. These are exactly where the nine hand-rolled peels it replaces currently
    // differ (per the BRIEF); pinning them here is what keeps the door itself honest.
    mod peel_param_spec_tests {
        use super::*;

        fn kw(s: &str) -> WatAST {
            WatAST::Keyword(s.to_string(), crate::rust_caller_span!())
        }
        fn sym(s: &str) -> WatAST {
            WatAST::Symbol(crate::scope::Identifier::bare(s), crate::rust_caller_span!())
        }
        fn vec_of(items: Vec<WatAST>) -> WatAST {
            WatAST::Vector(items, crate::rust_caller_span!())
        }

        /// No `:-` at all — the door is a no-op: `(None, args)`, args UNCHANGED.
        #[test]
        fn no_marker_returns_none_and_original_args() {
            let args = vec![sym("x"), sym("y")];
            let (peeled, rest) = peel_param_spec(&args);
            assert!(peeled.is_none());
            assert_eq!(rest.len(), 2);
            assert!(std::ptr::eq(rest.as_ptr(), args.as_ptr()), "rest must be the SAME slice, not a copy");
        }

        /// `:- []` — the empty binder is EXPRESSED, never absent. Must be
        /// `Some(&[])`, NEVER `None` — the builder's rule this stone exists to enforce.
        #[test]
        fn empty_bracket_marker_peels_to_some_empty_never_none() {
            let args = vec![kw(":-"), vec_of(vec![])];
            let (peeled, rest) = peel_param_spec(&args);
            assert_eq!(peeled, Some(&[][..]), "`:- []` must peel to Some(&[]), not None");
            assert!(rest.is_empty());
        }

        /// `:- [T]` — the ordinary case, sanity-checked alongside the empty one.
        #[test]
        fn nonempty_bracket_marker_peels_its_contents() {
            let args = vec![kw(":-"), vec_of(vec![sym("T")]), sym("rest-arg")];
            let (peeled, rest) = peel_param_spec(&args);
            let peeled = peeled.expect("marker present, must peel Some");
            assert_eq!(peeled.len(), 1);
            assert!(matches!(&peeled[0], WatAST::Symbol(id, _) if id.as_str() == "T"));
            assert_eq!(rest.len(), 1);
            assert!(matches!(&rest[0], WatAST::Symbol(id, _) if id.as_str() == "rest-arg"));
        }

        /// `:-` immediately followed by a NON-Vector — malformed shape. Left UNPEELED
        /// (`None, args`) so the caller's own diagnostic fires naturally, rather than
        /// this door inventing a second error path (mirrors `peel_type_binder`'s
        /// pre-existing rule for this exact shape).
        #[test]
        fn marker_followed_by_non_vector_is_left_unpeeled() {
            let args = vec![kw(":-"), sym("not-a-vector")];
            let (peeled, rest) = peel_param_spec(&args);
            assert!(peeled.is_none(), "a malformed binder must not silently peel");
            assert_eq!(rest.len(), 2, "args must be returned whole, untouched");
        }

        /// `:-` as the LAST element — nothing follows it, so there is no vector to
        /// peel. Falls through the same guard as the non-Vector case: `(None, args)`.
        #[test]
        fn marker_as_last_element_is_left_unpeeled() {
            let args = vec![sym("a"), kw(":-")];
            let (peeled, rest) = peel_param_spec(&args);
            assert!(peeled.is_none());
            assert_eq!(rest.len(), 2);
        }
    }

    // Arc 115 slice 2 — verify parse_type_expr rejects illegal
    // inner-colon forms.
    #[test]
    fn arc115_inner_colon_path_rejected() {
        let r = parse_type_expr(":Vec<:String>");
        assert!(r.is_err(), "should reject :Vec<:String>; got: {:?}", r);
    }

    #[test]
    fn arc115_inner_colon_fqdn_rejected() {
        let r = parse_type_expr(":Result<:wat::core::String,:wat::kernel::ThreadDiedError>");
        assert!(r.is_err(), "should reject inner colon on FQDN args; got: {:?}", r);
    }

    #[test]
    fn arc115_inner_colon_in_fn_args_rejected() {
        let r = parse_type_expr(":fn(:i64)->bool");
        assert!(r.is_err(), "should reject inner colon on fn arg; got: {:?}", r);
    }

    #[test]
    fn arc115_inner_colon_in_fn_ret_rejected() {
        let r = parse_type_expr(":fn(i64)->:bool");
        assert!(r.is_err(), "should reject inner colon on fn ret; got: {:?}", r);
    }

    #[test]
    fn arc115_legal_compound_args_pass() {
        // Canonical forms — no inner colons. Arc 109 ③ retired angle-bracket parametrics
        // entirely: there is no flat-string spelling for them any more (the reference
        // form `(Head :- [args])` only parses from a structural `WatAST::List`, never
        // from a keyword string via `parse_type_expr`) — so this now covers only the
        // compounds that still have a legal STRING spelling: non-parametric
        // `fn(...)->...` and the native tuple `:(...)`. The angle-bracket cases this
        // used to assert as legal are covered (as REFUSALS) by
        // `angle_bracket_parametric_head_is_illegal` below.
        for input in &[":fn(i64)->bool", ":(wat::core::i64,wat::core::String)"] {
            let r = parse_type_expr(input);
            assert!(r.is_ok(), "expected {} to parse; got: {:?}", input, r);
        }
    }

    #[test]
    fn angle_bracket_parametric_head_is_illegal() {
        // Arc 109 ③ — the sibling of `arc115_legal_compound_args_pass` above: every one
        // of THAT test's former angle-bracket cases must now be refused, not accepted.
        for input in &[
            ":Vec<String>",
            ":Vec<i64>",
            ":Result<Option<i64>,wat::kernel::ThreadDiedError>",
            ":fn(Vec<String>)->Option<i64>",
            ":HashMap<String,Vec<i64>>",
        ] {
            let r = parse_type_expr(input);
            assert!(r.is_err(), "expected {} to be REFUSED; got: {:?}", input, r);
        }
    }
    fn collect(src: &str) -> Result<(TypeEnv, Vec<WatAST>), TypeError> {
        let forms = crate::parse_all!(src).expect("parse ok");
        // Arc 293 decl-a — use with_builtins() so :wat::core::Struct (the new struct
        // nature-root) is in the registry before user structs try to register against it.
        let mut env = TypeEnv::with_builtins();
        let rest = register_types(forms, &mut env)?;
        Ok((env, rest))
    }

    /// Variant for tests where the lexer may reject the source
    /// before parsing reaches the type-registration phase. Arc 072
    /// extended the lexer's bracket-depth tracking to `<>`, so
    /// malformed type-keyword brackets now surface as
    /// LexError::UnclosedBracketInKeyword rather than slipping
    /// through to a TypeError downstream.
    fn collect_lenient(src: &str) -> Result<(TypeEnv, Vec<WatAST>), String> {
        let forms = crate::parse_all!(src).map_err(|e| format!("parse: {:?}", e))?;
        // Arc 293 decl-a — use with_builtins() so :wat::core::Struct is in the registry.
        let mut env = TypeEnv::with_builtins();
        let rest = register_types(forms, &mut env).map_err(|e| format!("type: {:?}", e))?;
        Ok((env, rest))
    }

    // ─── Struct ─────────────────────────────────────────────────────────

    #[test]
    fn simple_struct() {
        // Stone 241.8 — migrated from :wat::core::struct pair-form to defstruct triples.
        let (env, rest) = collect(
            r#"(:wat::core::defstruct :project::market::Candle
                  [open  <- :wat::core::f64
                   high  <- :wat::core::f64
                   low   <- :wat::core::f64
                   close <- :wat::core::f64])"#,
        )
        .unwrap();
        assert!(rest.is_empty());
        let def = env.get(":project::market::Candle").expect("registered");
        match def {
            TypeDef::Aggregate(a) => {
                assert_eq!(a.name, ":project::market::Candle");
                assert!(a.type_params.is_empty());
                assert_eq!(a.fields.len(), 4);
                assert_eq!(a.fields[0].0, "open");
                assert_eq!(a.fields[0].1, TypeExpr::Path(":wat::core::f64".into()));
            }
            _ => panic!("expected Aggregate"),
        }
    }

    #[test]
    fn parametric_struct() {
        // Stone 241.8 — migrated from :wat::core::struct pair-form to defstruct triples.
        // Arc 109 ③ — angle-bracket decl-name retired; `Head :- [T …]` siblings instead.
        let (env, _) = collect(
            r#"(:wat::core::defstruct :my::Container :- [T]
                  [value <- :T
                   count <- :i64])"#,
        )
        .unwrap();
        let def = env.get(":my::Container").expect("registered");
        match def {
            TypeDef::Aggregate(a) => {
                assert_eq!(a.type_params, vec!["T".to_string()]);
                assert_eq!(a.fields[0].1, TypeExpr::Path(":T".into()));
            }
            _ => panic!("expected Aggregate"),
        }
    }

    #[test]
    fn parametric_struct_multiple_params() {
        // Stone 241.8 — migrated from :wat::core::struct pair-form to defstruct triples.
        // Arc 109 ③ — angle-bracket decl-name retired; `Head :- [K V]` siblings instead.
        let (env, _) = collect(
            r#"(:wat::core::defstruct :my::Pair :- [K V]
                  [key   <- :K
                   value <- :V])"#,
        )
        .unwrap();
        let def = env.get(":my::Pair").expect("registered");
        if let TypeDef::Aggregate(a) = def {
            assert_eq!(a.type_params, vec!["K".to_string(), "V".to_string()]);
        } else {
            panic!("expected Aggregate");
        }
    }

    // ─── Enum ───────────────────────────────────────────────────────────

    #[test]
    fn unit_variant_enum() {
        // Stone 241.9 — migrated from :wat::core::enum to :wat::core::defenum (HARD CUT).
        let (env, _) = collect(r#"(:wat::core::defenum :my::Direction :wat::enum::Pure :up :down :left :right)"#).unwrap();
        if let TypeDef::Enum(e) = env.get(":my::Direction").unwrap() {
            assert_eq!(e.variants.len(), 4);
            assert!(matches!(&e.variants[0], EnumVariant::Unit(n) if n == "up"));
        } else {
            panic!("expected Enum");
        }
    }

    #[test]
    fn tagged_variant_enum() {
        // Stone 241.9 — migrated to defenum positional + argspec-Vector form.
        let (env, _) = collect(
            r#"(:wat::core::defenum :my::Event :wat::enum::Pure
                  :empty
                  :candle  [open <- :f64 close <- :f64]
                  :deposit [amount <- :f64])"#,
        )
        .unwrap();
        if let TypeDef::Enum(e) = env.get(":my::Event").unwrap() {
            assert_eq!(e.variants.len(), 3);
            assert!(matches!(&e.variants[0], EnumVariant::Unit(n) if n == "empty"));
            match &e.variants[1] {
                EnumVariant::Tagged { name, fields } => {
                    assert_eq!(name, "candle");
                    assert_eq!(fields.len(), 2);
                }
                _ => panic!(),
            }
        } else {
            panic!("expected Enum");
        }
    }

    #[test]
    fn parametric_enum() {
        // Stone 241.9 — migrated to defenum form.
        // Arc 109 ③ — angle-bracket decl-name retired; `Head :- [T]` siblings instead.
        let (env, _) = collect(
            r#"(:wat::core::defenum :my::Option :- [T] :wat::enum::Pure
                  :none
                  :some [value <- :T])"#,
        )
        .unwrap();
        if let TypeDef::Enum(e) = env.get(":my::Option").unwrap() {
            assert_eq!(e.type_params, vec!["T".to_string()]);
        } else {
            panic!();
        }
    }

    #[test]
    fn empty_enum_rejected() {
        // Stone 241.9 — migrated to defenum form. Empty defenum (no variants) is rejected.
        let err = collect(r#"(:wat::core::defenum :my::Empty)"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::MalformedDecl { .. }));
    }

    // ─── Newtype ────────────────────────────────────────────────────────

    #[test]
    fn simple_newtype() {
        let (env, _) = collect(r#"(:wat::core::newtype :my::trading::Price :wat::core::f64)"#).unwrap();
        if let TypeDef::Newtype(n) = env.get(":my::trading::Price").unwrap() {
            assert_eq!(n.inner, TypeExpr::Path(":wat::core::f64".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parametric_newtype() {
        // Arc 109 ③ — angle-bracket decl-name retired; `Head :- [T]` siblings instead.
        let (env, _) = collect(r#"(:wat::core::newtype :my::Wrap :- [T] :T)"#).unwrap();
        if let TypeDef::Newtype(n) = env.get(":my::Wrap").unwrap() {
            assert_eq!(n.type_params, vec!["T".to_string()]);
            assert_eq!(n.inner, TypeExpr::Path(":T".into()));
        } else {
            panic!();
        }
    }

    // ─── Typealias ──────────────────────────────────────────────────────

    #[test]
    fn simple_typealias() {
        let (env, _) = collect(r#"(:wat::core::typealias :my::Amount :wat::core::f64)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Amount").unwrap() {
            assert_eq!(a.expr, TypeExpr::Path(":wat::core::f64".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parametric_typealias() {
        // Arc 109 ③ — angle-bracket decl-name AND reference both retired: `Head :- [T]`
        // siblings for the decl, `(Head :- [T])` in parens for the reference.
        let (env, _) = collect(r#"(:wat::core::typealias :my::Series :- [T] (:wat::core::Vector :- [T]))"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Series").unwrap() {
            assert_eq!(a.type_params, vec!["T".to_string()]);
            assert_eq!(
                a.expr,
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":T".into())]
                }
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn typealias_function_type() {
        let (env, _) = collect(r#"(:wat::core::typealias :my::Predicate :fn(wat::holon::HolonAST)->wat::core::bool)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Predicate").unwrap() {
            match &a.expr {
                TypeExpr::Fn { args, ret } => {
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                    assert_eq!(**ret, TypeExpr::Path(":wat::core::bool".into()));
                }
                other => panic!("expected Fn, got {:?}", other),
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn typealias_nested_parametric() {
        // Arc 109 ③ — angle-bracket reference retired; `(Head :- [args])` in parens. `Atom`
        // stays a bare Symbol arg (no "::"), which `parse_type_node`'s Symbol arm prepends a
        // colon to (`ns_to_wat_path`'s bare-name branch) — same `Path(":Atom")` result.
        let (env, _) = collect(
            r#"(:wat::core::typealias :my::Scores (:wat::core::HashMap :- [Atom :wat::core::f64]))"#,
        )
        .unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Scores").unwrap() {
            match &a.expr {
                TypeExpr::Parametric { head, args } => {
                    assert_eq!(head, "wat::core::HashMap");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], TypeExpr::Path(":Atom".into()));
                    assert_eq!(args[1], TypeExpr::Path(":wat::core::f64".into()));
                }
                other => panic!("expected Parametric, got {:?}", other),
            }
        } else {
            panic!();
        }
    }

    // ─── Error paths ────────────────────────────────────────────────────

    #[test]
    fn duplicate_type_rejected() {
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let err = collect(
            r#"
            (:wat::core::defstruct :my::T [x <- :f64])
            (:wat::core::defstruct :my::T [y <- :i64])
            "#,
        )
        .unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::DuplicateType { .. }));
    }

    #[test]
    fn reserved_prefix_rejected() {
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let err = collect(r#"(:wat::core::defstruct :wat::core::MyStruct [x <- :f64])"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::ReservedPrefix { .. }));

        let err = collect(r#"(:wat::core::defstruct :wat::holon::Bad [x <- :f64])"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::ReservedPrefix { .. }));

        let err = collect(r#"(:wat::core::defstruct :wat::std::Bad [x <- :f64])"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::ReservedPrefix { .. }));
    }

    #[test]
    fn dotted_name_rejected() {
        // Arc 296 stone H-1 — the dot wall. A dot in a record's name segment (the part
        // after the last `::`) would forge stone H's tagged-variant wire discriminator
        // (`#ns/Enum.Variant`), so registration refuses it authoritatively rather than
        // relying on the corpus never happening to use one.
        //
        // Row 1 (negative): a record whose name contains a dot is refused, structurally
        // on the error kind — not on message text.
        let err = collect(r#"(:wat::core::defstruct :my::Shape.Circle [x <- :f64])"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::DottedName { .. }));

        // Row 2 (positive control): an ordinary undotted record in the SAME test still
        // registers successfully. Without this row, row 1 alone cannot distinguish "the
        // dot wall works" from "registration is broken for everything."
        let (env, _) = collect(r#"(:wat::core::defstruct :my::Shape [x <- :f64])"#).unwrap();
        assert!(env.get(":my::Shape").is_some());
    }

    #[test]
    fn malformed_newtype_arity_rejected() {
        let err = collect(r#"(:wat::core::newtype :my::T)"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::MalformedDecl { .. }));
    }

    #[test]
    fn malformed_field_rejected() {
        // Stone 241.8 — migrated to defstruct; old MalformedField (pair-form) replaced by
        // MalformedDecl from parse_argspec_triples (name-not-symbol / missing-arrow variants).
        // Incomplete triple [x] fails with MalformedDecl.
        let err = collect(r#"(:wat::core::defstruct :my::T [x])"#).unwrap_err();
        assert!(matches!(err.kind(), TypeErrorKind::MalformedDecl { .. }));
    }

    #[test]
    fn malformed_parametric_name_rejected() {
        // `:my::Bad<T` (unclosed `<`) hits whitespace mid-bracket.
        // Pre-arc-072 the lexer ignored `<>` so the keyword silently
        // truncated and the resulting decl errored as a malformed
        // name. Post-arc-072 the lexer rejects at lex layer with a
        // clean diagnostic — same property (rejection) at a better
        // layer.
        // Stone 241.8 — migrated to defstruct.
        let err = collect_lenient(r#"(:wat::core::defstruct :my::Bad<T [x <- :T])"#)
            .expect_err("expected rejection");
        // Stone B: {:?} and Display now emit EDN (not human prose). The error kind
        // is #wat.parse/Lex (a lex error at the keyword with whitespace inside `<`).
        // rune:lint(loose-assert) — EDN embeds variable Rust source path/line from the parser's
        // LexError construction site; tag discriminant is the stable contract
        assert!(
            err.contains("#wat.parse/Lex")
                || err.contains("MalformedName")
                || err.contains("MalformedDecl"),
            "expected lex or type-decl error, got: {}",
            err
        );
    }

    // ─── Non-type forms pass through ────────────────────────────────────

    #[test]
    fn non_type_forms_preserved() {
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let (_env, rest) = collect(
            r#"
            (:wat::core::defstruct :my::T [x <- :f64])
            (:wat::holon::Atom "hello")
            42
            "#,
        )
        .unwrap();
        assert_eq!(rest.len(), 2);
    }

    // ─── TypeExpr standalone parser ─────────────────────────────────────

    #[test]
    fn type_expr_path() {
        assert_eq!(
            parse_type_expr(":wat::core::f64").unwrap(),
            TypeExpr::Path(":wat::core::f64".into())
        );
        assert_eq!(
            parse_type_expr(":my::ns::MyType").unwrap(),
            TypeExpr::Path(":my::ns::MyType".into())
        );
    }

    #[test]
    fn type_expr_parametric() {
        // Arc 109 ③ — angle brackets have no flat-string spelling any more; `parse_type_expr`
        // (a `&str -> TypeExpr` fn) can no longer express a parametric reference at all. The
        // reference FORM `(Head :- [args])` only parses from a structural `WatAST::List` —
        // build one via `parse_one!` (real wat source syntax) and route it through
        // `parse_type_node`, the substrate's one door for every annotation-slot node shape.
        let form = crate::parse_one!("(:wat::core::Vector :- [:T])").unwrap();
        assert_eq!(
            parse_type_node(&form).unwrap(),
            TypeExpr::Parametric {
                head: "wat::core::Vector".into(),
                args: vec![TypeExpr::Path(":T".into())]
            }
        );
    }

    #[test]
    fn type_expr_parametric_nested() {
        // Arc 109 ③ — same structural-form migration as `type_expr_parametric` above; the
        // inner `fn(i32)->i32` stays string-spelled (non-parametric fn args are still legal
        // in the flat form) as one arg of the outer reference form.
        let form =
            crate::parse_one!("(:wat::core::HashMap :- [:wat::core::String :fn(i32)->i32])")
                .unwrap();
        let t = parse_type_node(&form).unwrap();
        match t {
            TypeExpr::Parametric { head, args } => {
                assert_eq!(head, "wat::core::HashMap");
                assert_eq!(args.len(), 2);
                match &args[1] {
                    TypeExpr::Fn { args: fn_args, ret } => {
                        assert_eq!(fn_args.len(), 1);
                        assert_eq!(fn_args[0], TypeExpr::Path(":i32".into()));
                        assert_eq!(**ret, TypeExpr::Path(":i32".into()));
                    }
                    _ => panic!("expected inner fn"),
                }
            }
            _ => panic!("expected outer Parametric"),
        }
    }

    #[test]
    fn type_expr_fn_no_args() {
        let t = parse_type_expr(":fn()->wat::holon::HolonAST").unwrap();
        match t {
            TypeExpr::Fn { args, ret } => {
                assert!(args.is_empty());
                assert_eq!(*ret, TypeExpr::Path(":wat::holon::HolonAST".into()));
            }
            _ => panic!(),
        }
    }

    // ─── Tuple literal types ────────────────────────────────────────────

    #[test]
    fn type_expr_tuple_unit() {
        // :() is the unit / 0-tuple.
        let t = parse_type_expr(":()").unwrap();
        match t {
            TypeExpr::Tuple(elements) => assert!(elements.is_empty()),
            other => panic!("expected Tuple([]), got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_pair() {
        let t = parse_type_expr(":(wat::core::i64,wat::core::String)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], TypeExpr::Path(":wat::core::i64".into()));
                assert_eq!(elements[1], TypeExpr::Path(":wat::core::String".into()));
            }
            other => panic!("expected Tuple(i64,String), got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_triple() {
        let t = parse_type_expr(":(Holon,wat::holon::HolonAST,Holon)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => assert_eq!(elements.len(), 3),
            other => panic!("expected 3-tuple, got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_one_element_is_grouping() {
        // :(T) is Rust grouping — flattens to T (not a 1-tuple).
        let t = parse_type_expr(":(wat::core::i64)").unwrap();
        assert_eq!(t, TypeExpr::Path(":wat::core::i64".into()));
    }

    #[test]
    fn type_expr_tuple_one_element_trailing_comma_is_tuple() {
        // :(T,) is the explicit 1-tuple.
        let t = parse_type_expr(":(wat::core::i64,)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], TypeExpr::Path(":wat::core::i64".into()));
            }
            other => panic!("expected 1-tuple, got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_with_nested_parametric_is_now_illegal() {
        // Arc 109 ③ — this used to assert `:(Vec<i64>,HashMap<String,i64>)` parses (nested
        // commas at depth > 0 must not split the outer tuple). A tuple element is parsed
        // from a raw STRING (`parse_tuple_body` -> `parse_type_inner` per element), which has
        // no path to the `(Head :- [args])` reference FORM at all — so a parametric tuple
        // element has no legal spelling any more, string or structural (the whole tuple would
        // need to move to `(:wat::core::Tuple :- [...])`, and even that inherits `parse_type_
        // node` per element, but this fn (`parse_type_expr`) only reads keyword STRINGS).
        // The comma-depth-tracking coverage `parse_tuple_body` still needs is carried by
        // `type_expr_tuple_with_nested_tuple` below instead (nested PARENS, still legal).
        let r = parse_type_expr(":(Vec<i64>,HashMap<String,i64>)");
        assert!(r.is_err(), "expected angle-bracket tuple element to be REFUSED; got: {:?}", r);
    }

    #[test]
    fn type_expr_tuple_with_nested_tuple() {
        // The comma-depth-tracking coverage the retired `type_expr_tuple_with_nested_parametric`
        // carried, over a shape that is STILL legal: nested tuples via parens. Nested commas at
        // depth > 0 (inside either inner tuple) must not split the outer tuple.
        let t = parse_type_expr(":((wat::core::i64,wat::core::String),(wat::core::bool,wat::core::f64))").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(&elements[0], TypeExpr::Tuple(inner) if inner.len() == 2));
                assert!(matches!(&elements[1], TypeExpr::Tuple(inner) if inner.len() == 2));
            }
            other => panic!("expected 2-tuple of tuples, got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_malformed_rejected() {
        // Missing closing ')'.
        assert!(parse_type_expr(":(i64,String").is_err());
    }

    #[test]
    fn type_expr_tuple_with_fn_element_arrow_not_a_bracket_close() {
        // Arc 170 W2 regression — a `Fn(...)->T` element in a NON-final tuple position.
        // Before the fix, `parse_type_list` decremented `depth` on the `>` of the `->` arrow,
        // underflowing to -1, so the comma AFTER the arrow was never seen as a top-level split:
        // the whole tail collapsed into one opaque `Path("wat::core::Fn(wat::core::i64)->wat::core::i64,wat::core::i64")`.
        // It must parse as a 2-element Tuple: [Fn(i64)->i64, i64].
        let t = parse_type_expr(":(wat::core::Fn(wat::core::i64)->wat::core::i64,wat::core::i64)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 2, "Fn(...)->T arrow must not swallow the trailing comma: {elements:?}");
                match &elements[0] {
                    TypeExpr::Fn { args, ret } => {
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], TypeExpr::Path(":wat::core::i64".into()));
                        assert_eq!(**ret, TypeExpr::Path(":wat::core::i64".into()));
                    }
                    other => panic!("expected element 0 = Fn(i64)->i64, got {other:?}"),
                }
                assert_eq!(elements[1], TypeExpr::Path(":wat::core::i64".into()));
            }
            other => panic!("expected 2-tuple (Fn(i64)->i64, i64), got {other:?}"),
        }
    }

    // ─── Arc 032 — :wat::holon::BundleResult builtin ────────────────

    #[test]
    fn bundle_result_alias_registered_with_builtins() {
        let env = TypeEnv::with_builtins();
        let def = env
            .get(":wat::holon::BundleResult")
            .expect(":wat::holon::BundleResult registered via with_builtins");
        match def {
            TypeDef::Alias(a) => {
                assert_eq!(a.name, ":wat::holon::BundleResult");
                assert!(a.type_params.is_empty(), "non-parametric alias");
                match &a.expr {
                    TypeExpr::Parametric { head, args } => {
                        assert_eq!(head, "wat::core::Result");
                        assert_eq!(args.len(), 2);
                        assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                        assert_eq!(
                            args[1],
                            TypeExpr::Path(":wat::holon::CapacityExceeded".into())
                        );
                    }
                    other => panic!("expected Result<_,_>, got {:?}", other),
                }
            }
            other => panic!("expected TypeDef::Alias, got {:?}", other),
        }
    }

    #[test]
    fn bundle_result_alias_expands_to_expected_result() {
        let env = TypeEnv::with_builtins();
        let alias_ref = TypeExpr::Path(":wat::holon::BundleResult".into());
        let expanded = expand_alias(&alias_ref, &env);
        match expanded {
            TypeExpr::Parametric { head, args } => {
                assert_eq!(head, "wat::core::Result");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                assert_eq!(args[1], TypeExpr::Path(":wat::holon::CapacityExceeded".into()));
            }
            other => panic!("expected expanded Result<HolonAST,CapacityExceeded>, got {:?}", other),
        }
    }

    // ─── Arc 033 — :wat::holon::Holons builtin ─────────────────────

    #[test]
    fn holons_alias_registered_with_builtins() {
        let env = TypeEnv::with_builtins();
        let def = env
            .get(":wat::holon::Holons")
            .expect(":wat::holon::Holons registered via with_builtins");
        match def {
            TypeDef::Alias(a) => {
                assert_eq!(a.name, ":wat::holon::Holons");
                assert!(a.type_params.is_empty(), "non-parametric alias");
                match &a.expr {
                    TypeExpr::Parametric { head, args } => {
                        assert_eq!(head, "wat::core::Vector");
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                    }
                    other => panic!("expected Vec<_>, got {:?}", other),
                }
            }
            other => panic!("expected TypeDef::Alias, got {:?}", other),
        }
    }

    #[test]
    fn holons_alias_expands_to_expected_vec() {
        let env = TypeEnv::with_builtins();
        let alias_ref = TypeExpr::Path(":wat::holon::Holons".into());
        let expanded = expand_alias(&alias_ref, &env);
        match expanded {
            TypeExpr::Parametric { head, args } => {
                assert_eq!(head, "wat::core::Vector");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
            }
            other => panic!("expected expanded Vec<HolonAST>, got {:?}", other),
        }
    }

    // ─── Arc 138 slice 2 — TypeError carries source coordinates ────
    //
    // Canary: a TypeError surfaced from user source MUST render with
    // `<file>:<line>:<col>:` as the leading prefix so consumers (humans
    // + agents) navigate straight to the offending decl. Mirrors
    // `check::tests::type_mismatch_message_carries_span`.
    #[test]
    fn arc138_type_error_message_carries_span() {
        // Stone 241.9 — migrated to defenum. `:my::Empty` is a defenum with no variants —
        // fires MalformedDecl. The form's outer span gets threaded all the way to the Display
        // arm via `decl_span`.
        let err = collect(r#"(:wat::core::defenum :my::Empty)"#).unwrap_err();
        let rendered = format!("{}", err);
        // rune:lint(loose-assert) — variable Rust source file path embedded in error Display output (varies by build environment)
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "expected TypeError Display to carry real source coordinates (file:line:col); got: {}",
            rendered
        );
        assert!(
            matches!(err.kind(), TypeErrorKind::MalformedDecl { .. }),
            "expected MalformedDecl, got: {:?}",
            err
        );
    }

    // ─── Arc 278 #16 Stone 16.1c — the ruling-A CONTRACT LOCK ────────────────────
    //
    // `synthesize_surface_protocol` enforces: every serviceable op-Response must be an
    // outcome ENUM carrying a well-shaped `RequestTooLarge [bytes <- i64  cap <- i64]`
    // variant. A `:nature :wat::kernel::Peer` surface OWNS its protocol types in a
    // mandatory `:messages` block; `expand_all`'s `hoist_surface_messages` lifts those
    // decls to the top-level form stream AHEAD of the surface form, so they are registered
    // in `env` before `synthesize_surface_protocol` runs. `expand_then_register` mirrors
    // that production pipeline (expand_all → register_types).
    //
    // Each RED error FIRING is itself the STOP-1 confirmation: the Aggregate branch only
    // fires when `env.get(ret)` returned `Some(Aggregate)`, the Enum branch only when it
    // returned `Some(Enum)` — i.e. the Response resolves at synthesize time (enums, unlike
    // the retired record Responses, are in `env` when the surface's protocol synthesizes).

    /// Mirror the production surface pipeline: parse → register defmacros → `expand_all`
    /// (which hoists a peer surface's `:messages` decls ahead of the surface form) →
    /// `register_types` (where `synthesize_surface_protocol`'s ruling-A lock lives).
    fn expand_then_register(src: &str) -> Result<TypeEnv, TypeError> {
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut reg = crate::macros::MacroRegistry::new();
        let rest = crate::macros::register_defmacros(forms, &mut reg)
            .expect("register_defmacros ok");
        let renv = crate::runtime::Environment::default();
        let sym = crate::runtime::SymbolTable::default();
        let expanded = crate::macros::expand_all(rest, &mut reg, &renv, &sym)
            .expect("expand_all ok");
        let mut env = TypeEnv::with_builtins();
        register_types(expanded, &mut env)?;
        Ok(env)
    }

    #[test]
    fn stone_16_1c_record_response_is_a_located_error() {
        // An op whose `<Op>Response` is a RECORD (records-as-Responses are retired for
        // services) is a located ruling-A error. Also confirms STOP-1: env.get(ret)
        // resolved to Aggregate at synthesize time (only then does the record branch fire).
        let err = expand_then_register(
            r#"(:wat::core::defsurface :t::Bad :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Bad::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::recordtype :t::Bad::FooResponse :wat::core::Record
                                [ok <- :wat::core::String])]
                  :features [(foo [self <- :t::Bad  req <- :t::Bad::FooRequest]
                               -> :t::Bad::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect_err("a record-typed op-Response must be a located ruling-A error");
        match err.kind() {
            TypeErrorKind::MalformedVariant { enum_name, offending, .. } => {
                assert_eq!(enum_name, ":t::Bad::FooResponse");
                assert_eq!(offending, "RequestTooLarge");
            }
            other => panic!("expected MalformedVariant (records-retired); got {other:?}"),
        }
    }

    #[test]
    fn stone_16_1c_enum_response_missing_rtl_is_a_located_error() {
        // An outcome enum that omits the `RequestTooLarge` variant is a located ruling-A
        // error. Also confirms STOP-1: env.get(ret) resolved to Enum at synthesize time.
        let err = expand_then_register(
            r#"(:wat::core::defsurface :t::Bad2 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Bad2::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Bad2::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String])]
                  :features [(foo [self <- :t::Bad2  req <- :t::Bad2::FooRequest]
                               -> :t::Bad2::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect_err("an enum Response lacking RequestTooLarge must be a located ruling-A error");
        match err.kind() {
            TypeErrorKind::MalformedVariant { enum_name, offending, .. } => {
                assert_eq!(enum_name, ":t::Bad2::FooResponse");
                assert_eq!(offending, "RequestTooLarge");
            }
            other => panic!("expected MalformedVariant (missing RequestTooLarge); got {other:?}"),
        }
    }

    #[test]
    fn stone_16_1c_enum_response_malformed_rtl_is_a_located_error() {
        // A `RequestTooLarge` variant whose fields are the WRONG shape (String, not i64) is
        // NOT well-shaped → located ruling-A error. Locks the field-shape, not just the name.
        let err = expand_then_register(
            r#"(:wat::core::defsurface :t::Bad3 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Bad3::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Bad3::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String]
                                :RequestTooLarge [bytes <- :wat::core::String  cap <- :wat::core::String])]
                  :features [(foo [self <- :t::Bad3  req <- :t::Bad3::FooRequest]
                               -> :t::Bad3::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect_err("a mis-shaped RequestTooLarge (non-i64 fields) must be a located error");
        match err.kind() {
            TypeErrorKind::MalformedVariant { enum_name, offending, .. } => {
                assert_eq!(enum_name, ":t::Bad3::FooResponse");
                assert_eq!(offending, "RequestTooLarge");
            }
            other => panic!("expected MalformedVariant (malformed RequestTooLarge); got {other:?}"),
        }
    }

    #[test]
    fn stone_2_enum_response_missing_request_malformed_is_a_located_error() {
        // Arc 278 Stone 2 — the SHAPE half of the lock, the exact standing ruling A gave the
        // SIZE half. An outcome enum that carries `RequestTooLarge` but omits
        // `RequestMalformed` is a located error: `defservice` generates the request-shape
        // guard into every op arm UNCONDITIONALLY (there is no clause to opt into and no
        // default to flip), and that guard refuses with this variant. Omitting it would leave
        // the generated code referencing a variant that does not exist.
        let err = expand_then_register(
            r#"(:wat::core::defsurface :t::Bad4 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Bad4::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Bad4::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String]
                                :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
                  :features [(foo [self <- :t::Bad4  req <- :t::Bad4::FooRequest]
                               -> :t::Bad4::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect_err("an enum Response lacking RequestMalformed must be a located error");
        match err.kind() {
            TypeErrorKind::MalformedVariant { enum_name, offending, .. } => {
                assert_eq!(enum_name, ":t::Bad4::FooResponse");
                assert_eq!(offending, "RequestMalformed");
            }
            other => panic!("expected MalformedVariant (missing RequestMalformed); got {other:?}"),
        }
    }

    #[test]
    fn stone_2_malformed_request_malformed_shape_is_a_located_error() {
        // The field SHAPE is locked, not just the name. `path` must be
        // `(Vector :- [String])` — the structured coordinate a caller indexes and walks. A `path`
        // rendered as a flat String would collapse the one field of this variant that is real
        // DATA rather than a rendering, so it is refused.
        let err = expand_then_register(
            r#"(:wat::core::defsurface :t::Bad5 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Bad5::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Bad5::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String]
                                :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                :RequestMalformed [path     <- :wat::core::String
                                                   expected <- :wat::core::String
                                                   got      <- :wat::core::String])]
                  :features [(foo [self <- :t::Bad5  req <- :t::Bad5::FooRequest]
                               -> :t::Bad5::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect_err("a mis-shaped RequestMalformed (String path) must be a located error");
        match err.kind() {
            TypeErrorKind::MalformedVariant { enum_name, offending, .. } => {
                assert_eq!(enum_name, ":t::Bad5::FooResponse");
                assert_eq!(offending, "RequestMalformed");
            }
            other => panic!("expected MalformedVariant (malformed RequestMalformed); got {other:?}"),
        }
    }

    #[test]
    fn stone_2_request_malformed_path_accepts_the_canonical_vector_spelling() {
        // Arc 109 "THE LAST DOORS" retired the bare parametric FORM
        // `(:wat::core::Vector :wat::core::String)` this test used to exercise — the type
        // position now accepts only the `:-` marker (door 1). `rm_fields` is built by
        // PARSING the canonical spelling rather than hand-assembling the `TypeExpr`, so this
        // still proves the lock accepts a parsed `Vector<String>` for `path` — it is now
        // exactly the same fixture as `stone_16_1c_wellshaped_enum_response_passes_and_synthesizes`
        // below, kept as an independent regression anchor for this lock specifically.
        expand_then_register(
            r#"(:wat::core::defsurface :t::Ok2 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Ok2::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Ok2::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String]
                                :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                                                   expected <- :wat::core::String
                                                   got      <- :wat::core::String])]
                  :features [(foo [self <- :t::Ok2  req <- :t::Ok2::FooRequest]
                               -> :t::Ok2::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect("the canonical Vector<String> spelling must clear the lock");
    }

    #[test]
    fn stone_16_1c_wellshaped_enum_response_passes_and_synthesizes() {
        // The GREEN half: a conforming outcome enum (`:Ok | :RequestTooLarge [bytes cap] |
        // :RequestMalformed [path expected got]`) clears BOTH halves of the lock, and the
        // protocol enums `::Op` / `::Reply` synthesize as before. Proves the lock does not
        // false-positive on the migrated (conforming) fleet.
        let env = expand_then_register(
            r#"(:wat::core::defsurface :t::Ok1 :nature :wat::kernel::Peer
                  :messages [(:wat::core::recordtype :t::Ok1::FooRequest :wat::core::Record
                                [x <- :wat::core::String])
                             (:wat::core::defenum :t::Ok1::FooResponse :wat::enum::Pure
                                :Ok [reply <- :wat::core::String]
                                :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                                                   expected <- :wat::core::String
                                                   got      <- :wat::core::String])]
                  :features [(foo [self <- :t::Ok1  req <- :t::Ok1::FooRequest]
                               -> :t::Ok1::FooResponse :max-request-bytes 524288)])"#,
        )
        .expect("a conforming outcome-enum Response must clear the ruling-A lock");
        assert!(
            matches!(env.get(":t::Ok1::Op"), Some(TypeDef::Enum(_))),
            "the synthesized `::Op` protocol enum must exist"
        );
        assert!(
            matches!(env.get(":t::Ok1::Reply"), Some(TypeDef::Enum(_))),
            "the synthesized `::Reply` protocol enum must exist"
        );
    }

    // ── Stone 255-builtin-registry — the door tells the truth for builtin leaves ──

    /// Acceptance row 1★ — THE DOOR (`SymbolTable::registrations`), not the new
    /// `TypeEnv` field. Testing `TypeEnv::contains` directly would prove only
    /// that the store works, not that the door — the entire point of the
    /// ruling — sees it.
    #[test]
    fn stone_255b_row1_door_answers_type_for_bare_primitive() {
        let mut sym = crate::value::SymbolTable::new();
        sym.set_types(std::sync::Arc::new(TypeEnv::with_builtins()));
        let regs = sym.registrations(":wat::core::i64");
        assert!(
            regs.contains(crate::value::symbol_table::RegistryKind::Type),
            "registrations(\":wat::core::i64\") = {regs:?}, expected it to contain Type"
        );
    }

    /// Acceptance row 2 — a container (derived from `BARE_CONTAINER_HEADS`), an
    /// opaque capability type, and a rust-backed type, all through THE DOOR.
    #[test]
    fn stone_255b_row2_door_answers_type_for_container_opaque_and_rust_backed() {
        let mut sym = crate::value::SymbolTable::new();
        sym.set_types(std::sync::Arc::new(TypeEnv::with_builtins()));
        for name in [
            ":wat::core::Vector",
            ":wat::kernel::Peer",
            ":rust::crossbeam_channel::Sender",
        ] {
            let regs = sym.registrations(name);
            assert!(
                regs.contains(crate::value::symbol_table::RegistryKind::Type),
                "registrations({name:?}) = {regs:?}, expected it to contain Type"
            );
        }
    }

    /// Acceptance row 3★★ — the NEGATIVE control. This is the only row that can
    /// distinguish a genuinely populated registry from a `contains` that says
    /// yes to everything: rows 1, 2 and 5 are all positives.
    #[test]
    fn stone_255b_row3_negative_control_unknown_name_is_unregistered() {
        let mut sym = crate::value::SymbolTable::new();
        sym.set_types(std::sync::Arc::new(TypeEnv::with_builtins()));
        let regs = sym.registrations(":user::NoSuchType");
        assert!(
            regs.is_empty(),
            "registrations(\":user::NoSuchType\") = {regs:?}, expected empty — \
             a non-empty result here means `contains` answers yes unconditionally"
        );
    }

    /// Acceptance row 4★ — membership WITHOUT structure, asserted as a test
    /// rather than left as a comment. This is also the guard against a future
    /// "fix" that makes `get` fabricate a `TypeDef` for a builtin leaf — doing
    /// so would be building option A (a new `TypeDef` variant) by accident,
    /// which was rejected in the DESIGN.
    #[test]
    fn stone_255b_row4_get_stays_none_for_builtin_leaf() {
        let mut sym = crate::value::SymbolTable::new();
        sym.set_types(std::sync::Arc::new(TypeEnv::with_builtins()));
        // Membership, asked through THE DOOR — the same way row 1 asks it, and the
        // way the ruling says every consumer should. (It also keeps `contains` with a
        // string-literal argument out of an `assert!`, which `no_loose_string_assert`
        // cannot distinguish from `String::contains` — see the lint finding in this
        // stone's SCORE.)
        let regs = sym.registrations(":wat::core::i64");
        assert!(
            regs.contains(crate::value::symbol_table::RegistryKind::Type),
            "membership must exist first; registrations = {regs:?}"
        );
        // …and STRUCTURE is still absent. This is the asymmetry the stone exists to
        // make queryable, asserted rather than merely commented.
        let env = TypeEnv::with_builtins();
        assert_eq!(
            env.get(":wat::core::i64"),
            None,
            "get must stay None — a builtin leaf has membership, not structure"
        );
    }

    /// Acceptance row 5 — the DERIVED gate. Reads `BARE_PRIMITIVES` and
    /// `BARE_CONTAINER_HEADS` directly (never a transcribed copy), so this test
    /// cannot drift from `check.rs`'s own source of truth: any future addition
    /// to either const is automatically covered.
    #[test]
    fn stone_255b_row5_every_bare_primitive_and_container_head_is_registered() {
        let env = TypeEnv::with_builtins();
        for (bare, fqdn) in crate::check::BARE_PRIMITIVES {
            assert!(
                env.contains(fqdn),
                "BARE_PRIMITIVES entry {bare:?} -> {fqdn:?} must be contains-true"
            );
        }
        for (bare, fqdn) in crate::check::BARE_CONTAINER_HEADS {
            // BARE_CONTAINER_HEADS's FQDN column carries NO leading colon —
            // TypeExpr::Parametric.head's convention, not the registry's.
            let colon_fqdn = format!(":{fqdn}");
            assert!(
                env.contains(&colon_fqdn),
                "BARE_CONTAINER_HEADS entry {bare:?} -> {fqdn:?} must be contains-true as {colon_fqdn:?}"
            );
        }
    }

    /// Every group-3 name this stone registers must answer `contains`-true and
    /// `get`-None — the asymmetry holds uniformly, not just for the row-1/row-4
    /// examples above. `:wat::core::Never` is deliberately absent from this
    /// list (STOP-2: no genuine corpus type-position usage found).
    #[test]
    fn stone_255b_group3_opaques_are_membership_without_structure() {
        let env = TypeEnv::with_builtins();
        for name in [
            ":wat::core::bigint",
            ":wat::core::rational",
            ":wat::core::keyword",
            ":wat::holon::HolonAST",
            ":wat::WatAST",
            ":wat::core::Value",
            ":wat::core::List",
            ":wat::core::Uuid",
            ":wat::holon::Hologram",
            ":wat::holon::Vector",
            ":wat::io::IOReader",
            ":wat::io::IOWriter",
            ":wat::kernel::Process",
            ":wat::kernel::Thread",
            ":wat::kernel::Address",
            ":wat::kernel::Listener",
            ":wat::kernel::Peer",
            ":wat::kernel::ThreadSelfPeer",
            ":wat::stream::Stream",
            ":wat::time::Duration",
            ":wat::time::Instant",
            ":rust::crossbeam_channel::Sender",
            ":rust::crossbeam_channel::Receiver",
        ] {
            assert!(env.contains(name), "{name:?} must be contains-true");
            assert_eq!(env.get(name), None, "{name:?} must be get-None (membership, not structure)");
        }
    }

    /// STOP-2 documented as a test: `:wat::core::Never` was evaluated against
    /// the same corpus bar as every other group-3 name and did not clear it
    /// (its only non-comment `.wat`-adjacent mention is a Rust-internal
    /// synthesized `TypeExpr::Path`, never a user-written type position) — so
    /// it is NOT registered, and this asserts that absence stays deliberate
    /// rather than silently drifting true on a future, unrelated change.
    #[test]
    fn stone_255b_never_is_deliberately_unregistered() {
        let mut sym = crate::value::SymbolTable::new();
        sym.set_types(std::sync::Arc::new(TypeEnv::with_builtins()));
        let regs = sym.registrations(":wat::core::Never");
        assert!(
            regs.is_empty(),
            "`:wat::core::Never` was refused registration (STOP-2, no corpus \
             type-position citation) — registrations = {regs:?}. If this is no longer \
             empty someone registered it; update this test deliberately, don't let it \
             happen by drift"
        );
    }
}
