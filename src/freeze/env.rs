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
use crate::check::{validate_aggregate_containment, validate_arc170_legacy_callsites, validate_bare_legacy_primitives, CheckError, CheckErrors};
use crate::macros::{expand_all, register_defmacros, register_stdlib_defmacros, MacroRegistry};
use crate::resolve::{normalize_symbol_refs, resolve_references};
use crate::runtime::{
    preregister_acronyms, preregister_protocol_names, preregister_stdlib_defclause_stub,
    register_aggregate_methods, register_defines, register_enum_methods, register_newtype_methods,
    register_stdlib_defines, register_stdlib_runtime_defs, register_struct_methods,
    register_type_predicates, EvalBreak, Environment, SymbolTable,
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
}

/// Build the full registered environment from already-parsed,
/// already-load-resolved user forms.
///
/// `user_forms = vec![]` yields the stdlib-only environment (the
/// test path). All user-side steps (`expand_all(user)`,
/// bare-legacy walker, `register_types(user)`, `register_defines(user)`,
/// `preregister_protocol_names`, `preregister_acronyms(runtime)`,
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
/// - 6.95. `preregister_protocol_names`
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

    // 4. Macro registration + expansion. Stdlib defmacros register
    //    first; user defmacros layer on top and can shadow (subject
    //    to the reserved-prefix gate) or reference stdlib forms.
    let mut macros = MacroRegistry::new();
    let stdlib_post_macros = register_stdlib_defmacros(stdlib, &mut macros)?;
    let post_macro_reg = register_defmacros(user_forms, &mut macros)?;

    // ORDER LOAD-BEARING: macro_eval purity (src/macros/eval.rs) depends on
    // expand_all preceding register_defines. See freeze.rs header comment.
    //
    // Arc 265 — pre-register declare-acronyms forms into macro_sym BEFORE
    // expand_all so defservice's pascal->kebab-in call at expand time can
    // consult the registry.
    let mut macro_sym = SymbolTable::default();
    preregister_acronyms(&post_macro_reg, &mut macro_sym)
        .map_err(|e| match e {
            EvalBreak::Diagnostic(re) => StartupError::Runtime(Box::new(re)),
            EvalBreak::Signal(_) => unreachable!(
                "interpreter bug: eval-loop control signal escaped to freeze layer"
            ),
        })?;
    // Expansion-born stdlib defmacros (e.g. a `defservice`'s `…/start` companion,
    // emitted by a macro-generating-macro) register through `expand_all` ->
    // `MacroRegistry::register`. Grant them the same reserved-prefix bypass the
    // literal top-level path (`register_stdlib`) already has — STDLIB ONLY. User
    // expansion below stays gated, so a mis-namespaced user macro still halts.
    macros.set_stdlib_privilege(true);
    let expanded_stdlib = expand_all(
        stdlib_post_macros,
        &mut macros,
        &Environment::default(),
        &macro_sym,
    )?;
    macros.set_stdlib_privilege(false);
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
    //     sees them as callable names (e.g. :wat::kernel::spawn-program').
    for form in &stdlib_residue {
        preregister_stdlib_defclause_stub(form, &mut symbols);
    }
    // (b) Extract stdlib forms that need RUNTIME registration via
    //     runtime_defs: defclause, defprotocol, extend-type, def.
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
                                | ":wat::core::defprotocol"
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
        let mut prefix_items =
            vec![WatAST::Keyword(":wat::core::Vector".into(), crate::rust_caller_span!())];
        for p in entry.prefixes {
            prefix_items.push(WatAST::Keyword(p.to_string(), crate::rust_caller_span!()));
        }
        let restricted_to_ast = WatAST::List(prefix_items, crate::rust_caller_span!());
        let mut meta: HashMap<String, WatAST> = HashMap::new();
        meta.insert(":restricted-to".to_string(), restricted_to_ast);
        symbols
            .binding_metadata
            .entry(name)
            .or_insert_with(HashMap::new)
            .extend(meta);
    }

    // 6.95. Arc 232 Stone 232.3 — pre-register defprotocol names into
    //       runtime_def_values BEFORE the resolve pass.
    preregister_protocol_names(&residue, &mut symbols)
        .map_err(|e| match e {
            EvalBreak::Diagnostic(re) => StartupError::Runtime(Box::new(re)),
            EvalBreak::Signal(_) => unreachable!(
                "interpreter bug: eval-loop control signal escaped to freeze layer"
            ),
        })?;

    // 6.96. Arc 265 — pre-register declare-acronyms forms into the
    //       runtime SymbolTable (macro_sym covered expand-time; this
    //       covers eval-time).
    preregister_acronyms(&residue, &mut symbols)
        .map_err(|e| match e {
            EvalBreak::Diagnostic(re) => StartupError::Runtime(Box::new(re)),
            EvalBreak::Signal(_) => unreachable!(
                "interpreter bug: eval-loop control signal escaped to freeze layer"
            ),
        })?;

    // 6.97. Arc 293.4b — pre-attach the TypeEnv to the SymbolTable BEFORE the
    //       resolve pass so `is_resolvable_call_head` can distinguish a
    //       `:S/method` surface-method call head from an UnresolvedReference.
    //       At this point `types` is fully populated (all steps 5–6.9x done).
    //       `FrozenWorld::freeze` later overwrites `sym.types` with the same
    //       data (via `symbols.set_types(Arc::new(types.clone()))`); the early
    //       attach here is strictly for the resolve pass.
    symbols.types = Some(std::sync::Arc::new(types.clone()));

    // 7. Name resolution.
    // Stone 251.1b — normalize before resolve so rewritten AST flows
    // through check + eval with keyword heads.
    residue = normalize_symbol_refs(residue, &symbols, &macros)?;
    resolve_references(&residue, &symbols, &macros)?;

    // 7.6. Stone 237.8b (+ arc 209 host-parity-4a) — register stdlib
    //      defclause / defprotocol / extend-type / def forms into
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
    for form in &residue {
        if let WatAST::List(items, _) = form {
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::extend-type")
            {
                crate::runtime::register_extend_type_surface_impls(form, &mut symbols, false)
                    .map_err(|e| StartupError::Runtime(Box::new(e)))?;
            }
        }
    }

    Ok(EnvBundle {
        types,
        macros,
        symbols,
        residue,
    })
}
