//! Canonical environment builder — the ONE pipeline that materialises
//! the registered environment (macros + types + symbols + residue) from
//! a set of already-parsed, already-load-resolved user forms plus the
//! baked stdlib.
//!
//! Previously, three hand-rolled copies of this pipeline existed:
//!   1. Production — inline in `freeze::startup_from_forms_post_config`.
//!   2. Test copy A — `check.rs::tests::stdlib_loaded`.
//!   3. Test copy B — `runtime.rs::tests::stdlib_loaded`.
//!
//! The two test copies had drifted from production (they discarded the
//! stdlib residue and therefore skipped `preregister_stdlib_defclause_stub`
//! and `register_stdlib_runtime_defs`), causing 13 `check::tests::` failures
//! because the checker could not project defclause return types.
//!
//! Cure: ONE canonical builder here; three thin callers; divergence is
//! unrepresentable.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::ast::WatAST;
use crate::check::{
    validate_aggregate_containment, validate_arc170_legacy_callsites,
    validate_bare_legacy_primitives, validate_named_type_annotations, CheckError, CheckErrors,
};
use crate::declare::preregister::{preregister_acronyms, preregister_stdlib_defclause_stub};
use crate::declare::register::{
    register_aggregate_methods, register_defines, register_enum_methods, register_newtype_methods,
    register_stdlib_defines, register_stdlib_runtime_defs, register_struct_methods,
    register_type_predicates,
};
use crate::load::stdlib::stdlib_forms;
use crate::macros::{
    expand_all, expand_all_with, expand_once, register_aggregate_kwargs_companions,
    register_defmacros, register_stdlib_defmacros, retract_divergent_stdlib_macros, MacroRegistry,
};
use crate::resolve::{
    normalize_stored_function_bodies, normalize_symbol_refs, resolve_references, ResolveError,
};
use crate::runtime::{Environment, EvalBreak, SymbolTable};
use crate::span::Span;
use crate::types::{
    register_stdlib_types, register_stdlib_types_replacing, register_types_with_acronyms, TypeEnv,
};

/// The output of [`build_env`]: all four build-time registries plus
/// the post-resolve user residue that the caller will type-check and
/// freeze.
pub(crate) struct EnvBundle {
    pub types: TypeEnv,
    pub macros: MacroRegistry,
    pub symbols: SymbolTable,
    /// Post-register, post-resolve user forms. Empty when called from
    /// the stdlib-only test path (`user_forms = vec![]`).
    pub residue: Vec<WatAST>,
    /// Arc 278 #88 — the canonical (`<T,…>`-stripped) names of every top-level
    /// `(:wat::rete::core::defn …)` declared anywhere in the load-resolved `user_forms`
    /// (`extract_rete_defn_names`, collected below, pre-macro-expansion — see that fn's
    /// doc). Carried out rather than consumed locally: the definition-site check moved to
    /// `register_runtime_defs` (STOP-3, one door — see `FrozenWorld::freeze`), which needs
    /// this exact set at BOTH its callers (the boot path and the live-session path).
    ///
    /// ⛔ `BTreeSet`, NOT `HashSet` — C20 (arc 278); see `FrozenWorld::declared_rete_defns`
    /// and `rete::purity::apply_rete_defn_contracts` for why the order is part of the type.
    pub declared_rete_defns: std::collections::BTreeSet<String>,
    /// Arc 278 — a resolve failure DEFERRED so `check_program` (step 8) runs first.
    ///
    /// A malformed definition does not register, so every CALL to it becomes an
    /// `UnresolvedReference` pointing at the CALL SITE — while the located `MalformedForm`
    /// naming the real cause lives in `check_program`, which `?` on resolve prevented from
    /// ever running. Measured 2026-08-13: the SAME file reports the cause when the caller is
    /// deleted and the symptom when it is present. Carrying the error lets step 8 run and the
    /// cause win; the symptom is re-raised only when check finds nothing.
    pub deferred_resolve: Option<ResolveError>,
}

/// Stdlib registries after `build_env(vec![])`. Built at most once per process.
/// The initializer is `build_env` itself — it must NOT run inside a `OnceLock`
/// that stdlib expansion can re-enter (`runtime.rs:10294`). This lock's
/// closure expands stdlib; stdlib must not call [`stdlib_snapshot`] or the
/// `declared-types` verb (proven by
/// `stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb`).
pub(crate) fn stdlib_snapshot() -> &'static (SymbolTable, MacroRegistry, TypeEnv) {
    static LOADED: OnceLock<(SymbolTable, MacroRegistry, TypeEnv)> = OnceLock::new();
    LOADED.get_or_init(|| {
        let b = build_env(vec![]).expect("stdlib env builds");
        (b.symbols, b.macros, b.types)
    })
}

/// A declaration that could not register. The form is the original AST (or the
/// first form of the program when the failing span cannot be recovered); the
/// cause is the underlying error's Display. Never a silent drop.
#[derive(Debug)]
pub(crate) struct DeclaredTypesFail {
    pub form: WatAST,
    pub cause: String,
}

/// Expand `forms`' declarations against a COPY of the stdlib registries, register
/// their types, and return the resulting TypeEnv. Stops before `register_defines`
/// and before `check_program` — a stale body does not prevent registration.
///
/// After `register_defmacros` and `preregister_acronyms`, each top-level form is
/// walked one step: a [`crate::types::classify_type_decl`] hit is kept; a do/let
/// walks BODY children only ([`crate::macros::expand::container_body_start`]); a
/// registered macro is [`expand_once`]'d and the result walked; anything else is
/// dropped and never expanded. Then `expand_all` → `register_types_with_acronyms`
/// → `register_variant_types` on the kept forms.
pub(crate) fn register_declared_types(
    forms: Vec<WatAST>,
    stdlib_sym: &SymbolTable,
    stdlib_macros: &MacroRegistry,
    stdlib_types: &TypeEnv,
) -> Result<TypeEnv, Box<DeclaredTypesFail>> {
    let fallback = forms
        .first()
        .cloned()
        .unwrap_or_else(|| WatAST::NilLit(crate::rust_caller_span!()));
    let fail_at = |span: &Span, cause: String| {
        Box::new(DeclaredTypesFail {
            form: top_level_form_for_span(&forms, span).unwrap_or_else(|| fallback.clone()),
            cause,
        })
    };
    let mut macros = stdlib_macros.clone();
    let rest = register_defmacros(forms.clone(), &mut macros)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let mut macro_sym = stdlib_sym.clone();
    preregister_acronyms(&rest, &mut macro_sym).map_err(|e| match e {
        EvalBreak::Diagnostic(re) => fail_at(re.span(), format!("{re}")),
        EvalBreak::Signal(_) => fail_at(fallback.span(), "eval-loop control signal escaped".into()),
    })?;
    let env = Environment::default();
    let kept = collect_type_forms(rest, &macros, &env, &macro_sym)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let expanded = expand_all(kept, &mut macros, &env, &macro_sym)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let mut types = stdlib_types.clone();
    register_types_with_acronyms(expanded, &mut types, &macro_sym.acronym_registry)
        .map_err(|e| fail_at(e.span(), format!("{e}")))?;
    types
        .register_variant_types()
        .map_err(|e| fail_at(e.span(), format!("{e}")))?;
    Ok(types)
}

/// 2a4 / 2a4c — `build_env`'s STDLIB half on `forms`, against a FRESH copy of
/// the snapshot. A divergent snapshot MACRO is retracted in this copy
/// (`retract_divergent_stdlib_macros`) then `register_stdlib_defmacros`. The
/// one-step walk (`collect_type_forms`) keeps only what can declare a type, so
/// a `defn` body is never expanded; kept forms expand under
/// `Privilege::Stdlib` so a companion minted mid-walk still registers. A
/// divergent snapshot type is replaced in this copy only. Returns the env, the
/// copy's macros (call 2's snapshot), and the names this file declared.
pub(crate) fn register_declared_stdlib_types(
    forms: Vec<WatAST>,
    stdlib_sym: &SymbolTable,
    stdlib_macros: &MacroRegistry,
    stdlib_types: &TypeEnv,
) -> Result<(TypeEnv, MacroRegistry, Vec<String>), Box<DeclaredTypesFail>> {
    let fallback = forms
        .first()
        .cloned()
        .unwrap_or_else(|| WatAST::NilLit(crate::rust_caller_span!()));
    let fail_at = |span: &Span, cause: String| {
        Box::new(DeclaredTypesFail {
            form: top_level_form_for_span(&forms, span).unwrap_or_else(|| fallback.clone()),
            cause,
        })
    };
    let mut macros = stdlib_macros.clone();
    retract_divergent_stdlib_macros(&forms, &mut macros)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let rest = register_stdlib_defmacros(forms.clone(), &mut macros)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let mut macro_sym = stdlib_sym.clone();
    preregister_acronyms(&rest, &mut macro_sym).map_err(|e| match e {
        EvalBreak::Diagnostic(re) => fail_at(re.span(), format!("{re}")),
        EvalBreak::Signal(_) => fail_at(fallback.span(), "eval-loop control signal escaped".into()),
    })?;
    let env = Environment::default();
    let kept = collect_type_forms(rest, &macros, &env, &macro_sym)
        .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let expanded = expand_all_with(
        kept,
        &mut macros,
        &env,
        &macro_sym,
        crate::resolve::Privilege::Stdlib,
    )
    .map_err(|e| fail_at(&e.span, format!("{e}")))?;
    let mut types = stdlib_types.clone();
    let (_rest, declared) = register_stdlib_types_replacing(expanded, &mut types)
        .map_err(|e| fail_at(e.span(), format!("{e}")))?;
    types
        .register_variant_types()
        .map_err(|e| fail_at(e.span(), format!("{e}")))?;
    Ok((types, macros, declared))
}

fn collect_type_forms(
    forms: Vec<WatAST>,
    macros: &MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Vec<WatAST>, crate::macros::MacroError> {
    let mut kept = Vec::new();
    for form in forms {
        walk_type_forms(form, macros, env, sym, &mut kept)?;
    }
    Ok(kept)
}

fn walk_type_forms(
    form: WatAST,
    macros: &MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
    kept: &mut Vec<WatAST>,
) -> Result<(), crate::macros::MacroError> {
    if crate::types::classify_type_decl(&form).is_some() {
        kept.push(form);
        return Ok(());
    }
    if let WatAST::List(items, _) = &form {
        if let Some(head) = items.first().and_then(crate::declare::parse::head_fqdn) {
            if let Some(body_start) = crate::macros::expand::container_body_start(head.as_ref()) {
                for child in items.iter().skip(body_start).cloned() {
                    walk_type_forms(child, macros, env, sym, kept)?;
                }
                return Ok(());
            }
        }
    }
    let expanded = expand_once(form.clone(), macros, env, sym)?;
    if expanded == form {
        return Ok(());
    }
    walk_type_forms(expanded, macros, env, sym, kept)
}

fn pos_le(line1: i64, col1: i64, line2: i64, col2: i64) -> bool {
    line1 < line2 || (line1 == line2 && col1 <= col2)
}

fn span_covers(outer: &Span, inner: &Span) -> bool {
    if *outer.file != *inner.file {
        return false;
    }
    if !pos_le(outer.line, outer.col, inner.line, inner.col) {
        return false;
    }
    match &outer.end {
        Some(oe) => {
            let (il, ic) = match &inner.end {
                Some(ie) => (ie.line, ie.col),
                None => (inner.line, inner.col),
            };
            pos_le(il, ic, oe.line, oe.col)
        }
        None => outer.line == inner.line && outer.col == inner.col,
    }
}

fn top_level_form_for_span(forms: &[WatAST], span: &Span) -> Option<WatAST> {
    forms.iter().find(|f| span_covers(f.span(), span)).cloned()
}

/// Names of types the source declares, so expand-time `reconstruct_call_path`
/// can join a member before `register_types` runs. Membership only — the
/// real `TypeDef` is still built by the registration pass.
fn seed_declared_type_names(forms: &[WatAST], env: &mut crate::types::TypeEnv) {
    fn one(form: &WatAST, env: &mut crate::types::TypeEnv) {
        match form {
            WatAST::List(items, _) => {
                if let Some(name) = declaration_type_name(items) {
                    env.register_use_declared_leaf(name);
                }
                for child in items {
                    one(child, env);
                }
            }
            WatAST::Vector(items, _) | WatAST::Set(items, _) => {
                for child in items {
                    one(child, env);
                }
            }
            WatAST::Map(pairs, _) => {
                for (k, v) in pairs {
                    one(k, env);
                    one(v, env);
                }
            }
            _ => {}
        }
    }
    for form in forms {
        one(form, env);
    }
}

fn rekey_type_member_functions(sym: &mut crate::runtime::SymbolTable) {
    let Some(types) = sym.types().cloned() else {
        return;
    };
    let names: Vec<String> = sym.functions_iter().map(|(n, _)| n.clone()).collect();
    for name in names {
        // rune:lint(one-variant-separator, namespace) — last `::` of a stored function name
        let Some((parent, method)) = name.rsplit_once("::") else {
            continue;
        };
        // `Enum.Variant` is a variant path (`decompose_variant`), not a method.
        // Rekeying it onto `/` makes the map-ctor look up a type that was
        // registered with `::`.
        if method.is_empty()
            || method.contains('/')
            || wat_reader::identifier::prime(method)
            || wat_reader::identifier::decompose_variant(&name).is_some()
            || !types.is_known_type(parent)
        {
            continue;
        }
        let member = format!("{parent}/{method}");
        if member == name {
            continue;
        }
        if let Some(func) = sym.remove_function(&name) {
            sym.register_function(member, func);
        }
    }
}

fn declaration_type_name(items: &[WatAST]) -> Option<String> {
    let head = crate::declare::parse::head_fqdn(items.first()?)?;
    match head.as_ref() {
        ":wat::core::defrecord"
        | ":wat::core::defstruct"
        | ":wat::core::defenum"
        | ":wat::core::defsurface"
        | ":wat::core::typealias"
        | ":wat::holon::defrecord" => {}
        _ => return None,
    }
    match items.get(1)? {
        WatAST::Keyword(k, _) => Some(crate::edn::render::canonical_identity(k)),
        WatAST::Symbol(id, _) if id.is_reference() => Some(crate::edn::render::ns_to_wat_path(
            id.receiver(),
            id.method(),
        )),
        _ => None,
    }
}

/// Build the full registered environment from already-parsed,
/// already-load-resolved user forms.
///
/// `user_forms = vec![]` yields the stdlib-only environment (the
/// test path). All user-side steps (`expand_all(user)`,
/// bare-legacy walker, `register_types(user)`, `register_defines(user)`,
/// `preregister_acronyms(runtime)`,
/// `normalize_symbol_refs`, `resolve_references`) are natural no-ops on
/// an empty slice, so the same function drives both paths.
///
/// Steps covered (mirrors `startup_from_forms_post_config` 3a–7.6):
///
/// - 3a. `stdlib_forms()` — bake the stdlib
/// - 4.  `register_stdlib_defmacros` + `register_defmacros(user)` +
///       `preregister_acronyms(macro_sym)` + `expand_all(stdlib)` +
///       `expand_all(user)`
/// - 4b. Bare-legacy walker + arc-170 legacy callsite walker
/// - 5.  `TypeEnv::with_builtins` + `register_stdlib_types` + `register_types(user)`
/// - 6.  `register_stdlib_defines` → `preregister_stdlib_defclause_stub` loop
///       → extract `stdlib_runtime_def_forms` → `register_defines(user)`
/// - 6a/6.5/6.7/6.8a/6.9.  Auto-method registration (struct/enum/newtype/record/predicate)
/// - 6.8. Inventory restriction-entry drain into `binding_metadata`
/// - 6.96. `preregister_acronyms(runtime)`
/// - 7.  `normalize_symbol_refs` + `resolve_references`
/// - 7.6. `register_stdlib_runtime_defs`
///
/// NOT included (caller responsibility):
///
/// - Step 3: `resolve_loads` (caller passes already-loaded forms)
/// - Step 7.5: config-flag propagation (`redef_allowed`, `eval_redef_allowed`)
/// - Step 8: `check_program`
/// - Step 9: `FrozenWorld::freeze`
pub(crate) fn build_env(user_forms: Vec<WatAST>) -> Result<EnvBundle, super::StartupError> {
    use super::StartupError;

    // 3a. Baked stdlib. Registered ahead of user code so any
    //     `(:wat::holon::Subtract …)` / `(:wat::holon::Amplify …)` call
    //     in user source resolves during step 4's macro expansion
    //     without an explicit `load!`.
    let stdlib = stdlib_forms()?;

    // 3b. Arc 278 #88 — pull the `(:wat::rete::core::defn …)` declarations out of the RAW,
    //     pre-macro-expansion user forms, and rewrite each head to plain `:wat::core::defn` so
    //     it flows through the EXACT SAME macro-expansion → registration → type-checking
    //     pipeline an ordinary defn does ("same parse, same registration, same symbol binding
    //     as defn" — the design stone's own framing; reusing that path rather than a parallel
    //     one). `declared_rete_defns` names WHICH registrations to check + stamp; v2 moved
    //     that check out of `build_env` entirely (see the note beside step 6.97, below) —
    //     this fn now only DERIVES the name set and carries it out on `EnvBundle` for the
    //     caller to thread to `register_runtime_defs`, the check's new (and only) home.
    crate::freeze::pass_order::record("3b-extract-rete-defn-names");
    let declared_rete_defns = extract_rete_defn_names(&user_forms);
    let user_forms = rewrite_rete_defn_heads(user_forms);

    // 4. Macro registration + expansion. Stdlib defmacros register
    //    first; user defmacros layer on top and can shadow (subject
    //    to the reserved-prefix gate) or reference stdlib forms.
    let mut macros = MacroRegistry::new();
    crate::freeze::pass_order::record("4-register-stdlib-defmacros");
    let stdlib_post_macros = register_stdlib_defmacros(stdlib, &mut macros)?;
    crate::freeze::pass_order::record("4-register-defmacros");
    let post_macro_reg = register_defmacros(user_forms, &mut macros)?;

    // Arc 294 item 9a — class closure: an aggregate registered directly in Rust
    // (`TypeEnv::with_builtins()` — `register_builtin_types` + the `inventory`
    // `EdnSchema` drain) never flows through a wat `defstruct`/`defrecord`
    // invocation, so it never gets a kwargs companion macro minted the way a
    // wat-declared aggregate does. Mint one here, structurally, for every such
    // aggregate that doesn't already have one (skip-if-present — a wat-emitted
    // companion, were one somehow already registered under the same bare name,
    // is never clobbered). MUST run before `expand_all` below: that's the pass
    // that actually resolves `(:T :field v ...)` call sites into the companion's
    // `kwargs-lower` forward. `TypeEnv::with_builtins()` is self-contained (no
    // stdlib/user forms needed) so it's safe to construct this early, ahead of
    // step 5's real `types` build.
    register_aggregate_kwargs_companions(&crate::types::TypeEnv::with_builtins(), &mut macros)?;

    // ORDER LOAD-BEARING: macro_eval purity (src/macros/eval.rs) depends on
    // expand_all preceding register_defines. See freeze.rs header comment.
    //
    // Arc 265 — pre-register declare-acronyms forms into macro_sym BEFORE
    // expand_all so defservice's pascal->kebab-in call at expand time can
    // consult the registry.
    let mut macro_sym = SymbolTable::default();
    // Macro bodies eval during expand, before register_types. A converted
    // `(wat.core.Option/expect …)` must still join as a member. Builtins are
    // the registry that exists at this point. Stdlib types are not registered
    // yet, but their NAMES are already in the source — seed membership so
    // `reconstruct_call_path` can see `wat.spawn.Locus/launch` as a member
    // while `defservice` is evaluating. This env is expand-only; the real
    // `TypeEnv` is built later and replaces nothing here.
    let mut expand_types = crate::types::TypeEnv::with_builtins();
    seed_declared_type_names(&stdlib_post_macros, &mut expand_types);
    seed_declared_type_names(&post_macro_reg, &mut expand_types);
    macro_sym.types_insert(std::sync::Arc::new(expand_types));
    preregister_acronyms(&post_macro_reg, &mut macro_sym).map_err(|e| match e {
        EvalBreak::Diagnostic(re) => StartupError::Runtime(re),
        EvalBreak::Signal(_) => {
            unreachable!("interpreter bug: eval-loop control signal escaped to freeze layer")
        }
    })?;
    // Expansion-born stdlib defmacros (e.g. a `defservice`'s `…/start` companion,
    // emitted by a macro-generating-macro) register through the ONE gate with an
    // EXPLICIT `Privilege::Stdlib` (threaded, no ambient flag) — the stdlib bypass. The
    // user pass below uses plain `expand_all` (Privilege::User), so a mis-namespaced
    // user macro still halts.
    let expanded_stdlib = crate::macros::expand::expand_all_with(
        stdlib_post_macros,
        &mut macros,
        &Environment::default(),
        &macro_sym,
        crate::resolve::Privilege::Stdlib,
    )?;
    crate::freeze::pass_order::record("4-expand-all");
    let expanded_user = expand_all(
        post_macro_reg,
        &mut macros,
        &Environment::default(),
        &macro_sym,
    )?;

    // 4b. Arc 163 slice 3g phase A — bare-legacy walker on raw
    //     post-expansion forms BEFORE register_types/register_defines.
    //     Walks user forms only; stdlib is substrate-authored.
    {
        let mut bare_errors: Vec<CheckError> = Vec::new();
        for form in &expanded_user {
            validate_bare_legacy_primitives(form, &mut bare_errors);
        }
        // Arc 170 slice 2 — substrate-as-teacher walker.
        for form in &expanded_user {
            validate_arc170_legacy_callsites(form, &mut bare_errors);
        }
        if !bare_errors.is_empty() {
            return Err(StartupError::Check(CheckErrors(bare_errors)));
        }
    }

    // 5. Type declarations. Seeded with built-in types before stdlib
    //    and user source land.
    let mut types = TypeEnv::with_builtins();
    crate::freeze::pass_order::record("5-register-stdlib-types");
    let stdlib_post_types = register_stdlib_types(expanded_stdlib, &mut types)?;
    // Thread the namespace-scoped acronym registry (populated by `preregister_acronyms`
    // above, BEFORE macro expansion) into type registration so a `:satisfies` surface's
    // S1 protocol synthesis restores acronym casing on its `::Op`/`::Reply` variants
    // identically to how `defservice :impls` does at expand time.
    crate::freeze::pass_order::record("5-register-types");
    let post_types =
        register_types_with_acronyms(expanded_user, &mut types, &macro_sym.acronym_registry)?;
    // Arc 293.W — containment rule: after BOTH stdlib and user types are fully
    // registered, verify that no portable aggregate (record/holon) declares a
    // non-portable (struct) field. Forward references are now resolved, so the
    // check is complete and sound. TypeError converts to StartupError::Type via
    // the From impl in freeze.rs.
    validate_aggregate_containment(&types)?;
    // Arc 296 A-2 RELAND-1 — every enum variant becomes its own type (`:Enum::Variant`, a
    // singleton `TypeDef::Enum` sharing the parent's type params) plus a head-level
    // subtype edge `Variant <: Enum`, BEFORE the P-3 annotation wall
    // (`validate_named_type_annotations`, below) and ctor-method synthesis
    // (`register_enum_methods`) see them. Stdlib enums are NOT excluded (RELAND-0's
    // `!is_reserved_prefix` scope cut relocated its failures into user code instead of
    // avoiding them — see `TypeEnv::register_variant_types`'s doc comment); `join_types`
    // (`src/check.rs`) is what makes registering every enum, stdlib included, safe.
    types.register_variant_types()?;

    // 6. Function definitions.
    let mut symbols = SymbolTable::new();
    // Arc 296 P-3 — stdlib `use!` ONLY. Kept unmerged so the annotation wall
    // can ask each declaring scope about its own declarations (a stdlib
    // annotation by stdlib `use!`; a user annotation by user `use!`).
    let mut stdlib_use = crate::rust_deps::UseDeclarations::new();
    {
        let registry = crate::rust_deps::registry();
        let mut unused = Vec::new();
        for form in &stdlib_post_types {
            crate::resolve::collect_use_declarations(form, registry, &mut stdlib_use, &mut unused);
        }
    }
    // Stone 237.8b — capture stdlib residue so defclause forms reach
    // register_runtime_defs.
    crate::freeze::pass_order::record("6-register-stdlib-defines");
    let stdlib_residue = register_stdlib_defines(stdlib_post_types, &mut symbols)?;
    // (a) Pre-register defclause stubs into sym.functions so the checker
    //     sees them as callable names (e.g. :wat::kernel::spawn-program).
    for form in &stdlib_residue {
        preregister_stdlib_defclause_stub(form, &mut symbols);
    }
    // (b) Extract stdlib forms that need RUNTIME registration via
    //     runtime_defs: defclause, extend-type, def.
    //     Arc 209 host-parity-4a broadened from defclause-only.
    let stdlib_runtime_def_forms: Vec<WatAST> = stdlib_residue
        .into_iter()
        .filter(|form| {
            let WatAST::List(items, _) = form else {
                return false;
            };
            // Symbol `(wat.core/defclause …)` is the same form as the keyword.
            // A keyword-only filter left the 0-param stub in place.
            matches!(
                items.first().and_then(crate::declare::parse::head_fqdn).as_deref(),
                Some(
                    ":wat::core::defclause"
                        | ":wat::core::extend-type"
                        // Arc 255 escape-hatch — scalar stdlib `def` forms
                        // (e.g. MAX-READLN-BYTES) must reach runtime_def_values.
                        | ":wat::core::def"
                )
            )
        })
        .collect();
    crate::freeze::pass_order::record("6-register-defines");
    let mut residue = register_defines(post_types, &mut symbols)?;
    // User-source `use!` only. Seeded into TypeEnv so `is-type?` agrees
    // with resolve for names THIS program declared (P-2 prereq). Not
    // merged into `stdlib_use` — that merge was the wall's scope-blindness.
    let mut user_use = crate::rust_deps::UseDeclarations::new();
    {
        let registry = crate::rust_deps::registry();
        let mut unused = Vec::new();
        for form in &residue {
            crate::resolve::collect_use_declarations(form, registry, &mut user_use, &mut unused);
        }
    }
    for path in user_use.list() {
        types.register_use_declared_leaf(path);
    }
    validate_named_type_annotations(&types, &symbols, &stdlib_use, &user_use)?;

    // 6a. Struct auto-methods (ctor only; accessors now in 6.8a).
    register_struct_methods(&types, &mut symbols)?;
    // 6.5. Enum variant constructors.
    register_enum_methods(&types, &mut symbols)?;
    // 6.7. Newtype auto-methods.
    register_newtype_methods(&types, &mut symbols)?;
    // 6.8a. Arc 293.R2.2 — ONE unified accessor codegen for all Aggregate natures
    // (Struct + Record + HolonRecord). Replaces the deleted register_record_methods
    // + the accessor loop that was in register_struct_methods.
    register_aggregate_methods(&types, &mut symbols)?;
    // 6.9. Type membership predicates.
    register_type_predicates(&types, &mut symbols)?;

    // 6.8. Arc 198 slice 2 Stone 1 — drain the `inventory` registry of
    //      Rust-side `RestrictionEntry` declarations into `binding_metadata`.
    // rune:sequi(ambient-context) — inventory::iter is link-time static state.
    for entry in inventory::iter::<crate::restriction_entry::RestrictionEntry> {
        let name = entry.wat_name.to_string();
        let mut prefix_items = vec![WatAST::Keyword(
            ":wat::core::Vector".into(),
            crate::rust_caller_span!(),
        )];
        for p in entry.prefixes {
            prefix_items.push(WatAST::Keyword(p.to_string(), crate::rust_caller_span!()));
        }
        let restricted_to_ast = WatAST::List(prefix_items, crate::rust_caller_span!());
        let mut meta: HashMap<String, WatAST> = HashMap::new();
        meta.insert(":restricted-to".to_string(), restricted_to_ast);
        symbols
            .binding_metadata
            .entry(name)
            .or_default()
            .extend(meta);
    }

    // 6.96. Arc 265 — pre-register declare-acronyms forms into the
    //       runtime SymbolTable (macro_sym covered expand-time; this
    //       covers eval-time).
    preregister_acronyms(&residue, &mut symbols).map_err(|e| match e {
        EvalBreak::Diagnostic(re) => StartupError::Runtime(re),
        EvalBreak::Signal(_) => {
            unreachable!("interpreter bug: eval-loop control signal escaped to freeze layer")
        }
    })?;

    // 6.97. Arc 293.4b — pre-attach the TypeEnv to the SymbolTable BEFORE the
    //       resolve pass so `is_resolvable_call_head` can distinguish a
    //       `:S/method` surface-method call head from an UnresolvedReference.
    //       At this point `types` is fully populated (all steps 5–6.9x done).
    //       `FrozenWorld::freeze` later overwrites `sym.types` with the same
    //       data (via `symbols.set_types(Arc::new(types.clone()))`); the early
    //       attach here is strictly for the resolve pass.
    symbols.types_insert(std::sync::Arc::new(types.clone()));
    // A defn name `wat.cache.Lru/new` is stored by `ns_to_wat_path` as
    // `:wat::cache::Lru::new`. The call site asks `reconstruct_call_path`,
    // and Lru is a type, so the call is `:wat::cache::Lru/new`. Rekey the
    // function to the member join once the type env exists.
    rekey_type_member_functions(&mut symbols);

    // Arc 278 #88 v2 — THE DEFINITION-SITE CHECK for every `(:wat::rete::core::defn …)`
    // collected at step 3b used to run HERE (step 6.975), stamping `Function::rete` on
    // `symbols` before `build_env` returns. But `FrozenWorld::freeze` (and the live-session
    // path, `eval_form_against_defs` in runtime.rs) both call `register_runtime_defs`
    // AFTER this point, and that pass RE-REGISTERS every `defn`-turned-`def`, rebuilding a
    // fresh `Function` (`rete: None`) and dropping the stamp — so the file loaded (the
    // check ran, correctly) while the runtime fence still refused every helper, because
    // the `Function` it read back was unstamped (DESIGN-STONE-the-rete-defn.md, "WHAT THE
    // FIRST STRIKE LEARNED" §3). The check now runs INSIDE `register_runtime_defs` itself —
    // the one door both the boot path (`freeze.rs`, `FrozenWorld::freeze`) and the
    // live-session path (`runtime.rs`, `eval_form_against_defs`) already call — so
    // `declared_rete_defns` is carried OUT on `EnvBundle` instead of being consumed here.

    // 7. Name resolution.
    // Stone 251.1b — normalize before resolve so rewritten AST flows
    // through check + eval with keyword heads.
    crate::freeze::pass_order::record("7-normalize-symbol-refs");
    residue = normalize_symbol_refs(residue, &symbols, &macros)?;
    crate::freeze::pass_order::record("7-normalize-stored-function-bodies");
    // Stone 251.8c — check_program type-checks FunctionBody snapshots from
    // register_defines, not this residue. Normalize those copies too so a
    // namespaced Symbol call head is a Keyword before infer_list.
    normalize_stored_function_bodies(&mut symbols, &macros)?;
    // DEFERRED, not swallowed: an unresolved reference is very often the SYMPTOM of a
    // malformed definition that failed to register. Running `check_program` first lets the
    // located cause be reported; if check is clean, this error is re-raised unchanged.
    crate::freeze::pass_order::record("7-resolve-references");
    let deferred_resolve = resolve_references(&residue, &symbols, &macros).err();

    // 7.6. Stone 237.8b (+ arc 209 host-parity-4a) — register stdlib
    //      defclause / extend-type / def forms into
    //      runtime_def_values.
    register_stdlib_runtime_defs(&stdlib_runtime_def_forms, &mut symbols)
        .map_err(|e| StartupError::Runtime(Box::new(e)))?;

    // 7.7. Arc 278 BRIEF-STONE-extend-user-checked — pre-register USER extend-type
    //      SURFACE impls into sym.functions with their REAL inherited sig (mirrors
    //      the stdlib step 7.6 call above via the SAME shared routine), BEFORE
    //      check_program's body-check sweep (check.rs:826) runs. Without this, a
    //      user satisfier's impl body is never type-checked against the surface it
    //      claims to satisfy (it only lands in sym.functions at freeze step 9,
    //      AFTER check_program already ran). Protocol-target extend-type forms are
    //      a no-op here (handled unchanged at freeze step 9,
    //      `register_runtime_defs_form`). This is the FIRST registration for these
    //      forms, so a colliding key is a genuine DuplicateDefine
    //      (skip_if_present=false).
    //
    //      W1a — a macro (e.g. `defservice`) can emit `(do … (extend-type …) …)`
    //      rather than a bare top-level extend-type, so this must recurse into
    //      `do`/`let` wrappers the same way freeze step 9's
    //      `register_runtime_defs_form` (runtime.rs) and `splice_type_decls`
    //      (types.rs) already do for the sibling type-decl / runtime-def passes —
    //      otherwise a macro-emitted extend-type's satisfaction scheme never
    //      reaches `sym.functions` before `check_program` runs. Step 9 continues
    //      to re-walk the SAME residue (skip_if_present=true, see runtime.rs:1935),
    //      so pre-registering do/let-nested forms here is idempotent with step 9
    //      exactly like the existing top-level case.
    for form in &residue {
        preregister_extend_type_in_do_let(form, &mut symbols)
            .map_err(|e| StartupError::Runtime(Box::new(e)))?;
    }
    crate::freeze::pass_order::record("7.7-normalize-stored-function-bodies");
    // extend-type / defclause bodies are stored from the pre-normalize
    // capture. The pass above only saw functions registered before it.
    normalize_stored_function_bodies(&mut symbols, &macros)?;

    // 7.8 — Arc 294 item 9a (DESIGN-rete-defrule-wall.md) lifted into a pluggable
    // `FreezeValidator` extension point (mirrors step 6.8's `RestrictionEntry` drain, same
    // fn): drain every `inventory`-registered freeze-time validator against the SAME
    // post-register (types are authoritative), post-resolve (quoted :when/:then survive
    // `resolve` un-mangled — proven by `rete_wall_probe` below) `residue` + `types` +
    // `symbols`. The `defrule` wall (`crate::rete::validate::validate_rete_rules`) is the
    // FIRST registered consumer (see its `inventory::submit!` in `src/rete/validate.rs`) —
    // it still walks every `defrule`'s expanded `make-rule` call, validates `:when`/`:then`
    // against `types`, and REWRITES `:then` kwargs to declaration order in place. A
    // malformed rule is a LOCATED `#wat.rete/*` freeze error (dynamic dispatch through the
    // box preserves the concrete namespace) instead of a silent fire-time `None` / scrambled
    // fact (the 9a codemod's corruption class). Any OTHER crate depending on `wat` can
    // register its own validator the same way — zero special-casing for the rete wall here.
    // rune:sequi(ambient-context) — inventory::iter is link-time static state.
    for v in inventory::iter::<crate::freeze::validator::FreezeValidator> {
        (v.validate)(&mut residue, &types, &symbols).map_err(StartupError::Validator)?;
    }

    Ok(EnvBundle {
        types,
        macros,
        symbols,
        residue,
        declared_rete_defns,
        deferred_resolve,
    })
}

/// Arc 278 #88 — step 3b's SCAN half: collect the name of every TOP-LEVEL
/// `(:wat::rete::core::defn :name [args] -> :Ret body…)` declaration in `forms`, before
/// macro expansion touches anything. Top-level only — every corpus site the design stone
/// measured is a bare top-level declaration (mirrors the fixture and every `where`-callee in
/// the corpus); a form nested inside a macro-emitted `do`/`let` is out of this slice's scope.
///
/// ⛔ RETURNS A `BTreeSet`, NOT A `HashSet` — C20 (arc 278). This is the ONLY construction site
/// for the value, and the consumer's loop picks the first cycle it finds, so the collection type
/// here decides which function a user is told to go look at. See
/// `rete::purity::apply_rete_defn_contracts`.
fn extract_rete_defn_names(forms: &[WatAST]) -> std::collections::BTreeSet<String> {
    let mut declared = std::collections::BTreeSet::new();
    for form in forms {
        let WatAST::List(items, _) = form else {
            continue;
        };
        // 255.10 — head and slot-1 are both NAMES, read through the identity door so
        // `wat.rete.core/defn` is the same declaration as `:wat::rete::core::defn`.
        let Some(k) = items.first().and_then(crate::form_match::canonical_identity_of) else {
            continue;
        };
        if k != ":wat::rete::core::defn" {
            continue;
        }
        if let Some(name_kw) = items.get(1).and_then(crate::form_match::canonical_identity_of) {
            // STONE reap-the-angle-machinery (arc 109) — `name_kw` used to be run through
            // `split_name_and_type_params` to strip a `<T,...>` suffix. Angle syntax is
            // unexpressible now, so the name can never carry one; insert it directly.
            declared.insert(name_kw);
        }
    }
    declared
}

/// Arc 278 #88 — step 3b's REWRITE half: every top-level `(:wat::rete::core::defn …)` head
/// becomes `:wat::core::defn`, so `expand_all` / `register_defines` / `check_program` see an
/// ordinary defn — the SAME parse, the SAME registration, the SAME symbol binding
/// (DESIGN-STONE-the-rete-defn.md). `extract_rete_defn_names` (above) ran first, on the
/// UNREWRITTEN forms, so the rewrite here loses no information: it only erases the surface
/// distinction the rest of the pipeline doesn't need to see.
fn rewrite_rete_defn_heads(forms: Vec<WatAST>) -> Vec<WatAST> {
    forms
        .into_iter()
        .map(|form| match form {
            WatAST::List(mut items, span) => {
                // 255.10 — the head is a NAME: read both spellings through the identity
                // door. The REWRITE still emits the keyword, which is what the rest of the
                // pipeline (expand_all → register_defines → check) already expects.
                if let Some(k) = items.first().and_then(crate::form_match::canonical_identity_of) {
                    if k == ":wat::rete::core::defn" {
                        let kspan = items[0].span().clone();
                        items[0] = WatAST::Keyword(":wat::core::defn".to_string(), kspan);
                    }
                }
                WatAST::List(items, span)
            }
            other => other,
        })
        .collect()
}

/// W1a — recursive walk for build_env step 7.7: find every `extend-type` form
/// reachable from `form` through `do`/`let` wrappers (a macro-emitted
/// extend-type is never a bare top-level form; it arrives nested inside a
/// `(do …)` splice, and possibly nested `do`s within that), pre-registering
/// each one's surface impls via the shared `register_extend_type_surface_impls`
/// routine (`skip_if_present=false` — this is the FIRST registration for a
/// given form, so a colliding key here is a genuine `DuplicateDefine`, mirroring
/// the top-level case this replaces).
///
/// Mirrors the `do`/`let` recursion shape in `register_runtime_defs_form`
/// (runtime.rs) and `splice_type_decls` (types.rs) — same two keywords, same
/// body-start offset (`do` body is `items[1..]`, `let` body is `items[2..]`
/// to skip the bindings vector).
fn preregister_extend_type_in_do_let(
    form: &WatAST,
    symbols: &mut SymbolTable,
) -> Result<(), crate::runtime::RuntimeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(()),
    };
    let head = match items.first().and_then(crate::declare::parse::head_fqdn) {
        Some(k) => k,
        None => return Ok(()),
    };
    match head.as_ref() {
        ":wat::core::extend-type" => {
            crate::declare::register::register_extend_type_surface_impls(form, symbols, false)
        }
        ":wat::core::do" => {
            for child in &items[1..] {
                preregister_extend_type_in_do_let(child, symbols)?;
            }
            Ok(())
        }
        ":wat::core::let" => {
            for child in items.iter().skip(2) {
                preregister_extend_type_in_do_let(child, symbols)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod rete_wall_probe {
    //! Disconfirming probe for DESIGN-rete-defrule-wall.md — proves the wall's three load-bearing
    //! assumptions BEFORE a shadowdancer builds it:
    //!   (1) build_env's `residue` (post-register, post-resolve) holds the defrule's expanded
    //!       `make-rule` call, reachable as WatAST;
    //!   (2) the quoted :when/:then survive `resolve` UN-MANGLED (head keyword + clause list intact,
    //!       and `resolve` does NOT choke on the free `?loc`/`<-` inside the quote);
    //!   (3) the head fact-type's field ORDER reads from env.types() (the validate + reorder core).
    //! Fail here → STOP; the wall's post-register hook is not where the design assumes.
    //!
    //! Arc 294 item 9a — the wall landed (`crate::rete::validate::validate_rete_rules`, hooked
    //! in `build_env` step 7.8, below). This probe's fixture is now a CORRECT rule: with the
    //! wall live, `build_env` itself raises `StartupError::Validator(..)` (the rete wall registers through the generic freeze-validator hook; there is no `Rete` variant) on the 9a codemod's
    //! injected-keyword corruption this probe originally carried — a corrupt fixture here would
    //! make `build_env` fail, defeating the reachability assertions this probe exists to prove.
    //! The corruption-is-caught proof now lives in `src/rete/validate.rs`'s own test module
    //! (`corrupt_when_clause_is_a_located_error`, same fixture, asserting the located error).
    use super::*;
    use crate::ast::WatAST;

    fn find_make_rule(forms: &[WatAST]) -> Option<&Vec<WatAST>> {
        for f in forms {
            if let WatAST::List(items, _) = f {
                if let Some(WatAST::Keyword(k, _)) = items.first() {
                    if k == ":wat::rete::make-rule" {
                        return Some(items);
                    }
                }
                if let Some(found) = find_make_rule(items) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn quote_vec(form: &WatAST) -> &[WatAST] {
        // form = (:wat::core::quote [<items>...]) → the Vector's items
        if let WatAST::List(items, _) = form {
            if let Some(WatAST::Vector(v, _)) = items.get(1) {
                return v.as_slice();
            }
        }
        &[]
    }

    #[test]
    fn probe_hook_reaches_rule_forms_and_field_order() {
        // A CORRECT defrule — the wall (validate_rete_rules, hooked below) now runs INSIDE
        // build_env, so a corrupt :when here would make build_env itself fail (proven
        // separately by src/rete/validate.rs's own test module).
        let src = r#"
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc :- :location) (?c :- :celsius))]
  :then
  [(:alert::Unattended :location ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let env = build_env(forms).expect("build_env must not choke on the quoted rule interior");

        // (1)+(2): reach the make-rule + its quoted :when, un-mangled by resolve.
        let mr = find_make_rule(&env.residue).expect("make-rule reachable in residue");
        let when = quote_vec(&mr[2]); // child[2] = (:wat::core::quote [conds])
        assert!(
            !when.is_empty(),
            "the :when quote survives resolve as a non-empty vector"
        );
        let cond_items = match &when[0] {
            WatAST::List(i, _) => i,
            other => panic!("cond0 is a List; got {other:?}"),
        };
        let head = match &cond_items[0] {
            WatAST::Keyword(k, _) => k.as_str(),
            other => panic!("cond head is a Keyword; got {other:?}"),
        };
        assert_eq!(
            head, ":weather::Temperature",
            "cond head keyword intact through resolve"
        );

        // (3): the head type's field ORDER reads from the registry — the validate + reorder core.
        // The registry key carries the leading colon (matcher.rs:126: format!(":{}", class_fqdn)).
        let td = env
            .types
            .get(":weather::Temperature")
            .expect("registered record in env.types() (colon-prefixed key)");
        let fields: Vec<&str> = match td {
            crate::types::TypeDef::Aggregate(a) => a.field_names().collect(),
            other => panic!("Temperature is an Aggregate; got {other:?}"),
        };
        assert_eq!(
            fields,
            vec!["celsius", "location"],
            "field names in declaration order"
        );

        // The clause itself is a well-formed bind — `(?loc :- :location)`, a List, not a bare
        // keyword (the shape the 9a corruption injected; see `src/rete/validate.rs`'s
        // `corrupt_when_clause_is_a_located_error` for that case).
        assert!(
            matches!(&cond_items[1], WatAST::List(_, _)),
            "a well-formed bind clause is a List; got {:?}",
            cond_items[1]
        );
    }
}


/// ⭐ ARC 251.8d-ii (EIGHTH DRAW) — **THE FAITHFUL-SURFACE ROUND-TRIP GATE.**
///
/// The 8d campaign's premise is ONE canonical spelling: a `.wat` file rewritten by
/// `wat-scripts/fixes/to-faithful-clojure.wat` must declare the SAME names it declared
/// before. For a DECLARATION NAME that is a round trip through machinery that already
/// exists, in both directions, and is the declared grammar:
///
/// - forward — [`crate::edn::render::wat_keyword_to_clojure_symbol`]: strip the `:`,
///   split on `::`, fold a `Type/method` leaf's `Type` INTO the namespace, join with `.`.
///   This is what the codemod writes.
/// - back — the name slot of `declare::parse` / `macros::parse`, which is
///   [`crate::edn::render::ns_to_wat_path`] and writes `::` ALWAYS; then, for a FUNCTION
///   only, [`rekey_type_member_functions`] above, which restores the `/` member join iff
///   the parent segment names a TYPE (255.4: a member join is `/`, always).
///
/// ⛔ **The two are not inverses.** A faithful symbol carries exactly ONE `/` and it is
/// the namespace/name split, so `wat.spawn.process/post-spawn` is the image of BOTH
/// `:wat::spawn::process/post-spawn` and `:wat::spawn::process::post-spawn`. The rekey
/// pass is the only thing that can tell them apart and it can only answer when the parent
/// is a type. **When the parent is NOT a type, the `/` join is UNSPELLABLE in the faithful
/// surface**: the declaration registers under `::`, every keyword-spelled caller still
/// asks for `/`, and the name is simply gone. That is the eighth draw's address — measured
/// by converting `wat/spawn.wat` alone, which turns the ONE deliberate unresolved
/// reference in `probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat` into TWO.
///
/// ⭐ A MACRO is on the list for a DIFFERENT reason, and `:wat::core::Fault/of` is the
/// only one: macros are registered and expanded at steps 4–5, before the `TypeEnv` exists
/// at step 6.97, so `rekey_type_member_functions` cannot reach them at all — not even when
/// the parent IS a type, as `Fault` is. Its rescue is at the CONSULT instead: the keyword
/// arm of `macros::expand`'s macro-call dispatch asks `other_join_spelling` when the
/// primary key misses, the same second question its Symbol arm already asked. ⛔ This gate
/// deliberately does NOT model that rescue — a gate that models a cure passes whether or
/// not the cure is there. The rescue is proven behaviourally, on two binaries, by
/// `tests/macros/probe_arc251_8d_macro_member_join.rs`.
///
/// ⭐ DERIVED, not a hand-list: every top-level declaration name in every baked stdlib
/// file is asked the same question. ⛔ The frozen list is the ANSWER, re-frozen by hand,
/// so the class can never grow silently.
///
/// ⛔⛔ **THIS GATE GOES RED ON A 8d-CONVERTED STDLIB, BY DESIGN AND BY MEASUREMENT.** Once
/// the corpus is faithful-spelled there is no keyword surface left to round-TRIP from:
/// every declaration name arrives as a `Symbol`, `ns_to_wat_path` writes `::`, and the
/// `/`-joined population this gate discriminates on collapses to ONE. Measured — the
/// eighth draw's converted floor fires the `slash_joined` floor with
/// `only 1 stdlib declaration names carry a `/` member join`, which is a SECOND,
/// whole-corpus confirmation that the conversion destroys the join. The floor is not a
/// bug to tune away: a conditional skip here could not tell "clean" from "never ran"
/// (`[[feedback_a_conditional_probe_cannot_tell_clean_from_never_ran]]`). **When 8d-iii
/// lands, this gate is RETIRED or RE-AIMED at the converted surface — it is not
/// loosened.**
///
/// ⛔ **The frozen seven are a SURFACE question, not a defect this pass may cure.** They
/// are `wat/spawn.wat`'s documented per-env builder constructors — `(thread/init f)`,
/// `(process/post-spawn f)`, `(process/runner-count n)`, … — spelled out in that file's
/// own header at lines 90–98 and written across 26 more live files. Curing them
/// means RESPELLING a documented public API across the corpus, which is the builder's
/// ruling and 255.8's already-scheduled *"stone for the wrong-join acceptance BEFORE
/// 8d-iii"*, not a rider's. This gate exists so that stone has an exact, derived,
/// non-growing list to work from.
#[cfg(test)]
mod faithful_surface_round_trip {
    use super::*;

    /// Every stdlib declaration name that does NOT survive the round trip through the
    /// faithful-Clojure surface. A name JOINING this list is a new hole in 8d's surface;
    /// a name LEAVING it is a cure, and the SCORE must say which side of the join moved.
    const UNSPELLABLE_IN_THE_FAITHFUL_SURFACE: &[&str] = &[
        // ⛔ UNREACHABLE once `wat/spawn.wat` is converted: the parent segment
        // (`process` / `thread`) is not a TYPE, so nothing can restore the `/`, and
        // every keyword-spelled caller keeps asking for a key that no longer exists.
        // This is the eighth draw's measured address. Curing it means RESPELLING a
        // documented public API (see this module's doc) — the builder's ruling.
        ":wat::core::Fault/of",
        ":wat::spawn::process/env",
        ":wat::spawn::process/max-message-bytes",
        ":wat::spawn::process/post-spawn",
        ":wat::spawn::process/runner-count",
        ":wat::spawn::thread/init",
        ":wat::spawn::thread/post-spawn",
        ":wat::spawn::thread/runner-count",
    ];

    /// The declaration heads whose item 1 is a NAME. `defservice`/`defrule` mint their
    /// members by string concatenation from this same name, so covering the declaration
    /// covers them; a minted name is never written in a source file and is therefore not
    /// something the codemod can rewrite.
    fn is_named_declaration(head: &str) -> bool {
        matches!(
            head,
            ":wat::core::defn"
                | ":wat::core::def"
                | ":wat::core::defmacro"
                | ":wat::core::defclause"
                | ":wat::core::defrecord"
                | ":wat::core::defstruct"
                | ":wat::core::defenum"
                | ":wat::core::defsurface"
                | ":wat::core::defservice"
                | ":wat::core::typealias"
                | ":wat::holon::defrecord"
        )
    }

    #[test]
    fn every_stdlib_declaration_name_survives_the_faithful_surface() {
        let (_symbols, _macros, types) = stdlib_snapshot();

        let mut measured = 0usize;
        let mut slash_joined = 0usize;
        let mut offenders: Vec<String> = Vec::new();

        for src in crate::load::stdlib::stdlib_files() {
            let forms = crate::parser::parse_all_with_file(src.source, src.path)
                .unwrap_or_else(|e| panic!("stdlib file {} must parse: {e:?}", src.path));
            for form in &forms {
                let WatAST::List(items, _) = form else {
                    continue;
                };
                let Some(head) = items.first().and_then(crate::declare::parse::head_fqdn) else {
                    continue;
                };
                if !is_named_declaration(head.as_ref()) {
                    continue;
                }
                let is_macro = head.as_ref() == ":wat::core::defmacro";
                let name = match items.get(1) {
                    Some(WatAST::Keyword(k, _)) => k.clone(),
                    Some(WatAST::Symbol(id, _)) if id.is_reference() => {
                        crate::edn::render::ns_to_wat_path(id.receiver(), id.method())
                    }
                    _ => continue,
                };
                // Not a call-head-shaped name: nothing for the codemod to rewrite.
                let Some(clj) = crate::edn::render::wat_keyword_to_clojure_symbol(&name) else {
                    continue;
                };
                // rune:lint(one-variant-separator, namespace) — a faithful symbol's ONE `/`
                // is its namespace/name split; no enum/variant is involved.
                if !clj.contains('/') {
                    continue;
                }
                measured += 1;
                // rune:lint(one-variant-separator, namespace) — the leaf's member join
                if name.contains('/') {
                    slash_joined += 1;
                }

                // What `declare::parse` / `macros::parse` would register for the CONVERTED
                // declaration: `ns_to_wat_path`, which writes `::` always.
                let stored = crate::edn::render::ns_to_wat_path(
                    wat_reader::identifier::receiver(&clj),
                    wat_reader::identifier::method(&clj),
                );
                // ⛔ ONE question, asked of the LIVE `TypeEnv`, never modelled: would
                // `rekey_type_member_functions` put this join back? That is
                // `reconstruct_call_path`, the same door the rekey pass itself calls.
                //
                // ⛔ AND THE REKEY PASS WALKS `sym.functions_iter()` ONLY — a MACRO is
                // never rekeyed, whatever its parent is. That is a fact about the code
                // above, not a model of any cure: a `defmacro` registers at step 4 and
                // expands at step 5, both before the `TypeEnv` is attached at step 6.97.
                let survives = stored == name
                    || (!is_macro
                        && crate::types::reconstruct_call_path(
                            wat_reader::identifier::receiver(&clj),
                            wat_reader::identifier::method(&clj),
                            types,
                        ) == name);
                if !survives {
                    offenders.push(name);
                }
            }
        }
        offenders.sort();
        offenders.dedup();

        // ⛔ NON-VACUITY, on two axes. A census that parsed nothing and a census whose
        // discriminating population is empty both read exactly like a clean one
        // (`[[feedback_a_green_test_can_prove_nothing]]`).
        assert!(
            measured > 500,
            "the declaration census measured only {measured} names — the stdlib list or the \
             forward map changed shape; this green is worthless"
        );
        assert!(
            slash_joined >= 16,
            "only {slash_joined} stdlib declaration names carry a `/` member join — the \
             population this gate discriminates ON has vanished, so a pass proves nothing"
        );

        let want: Vec<String> = UNSPELLABLE_IN_THE_FAITHFUL_SURFACE
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            offenders, want,
            "\n🔥 THE FAITHFUL-SURFACE DECLARATION ROUND TRIP CHANGED \
             ({measured} names measured, {slash_joined} of them `/`-joined).\n\
             A name here converts to a faithful-Clojure symbol that registers under a \
             DIFFERENT key, so every keyword-spelled caller loses it the moment its \
             declaring file is converted.\n\
             If you ADDED one: the `/` join means Type/member (255.4) — use `::` unless the \
             segment before the `/` is a declared TYPE.\n\
             If you CURED one: re-freeze the list above and say, in the SCORE, which side of \
             the join moved."
        );
    }
}
