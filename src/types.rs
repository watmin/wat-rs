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
//! # What's deferred
//!
//! - Validation that every field-type reference resolves to a declared
//!   type. The name-resolution pass handles call heads but doesn't
//!   yet walk nested field-position references.
//! - Code generation for Rust-backed compiled binaries (wat-to-rust,
//!   Track 2 of the 058 backlog — not slated for wat-rs).

use crate::ast::WatAST;
use std::collections::HashMap;
use std::fmt;

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
    /// `:Vec<T>`, `:HashMap<K,V>`, `:my::ns::Container<wat::holon::HolonAST,f64>`.
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

/// Struct declaration — named product type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, TypeExpr)>,
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

/// One of the four declaration variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
}

impl TypeDef {
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Struct(s) => &s.name,
            TypeDef::Enum(e) => &e.name,
            TypeDef::Newtype(n) => &n.name,
            TypeDef::Alias(a) => &a.name,
        }
    }
}

/// Keyword-path ↦ `TypeDef` registry.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    types: HashMap<String, TypeDef>,
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

    pub fn register(&mut self, def: TypeDef) -> Result<(), TypeError> {
        let name = def.name().to_string();
        if crate::resolve::is_reserved_prefix(&name) {
            return Err(TypeError::ReservedPrefix { name });
        }
        if self.types.contains_key(&name) {
            return Err(TypeError::DuplicateType { name });
        }
        // Reject cyclic aliases BEFORE insertion so `expand_alias` can
        // assume every alias in the registry is non-cyclic.
        if let TypeDef::Alias(alias) = &def {
            check_alias_no_cycle(&name, &alias.expr, self)?;
        }
        self.types.insert(name, def);
        Ok(())
    }

    /// Register a TRUSTED stdlib type declaration. Bypasses the
    /// reserved-prefix gate because stdlib wat files live under
    /// `:wat::std::*` by design — same privilege that
    /// [`crate::macros::MacroRegistry::register_stdlib`] grants
    /// stdlib defmacros. User source still goes through
    /// [`Self::register`] where the prefix check catches
    /// mis-namespaced user declarations.
    ///
    /// Duplicates and cyclic aliases are still rejected.
    pub fn register_stdlib(&mut self, def: TypeDef) -> Result<(), TypeError> {
        let name = def.name().to_string();
        if self.types.contains_key(&name) {
            return Err(TypeError::DuplicateType { name });
        }
        if let TypeDef::Alias(alias) = &def {
            check_alias_no_cycle(&name, &alias.expr, self)?;
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
            ("cost".into(), TypeExpr::Path(":i64".into())),
            ("budget".into(), TypeExpr::Path(":i64".into())),
        ],
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
            head: "Result".into(),
            args: vec![
                TypeExpr::Path(":wat::holon::HolonAST".into()),
                TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
            ],
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
            ("kind".into(), TypeExpr::Path(":String".into())),
            ("message".into(), TypeExpr::Path(":String".into())),
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
            ("file".into(), TypeExpr::Path(":String".into())),
            ("line".into(), TypeExpr::Path(":i64".into())),
            ("col".into(), TypeExpr::Path(":i64".into())),
        ],
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
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
            (
                "line".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":i64".into())],
                },
            ),
            (
                "symbol".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
        ],
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
            ("message".into(), TypeExpr::Path(":String".into())),
            (
                "location".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Location".into())],
                },
            ),
            (
                "frames".into(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Frame".into())],
                },
            ),
            (
                "actual".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
            (
                "expected".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
        ],
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
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
            (
                "stderr".into(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ),
            (
                "failure".into(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::Failure".into())],
                },
            ),
        ],
    }));

    // :wat::kernel::ForkedChild — return type of
    // `:wat::kernel::fork-with-forms` (arc 012 slice 2). Holds the
    // child's pid-bearing handle plus the three parent-side pipe
    // ends. Fields:
    //   - handle — opaque ChildHandle; feeds into wait-child.
    //   - stdin  — parent writes, child reads fd 0.
    //   - stdout — parent reads, child wrote fd 1.
    //   - stderr — parent reads, child wrote fd 2.
    //
    // Auto-generated `ForkedChild/new` + per-field accessors land
    // in the symbol table at freeze time via register_struct_methods.
    env.register_builtin(TypeDef::Struct(StructDef {
        name: ":wat::kernel::ForkedChild".into(),
        type_params: vec![],
        fields: vec![
            (
                "handle".into(),
                TypeExpr::Path(":wat::kernel::ChildHandle".into()),
            ),
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
        ],
    }));
}

/// Type-declaration errors.
#[derive(Debug)]
pub enum TypeError {
    DuplicateType { name: String },
    ReservedPrefix { name: String },
    MalformedDecl { head: String, reason: String },
    MalformedName { raw: String, reason: String },
    MalformedField { reason: String },
    MalformedVariant { reason: String },
    MalformedTypeExpr { raw: String, reason: String },
    /// User source wrote `:Any` (as a bare path or parametric head).
    /// 058-030 forbids the escape hatch; every apparent use has a
    /// principled alternative (`:wat::holon::HolonAST`, parametric T, or a named
    /// enum).
    AnyBanned { raw: String },
    /// A typealias's expansion, traced through the currently-registered
    /// aliases, reaches the alias's own name. Detected at registration
    /// time so the wat refuses to start rather than looping at
    /// unification later. Example:
    /// `(typealias :A :B) (typealias :B :A)` — the second registration
    /// fires this error because walking `:B`'s expression reaches `:A`
    /// which already expands to `:B`.
    CyclicAlias { name: String },
    /// A parametric typealias was referenced with the wrong number of
    /// type arguments. Example: `(typealias :Pair<A,B> :(A,B))` used as
    /// `:Pair<i64>` — declared 2 params, supplied 1.
    AliasArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::DuplicateType { name } => {
                write!(f, "duplicate type declaration: {}", name)
            }
            TypeError::ReservedPrefix { name } => write!(
                f,
                "type name {} uses a reserved prefix ({}); user types must use their own prefix",
                name,
                crate::resolve::reserved_prefix_list()
            ),
            TypeError::MalformedDecl { head, reason } => {
                write!(f, "malformed {} declaration: {}", head, reason)
            }
            TypeError::MalformedName { raw, reason } => {
                write!(f, "malformed type name {:?}: {}", raw, reason)
            }
            TypeError::MalformedField { reason } => {
                write!(f, "malformed field: {}", reason)
            }
            TypeError::MalformedVariant { reason } => {
                write!(f, "malformed enum variant: {}", reason)
            }
            TypeError::MalformedTypeExpr { raw, reason } => {
                write!(f, "malformed type expression {:?}: {}", raw, reason)
            }
            TypeError::AnyBanned { raw } => write!(
                f,
                ":Any is not part of the type system (058-030); use :wat::holon::HolonAST for any algebra value, a named enum for closed heterogeneous sets, or parametric T/K/V for generics. Offending expression: {}",
                raw
            ),
            TypeError::CyclicAlias { name } => write!(
                f,
                "typealias {} forms a cycle through the current alias graph — refused at registration time so unification doesn't loop",
                name
            ),
            TypeError::AliasArityMismatch { name, expected, got } => write!(
                f,
                "typealias {} declared with {} type parameter(s), used with {}",
                name, expected, got
            ),
        }
    }
}

impl std::error::Error for TypeError {}

/// Walk `forms`, register every type declaration, return the remaining
/// forms in order.
pub fn register_types(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
) -> Result<Vec<WatAST>, TypeError> {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        match classify_type_decl(&form) {
            Some(head) => {
                let def = parse_type_decl(head, form)?;
                env.register(def)?;
            }
            None => rest.push(form),
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant of [`register_types`] that bypasses the
/// `:wat::*` reserved-prefix gate. Called by the startup pipeline on
/// the baked stdlib sources so stdlib wat files can declare types
/// (typealiases, structs, enums, newtypes) under `:wat::std::*`.
/// Mirrors [`crate::macros::register_stdlib_defmacros`]'s privileged
/// path.
pub fn register_stdlib_types(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
) -> Result<Vec<WatAST>, TypeError> {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        match classify_type_decl(&form) {
            Some(head) => {
                let def = parse_type_decl(head, form)?;
                env.register_stdlib(def)?;
            }
            None => rest.push(form),
        }
    }
    Ok(rest)
}

fn classify_type_decl(form: &WatAST) -> Option<&'static str> {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            match k.as_str() {
                ":wat::core::struct" => return Some("struct"),
                ":wat::core::enum" => return Some("enum"),
                ":wat::core::newtype" => return Some("newtype"),
                ":wat::core::typealias" => return Some("typealias"),
                _ => {}
            }
        }
    }
    None
}

fn parse_type_decl(head: &str, form: WatAST) -> Result<TypeDef, TypeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError::MalformedDecl {
                head: head.into(),
                reason: "expected list form".into(),
            })
        }
    };
    let mut iter = items.into_iter();
    let _head_kw = iter.next();
    match head {
        "struct" => parse_struct(iter.collect()),
        "enum" => parse_enum(iter.collect()),
        "newtype" => parse_newtype(iter.collect()),
        "typealias" => parse_typealias(iter.collect()),
        _ => unreachable!(),
    }
}

fn parse_struct(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    let mut iter = args.into_iter();
    let name_kw = iter.next().ok_or_else(|| TypeError::MalformedDecl {
        head: "struct".into(),
        reason: "missing name".into(),
    })?;
    let (name, type_params) = parse_declared_name("struct", &name_kw)?;
    let mut fields = Vec::new();
    for item in iter {
        fields.push(parse_field(item)?);
    }
    Ok(TypeDef::Struct(StructDef {
        name,
        type_params,
        fields,
    }))
}

fn parse_enum(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    let mut iter = args.into_iter();
    let name_kw = iter.next().ok_or_else(|| TypeError::MalformedDecl {
        head: "enum".into(),
        reason: "missing name".into(),
    })?;
    let (name, type_params) = parse_declared_name("enum", &name_kw)?;
    let mut variants = Vec::new();
    for item in iter {
        variants.push(parse_enum_variant(item)?);
    }
    if variants.is_empty() {
        return Err(TypeError::MalformedDecl {
            head: "enum".into(),
            reason: "enum must have at least one variant".into(),
        });
    }
    Ok(TypeDef::Enum(EnumDef {
        name,
        type_params,
        variants,
    }))
}

fn parse_newtype(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError::MalformedDecl {
            head: "newtype".into(),
            reason: format!(
                "expected (:wat::core::newtype :name :InnerType); got {} args",
                args.len()
            ),
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let inner_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("newtype", &name_kw)?;
    let inner = match inner_kw {
        WatAST::Keyword(k, _) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedDecl {
                head: "newtype".into(),
                reason: format!(
                    "inner type must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok(TypeDef::Newtype(NewtypeDef {
        name,
        type_params,
        inner,
    }))
}

fn parse_typealias(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError::MalformedDecl {
            head: "typealias".into(),
            reason: format!(
                "expected (:wat::core::typealias :name :Expr); got {} args",
                args.len()
            ),
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let expr_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("typealias", &name_kw)?;
    let expr = match expr_kw {
        WatAST::Keyword(k, _) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedDecl {
                head: "typealias".into(),
                reason: format!(
                    "alias expression must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok(TypeDef::Alias(AliasDef {
        name,
        type_params,
        expr,
    }))
}

/// `(field-name :Type)` — typed field form used by structs + tagged enum variants.
fn parse_field(form: WatAST) -> Result<(String, TypeExpr), TypeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError::MalformedField {
                reason: "field must be a (name :Type) list".into(),
            })
        }
    };
    if items.len() != 2 {
        return Err(TypeError::MalformedField {
            reason: format!(
                "field must be exactly (name :Type); got {} elements",
                items.len()
            ),
        });
    }
    let mut iter = items.into_iter();
    let name = match iter.next().unwrap() {
        WatAST::Symbol(ident, _) => ident.name,
        other => {
            return Err(TypeError::MalformedField {
                reason: format!(
                    "field name must be a bare symbol; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    let ty = match iter.next().unwrap() {
        WatAST::Keyword(k, _) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedField {
                reason: format!(
                    "field type must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok((name, ty))
}

/// A variant is either a bare keyword (`:unit-variant`) or a list
/// `(tagged-variant (field :Type) ...)`.
fn parse_enum_variant(form: WatAST) -> Result<EnumVariant, TypeError> {
    match form {
        WatAST::Keyword(k, _) => {
            let name = k
                .strip_prefix(':')
                .ok_or_else(|| TypeError::MalformedVariant {
                    reason: format!("unit variant must be a keyword; got {:?}", k),
                })?
                .to_string();
            Ok(EnumVariant::Unit(name))
        }
        WatAST::List(items, _) => {
            let mut iter = items.into_iter();
            let name_sym = iter.next().ok_or_else(|| TypeError::MalformedVariant {
                reason: "tagged variant must have a name".into(),
            })?;
            let name = match name_sym {
                WatAST::Symbol(ident, _) => ident.name,
                WatAST::Keyword(k, _) => k
                    .strip_prefix(':')
                    .map(str::to_string)
                    .unwrap_or(k),
                other => {
                    return Err(TypeError::MalformedVariant {
                        reason: format!(
                            "variant name must be a symbol or keyword; got {}",
                            ast_variant_name(&other)
                        ),
                    })
                }
            };
            let mut fields = Vec::new();
            for item in iter {
                fields.push(parse_field(item)?);
            }
            Ok(EnumVariant::Tagged { name, fields })
        }
        other => Err(TypeError::MalformedVariant {
            reason: format!(
                "variant must be a keyword (unit) or list (tagged); got {}",
                ast_variant_name(&other)
            ),
        }),
    }
}

/// Parse a declared type name. Accepts:
/// - `:my::ns::MyType` → ("my/ns/MyType", [])
/// - `:my::ns::Wrapper<T>` → ("my/ns/Wrapper", ["T"])
/// - `:my::ns::Container<K,V>` → ("my/ns/Container", ["K", "V"])
fn parse_declared_name(
    head: &str,
    form: &WatAST,
) -> Result<(String, Vec<String>), TypeError> {
    let raw = match form {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(TypeError::MalformedDecl {
                head: head.into(),
                reason: format!(
                    "name must be a keyword; got {}",
                    ast_variant_name(other)
                ),
            })
        }
    };
    // Strip the colon but keep the rest as the key for TypeEnv.
    let stripped = raw.strip_prefix(':').ok_or_else(|| TypeError::MalformedName {
        raw: raw.clone(),
        reason: "keyword must begin with ':'".into(),
    })?;
    // Split at first '<' if present.
    match stripped.find('<') {
        None => Ok((raw, Vec::new())),
        Some(lt_index) => {
            let base = &stripped[..lt_index];
            let params_part = &stripped[lt_index..];
            if !params_part.ends_with('>') {
                return Err(TypeError::MalformedName {
                    raw: raw.clone(),
                    reason: "parametric name must close with '>'".into(),
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
                    return Err(TypeError::MalformedName {
                        raw: raw.clone(),
                        reason: format!("type parameter {:?} has invalid chars", p),
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
pub fn parse_type_expr(kw: &str) -> Result<TypeExpr, TypeError> {
    let stripped = kw.strip_prefix(':').ok_or_else(|| TypeError::MalformedTypeExpr {
        raw: kw.into(),
        reason: "type expression keyword must begin with ':'".into(),
    })?;
    let expr = parse_type_inner(stripped, kw)?;
    reject_any(&expr, kw)?;
    Ok(expr)
}

/// Walk a parsed [`TypeExpr`] and raise [`TypeError::AnyBanned`] if
/// `:Any` appears anywhere. Protects the type universe's closure.
fn reject_any(expr: &TypeExpr, raw: &str) -> Result<(), TypeError> {
    match expr {
        TypeExpr::Path(p) => {
            if p == ":Any" {
                return Err(TypeError::AnyBanned { raw: raw.into() });
            }
        }
        TypeExpr::Parametric { head, args } => {
            if head == "Any" {
                return Err(TypeError::AnyBanned { raw: raw.into() });
            }
            for a in args {
                reject_any(a, raw)?;
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                reject_any(a, raw)?;
            }
            reject_any(ret, raw)?;
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                reject_any(e, raw)?;
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
fn parse_type_inner(s: &str, original: &str) -> Result<TypeExpr, TypeError> {
    // Tuple literal — `(T,U,...)`. Must appear at the start; inner
    // types respect top-level comma splitting.
    if let Some(rest) = s.strip_prefix('(') {
        if !rest.ends_with(')') {
            return Err(TypeError::MalformedTypeExpr {
                raw: original.into(),
                reason: "tuple-literal type must close with ')'".into(),
            });
        }
        let inside = &rest[..rest.len() - 1];
        return parse_tuple_body(inside, original);
    }
    // `fn(args)->ret` function type — detect at the start.
    if let Some(body) = s.strip_prefix("fn(") {
        return parse_fn_body(body, original);
    }
    // `Head<args>` parametric.
    if let Some(lt_index) = find_top_level_char(s, '<') {
        let head = s[..lt_index].to_string();
        let rest = &s[lt_index..];
        if !rest.ends_with('>') {
            return Err(TypeError::MalformedTypeExpr {
                raw: original.into(),
                reason: "parametric type must close with '>'".into(),
            });
        }
        let inside = &rest[1..rest.len() - 1];
        let args = parse_type_list(inside, original)?;
        return Ok(TypeExpr::Parametric { head, args });
    }
    // Plain path.
    Ok(TypeExpr::Path(format!(":{}", s)))
}

/// Parse the body of a tuple-literal type.
///
/// - Empty body `` → unit (0-tuple): `Tuple(vec![])`.
/// - Single type with no trailing comma: Rust grouping — returns the
///   inner type directly (NOT wrapped in Tuple).
/// - Trailing comma or multiple elements: `Tuple(vec![...])`.
///
/// Matches Rust's tuple-type syntax exactly.
fn parse_tuple_body(inside: &str, original: &str) -> Result<TypeExpr, TypeError> {
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
    let elements = parse_type_list(effective, original)?;
    if elements.len() == 1 && !has_trailing_comma {
        // `:(T)` is grouping — return the inner type unwrapped.
        return Ok(elements.into_iter().next().unwrap());
    }
    Ok(TypeExpr::Tuple(elements))
}

fn parse_fn_body(body: &str, original: &str) -> Result<TypeExpr, TypeError> {
    // body is `T,U)->R` — find the matching `)` at depth 0.
    let close = find_matching_close(body, '(', ')').ok_or_else(|| {
        TypeError::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing matching ')'".into(),
        }
    })?;
    let args_part = &body[..close];
    let tail = &body[close + 1..];
    let ret_part = tail
        .strip_prefix("->")
        .ok_or_else(|| TypeError::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing '->' before return".into(),
        })?;
    let args = if args_part.trim().is_empty() {
        Vec::new()
    } else {
        parse_type_list(args_part, original)?
    };
    let ret = parse_type_inner(ret_part, original)?;
    Ok(TypeExpr::Fn {
        args,
        ret: Box::new(ret),
    })
}

/// Parse a comma-separated list of types (respecting nested `<>` and `()`).
fn parse_type_list(s: &str, original: &str) -> Result<Vec<TypeExpr>, TypeError> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                let piece = &s[start..i];
                out.push(parse_type_inner(piece.trim(), original)?);
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = &s[start..];
    if !tail.trim().is_empty() {
        out.push(parse_type_inner(tail.trim(), original)?);
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

fn ast_variant_name(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_, _) => "int literal",
        WatAST::FloatLit(_, _) => "float literal",
        WatAST::BoolLit(_, _) => "bool literal",
        WatAST::StringLit(_, _) => "string literal",
        WatAST::Keyword(_, _) => "keyword",
        WatAST::Symbol(_, _) => "symbol",
        WatAST::List(_, _) => "list",
    }
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
) -> Result<(), TypeError> {
    let mut visiting = std::collections::HashSet::new();
    check_alias_reaches(target_name, expr, env, &mut visiting)
}

fn check_alias_reaches(
    target_name: &str,
    expr: &TypeExpr,
    env: &TypeEnv,
    visiting: &mut std::collections::HashSet<String>,
) -> Result<(), TypeError> {
    match expr {
        TypeExpr::Path(name) => {
            if name == target_name {
                return Err(TypeError::CyclicAlias {
                    name: target_name.to_string(),
                });
            }
            if let Some(TypeDef::Alias(alias)) = env.get(name) {
                if visiting.insert(name.clone()) {
                    check_alias_reaches(target_name, &alias.expr, env, visiting)?;
                    visiting.remove(name);
                }
            }
        }
        TypeExpr::Parametric { head, args } => {
            let qualified = format!(":{}", head);
            if qualified == target_name {
                return Err(TypeError::CyclicAlias {
                    name: target_name.to_string(),
                });
            }
            if let Some(TypeDef::Alias(alias)) = env.get(&qualified) {
                if visiting.insert(qualified.clone()) {
                    check_alias_reaches(target_name, &alias.expr, env, visiting)?;
                    visiting.remove(&qualified);
                }
            }
            for a in args {
                check_alias_reaches(target_name, a, env, visiting)?;
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                check_alias_reaches(target_name, a, env, visiting)?;
            }
            check_alias_reaches(target_name, ret, env, visiting)?;
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                check_alias_reaches(target_name, e, env, visiting)?;
            }
        }
        TypeExpr::Var(_) => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_all;

    fn collect(src: &str) -> Result<(TypeEnv, Vec<WatAST>), TypeError> {
        let forms = parse_all(src).expect("parse ok");
        let mut env = TypeEnv::new();
        let rest = register_types(forms, &mut env)?;
        Ok((env, rest))
    }

    // ─── Struct ─────────────────────────────────────────────────────────

    #[test]
    fn simple_struct() {
        let (env, rest) = collect(
            r#"(:wat::core::struct :project::market::Candle
                  (open :f64)
                  (high :f64)
                  (low :f64)
                  (close :f64))"#,
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
                assert_eq!(s.fields[0].1, TypeExpr::Path(":f64".into()));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn parametric_struct() {
        let (env, _) = collect(
            r#"(:wat::core::struct :my::Container<T>
                  (value :T)
                  (count :i64))"#,
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
        let (env, _) = collect(
            r#"(:wat::core::struct :my::Pair<K,V>
                  (key :K)
                  (value :V))"#,
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
        let (env, _) = collect(r#"(:wat::core::enum :my::Direction :up :down :left :right)"#).unwrap();
        if let TypeDef::Enum(e) = env.get(":my::Direction").unwrap() {
            assert_eq!(e.variants.len(), 4);
            assert!(matches!(&e.variants[0], EnumVariant::Unit(n) if n == "up"));
        } else {
            panic!("expected Enum");
        }
    }

    #[test]
    fn tagged_variant_enum() {
        let (env, _) = collect(
            r#"(:wat::core::enum :my::Event
                  :empty
                  (candle (open :f64) (close :f64))
                  (deposit (amount :f64)))"#,
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
        let (env, _) = collect(
            r#"(:wat::core::enum :my::Option<T>
                  :none
                  (some (value :T)))"#,
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
        let err = collect(r#"(:wat::core::enum :my::Empty)"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedDecl { .. }));
    }

    // ─── Newtype ────────────────────────────────────────────────────────

    #[test]
    fn simple_newtype() {
        let (env, _) = collect(r#"(:wat::core::newtype :my::trading::Price :f64)"#).unwrap();
        if let TypeDef::Newtype(n) = env.get(":my::trading::Price").unwrap() {
            assert_eq!(n.inner, TypeExpr::Path(":f64".into()));
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
        let (env, _) = collect(r#"(:wat::core::typealias :my::Amount :f64)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Amount").unwrap() {
            assert_eq!(a.expr, TypeExpr::Path(":f64".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parametric_typealias() {
        let (env, _) = collect(r#"(:wat::core::typealias :my::Series<T> :Vec<T>)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Series").unwrap() {
            assert_eq!(a.type_params, vec!["T".to_string()]);
            assert_eq!(
                a.expr,
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":T".into())]
                }
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn typealias_function_type() {
        let (env, _) = collect(r#"(:wat::core::typealias :my::Predicate :fn(wat::holon::HolonAST)->bool)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Predicate").unwrap() {
            match &a.expr {
                TypeExpr::Fn { args, ret } => {
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                    assert_eq!(**ret, TypeExpr::Path(":bool".into()));
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
            r#"(:wat::core::typealias :my::Scores :HashMap<Atom,f64>)"#,
        )
        .unwrap();
        if let TypeDef::Alias(a) = env.get(":my::Scores").unwrap() {
            match &a.expr {
                TypeExpr::Parametric { head, args } => {
                    assert_eq!(head, "HashMap");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], TypeExpr::Path(":Atom".into()));
                    assert_eq!(args[1], TypeExpr::Path(":f64".into()));
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
        let err = collect(
            r#"
            (:wat::core::struct :my::T (x :f64))
            (:wat::core::struct :my::T (y :i64))
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, TypeError::DuplicateType { .. }));
    }

    #[test]
    fn reserved_prefix_rejected() {
        let err = collect(r#"(:wat::core::struct :wat::core::MyStruct (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));

        let err = collect(r#"(:wat::core::struct :wat::holon::Bad (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));

        let err = collect(r#"(:wat::core::struct :wat::std::Bad (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));
    }

    #[test]
    fn malformed_newtype_arity_rejected() {
        let err = collect(r#"(:wat::core::newtype :my::T)"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedDecl { .. }));
    }

    #[test]
    fn malformed_field_rejected() {
        let err = collect(r#"(:wat::core::struct :my::T (oops))"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedField { .. }));
    }

    #[test]
    fn malformed_parametric_name_rejected() {
        let err = collect(r#"(:wat::core::struct :my::Bad<T (x :T))"#).unwrap_err();
        // `:my::Bad<T` (no closing `>`) — under the keyword-lexer rules
        // either the lexer errors out (unterminated) or the type
        // declaration complains. Either way, an error surfaces.
        assert!(matches!(err, TypeError::MalformedName { .. } | TypeError::MalformedDecl { .. }));
    }

    // ─── Non-type forms pass through ────────────────────────────────────

    #[test]
    fn non_type_forms_preserved() {
        let (_env, rest) = collect(
            r#"
            (:wat::core::struct :my::T (x :f64))
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
            parse_type_expr(":f64").unwrap(),
            TypeExpr::Path(":f64".into())
        );
        assert_eq!(
            parse_type_expr(":my::ns::MyType").unwrap(),
            TypeExpr::Path(":my::ns::MyType".into())
        );
    }

    #[test]
    fn type_expr_parametric() {
        assert_eq!(
            parse_type_expr(":Vec<T>").unwrap(),
            TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Path(":T".into())]
            }
        );
    }

    #[test]
    fn type_expr_parametric_nested() {
        let t = parse_type_expr(":HashMap<String,fn(i32)->i32>").unwrap();
        match t {
            TypeExpr::Parametric { head, args } => {
                assert_eq!(head, "HashMap");
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
        let t = parse_type_expr(":(i64,String)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], TypeExpr::Path(":i64".into()));
                assert_eq!(elements[1], TypeExpr::Path(":String".into()));
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
        let t = parse_type_expr(":(i64)").unwrap();
        assert_eq!(t, TypeExpr::Path(":i64".into()));
    }

    #[test]
    fn type_expr_tuple_one_element_trailing_comma_is_tuple() {
        // :(T,) is the explicit 1-tuple.
        let t = parse_type_expr(":(i64,)").unwrap();
        match t {
            TypeExpr::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], TypeExpr::Path(":i64".into()));
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
                        assert_eq!(head, "Result");
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
                assert_eq!(head, "Result");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], TypeExpr::Path(":wat::holon::HolonAST".into()));
                assert_eq!(args[1], TypeExpr::Path(":wat::holon::CapacityExceeded".into()));
            }
            other => panic!("expected expanded Result<HolonAST,CapacityExceeded>, got {:?}", other),
        }
    }
}
