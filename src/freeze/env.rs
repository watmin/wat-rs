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

use crate::ast::WatAST;
use crate::check::{
    validate_aggregate_containment, validate_arc170_legacy_callsites,
    validate_bare_legacy_primitives, CheckError, CheckErrors,
};
use crate::macros::{
    expand_all, register_aggregate_kwargs_companions, register_defmacros,
    register_stdlib_defmacros, MacroRegistry,
};
use crate::resolve::{normalize_symbol_refs, resolve_references, ResolveError};
use crate::runtime::{
    preregister_acronyms, preregister_stdlib_defclause_stub, register_aggregate_methods,
    register_defines, register_enum_methods, register_newtype_methods, register_stdlib_defines,
    register_stdlib_runtime_defs, register_struct_methods, register_type_predicates, Environment,
    EvalBreak, SymbolTable,
};
use crate::stdlib::stdlib_forms;
use crate::types::{register_stdlib_types, register_types_with_acronyms, TypeEnv};

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
    pub declared_rete_defns: std::collections::HashSet<String>,
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
    let declared_rete_defns = extract_rete_defn_names(&user_forms);
    let user_forms = rewrite_rete_defn_heads(user_forms);

    // 4. Macro registration + expansion. Stdlib defmacros register
    //    first; user defmacros layer on top and can shadow (subject
    //    to the reserved-prefix gate) or reference stdlib forms.
    let mut macros = MacroRegistry::new();
    let stdlib_post_macros = register_stdlib_defmacros(stdlib, &mut macros)?;
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
    let stdlib_post_types = register_stdlib_types(expanded_stdlib, &mut types)?;
    // Thread the namespace-scoped acronym registry (populated by `preregister_acronyms`
    // above, BEFORE macro expansion) into type registration so a `:satisfies` surface's
    // S1 protocol synthesis restores acronym casing on its `::Op`/`::Reply` variants
    // identically to how `defservice :impls` does at expand time.
    let post_types =
        register_types_with_acronyms(expanded_user, &mut types, &macro_sym.acronym_registry)?;
    // Arc 293.W — containment rule: after BOTH stdlib and user types are fully
    // registered, verify that no portable aggregate (record/holon) declares a
    // non-portable (struct) field. Forward references are now resolved, so the
    // check is complete and sound. TypeError converts to StartupError::Type via
    // the From impl in freeze.rs.
    validate_aggregate_containment(&types)?;

    // 6. Function definitions.
    let mut symbols = SymbolTable::new();
    // Stone 237.8b — capture stdlib residue so defclause forms reach
    // register_runtime_defs.
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
            if let WatAST::List(items, _) = form {
                matches!(
                    items.first(),
                    Some(WatAST::Keyword(k, _))
                        if matches!(
                            k.as_str(),
                            ":wat::core::defclause"
                                | ":wat::core::extend-type"
                                // Arc 255 escape-hatch — scalar stdlib `def` forms
                                // (e.g. MAX-READLN-BYTES) must reach runtime_def_values.
                                | ":wat::core::def"
                        )
                )
            } else {
                false
            }
        })
        .collect();
    let mut residue = register_defines(post_types, &mut symbols)?;

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
    residue = normalize_symbol_refs(residue, &symbols, &macros)?;
    // DEFERRED, not swallowed: an unresolved reference is very often the SYMPTOM of a
    // malformed definition that failed to register. Running `check_program` first lets the
    // located cause be reported; if check is clean, this error is re-raised unchanged.
    let deferred_resolve = resolve_references(&residue, &symbols, &macros, &types).err();

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

/// Arc 278 #88 — step 3b's SCAN half: collect the canonical (`<T,…>`-stripped) name of every
/// TOP-LEVEL `(:wat::rete::core::defn :name [args] -> :Ret body…)` declaration in `forms`, before
/// macro expansion touches anything. Top-level only — every corpus site the design stone
/// measured is a bare top-level declaration (mirrors the fixture and every `where`-callee in
/// the corpus); a form nested inside a macro-emitted `do`/`let` is out of this slice's scope.
fn extract_rete_defn_names(forms: &[WatAST]) -> std::collections::HashSet<String> {
    let mut declared = std::collections::HashSet::new();
    for form in forms {
        let WatAST::List(items, _) = form else {
            continue;
        };
        let Some(WatAST::Keyword(k, _)) = items.first() else {
            continue;
        };
        if k != ":wat::rete::core::defn" {
            continue;
        }
        if let Some(WatAST::Keyword(name_kw, _)) = items.get(1) {
            if let Ok((name, _type_params)) = crate::runtime::split_name_and_type_params(name_kw) {
                declared.insert(name);
            }
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
                if let Some(WatAST::Keyword(k, kspan)) = items.first() {
                    if k == ":wat::rete::core::defn" {
                        items[0] = WatAST::Keyword(":wat::core::defn".to_string(), kspan.clone());
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
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(()),
    };
    match head {
        ":wat::core::extend-type" => {
            crate::runtime::register_extend_type_surface_impls(form, symbols, false)
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
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))]
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

        // The clause itself is a well-formed bind — `(?loc <- :location)`, a List, not a bare
        // keyword (the shape the 9a corruption injected; see `src/rete/validate.rs`'s
        // `corrupt_when_clause_is_a_located_error` for that case).
        assert!(
            matches!(&cond_items[1], WatAST::List(_, _)),
            "a well-formed bind clause is a List; got {:?}",
            cond_items[1]
        );
    }
}
