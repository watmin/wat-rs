//! Arc 109 (DESIGN-STONE-a-type-reference-must-resolve) — the declared-type-position sweep.
//!
//! [`sweep_type_references`] closes the mirror-image gap next to `is_resolvable_call_head`:
//! a CALL head that names nothing fails resolve (`UnresolvedReferences`); a TYPE reference that
//! names nothing was accepted forever, silently — surfacing only if some caller happened to
//! exercise it, and then blaming the CALLER (`TypeMismatch`) rather than the declaration.
//!
//! ## Why this is a REGISTRY sweep, not a form walk
//!
//! `register_types_impl` (`types.rs`) CONSUMES every type declaration form
//! (`defrecord`/`defstruct`/`defenum`/`typealias`/…) at freeze step 5 — the residue step 7 walks
//! never contains them again. So this pass iterates the two registries that step 5/6 already
//! built, rather than re-walking forms:
//!
//! - `TypeEnv`'s own `TypeDef`s — field types (Aggregate), variant payloads (Enum), the aliased
//!   expression (Alias), the newtype's inner, typeunion members, and surface member (field +
//!   method) signatures.
//! - `SymbolTable`'s registered `Function`s — declared param types, return type, rest-param
//!   type. Anonymous `fn` VALUES (`name: None`) never reach `SymbolTable.functions` — they are
//!   runtime values evaluated inside a body, and D2-A ("declared positions only") already puts
//!   function bodies out of scope. There is consequently no NESTED scope to construct here: every
//!   walked `TypeExpr` belongs to exactly one owning declaration (one `TypeDef` or one
//!   `Function`), and its bound set is that declaration's OWN `type_params` — flat, never merged
//!   with an enclosing one, mirroring `check.rs`'s `derive_scheme_from_function` (which likewise
//!   builds each scheme's `type_params` solely from that function's own declared binder, with no
//!   parent-scope concept — and returns `None` for `name: None` functions, i.e. the checker has
//!   no scheme, and so no scope, for a bare `fn` value at all).
//!
//! ## The one hard constraint — type VARIABLES are `Path`s
//!
//! `TypeExpr::Path` is the SAME node kind for a bare type variable (`T`, `K`, `V`) and an
//! unresolvable name (`NoSuchType`) — `TypeExpr::Var(u64)` is synthetic and never produced by
//! parsing (see `types.rs`'s `TypeExpr` doc). So a `Path` is resolved iff it names a registered
//! type OR its bare (colon-stripped) form is a member of the enclosing declaration's
//! `type_params` — never by a naming-convention heuristic (`runtime::is_type_var_path` exists
//! for a DIFFERENT purpose — inferring free-variable names when no declared binder is in scope —
//! and is deliberately NOT reused here: this pass always has the real declared binder, and using
//! it is strictly more correct than pattern-matching on capitalization).
//!
//! ## ★ A second gap, measured rather than assumed — `TypeEnv` does not know the primitives
//!
//! See [`known_builtin_leaf_types`]'s doc for the full measurement. In short: `TypeEnv::
//! with_builtins()` registers 80 names, every one an aggregate-shaped record; scalar primitives
//! (`i64`, `f64`, `bool`, …) and the built-in parametric containers (`Vector`, `HashMap`, …) are
//! NOT among them. `known_builtin_leaf_types` is the closed-world list this sweep needs in
//! addition to `TypeEnv::contains`.

use std::collections::HashSet;

use crate::runtime::{Function, FunctionBody, SymbolTable};
use crate::span::Span;
use crate::types::{
    parametric_head_fqdn, AggregateDef, AliasDef, EnumDef, EnumVariant, NewtypeDef, SurfaceDef,
    SurfaceMember, TypeDef, TypeEnv, TypeExpr, UnionDef,
};

use super::error::{ReferenceKind, UnresolvedReference};

/// ★ MEASURED, NOT ASSUMED — `TypeEnv::with_builtins()` does NOT register the scalar
/// primitives or the built-in parametric containers as `TypeDef`s. Probed directly against the
/// disk (a throwaway test dumping `TypeEnv::with_builtins().iter()`): 80 names total, every one
/// of them an aggregate-shaped error/outcome/result record (`:wat::core::Option`,
/// `:wat::core::Result`, `:wat::core::Struct`, `:wat::core::Record`, `:wat::core::nil`, …).
/// `contains(":wat::core::i64")` is **false**. This directly contradicts the DESIGN's own "the
/// ground" section ("`with_builtins()` → `register_builtin_types` so `contains` covers
/// `:wat::core::i64` etc.") — a load-bearing premise that does not hold on today's disk.
///
/// The reason: nothing in this codebase has EVER validated "is this a real type name" before
/// this stone. Literal inference (`check.rs::infer`, e.g. `WatAST::IntLit => Path(":wat::core::
/// i64")`) constructs these paths directly; `register_builtins` (the CHECK-layer's native-op
/// scheme table, a SEPARATE registry from `TypeEnv`) uses them as hardcoded Rust literals; unify
/// only ever compares two `TypeExpr`s STRUCTURALLY, never asks a registry "does this exist". A
/// phantom sails through today for exactly this reason — nothing has ever had to know the full
/// set of real names, until now.
///
/// This function returns that set — the first explicit closed-world list of built-in LEAF type
/// names this language has had. Two sources are REUSED, not re-derived (`check::BARE_PRIMITIVES`
/// / `check::BARE_CONTAINER_HEADS` — the legacy-bare-spelling migration consts, whose FQDN
/// column is exactly this same set for the primitives+containers that once had a bare spelling);
/// the remainder is every additional scalar/sentinel Path the checker constructs as a literal
/// (`check.rs::infer`, `is_atomizable`) that has no `TypeDef` and never had a bare spelling to
/// leave a const trail. Validated empirically: `target/release/wat --check` against the full
/// stdlib (loaded by ANY user program, `.wat` and `wat-scripts/`) plus the scoped
/// `wat::types`/`wat::resolve` test binaries.
fn known_builtin_leaf_types() -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();
    for (_, fqdn) in crate::check::BARE_PRIMITIVES {
        set.insert((*fqdn).to_string());
    }
    for (_, fqdn) in crate::check::BARE_CONTAINER_HEADS {
        // `BARE_CONTAINER_HEADS`'s FQDN column follows `TypeExpr::Parametric.head`'s own
        // convention (no leading colon — see `parametric_head_fqdn`'s doc); add one back for a
        // uniform, colon-prefixed `contains` check against every entry in this set.
        set.insert(format!(":{fqdn}"));
    }
    for extra in [
        // Additional scalar primitives with no bare-legacy trail (check.rs::infer literal
        // construction sites: RationalLit, BigIntLit, Keyword).
        ":wat::core::bigint",
        ":wat::core::rational",
        ":wat::core::keyword",
        // The two hologram-algebra leaf types (`is_atomizable`'s own enumeration).
        ":wat::holon::HolonAST",
        ":wat::WatAST",
        // Generic escape-hatch sentinels used as literal return types by native ops
        // (`register_builtins` — the check-layer's own, SEPARATE scheme registry).
        ":wat::core::Value",
        ":wat::core::Never",
        // A bare parametric container with no bare-legacy pairing (never had a bare spelling
        // to retire, per `check.rs`'s harvest of Parametric.head literals).
        ":wat::core::List",
        // Arc 109 — measured empirically (`target/release/wat --check` against a trivial
        // program, which loads the full stdlib): 25 distinct phantom names, 312 sites, before
        // this batch. Every name below is an OPAQUE capability/handle type — a Rust struct
        // exposed to wat with no `TypeDef` (no fields a wat program can see; it's a token, not
        // a structure) — used pervasively (up to 92 sites each) across `wat/kernel*.wat`,
        // `wat/io.wat`, `wat/stream.wat`, `wat/time.wat`, `wat/holon.wat`. None had a bare-
        // legacy spelling to leave a const trail (they're arc-278+ additions, past the arc 109
        // migration window `BARE_PRIMITIVES`/`BARE_CONTAINER_HEADS` date from).
        ":wat::core::Uuid",
        ":wat::holon::Hologram",
        ":wat::holon::Vector",
        ":wat::io::IOReader",
        ":wat::io::IOWriter",
        // Found by the SAME empirical sweep, one layer later: the scoped `wat::types` test
        // binary's broader fixture corpus (not just the trivial stdlib-only stub) surfaced two
        // more opaque handles from `wat/kernel_process.wat`/`wat/kernel_thread.wat`-shaped
        // fixtures.
        ":wat::kernel::Process",
        ":wat::kernel::Thread",
        ":wat::kernel::Address",
        ":wat::kernel::Listener",
        ":wat::kernel::Peer",
        ":wat::kernel::ThreadSelfPeer",
        ":wat::stream::Stream",
        ":wat::time::Duration",
        ":wat::time::Instant",
        // `:wat::kernel::Sender`/`Receiver`'s typealias RHS. `crossbeam_channel` backs the
        // kernel's channel primitives DIRECTLY (a core runtime dependency, not a pluggable
        // shim), so — unlike `:rust::cache::Lru` / `:rust::sqlite::Connection` (registered via
        // `rust_deps`, a consumer-registrable shim mechanism for OPTIONAL integrations) —
        // it was never registered as a `RustTypeDecl`. Measured: `rust_deps::registry()
        // .has_type(":rust::crossbeam_channel::Sender")` is false on today's disk.
        ":rust::crossbeam_channel::Sender",
        ":rust::crossbeam_channel::Receiver",
    ] {
        set.insert(extra.to_string());
    }
    set
}

/// The two registries this pass consults on every `Path` / `Parametric.head`, bundled so every
/// walker fn takes one reference instead of two. `builtins` is built once per
/// [`sweep_type_references`] call — see [`known_builtin_leaf_types`].
struct Registry<'a> {
    types: &'a TypeEnv,
    builtins: HashSet<String>,
    unresolved: &'a mut Vec<UnresolvedReference>,
}

impl Registry<'_> {
    /// A `:rust::*` type reference (e.g. `:rust::cache::Lru<K,V>` on a `typealias` RHS bridging
    /// to a Rust-backed shim — see `wat/cache.wat`) is validated against the SAME build-time
    /// `rust_deps` registry the call-head resolver's `:rust::*` coverage check already
    /// consults (`rust_use.rs`'s `collect_use_declarations`, via `registry.has_type`) — one
    /// door, not a second copy of "which `:rust::*` paths are real". Unlike the call-head
    /// check, a type-position reference does NOT additionally require a `(:wat::core::use!
    /// ...)` declaration in scope: `use!` gates CALLING a rust type's methods (pass 2's
    /// concern); naming the type in a declaration is a weaker fact that doesn't need import
    /// coverage, mirroring how a Rust `use` isn't required to write a fully-qualified path.
    fn is_known(&self, path: &str) -> bool {
        if self.types.contains(path) || self.builtins.contains(path) {
            return true;
        }
        path.starts_with(":rust::") && crate::rust_deps::registry().has_type(path)
    }
}

/// Sweep every registered type declaration and every registered function signature for a
/// declared type-expression naming something that isn't a registered type and isn't a type
/// variable bound by the enclosing declaration. Appends findings to `unresolved` (never clears
/// it — this is one of several passes contributing to the same collection, mirroring
/// [`super::walk::resolve_references`]'s own two-pass shape).
pub(super) fn sweep_type_references(
    sym: &SymbolTable,
    types: &TypeEnv,
    unresolved: &mut Vec<UnresolvedReference>,
) {
    let mut reg = Registry { types, builtins: known_builtin_leaf_types(), unresolved };
    for (_name, def) in types.iter() {
        walk_type_def(def, &mut reg);
    }
    for (name, func) in sym.functions_iter() {
        walk_function_signature(name, func, &mut reg);
    }
}

/// The best available location for a diagnostic naming `name`'s declaration. User (non-reserved)
/// types retain their original source form (`TypeEnv::source_form`) — its span IS the
/// declaration's span. Reserved (`:wat::*`) builtins are Rust literals with no source form; there
/// is genuinely no user-file location to point at, so `rust_caller_span!()` names this pass's own
/// call site (the documented meaning of that macro: "no recoverable location").
fn decl_span(types: &TypeEnv, name: &str) -> Span {
    types
        .source_form(name)
        .map(|f| f.span().clone())
        .unwrap_or_else(|| crate::rust_caller_span!())
}

/// The best available location for a registered function's signature: the body form's span.
/// `Function` carries no separate signature span, and the body sits inside the same top-level
/// declaration, immediately after it — closer to the declaration than any alternative available
/// from the registry (see this module's doc — the registry sweep has no per-field spans at all).
fn body_span(func: &Function) -> Span {
    match &func.body {
        FunctionBody::Wat(ast) => ast.span().clone(),
        FunctionBody::Native => crate::rust_caller_span!(),
    }
}

fn walk_type_def(def: &TypeDef, reg: &mut Registry<'_>) {
    let span = decl_span(reg.types, def.name());
    match def {
        TypeDef::Aggregate(a) => walk_aggregate(a, &span, reg),
        TypeDef::Enum(e) => walk_enum(e, &span, reg),
        TypeDef::Newtype(n) => walk_newtype(n, &span, reg),
        TypeDef::Alias(a) => walk_alias(a, &span, reg),
        TypeDef::Union(u) => walk_union(u, &span, reg),
        TypeDef::Surface(s) => walk_surface(s, &span, reg),
    }
}

fn walk_aggregate(a: &AggregateDef, span: &Span, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = a.type_params.iter().map(String::as_str).collect();
    for (fname, ty) in &a.fields {
        let context = format!("field type of {}.{}", a.name, fname);
        walk_type_expr(ty, &bound, &context, span, reg);
    }
}

fn walk_enum(e: &EnumDef, span: &Span, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = e.type_params.iter().map(String::as_str).collect();
    for variant in &e.variants {
        if let EnumVariant::Tagged { name: vname, fields } = variant {
            for (fname, ty) in fields {
                let context = format!("variant payload type of {}::{}.{}", e.name, vname, fname);
                walk_type_expr(ty, &bound, &context, span, reg);
            }
        }
    }
}

fn walk_newtype(n: &NewtypeDef, span: &Span, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = n.type_params.iter().map(String::as_str).collect();
    let context = format!("inner type of newtype {}", n.name);
    walk_type_expr(&n.inner, &bound, &context, span, reg);
}

fn walk_alias(a: &AliasDef, span: &Span, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = a.type_params.iter().map(String::as_str).collect();
    let context = format!("alias body of {}", a.name);
    walk_type_expr(&a.expr, &bound, &context, span, reg);
}

fn walk_union(u: &UnionDef, span: &Span, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = u.type_params.iter().map(String::as_str).collect();
    for (i, ty) in u.members.iter().enumerate() {
        let context = format!("member #{} of typeunion {}", i + 1, u.name);
        walk_type_expr(ty, &bound, &context, span, reg);
    }
}

/// Surface methods declare type params of their OWN (`SurfaceMember::Method.type_params`,
/// parsed from a `method<X>` suffix), IN ADDITION to the surface's own `<K,V>` — a method body
/// routinely uses both (e.g. `Cache<K,V>`'s methods use `K`/`V`; a method could also bind an
/// extra param of its own). The bound set per method is therefore the UNION of the surface's
/// declared `type_params` and that method's own — never the surface's alone, which would
/// wrongly flag a method-only type variable as unresolved.
fn walk_surface(s: &SurfaceDef, span: &Span, reg: &mut Registry<'_>) {
    let surface_bound: HashSet<&str> = s.type_params.iter().map(String::as_str).collect();
    for member in &s.members {
        match member {
            SurfaceMember::Field { name, ty } => {
                let context = format!("field type of surface {}.{}", s.name, name);
                walk_type_expr(ty, &surface_bound, &context, span, reg);
            }
            SurfaceMember::Method { name, args, ret, type_params, .. } => {
                let mut bound = surface_bound.clone();
                for tp in type_params {
                    bound.insert(tp.as_str());
                }
                for (i, (_, ty)) in args.fixed_params.iter().enumerate() {
                    let context =
                        format!("parameter #{} of surface method {}/{}", i + 1, s.name, name);
                    walk_type_expr(ty, &bound, &context, span, reg);
                }
                if let Some((_, ty)) = &args.rest_param {
                    let context = format!("rest-parameter type of surface method {}/{}", s.name, name);
                    walk_type_expr(ty, &bound, &context, span, reg);
                }
                let context = format!("return type of surface method {}/{}", s.name, name);
                walk_type_expr(ret, &bound, &context, span, reg);
            }
        }
    }
}

fn walk_function_signature(name: &str, func: &Function, reg: &mut Registry<'_>) {
    let bound: HashSet<&str> = func.type_params.iter().map(String::as_str).collect();
    let span = body_span(func);
    for (i, ty) in func.param_types.iter().enumerate() {
        let context = format!("type in the signature of {name}, parameter #{}", i + 1);
        walk_type_expr(ty, &bound, &context, &span, reg);
    }
    let ret_context = format!("return type in the signature of {name}");
    walk_type_expr(&func.ret_type, &bound, &ret_context, &span, reg);
    if let Some(rt) = &func.rest_param_type {
        let rest_context = format!("rest-parameter type in the signature of {name}");
        walk_type_expr(rt, &bound, &rest_context, &span, reg);
    }
}

/// Recurse structurally through a `TypeExpr`, checking every `Path` and every
/// `Parametric.head` against the registry + bound set. `Var(_)` is synthetic (never produced by
/// parsing/registration) and is skipped.
fn walk_type_expr(ty: &TypeExpr, bound: &HashSet<&str>, context: &str, span: &Span, reg: &mut Registry<'_>) {
    match ty {
        TypeExpr::Path(p) => check_name(p, bound, context, span, reg),
        TypeExpr::Parametric { head, args } => {
            let head_fqdn = parametric_head_fqdn(head);
            check_name(&head_fqdn, bound, context, span, reg);
            for a in args {
                walk_type_expr(a, bound, context, span, reg);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                walk_type_expr(a, bound, context, span, reg);
            }
            walk_type_expr(ret, bound, context, span, reg);
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                walk_type_expr(e, bound, context, span, reg);
            }
        }
        TypeExpr::Var(_) => {}
    }
}

/// `path` is resolved iff it names a registered type (`TypeEnv::contains`, colon-prefixed FQDN),
/// OR it's one of the built-in scalar/container leaf names ([`known_builtin_leaf_types`]), OR its
/// bare form is bound by the enclosing declaration's own `type_params`. D3-B: no reserved-prefix
/// exemption — every namespace, `:wat::*` included.
fn check_name(path: &str, bound: &HashSet<&str>, context: &str, span: &Span, reg: &mut Registry<'_>) {
    if reg.is_known(path) {
        return;
    }
    let bare = path.strip_prefix(':').unwrap_or(path);
    if bound.contains(bare) {
        return;
    }
    reg.unresolved.push(UnresolvedReference {
        path: path.to_string(),
        context: context.to_string(),
        span: span.clone(),
        kind: ReferenceKind::Type,
    });
}
