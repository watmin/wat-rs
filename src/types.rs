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
//! Parametric polymorphism (058-030 Q1 resolved YES): the name keyword
//! may carry a `<T,U,V>` suffix declaring type parameters. Example:
//! `:my::Wrapper<T>` declares a type with one type variable `T`.
//!
//! # What this slice does
//!
//! - Classifies each declaration form at startup.
//! - Extracts the name, type parameters, and structural shape (field
//!   name/type pairs, enum variants).
//! - Parses type expressions (`:f64`, `:Vec<T>`, `:fn(T,U)->R`,
//!   `:my::ns::MyType`) into structured [`TypeExpr`] values.
//! - Stores the result in a [`TypeEnv`], keyed by the bare declaration
//!   name (no `<T>` in the key — parametric types are registered once;
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

/// Arc 215 stone 1 — type-placeholder path for HM-style inference.
///
/// Appears in type-arg slots of parametric constructor calls to signal
/// "infer this type from the values." Used by:
///
/// - `{...}` map literals (V slot) — desugar emits `:wat::type::Infer`
///   as V; `infer_hashmap_constructor` detects it and uses `fresh.fresh()`.
/// - `#{...}` set literals (T slot) — desugar emits `:wat::type::Infer`
///   as T; `infer_hashset_constructor` detects it and uses `fresh.fresh()`.
/// - Explicit verb-form with inference: `(:wat::core::HashMap :wat::core::keyword
///   :wat::type::Infer :k v)` — K is explicit, V is inferred.
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
    /// `:wat::core::Vector<T>`, `:wat::core::HashMap<K,V>`, `:my::ns::Container<wat::holon::HolonAST,f64>`.
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

/// Struct declaration — named product type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, TypeExpr)>,
    /// Arc 203 — `None` for plain `:wat::core::struct` declarations;
    /// `Some(_)` for `:wat::core::struct-restricted` declarations.
    /// When present, `register_struct_methods` writes the ctor + per-field
    /// whitelists into `SymbolTable.binding_metadata` (under `:restricted-to`)
    /// alongside the synthesized Function entries (Stone 241.14 — migrated
    /// from the deleted `defined_value_restrictions` field).
    pub restrictions: Option<StructRestrictions>,
}

/// Enum declaration — coproduct type. Variants are either unit
/// (payload-free) or tagged (with named typed fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
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
/// (e.g. `typeunion :Result<T,E>`); arc 237 ships non-parametric only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<TypeExpr>,
}

/// Record class declaration — Stone S-B.1.
///
/// `(:wat::core::recordtype :my::Circle :wat::Record ["field1" "field2"])` declares
/// a record class as a real `TypeDef` so it inherits the type system's uniform
/// services: ∀T `is-<Name>?` synthesis + `typesub` hierarchy membership.
/// Field names are a CLASS property (Ruby model: class defines attrs, instance
/// holds values); `struct_form` stays positional `Arc<Vec<Value>>`.
/// NOT fed to `register_struct_methods` — dedicated kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub parent: String,
    /// Field names in declaration order. Empty for zero-field records.
    /// Name-based access (keyword-accessor, assoc, record->map) looks up
    /// the index here, then reads/writes `struct_form[index]`.
    pub field_names: Vec<String>,
    /// Field types in declaration order, parallel to `field_names`.
    /// Populated when the `recordtype` form uses the typed-field syntax
    /// `[name <- :type ...]`; `None` when the string-literal syntax
    /// `["name" ...]` is used (type information not available at that layer).
    pub field_types: Option<Vec<TypeExpr>>,
}

/// Surface declaration — structural interface (arc 293.3-core).
///
/// `(:wat::core::defsurface :Name [member <- :T ...])` declares a named
/// structural surface. A struct (or record, future arc) satisfies a surface
/// by having every member with a field-type assignable to the member's type
/// (row-polymorphic width subtyping — extra fields are fine). No `:satisfies`,
/// no `:parent`, no declaration at the use site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<(String, TypeExpr)>,
}

/// One of the seven declaration variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
    /// Stone 237.1 — named bounded set of types for bounded-existential
    /// unification. See [`UnionDef`].
    Union(UnionDef),
    /// Stone S-B.1 — record class as a real TypeDef. See [`RecordDef`].
    Record(RecordDef),
    /// Arc 293.3-core — structural surface for row-polymorphic width subtyping.
    Surface(SurfaceDef),
}

impl TypeDef {
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Struct(s) => &s.name,
            TypeDef::Enum(e) => &e.name,
            TypeDef::Newtype(n) => &n.name,
            TypeDef::Alias(a) => &a.name,
            TypeDef::Union(u) => &u.name,
            // Stone S-B.1
            TypeDef::Record(r) => &r.name,
            // Arc 293.3-core
            TypeDef::Surface(s) => &s.name,
        }
    }
}

/// Keyword-path ↦ `TypeDef` registry.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    types: HashMap<String, TypeDef>,
    /// Stone S-A — the `typesub` child→parent edge registry.
    /// Maps a child FQDN (e.g. `":wat::holon::Record"`) to the list of its direct
    /// parent FQDNs (e.g. `[":wat::Record"]`). Populated by `register_subtype`;
    /// walked (transitively) by `is_subtype`. Distinct from `typeunion` membership:
    /// this is the Clojure `derive`/`isa?` axis — an open directional is-a hierarchy.
    subtype_edges: HashMap<String, Vec<String>>,
}

/// Distinguishes user-source registration (subject to reserved-prefix gate)
/// from stdlib registration (privileged to register `:wat::*` directly).
enum RegistrationPrivilege {
    User,
    Stdlib,
}

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

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
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
        self.register_with_span(def, Span::unknown())
    }

    /// Arc 138 slice 2 — span-carrying variant. The decl's name keyword
    /// span surfaces through `ReservedPrefix` / `DuplicateType` /
    /// `CyclicAlias` errors so consumers (humans + agents) navigate to
    /// the offending decl.
    pub fn register_with_span(&mut self, def: TypeDef, span: Span) -> Result<(), TypeError> {
        self.register_validated(def, span, RegistrationPrivilege::User)
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
        self.register_stdlib_with_span(def, Span::unknown())
    }

    /// Arc 138 slice 2 — span-carrying variant of [`Self::register_stdlib`].
    pub fn register_stdlib_with_span(
        &mut self,
        def: TypeDef,
        span: Span,
    ) -> Result<(), TypeError> {
        self.register_validated(def, span, RegistrationPrivilege::Stdlib)
    }

    /// Shared guard chain for [`register_with_span`] and
    /// [`register_stdlib_with_span`]. The `privilege` parameter
    /// distinguishes the stdlib path (which bypasses the reserved-prefix
    /// check because stdlib types ARE in the reserved namespace).
    fn register_validated(
        &mut self,
        def: TypeDef,
        span: Span,
        privilege: RegistrationPrivilege,
    ) -> Result<(), TypeError> {
        let name = def.name().to_string();
        if matches!(privilege, RegistrationPrivilege::User)
            && crate::resolve::is_reserved_prefix(&name)
        {
            return Err(TypeError { span, kind: TypeErrorKind::ReservedPrefix { name } });
        }
        // Arc 054: idempotent re-declaration. If the same name is already
        // registered with a byte-equivalent definition, the second
        // registration is a no-op. Divergent re-declarations remain an
        // error. Unblocks in-crate shims whose wat surface is delivered
        // both via `wat_sources()` and on-disk loading (the natural
        // pattern for lab-side shims like CandleStream).
        if let Some(existing) = self.types.get(&name) {
            if existing == &def {
                return Ok(());
            }
            return Err(TypeError { span, kind: TypeErrorKind::DuplicateType { name } });
        }
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
        // Stone S-B.1 — record type: verify parent is known, wire subtype edge.
        if let TypeDef::Record(rec) = &def {
            let parent = rec.parent.clone();
            // Parent must already be registered in the TypeEnv. The built-in
            // roots `:wat::Record` and `:wat::holon::Record` are registered as
            // opaque TypeDef::Struct entries, so `contains_key` is the right gate.
            if !self.types.contains_key(&parent) {
                return Err(TypeError {
                    span,
                    kind: TypeErrorKind::MalformedDecl {
                        head: "recordtype".into(),
                        reason: format!(
                            "parent '{}' is not a known type; declare it before this recordtype",
                            parent
                        ),
                    },
                });
            }
            self.types.insert(name.clone(), def);
            return self.register_subtype(&name, &parent, span);
        }
        self.types.insert(name, def);
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
            return Err(TypeError {
                span,
                kind: TypeErrorKind::CyclicSubtype {
                    child: child.to_string(),
                    parent: parent.to_string(),
                },
            });
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

/// Seeds a fresh [`TypeEnv`] with wat-rs's own `:wat::*` declarations.
/// Called exactly once, from [`TypeEnv::with_builtins`]. New builtins
/// land here as the algebra grows; each entry documents why the
/// declaration is `:wat::*`-scoped.
fn register_builtin_types(env: &mut TypeEnv) {
    // :wat::holon::CapacityExceeded — populated in the Err slot of
    // :wat::holon::Bundle's :Result return when a frame's
    // constituent count exceeds `floor(sqrt(dims))` (Kanerva's capacity
    // budget). The two fields are honest: cost is what the Bundle was
    // asked to hold; budget is what the substrate could hold. Both
    // i64 because wat integer literals are i64.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::holon::CapacityExceeded".into(),
        type_params: vec![],
        fields: vec![
            ("cost".into(), TypeExpr::Path(":wat::core::i64".into())),
            ("budget".into(), TypeExpr::Path(":wat::core::i64".into())),
        ],
        restrictions: None,
    }));

    // :wat::holon::BundleResult — arc 032. Typealias for the
    // canonical Result shape Bundle (and every downstream caller
    // that threads through Bundle) returns. 44 characters wide
    // collapsed to one named type. Non-parametric: Bundle's Ok
    // arm is always HolonAST; CapacityExceeded is the algebra's
    // only capacity-failure shape.
    //
    //   typealias :wat::holon::BundleResult
    //     = :Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>
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
    //   typealias :wat::holon::Holons = :Vec<wat::holon::HolonAST>
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
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::core::EvalError".into(),
        type_params: vec![],
        fields: vec![
            ("kind".into(), TypeExpr::Path(":wat::core::String".into())),
            ("message".into(), TypeExpr::Path(":wat::core::String".into())),
        ],
        restrictions: None,
    }));

    // :wat::core::Bytes — substrate-general byte buffer. Alias for
    // :Vec<u8>. Per arc 062 + /gaze: the universal name "Bytes" wins
    // across adjacent ecosystems (Rust's bytes::Bytes, Python's
    // bytes, Go's []byte, Haskell's ByteString). Lives in :wat::core::*
    // because byte buffers are substrate-general — they predate every
    // current and future consumer (vector serde via arc 061, future
    // crypto/IO/hashing/network arcs). The alias resolves structurally;
    // both `:wat::core::Bytes` and `:Vec<u8>` work at call sites.
    //
    //   typealias :wat::core::Bytes = :Vec<u8>
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
    // `Option<T>::None` / `Some(t)` discipline (per arc 153
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

    // Arc 070 — :wat::eval::WalkStep<A> — what the visitor passed to
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

    // :wat::kernel::ThreadDiedError — populated in the Err slot of the
    // :Result returned by :wat::kernel::join-result (arc 060) when a
    // spawned thread does NOT yield a value normally. Three variants
    // discriminate cause; supervisors / restart policies / debugging
    // traces want to tell them apart:
    //
    //   Panic(message)         — the thread's eval panicked; catch_unwind
    //                            captured the payload as a String.
    //   RuntimeError(message)  — the thread's eval returned :Err normally
    //                            (the spawned function was Result-typed
    //                            and produced an Err).
    //   ChannelDisconnected    — substrate bug; the channel dropped
    //                            without sending. In practice should
    //                            never fire under arc-060's catch_unwind
    //                            wrap; emitted as a distinct variant so
    //                            consumers can tell "my function ran and
    //                            died" from "the substrate ate my child."
    //
    // The String fields aren't typed-error-objects on purpose — wat-rs's
    // RuntimeError enum carries its own Display impl; we extract the
    // formatted message at the substrate boundary. Future arc may widen
    // to a typed payload if a caller surfaces real need.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::ThreadDiedError".into(),
        type_params: vec![],
        variants: vec![
            // Arc 105c: Panic variant carries TWO fields. `message`
            // is always populated. `failure` is `:Some(...)` when
            // the panic was an arc-016/064 AssertionPayload (assert-eq
            // failure), `:None` for plain `panic!()`. Wat callers
            // route through `:wat::kernel::ThreadDiedError/to-failure`
            // (also arc 105c) which builds a Failure regardless of
            // variant — sandbox.wat doesn't pattern-match the variant
            // at all.
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
            EnumVariant::Unit("ChannelDisconnected".into()),
            // arc 170 Slice A — process-wide shutdown signal fired during recv.
            // The channel partner did NOT drop — the process is terminating.
            // Distinguishable from ChannelDisconnected for shutdown-specific cleanup.
            // Slice B wires recv to surface this; Slice A only registers the variant.
            EnumVariant::Unit("Shutdown".into()),
        ],
    }));

    // :wat::kernel::ProcessDiedError — populated in the Err slot of
    // the :Result returned by verbs that operate on
    // :wat::kernel::Process<I,O> (arc 112): join-result on a
    // Process/join handle, process-recv, process-send. Three
    // variants identical in shape to ThreadDiedError; the name
    // tracks the SUBJECT (Process — a running Program — vs
    // ThreadDiedError's thread peer on a channel). After arc 112
    // unifies the in-thread (spawn-program') and OS-fork (spawn-process)
    // paths under a single Process<I,O> return type, the
    // Forked variant of ProgramHandle synthesizes ProcessDiedError
    // from waitpid + exit code; the InThread variant of
    // ProgramHandle (returned by :wat::kernel::spawn) keeps
    // ThreadDiedError because its peer is genuinely a thread.
    env.register_builtin(TypeDef::Enum(EnumDef {
        name: ":wat::kernel::ProcessDiedError".into(),
        type_params: vec![],
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
            EnumVariant::Unit("ChannelDisconnected".into()),
            // Arc 170 slice 1i — new structured exit variants for all child
            // exit paths (spawn-process + fork). extract-panics uses the
            // TypeEnv to reconstruct these variants from EDN on round-trip;
            // they must be registered here so edn_to_value can find them.
            EnumVariant::Tagged {
                name: "StartupError".into(),
                fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
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
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::Location".into(),
        type_params: vec![],
        fields: vec![
            ("file".into(), TypeExpr::Path(":wat::core::String".into())),
            ("line".into(), TypeExpr::Path(":wat::core::i64".into())),
            ("col".into(), TypeExpr::Path(":wat::core::i64".into())),
        ],
        restrictions: None,
    }));

    // :wat::kernel::Frame — one entry from a Rust backtrace. The wat-
    // rs runtime populates these by iterating `std::backtrace::Backtrace`
    // frames when a sandboxed program panics; only populated if
    // `RUST_BACKTRACE` is enabled (otherwise the frames vec is empty).
    // Each field is Option because Rust's backtrace symbol resolution
    // can fail per-frame (stripped symbols, jit frames).
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::Frame".into(),
        type_params: vec![],
        fields: vec![
            (
                "file".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
            (
                "line".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::core::i64".into())],
                },
            ),
            (
                "symbol".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::Failure — structured panic / assertion payload
    // populated when a sandboxed `:user::main` fails. Slice 2b fills
    // message / location / frames from `catch_unwind`; slice 3's
    // `:wat::test::assert-*` primitives additionally populate actual /
    // expected when the panic payload carries an AssertionPayload.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::Failure".into(),
        type_params: vec![],
        fields: vec![
            ("message".into(), TypeExpr::Path(":wat::core::String".into())),
            (
                "location".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Location".into())],
                },
            ),
            (
                "frames".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Frame".into())],
                },
            ),
            (
                "actual".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
            (
                "expected".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::RunResult — return type of
    // `:wat::kernel::run-sandboxed`. `stdout` and `stderr` accumulate
    // everything the sandboxed `:user::main` wrote through its stdio
    // channels, line by line. `failure` is `:None` on success; slice 2b
    // populates it with a `Failure` when `catch_unwind` catches.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::RunResult".into(),
        type_params: vec![],
        fields: vec![
            (
                "stdout".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
            (
                "stderr".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
            (
                "failure".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Failure".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::ForkedChild RETIRED 2026-04-30 (arc 112).
    // The struct collapsed into :wat::kernel::Process<I,O> — both
    // spawn-process and spawn-program' now return the unified Process
    // shape. The wait mechanism lives inside ProgramHandle's
    // InThread / Forked enum variant; the ChildHandle is no longer
    // wat-visible. Pre-arc-112 fixtures used:
    //   (child :wat::kernel::ForkedChild<I,O>) (spawn-process forms)
    //   (handle :wat::kernel::ChildHandle)     (ForkedChild/handle child)
    //   (exit  :i64)                           (wait-child handle)
    // Migration:
    //   (proc  :wat::kernel::Process<I,O>)     (spawn-process forms)
    //   (rcv   :Result<:(),:ProcessDiedError>) (Process/join-result proc)

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
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::StartupError".into(),
        type_params: vec![],
        fields: vec![("message".into(), TypeExpr::Path(":wat::core::String".into()))],
        restrictions: None,
    }));

    // :wat::kernel::Process<I,O> — return type of
    // `:wat::kernel::spawn-process` (arc 012 + arc 112) and
    // `:wat::kernel::spawn-program'` (arc 214).
    //
    // Arc 170 slice 1c: ADDITIVE reshape. Existing fields (stdin /
    // stdout / stderr / join) preserved for back-compat with the
    // bundled stdlib (`wat/kernel/sandbox.wat` and
    // `wat/kernel/hermetic.wat`); two new fields appended (`tx` / `rx`)
    // expose the typed-channel surface the DESIGN settled on.
    // Slice 3 retires the byte-pipe accessors when the testing
    // tooling rebuilds against the new surface.
    //
    // Decision honest delta vs. BRIEF-SLICE-1C.md row D ("drop"):
    // a destructive reshape would brick `wat/kernel/sandbox.wat` (a
    // bundled stdlib used by every `:wat::test::deftest` expansion)
    // because its `Process/stdin` / `/stdout` / `/stderr` calls
    // would fail type-check at substrate startup, blocking every
    // test. Additive ships without bricking; slice 3 sweeps.
    //
    // `tx :Sender<I>` and `rx :Receiver<O>` are typed-channel handles
    // wrapped over the same kernel pipes that back `stdin` / `stdout`.
    // The substrate populates both views: byte-pipe view (legacy
    // Stone C — Process is 4 fields: real OS stdio pipes + join handle.
    // The slice-1c typed-channel tx/rx fields are REMOVED. Real stdio
    // is canonical at the OS boundary. Users wanting typed semantics
    // wrap Process/stdin / Process/stdout with
    // (:wat::kernel::Sender/from-pipe writer) /
    // (:wat::kernel::Receiver/from-pipe reader).
    //
    // Type params I/O are KEPT for TIERS.md uniformity and backwards
    // compatibility with Program<I,O> alias (Program<I,O> = Process<I,O>).
    // The params are annotation-only — no field uses them after Stone C.
    //
    // Auto-generated `Process/new` + per-field accessors land in the
    // symbol table at freeze time via register_struct_methods.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::Process".into(),
        type_params: vec!["I".into(), "O".into()],
        fields: vec![
            (
                "stdin".into(),
                TypeExpr::Path(":wat::io::IOWriter".into()),
            ),
            (
                "stdout".into(),
                TypeExpr::Path(":wat::io::IOReader".into()),
            ),
            (
                "stderr".into(),
                TypeExpr::Path(":wat::io::IOReader".into()),
            ),
            (
                "join".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::ProgramHandle".into(),
                    args: vec![TypeExpr::Tuple(vec![])],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::Thread<I,O> — arc 114 slice 1.
    //
    // Concrete satisfier of Program<I,O> for the in-thread host
    // (the other satisfier is Process<I,O> for forked OS processes).
    // Threads share memory with the parent; communication is via
    // crossbeam channels (zero-copy Arc<Value>) — NOT OS pipes.
    // Threads have NO stderr stream because panic info travels
    // through the join handle's chain, not through a side stream
    // (that's how processes handle it; threads don't need to).
    //
    // Three fields, mapping to the Program contract's three concerns:
    //
    //   input  — Sender<I>:    parent writes IN; thread reads
    //   output — Receiver<O>:  thread writes; parent reads OUT
    //   join   — ProgramHandle<()>: panic surfaces here on
    //                              `Thread/join-result` as
    //                              Result<unit, ThreadDiedError>
    //
    // Auto-generated `Thread/new` + per-field accessors (`Thread/input`,
    // `Thread/output`, `Thread/join`) land in the symbol table at
    // freeze time via register_struct_methods.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::Thread".into(),
        type_params: vec!["I".into(), "O".into()],
        fields: vec![
            (
                "input".into(),
                TypeExpr::Parametric {
                    head: "rust::crossbeam_channel::Sender".into(),
                    args: vec![TypeExpr::Path(":I".into())],
                },
            ),
            (
                "output".into(),
                TypeExpr::Parametric {
                    head: "rust::crossbeam_channel::Receiver".into(),
                    args: vec![TypeExpr::Path(":O".into())],
                },
            ),
            (
                "join".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::ProgramHandle".into(),
                    args: vec![TypeExpr::Tuple(vec![])],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::ThreadPeer<I, O> — arc 170 Stone C1.
    //
    // Peer-relative view of one end of a thread↔thread typed-channel
    // conversation. Per `INTERSTITIAL-REALIZATIONS.md § 2026-05-16
    // (Stone C revision)`, the conceptual client/server distinction is
    // encoded by mirror bindings of the two type parameters — both
    // peers are instances of the SAME struct.
    //
    // Type parameters:
    //   I — what THIS peer reads (input direction; comes IN from the
    //       partner)
    //   O — what THIS peer writes (output direction; goes OUT to the
    //       partner)
    //
    // Fields, in declaration order:
    //   rx — Receiver<I> the peer pulls inbound messages from
    //   tx — Sender<O>   the peer pushes outbound messages through
    //
    // For a Request/Reply protocol the substrate provisions two
    // typed-channel pairs, then constructs one ThreadPeer per side
    // with the mirror parameter binding: server peer gets
    // ThreadPeer<Request, Reply>; client peer gets
    // ThreadPeer<Reply, Request>. Same struct, opposite directions.
    //
    // Stone D (`run-threads` bracket macro) is the user-facing
    // constructor. Stone C1 itself only mints the type and the two
    // peer-relative verbs (`Thread/readln`, `Thread/println`); test
    // peer-pair construction goes through the substrate-internal
    // `make_thread_peer_pair_for_test` helper in
    // `typed_channel.rs`.
    //
    // Auto-generated `ThreadPeer/new` + per-field accessors
    // (`ThreadPeer/rx`, `ThreadPeer/tx`) land at freeze time via
    // `register_struct_methods`. Stone C1 does NOT exercise those
    // accessors directly — the substrate verbs reach into the struct
    // by index — but they exist for future stones and for diagnostic
    // introspection.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::ThreadPeer".into(),
        type_params: vec!["I".into(), "O".into()],
        fields: vec![
            (
                "rx".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::Receiver".into(),
                    args: vec![TypeExpr::Path(":I".into())],
                },
            ),
            (
                "tx".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::Sender".into(),
                    args: vec![TypeExpr::Path(":O".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::ProcessPeer<I, O> — arc 170 Stone C2.
    //
    // CLIENT-side-only wrapper around the parent's view of a spawned
    // process's stdin + stdout pipe ends. Per
    // `INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (Stone C revision)`,
    // the Process side of the bracket-combinator family is ASYMMETRIC:
    // the OS process has exactly one stdin / stdout per child, so only
    // the parent (client) holds the peer struct; the child (server)
    // uses ambient `(:wat::kernel::readln)` / `(:wat::kernel::println …)`
    // over its own real stdio. NO `ProcessPeer/Server` variant is
    // emitted — that's the design.
    //
    // Type parameters (mirror of ThreadPeer):
    //   I — what the parent (client) READS from the server's stdout
    //   O — what the parent (client) WRITES to the server's stdin
    //
    // Fields, in declaration order:
    //   rx — Receiver<I> the parent pulls server output from. The
    //        underlying transport is the PipeFd-backed Receiver inner
    //        from `:wat::kernel::Receiver/from-pipe` over the
    //        Process/stdout reader.
    //   tx — Sender<O>   the parent pushes inbound messages to. The
    //        underlying transport is the PipeFd-backed Sender inner
    //        from `:wat::kernel::Sender/from-pipe` over the
    //        Process/stdin writer.
    //
    // Stone D's `run-processes` bracket macro is the user-facing
    // constructor (wires `spawn-process` → `Process/stdin` +
    // `Process/stdout` → typed channels → ProcessPeer). Stone C2 itself
    // only mints the type and the two peer-relative verbs
    // (`Process/readln`, `Process/println`); the substrate-composition
    // proof in `tests/wat_process_peer_ipc_round_trip.rs` exercises the
    // peer via the auto-generated `:wat::kernel::ProcessPeer/new`
    // constructor composing `Sender/from-pipe` + `Receiver/from-pipe`
    // over `Process/stdin` + `Process/stdout` — the same composition
    // Stone D's bracket macro will encapsulate for everyday use.
    //
    // Stone C3 — field-type honesty fix (arc 170). Previously, the
    // Receiver<I> / Sender<O> field types were deliberately named after
    // the THREAD-TIER backing crate (`rust::crossbeam_channel::*`) so
    // both Process verbs and Thread verbs could share dispatch logic.
    // Stone C2's confession comment noted:
    //   "The Receiver<I> / Sender<O> field types are deliberately the
    //   SAME typed-channel substrate ThreadPeer uses — `typed_recv` /
    //   `typed_send` are transport-polymorphic (Crossbeam tier-1 for
    //   threads, PipeFd tier-2 for processes), so the Process/readln +
    //   Process/println eval handlers can mirror Thread/readln +
    //   Thread/println verbatim modulo the struct tag."
    //
    // The architectural lesson: dispatch logic is shared via runtime
    // polymorphism (`typed_recv` branches on the Value variant); the
    // TYPE-KEYWORD should name the ABSTRACTION (`:wat::kernel::Sender/Receiver`),
    // not the implementation crate (crossbeam happens to back one tier).
    // The `:wat::kernel::Sender<T>` / `:wat::kernel::Receiver<T>` aliases
    // established by arc 109 K-channel rename (src/check.rs:3056-3057)
    // already unify at the type-system level, so this rename is
    // behavior-preserving. The dishonest `rust::crossbeam_channel::*`
    // names are retired from the FIELD DECLARATIONS here (Stone C3);
    // the alias registrations in channel.wat remain as the alias target.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::ProcessPeer".into(),
        type_params: vec!["I".into(), "O".into()],
        fields: vec![
            (
                "rx".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::Receiver".into(),
                    args: vec![TypeExpr::Path(":I".into())],
                },
            ),
            (
                "tx".into(),
                TypeExpr::Parametric {
                    head: "wat::kernel::Sender".into(),
                    args: vec![TypeExpr::Path(":O".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::kernel::Program<I,O> — arc 109 § J slice 10a.
    //
    // Typealias for today's :wat::kernel::Process<I,O>. The "supertype
    // kind" the slice plan calls for is just an alias under existing
    // substrate machinery — `unify` already collapses aliases (queue.wat
    // CommResult<T> / Chosen<T> precedent), so :Program<I,O> and
    // :Process<I,O> are interchangeable at every annotation site
    // post-slice-10a.
    //
    // Slice 10b (sonnet rename sweep) flips this around: the underlying
    // struct gets renamed to Program<I,O>; Process<I,O> becomes the
    // alias-for-back-compat. Slice 10c adds Thread<I,O> as another
    // alias (until arc 114's transport asymmetry forces the structural
    // split — Sender<Value> vs IOWriter). Slice 10d wires the typeclass
    // dispatch for the polymorphic verbs once concrete types diverge.
    //
    // No new TypeDef variant. No unify changes. The substrate's existing
    // typealias machinery carries the abstraction.
    env.register_builtin(TypeDef::Alias(AliasDef {
        name: ":wat::kernel::Program".into(),
        type_params: vec!["I".into(), "O".into()],
        expr: TypeExpr::Parametric {
            head: "wat::kernel::Process".into(),
            args: vec![
                TypeExpr::Path(":I".into()),
                TypeExpr::Path(":O".into()),
            ],
        },
    }));

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
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::holon::CoincidentExplanation".into(),
        type_params: vec![],
        fields: vec![
            ("cosine".into(), TypeExpr::Path(":wat::core::f64".into())),
            ("floor".into(), TypeExpr::Path(":wat::core::f64".into())),
            ("dim".into(), TypeExpr::Path(":wat::core::i64".into())),
            ("sigma".into(), TypeExpr::Path(":wat::core::i64".into())),
            ("coincident".into(), TypeExpr::Path(":wat::core::bool".into())),
            (
                "min-sigma-to-pass".into(),
                TypeExpr::Path(":wat::core::i64".into()),
            ),
        ],
        restrictions: None,
    }));

    // :wat::test::RunResultIO<O> — return type of
    // `:wat::test::run-hermetic-with-io` (arc 170 slice 3 phase D).
    // Layer 2 testing API: typed-channel I/O round-trip result.
    //
    // Three fields:
    //   outputs  :Vector<O>              — values received from the child's tx
    //   stderr   :Vector<String>         — raw stderr lines (for diagnostic)
    //   failure  :Option<Failure>        — :None on success; :Some on child panic
    //
    // Parallel to :wat::kernel::RunResult (Layer 1) but replaces
    // `stdout :Vector<String>` with `outputs :Vector<O>` (typed channel
    // values, not byte-stream lines). D2 decision: registered in
    // src/types.rs (Rust-side StructDef) so the accessor methods
    // RunResultIO/outputs, /stderr, /failure are auto-generated via
    // register_struct_methods at freeze time. A wat-side `:struct` form
    // was the alternative; the substrate registration is preferred
    // because it gives the struct the same first-class status as RunResult
    // without relying on a user-space `:struct` parse path.
    //
    // Auto-generated `RunResultIO/new` + per-field accessors land in
    // the symbol table at freeze time via register_struct_methods.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::test::RunResultIO".into(),
        type_params: vec!["O".into()],
        fields: vec![
            (
                "outputs".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":O".into())],
                },
            ),
            (
                "stderr".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Vector".into(),
                    args: vec![TypeExpr::Path(":wat::core::String".into())],
                },
            ),
            (
                "failure".into(),
                TypeExpr::Parametric {
                    head: "wat::core::Option".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Failure".into())],
                },
            ),
        ],
        restrictions: None,
    }));

    // :wat::Record — Arc 234 Stone 234.1.5. Opaque umbrella type for the
    // wat-record hologram (Value::wat__holon__Record). Pascal-Case namespace per
    // the `::`/`/` semantic-split doctrine: the namespace IS the umbrella
    // type; `::` verbs operate at the type tier (Record::of, Record::def,
    // Record::is?); `/` methods operate on instances (Record/field-at,
    // Record/to-map). Registered as opaque zero-field struct so the TypeEnv
    // contains the path and `env.types().get(":wat::Record")` resolves
    // cleanly. Per-class types (`:myapp::Voltage` as `:wat::Record` aliases)
    // ship in Stone 234.2b when the defrecord macro lands.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::Record".into(),
        type_params: vec![],
        fields: vec![],
        restrictions: None,
    }));

    // Stone S-A — `:wat::holon::Record` opaque umbrella type + typesub root edge.
    //
    // `:wat::holon::Record` is the "holonic record" flavor — a record that carries
    // a HolonAST alongside its struct-form. Registered as an opaque zero-field
    // struct (mirrors `:wat::Record` exactly). The `typesub` edge seeds the
    // built-in is-a root: `:wat::holon::Record` is-a `:wat::Record`.
    //
    // NOTE: registering `:wat::holon::Record` as a struct causes
    // `register_type_predicates` to synthesize `:wat::holon::is-Record?` for it
    // (same as `:wat::Record` already gets `:wat::is-Record?`). This is correct —
    // it is a type. See SCORE-STONE-S-A § Honest deltas.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::holon::Record".into(),
        type_params: vec![],
        fields: vec![],
        restrictions: None,
    }));
    // Seed the built-in typesub root: `:wat::holon::Record` is-a `:wat::Record`.
    // Cannot cycle (fresh registry with no edges yet); `expect` is correct here.
    // built-in root hierarchy seed — no source form exists; unreachable cycle path (two distinct roots).
    env.register_subtype(":wat::holon::Record", ":wat::Record", Span::unknown())
        .expect("built-in typesub root cannot cycle");
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
) -> Result<Vec<WatAST>, TypeError> {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        match classify_type_decl(&form) {
            Some(head) => {
                // Arc 138 slice 2 — capture decl span BEFORE the form
                // is consumed by `parse_type_decl`. Threaded through
                // every emission site for source-coordinate prefixes.
                let decl_span = form.span().clone();
                let def = parse_type_decl(head, form, decl_span.clone())?;
                register(env, def, decl_span)?;
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
    register_types_impl(
        forms,
        env,
        &|env, def, span| env.register_with_span(def, span),
        &splice_type_decls_user,
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
    register_types_impl(
        forms,
        env,
        &|env, def, span| env.register_stdlib_with_span(def, span),
        &splice_type_decls_stdlib,
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
                        let def = parse_type_decl(head, child, decl_span.clone())?;
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
                        let def = parse_type_decl(head, child, decl_span.clone())?;
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
                    return Err(TypeError {
                        span: decl_span,
                        kind: TypeErrorKind::MalformedDecl {
                            head: "derive".into(),
                            reason: "expected keyword child type name at position 1".into(),
                        },
                    })
                }
            };
            let parent = match items.get(2) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => {
                    return Err(TypeError {
                        span: decl_span,
                        kind: TypeErrorKind::MalformedDecl {
                            head: "derive".into(),
                            reason: "expected keyword parent type name at position 2".into(),
                        },
                    })
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
            let type_name = match items.get(1) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => {
                    return Err(TypeError {
                        span: decl_span,
                        kind: TypeErrorKind::MalformedDecl {
                            head: "extend-type".into(),
                            reason: "expected keyword type name at position 1".into(),
                        },
                    })
                }
            };
            let protocol_name = match items.get(2) {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => {
                    return Err(TypeError {
                        span: decl_span,
                        kind: TypeErrorKind::MalformedDecl {
                            head: "extend-type".into(),
                            reason: "expected keyword protocol name at position 2".into(),
                        },
                    })
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
                // Arc 293.3-core — structural surface.
                ":wat::core::defsurface" => return Some("defsurface"),
                _ => {}
            }
        }
    }
    None
}

fn parse_type_decl(
    head: &str,
    form: WatAST,
    decl_span: Span,
) -> Result<TypeDef, TypeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: "expected list form".into(),
                },
            })
        }
    };
    let mut iter = items.into_iter();
    let _head_kw = iter.next();
    match head {
        // Stone 241.8 — defstruct replaces struct + struct-restricted (HARD CUT).
        "defstruct" => parse_defstruct(iter.collect(), decl_span),
        // Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
        "structtype" => parse_defstruct(iter.collect(), decl_span),
        // Stone 241.9 — defenum replaces enum (HARD CUT).
        "defenum" => parse_defenum(iter.collect(), decl_span),
        "newtype" => parse_newtype(iter.collect(), decl_span),
        "typealias" => parse_typealias(iter.collect(), decl_span),
        // Stone 237.1 — named bounded set of types.
        "typeunion" => parse_typeunion(iter.collect(), decl_span),
        // Stone S-B.1 — record class as a real TypeDef.
        "recordtype" => parse_recordtype(iter.collect(), decl_span),
        // Arc 293.3-core — structural surface.
        "defsurface" => parse_defsurface(iter.collect(), decl_span),
        _ => unreachable!(),
    }
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
fn parse_defenum(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    const HEAD: &str = ":wat::core::defenum";

    // Need at least: name + one variant (2 args minimum).
    if args.len() < 2 {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defenum :Name :V1 ...) with at least one variant; got {} args after head",
                    args.len()
                ),
            },
        });
    }

    let mut iter = args.into_iter();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name(HEAD, &name_kw, &decl_span)?;

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
        let pairs = meta_node.metadata_map_pairs().ok_or_else(|| TypeError {
            span: meta_node.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "malformed metadata-map (internal structure corrupt)".into(),
            },
        })?;
        // Empty {} → pairs.len() == 0 → REJECTED (FORM-COLLAPSE D4).
        if pairs.is_empty() {
            return Err(TypeError {
                span: meta_node.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "empty `{}` metadata-map is illegal (use no metadata-map arg for plain defenum)".into(),
                },
            });
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
                    return Err(TypeError {
                        span: other.span().clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: "metadata-map keys must be keywords".into(),
                        },
                    });
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
                let variant_name = k.strip_prefix(':').ok_or_else(|| TypeError {
                    span: item.span().clone(),
                    kind: TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: format!("{:?}", k),
                        reason: "defenum variant must be a keyword starting with ':'".to_string(),
                        remedies: vec![],
                    },
                })?.to_string();

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
                return Err(TypeError {
                    span: item.span().clone(),
                    kind: TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: ident.as_str().to_owned(),
                        reason: format!(
                            "defenum variant must be a keyword; got bare symbol '{}' — write it as the keyword '{}'",
                            ident.as_str(), needle,
                        ),
                        remedies: vec![],
                    },
                });
            }
            other => {
                return Err(TypeError {
                    span: other.span().clone(),
                    kind: TypeErrorKind::MalformedVariant {
                        enum_name: name.clone(),
                        offending: format!("{:?}", other),
                        reason: "defenum variant must be a keyword (unit) or keyword followed by Vector (tagged)".to_string(),
                        remedies: vec![],
                    },
                });
            }
        }
    }

    if variants.is_empty() {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "defenum must have at least one variant".into(),
            },
        });
    }

    Ok(TypeDef::Enum(EnumDef {
        name,
        type_params,
        variants,
    }))
}

fn parse_newtype(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: "newtype".into(),
                reason: format!(
                    "expected (:wat::core::newtype :name :InnerType); got {} args",
                    args.len()
                ),
            },
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let inner_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("newtype", &name_kw, &decl_span)?;
    // Arc 251.3a — accept Keyword, Symbol (wat.type/X), or List (parametric form).
    let inner = match &inner_kw {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(&inner_kw)?
        }
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "newtype".into(),
                    reason: format!(
                        "inner type must be a keyword or type form; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };
    Ok(TypeDef::Newtype(NewtypeDef {
        name,
        type_params,
        inner,
    }))
}

fn parse_typealias(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: "typealias".into(),
                reason: format!(
                    "expected (:wat::core::typealias :name :Expr); got {} args",
                    args.len()
                ),
            },
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let expr_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("typealias", &name_kw, &decl_span)?;
    // Arc 251.3a — accept Keyword, Symbol (wat.type/X), or List (parametric form).
    let expr = match &expr_kw {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(&expr_kw)?
        }
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "typealias".into(),
                    reason: format!(
                        "alias expression must be a keyword or type form; got {}",
                        other.variant_name()
                    ),
                },
            })
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
    if args.len() != 2 {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: "typeunion".into(),
                reason: format!(
                    "expected (:wat::core::typeunion :Name [:T1 :T2 ...]); got {} args",
                    args.len()
                ),
            },
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let members_ast = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("typeunion", &name_kw, &decl_span)?;
    let member_items = match members_ast {
        WatAST::Vector(items, _) => items,
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "typeunion".into(),
                    reason: format!(
                        "member list must be a Vector `[...]`; got {}",
                        other.variant_name()
                    ),
                },
            })
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
                return Err(TypeError {
                    span: item_span,
                    kind: TypeErrorKind::MalformedDecl {
                        head: "typeunion".into(),
                        reason: format!(
                            "member must be a type keyword or type form; got {}",
                            other.variant_name()
                        ),
                    },
                })
            }
        }
    }
    Ok(TypeDef::Union(UnionDef {
        name,
        type_params,
        members,
    }))
}

/// Stone S-C.2ab — parse `(:wat::core::recordtype :Name :Parent [field-name-strings])`.
///
/// Three positional slots after the head keyword (consumed by `parse_type_decl`):
///   args[0] — name keyword (e.g. `:my::Circle`)
///   args[1] — parent type keyword (e.g. `:wat::Record` or `:wat::holon::Record`)
///   args[2] — vector of field-name string literals in declaration order
///              (e.g. `["radius"]` or `[]` for zero-field records)
///
/// HARD CUT: 2-arg form is rejected. A 0-field record passes `[]`.
///
/// → `TypeDef::Record(RecordDef { name, parent, field_names })`. Parent validity is
/// checked at registration time (in `register_with_span`).
fn parse_recordtype(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    if args.len() != 3 {
        return Err(TypeError {
            span: decl_span,
            kind: TypeErrorKind::MalformedDecl {
                head: "recordtype".into(),
                reason: format!(
                    "expected (:wat::core::recordtype :Name :Parent [field-name-strings]); got {} args (HARD CUT: the 3rd arg [field-names] is required; pass [] for zero-field records)",
                    args.len()
                ),
            },
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let parent_kw = iter.next().unwrap();
    let fields_arg = iter.next().unwrap();
    // Name: plain keyword (no type params for records).
    let name = match &name_kw {
        WatAST::Keyword(k, _) => {
            if !k.starts_with(':') {
                return Err(TypeError {
                    span: decl_span,
                    kind: TypeErrorKind::MalformedDecl {
                        head: "recordtype".into(),
                        reason: format!("name must begin with ':'; got {}", k),
                    },
                });
            }
            k.clone()
        }
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "recordtype".into(),
                    reason: format!(
                        "name must be a keyword; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };
    // Parent: plain type keyword.
    let parent = match parent_kw {
        WatAST::Keyword(k, _) => {
            if !k.starts_with(':') {
                return Err(TypeError {
                    span: decl_span,
                    kind: TypeErrorKind::MalformedDecl {
                        head: "recordtype".into(),
                        reason: format!("parent must begin with ':'; got {}", k),
                    },
                });
            }
            k
        }
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "recordtype".into(),
                    reason: format!(
                        "parent must be a type keyword; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };
    // Field names: a vector literal in one of two forms:
    //   1. String-literal form (emitted by :wat::Record::def macro):
    //      ["field1" "field2"]
    //      field_types = None (type info not provided at this layer).
    //   2. Typed-declaration form (direct user code, mirrors Record::def input syntax):
    //      [name <- :type  name2 <- :type2]
    //      Groups of 3 elements: (Symbol|Keyword name, Symbol "<-", Keyword type).
    //      field_types = Some(vec![...]) populated for register_record_methods.
    let (field_names, field_types) = match fields_arg {
        WatAST::Vector(elems, _) => {
            if elems.is_empty() {
                (Vec::new(), None)
            } else if matches!(&elems[0], WatAST::StringLit(_, _)) {
                // String-literal form: every element must be a StringLit.
                let mut names = Vec::with_capacity(elems.len());
                for elem in elems.iter() {
                    match elem {
                        WatAST::StringLit(s, _) => names.push(s.clone()),
                        other => {
                            return Err(TypeError {
                                span: decl_span,
                                kind: TypeErrorKind::MalformedDecl {
                                    head: "recordtype".into(),
                                    reason: format!(
                                        "field-names vector must contain string literals; got {}",
                                        other.variant_name()
                                    ),
                                },
                            });
                        }
                    }
                }
                (names, None)
            } else {
                // Typed-declaration form: groups of 3 — (name, <-, type).
                if elems.len() % 3 != 0 {
                    return Err(TypeError {
                        span: decl_span,
                        kind: TypeErrorKind::MalformedDecl {
                            head: "recordtype".into(),
                            reason: format!(
                                "typed field vector must have a multiple of 3 elements (name <- :type); got {}",
                                elems.len()
                            ),
                        },
                    });
                }
                let nf = elems.len() / 3;
                let mut names = Vec::with_capacity(nf);
                let mut types_out = Vec::with_capacity(nf);
                for i in 0..nf {
                    let name_elem = &elems[i * 3];
                    let arrow_elem = &elems[i * 3 + 1];
                    let type_elem = &elems[i * 3 + 2];
                    // Arrow must be a Symbol named "<-".
                    match arrow_elem {
                        WatAST::Symbol(ident, _) if ident.as_str() == "<-" => {}
                        other => {
                            return Err(TypeError {
                                span: decl_span,
                                kind: TypeErrorKind::MalformedDecl {
                                    head: "recordtype".into(),
                                    reason: format!(
                                        "typed field [{}]: expected '<-' arrow; got {}",
                                        i, other.variant_name()
                                    ),
                                },
                            });
                        }
                    }
                    // Field name: Symbol or Keyword.
                    let field_name = match name_elem {
                        WatAST::Symbol(ident, _) => ident.as_str().to_string(),
                        WatAST::Keyword(k, _) => {
                            // Strip leading ':' for the stored name.
                            k.trim_start_matches(':').to_string()
                        }
                        other => {
                            return Err(TypeError {
                                span: decl_span,
                                kind: TypeErrorKind::MalformedDecl {
                                    head: "recordtype".into(),
                                    reason: format!(
                                        "typed field [{}]: name must be a symbol or keyword; got {}",
                                        i, other.variant_name()
                                    ),
                                },
                            });
                        }
                    };
                    // Field type: parse the type node (Keyword, Symbol, or List).
                    let field_type = parse_type_node(type_elem).map_err(|e| TypeError {
                        span: decl_span.clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: "recordtype".into(),
                            reason: format!("typed field [{}]: bad type expr: {:?}", i, e.kind),
                        },
                    })?;
                    names.push(field_name);
                    types_out.push(field_type);
                }
                (names, Some(types_out))
            }
        }
        other => {
            return Err(TypeError {
                span: decl_span,
                kind: TypeErrorKind::MalformedDecl {
                    head: "recordtype".into(),
                    reason: format!(
                        "third arg must be a vector of field-name strings (e.g. [\"field1\" \"field2\"] or []); got {}",
                        other.variant_name()
                    ),
                },
            });
        }
    };
    Ok(TypeDef::Record(RecordDef { name, parent, field_names, field_types }))
}

// Stone 241.9 — `parse_field` DELETED. Its only caller was `parse_enum_variant`,
// which was also deleted (HARD CUT). `parse_defenum` uses `parse_argspec_triples`
// for tagged-variant fields instead of the legacy pair-form parser.

/// Parse a declared type name. Accepts:
/// - `:my::ns::MyType` → ("my/ns/MyType", [])
/// - `:my::ns::Wrapper<T>` → ("my/ns/Wrapper", ["T"])
/// - `:my::ns::Container<K,V>` → ("my/ns/Container", ["K", "V"])
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
            return Err(TypeError {
                span: decl_span.clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: format!(
                        "name must be a keyword; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };
    // Strip the colon but keep the rest as the key for TypeEnv.
    let stripped = raw.strip_prefix(':').ok_or_else(|| TypeError {
        span: name_span.clone(),
        kind: TypeErrorKind::MalformedName {
            raw: raw.clone(),
            reason: "keyword must begin with ':'".into(),
        },
    })?;
    // Split at first '<' if present.
    match stripped.find('<') {
        None => Ok((raw, Vec::new())),
        Some(lt_index) => {
            let base = &stripped[..lt_index];
            let params_part = &stripped[lt_index..];
            if !params_part.ends_with('>') {
                return Err(TypeError {
                    span: name_span,
                    kind: TypeErrorKind::MalformedName {
                        raw: raw.clone(),
                        reason: "parametric name must close with '>'".into(),
                    },
                });
            }
            let inner = &params_part[1..params_part.len() - 1];
            let params: Vec<String> = inner
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for p in &params {
                if p.contains(char::is_whitespace) || p.contains('<') || p.contains(':') {
                    return Err(TypeError {
                        span: name_span,
                        kind: TypeErrorKind::MalformedName {
                            raw: raw.clone(),
                            reason: format!("type parameter {:?} has invalid chars", p),
                        },
                    });
                }
            }
            // Key the registry by the bare name (no <T> suffix), but
            // preserve the colon for the stored name field.
            let stored_name = format!(":{}", base);
            Ok((stored_name, params))
        }
    }
}

/// Parse a type-expression keyword into a structured [`TypeExpr`].
///
/// Refuses `:Any` at any position (bare path or parametric head) per
/// 058-030's closed-type-universe discipline. Every apparent need for
/// `:Any` has a principled named alternative (`:wat::holon::HolonAST` for algebra
/// values, parametric `T`/`K`/`V` for generics, a named enum for
/// closed heterogeneous sets).
// rune:struere(host-constraint) — public surface preserved for callers
// without a keyword span in scope (arc 138 lineage); Span::unknown() is
// the honest placeholder when no source position is available. Span-aware
// callers use parse_type_expr_with_span directly.
pub fn parse_type_expr(kw: &str) -> Result<TypeExpr, TypeError> {
    parse_type_expr_with_span(kw, &Span::unknown())
}

/// Arc 138 slice 2 — span-carrying variant. Consumers with a real
/// keyword span (the type-registration call chain in this file) use
/// this entry point so emitted errors prefix `<file>:<line>:<col>:`.
pub fn parse_type_expr_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError> {
    let stripped = kw.strip_prefix(':').ok_or_else(|| TypeError {
        span: span.clone(),
        kind: TypeErrorKind::MalformedTypeExpr {
            raw: kw.into(),
            reason: "type expression keyword must begin with ':'".into(),
        },
    })?;
    let expr = parse_type_inner(stripped, kw, true, span)?;
    reject_any(&expr, kw, span)?;
    Ok(expr)
}

/// Arc 251.3a — dispatch a `WatAST` node in a type-annotation slot.
///
/// Accepts the three node shapes that can appear in a type slot after the
/// dual-read transition begins at 251.3:
///
/// - `WatAST::Keyword(kw, span)` — the existing surface: delegates to
///   `parse_type_expr_with_span`. Covers `:wat::core::i64`,
///   `:wat::core::Vector<wat::core::i64>`, etc.
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
            let kw = match s.rfind('/') {
                Some(slash) => crate::edn_shim::ns_to_wat_path(&s[..slash], &s[slash + 1..]),
                // Bare symbol without namespace — treat as a keyword by prepending `:`.
                None => format!(":{}", s),
            };
            parse_type_expr_with_span(&kw, span)
        }
        WatAST::List(_, _) => parse_type_form(node),
        // Arc 251.4c — a `[T… :-> R]` bracket is a function type (core.typed parity).
        WatAST::Vector(items, span) => parse_fn_type_bracket(items, span),
        other => Err(TypeError {
            span: other.span().clone(),
            kind: TypeErrorKind::MalformedTypeExpr {
                raw: format!("{:?}", other),
                reason: format!(
                    "type annotation must be a keyword, namespaced symbol, parametric form `(Ctor arg…)`, or function-type bracket `[arg… :-> ret]`; got {}",
                    other.variant_name()
                ),
            },
        }),
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
            return Err(TypeError {
                span: span.clone(),
                kind: TypeErrorKind::MalformedTypeExpr {
                    raw: "[…]".into(),
                    reason: "function-type bracket needs a `:->` arrow: `[arg… :-> ret]`".into(),
                },
            })
        }
    };
    let ret_nodes = &items[arrow_pos + 1..];
    if ret_nodes.len() != 1 {
        return Err(TypeError {
            span: span.clone(),
            kind: TypeErrorKind::MalformedTypeExpr {
                raw: "[…]".into(),
                reason: format!(
                    "function-type bracket needs exactly one return type after `:->`; got {}",
                    ret_nodes.len()
                ),
            },
        });
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
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedTypeExpr {
                    raw: format!("{:?}", other),
                    reason: "parse_type_form expects a List node".into(),
                },
            })
        }
    };
    if items.is_empty() {
        return Err(TypeError {
            span: span.clone(),
            kind: TypeErrorKind::MalformedTypeExpr {
                raw: "()".into(),
                reason: "parametric type form must not be empty; expected `(Ctor arg…)`".into(),
            },
        });
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
            match s.rfind('/') {
                Some(slash) => {
                    let kw = crate::edn_shim::ns_to_wat_path(&s[..slash], &s[slash + 1..]);
                    kw.strip_prefix(':').unwrap_or(&kw).to_string()
                }
                None => s.to_string(),
            }
        }
        WatAST::Keyword(kw, _) => {
            // Post-normalize keyword (`:wat::type::Vector`) or already canonical (`:wat::core::Vector`).
            // Strip the leading `:`.
            kw.strip_prefix(':').unwrap_or(kw).to_string()
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedTypeExpr {
                    raw: format!("{:?}", other),
                    reason: "parametric type form head must be a symbol or keyword".into(),
                },
            })
        }
    };
    // Arc 251.2 alias: `wat::type::` → `wat::core::` (dual-read, mirrors parse_type_inner ~line 2374).
    let raw_head = match raw_head.strip_prefix("wat::type::") {
        Some(tail) => format!("wat::core::{}", tail),
        None => raw_head,
    };
    // Parse args recursively.
    let args: Result<Vec<TypeExpr>, TypeError> = items[1..].iter()
        .map(parse_type_node)
        .collect();
    let args = args?;
    // Arc 251 — the `Tuple` constructor head produces a TUPLE type, not a generic Parametric:
    // `(wat.type/Tuple A B)` → `TypeExpr::Tuple([A,B])`; the empty `(wat.type/Tuple)` → the
    // 0-tuple. This is the faithful-Clojure spelling of the legacy `:(A,B)` keyword tuple
    // (both produce the SAME `TypeExpr::Tuple`, so they unify identically).
    let result = if raw_head == "wat::core::Tuple" {
        TypeExpr::Tuple(args)
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
    parse_type_inner(stripped, kw, false, &Span::unknown()).ok()
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
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::AnyBanned { raw: raw.into() },
                });
            }
        }
        TypeExpr::Parametric { head, args } => {
            if head == "Any" {
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::AnyBanned { raw: raw.into() },
                });
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
        return Err(TypeError {
            span: span.clone(),
            kind: TypeErrorKind::InnerColonInCompoundArg {
                raw: original.into(),
                offending: s.to_string(),
            },
        });
    }
    // Tuple literal — `(T,U,...)`. Must appear at the start; inner
    // types respect top-level comma splitting.
    if let Some(rest) = s.strip_prefix('(') {
        if !rest.ends_with(')') {
            return Err(TypeError {
                span: span.clone(),
                kind: TypeErrorKind::MalformedTypeExpr {
                    raw: original.into(),
                    reason: "tuple-literal type must close with ')'".into(),
                },
            });
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
    // `Head<args>` parametric.
    if let Some(lt_index) = find_top_level_char(s, '<') {
        let raw_head = s[..lt_index].to_string();
        let rest = &s[lt_index..];
        if !rest.ends_with('>') {
            return Err(TypeError {
                span: span.clone(),
                kind: TypeErrorKind::MalformedTypeExpr {
                    raw: original.into(),
                    reason: "parametric type must close with '>'".into(),
                },
            });
        }
        let inside = &rest[1..rest.len() - 1];
        let args = parse_type_list(inside, original, canonicalize, span)?;
        // Arc 163 slice 3e + 3h — FQDN IS the canonical storage form.
        // Source FQDN flows through unchanged. Source bare-form is
        // rejected by `BareLegacyContainerHead` walker at check time
        // (slice 3g phase A wired the walker on raw post-expansion
        // forms so define-sig type positions are covered). The
        // canonicalize=true UPGRADE arm (Vec → wat::core::Vector
        // etc.) retired in slice 3h — raw_head passes through identity.
        return Ok(TypeExpr::Parametric { head: raw_head, args });
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
    let close = find_matching_close(body, '(', ')').ok_or_else(|| TypeError {
        span: span.clone(),
        kind: TypeErrorKind::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing matching ')'".into(),
        },
    })?;
    let args_part = &body[..close];
    let tail = &body[close + 1..];
    let ret_part = tail
        .strip_prefix("->")
        .ok_or_else(|| TypeError {
            span: span.clone(),
            kind: TypeErrorKind::MalformedTypeExpr {
                raw: original.into(),
                reason: "fn type missing '->' before return".into(),
            },
        })?;
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
fn parse_type_list(
    s: &str,
    original: &str,
    canonicalize: bool,
    span: &Span,
) -> Result<Vec<TypeExpr>, TypeError> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                let piece = &s[start..i];
                out.push(parse_type_inner(piece.trim(), original, canonicalize, span)?);
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = &s[start..];
    if !tail.trim().is_empty() {
        out.push(parse_type_inner(tail.trim(), original, canonicalize, span)?);
    }
    Ok(out)
}

/// Find the first occurrence of `c` at bracket-depth 0.
///
/// Checks the match BEFORE adjusting depth so that `c` itself being a
/// bracket (`<` or `(`) is correctly detected at the outermost level —
/// finding `<` in `List<T>` matches position 4, not None.
fn find_top_level_char(s: &str, c: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        if depth == 0 && ch == c {
            return Some(i);
        }
        match ch {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            _ => {}
        }
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
// `:Alias<K,V>` and its expansion are the SAME type. The runtime shape
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
                let qualified = format!(":{}", head);
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
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::CyclicAlias { name: target_name.to_string() },
                });
            }
            if let Some(TypeDef::Alias(alias)) = env.get(name) {
                if visiting.insert(name.clone()) {
                    check_alias_reaches(target_name, &alias.expr, env, visiting, span)?;
                    visiting.remove(name);
                }
            }
        }
        TypeExpr::Parametric { head, args } => {
            let qualified = format!(":{}", head);
            if qualified == target_name {
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::CyclicAlias { name: target_name.to_string() },
                });
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
        return Err(TypeError {
            span: span.clone(),
            kind: TypeErrorKind::EmptyUnion { name: name.to_string() },
        });
    }
    if members.len() == 1 {
        return Err(TypeError {
            span: span.clone(),
            kind: TypeErrorKind::SingleMemberUnion { name: name.to_string() },
        });
    }
    for member in members {
        match member {
            TypeExpr::Path(_) | TypeExpr::Parametric { .. } | TypeExpr::Tuple(_) => {}
            TypeExpr::Fn { .. } => {
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::InvalidUnionMember {
                        union_name: name.to_string(),
                        member_form: format!("{:?}", member),
                        reason: "Fn types are not valid union members (weird dispatch semantics; revisit if a use case surfaces)".to_string(),
                    },
                });
            }
            TypeExpr::Var(_) => {
                return Err(TypeError {
                    span: span.clone(),
                    kind: TypeErrorKind::InvalidUnionMember {
                        union_name: name.to_string(),
                        member_form: format!("{:?}", member),
                        reason: "Var (synthetic unification variable) is not valid in user-written declarations".to_string(),
                    },
                });
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
            return Err(TypeError {
                span: span.clone(),
                kind: TypeErrorKind::CyclicUnion { name: target_name.to_string() },
            });
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
    use super::*;

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
        // Canonical forms — no inner colons.
        for input in &[
            ":Vec<String>",
            ":Vec<i64>",
            ":Result<Option<i64>,wat::kernel::ThreadDiedError>",
            ":fn(i64)->bool",
            ":fn(Vec<String>)->Option<i64>",
            ":HashMap<String,Vec<i64>>",
        ] {
            let r = parse_type_expr(input);
            assert!(r.is_ok(), "expected {} to parse; got: {:?}", input, r);
        }
    }
    fn collect(src: &str) -> Result<(TypeEnv, Vec<WatAST>), TypeError> {
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut env = TypeEnv::new();
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
        let mut env = TypeEnv::new();
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
            TypeDef::Struct(s) => {
                assert_eq!(s.name, ":project::market::Candle");
                assert!(s.type_params.is_empty());
                assert_eq!(s.fields.len(), 4);
                assert_eq!(s.fields[0].0, "open");
                assert_eq!(s.fields[0].1, TypeExpr::Path(":wat::core::f64".into()));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn parametric_struct() {
        // Stone 241.8 — migrated from :wat::core::struct pair-form to defstruct triples.
        let (env, _) = collect(
            r#"(:wat::core::defstruct :my::Container<T>
                  [value <- :T
                   count <- :i64])"#,
        )
        .unwrap();
        let def = env.get(":my::Container").expect("registered");
        match def {
            TypeDef::Struct(s) => {
                assert_eq!(s.type_params, vec!["T".to_string()]);
                assert_eq!(s.fields[0].1, TypeExpr::Path(":T".into()));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn parametric_struct_multiple_params() {
        // Stone 241.8 — migrated from :wat::core::struct pair-form to defstruct triples.
        let (env, _) = collect(
            r#"(:wat::core::defstruct :my::Pair<K,V>
                  [key   <- :K
                   value <- :V])"#,
        )
        .unwrap();
        let def = env.get(":my::Pair").expect("registered");
        if let TypeDef::Struct(s) = def {
            assert_eq!(s.type_params, vec!["K".to_string(), "V".to_string()]);
        } else {
            panic!("expected Struct");
        }
    }

    // ─── Enum ───────────────────────────────────────────────────────────

    #[test]
    fn unit_variant_enum() {
        // Stone 241.9 — migrated from :wat::core::enum to :wat::core::defenum (HARD CUT).
        let (env, _) = collect(r#"(:wat::core::defenum :my::Direction :up :down :left :right)"#).unwrap();
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
            r#"(:wat::core::defenum :my::Event
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
        let (env, _) = collect(
            r#"(:wat::core::defenum :my::Option<T>
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
        assert!(matches!(err, TypeError { kind: TypeErrorKind::MalformedDecl { .. }, .. }));
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
        let (env, _) = collect(r#"(:wat::core::newtype :my::Wrap<T> :T)"#).unwrap();
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
        let (env, _) = collect(r#"(:wat::core::typealias :my::Series<T> :wat::core::Vector<T>)"#).unwrap();
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
        let (env, _) = collect(
            r#"(:wat::core::typealias :my::Scores :wat::core::HashMap<Atom,wat::core::f64>)"#,
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
        assert!(matches!(err, TypeError { kind: TypeErrorKind::DuplicateType { .. }, .. }));
    }

    #[test]
    fn reserved_prefix_rejected() {
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let err = collect(r#"(:wat::core::defstruct :wat::core::MyStruct [x <- :f64])"#).unwrap_err();
        assert!(matches!(err, TypeError { kind: TypeErrorKind::ReservedPrefix { .. }, .. }));

        let err = collect(r#"(:wat::core::defstruct :wat::holon::Bad [x <- :f64])"#).unwrap_err();
        assert!(matches!(err, TypeError { kind: TypeErrorKind::ReservedPrefix { .. }, .. }));

        let err = collect(r#"(:wat::core::defstruct :wat::std::Bad [x <- :f64])"#).unwrap_err();
        assert!(matches!(err, TypeError { kind: TypeErrorKind::ReservedPrefix { .. }, .. }));
    }

    #[test]
    fn malformed_newtype_arity_rejected() {
        let err = collect(r#"(:wat::core::newtype :my::T)"#).unwrap_err();
        assert!(matches!(err, TypeError { kind: TypeErrorKind::MalformedDecl { .. }, .. }));
    }

    #[test]
    fn malformed_field_rejected() {
        // Stone 241.8 — migrated to defstruct; old MalformedField (pair-form) replaced by
        // MalformedDecl from parse_argspec_triples (name-not-symbol / missing-arrow variants).
        // Incomplete triple [x] fails with MalformedDecl.
        let err = collect(r#"(:wat::core::defstruct :my::T [x])"#).unwrap_err();
        assert!(matches!(err, TypeError { kind: TypeErrorKind::MalformedDecl { .. }, .. }));
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
        assert!(
            err.contains("UnclosedBracketInKeyword")
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
        assert_eq!(
            parse_type_expr(":wat::core::Vector<T>").unwrap(),
            TypeExpr::Parametric {
                head: "wat::core::Vector".into(),
                args: vec![TypeExpr::Path(":T".into())]
            }
        );
    }

    #[test]
    fn type_expr_parametric_nested() {
        let t = parse_type_expr(":wat::core::HashMap<wat::core::String,fn(i32)->i32>").unwrap();
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
    fn type_expr_tuple_with_nested_parametric() {
        // :(Vec<i64>,HashMap<String,i64>) — nested commas at depth > 0
        // must not split the outer tuple.
        let t = parse_type_expr(":(Vec<i64>,HashMap<String,i64>)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], TypeExpr::Parametric { .. }));
                assert!(matches!(elements[1], TypeExpr::Parametric { .. }));
            }
            other => panic!("expected 2-tuple of parametrics, got {:?}", other),
        }
    }

    #[test]
    fn type_expr_tuple_malformed_rejected() {
        // Missing closing ')'.
        assert!(parse_type_expr(":(i64,String").is_err());
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
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "expected TypeError Display to carry real source coordinates (file:line:col); got: {}",
            rendered
        );
        assert!(
            matches!(err, TypeError { kind: TypeErrorKind::MalformedDecl { .. }, .. }),
            "expected MalformedDecl, got: {:?}",
            err
        );
    }
}
