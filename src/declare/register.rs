//! Arc 109 Stone 2 — the declare home's REGISTER phase.
//!
//! Split by PHASE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-declare-home.md`): `defn`/`defstruct`/
//! `defenum`/`defalias`/`extend`/`declare-acronyms` are forms this substrate mints regularly, and
//! a per-form layout (`defn.rs`, `defstruct.rs`, …) would multiply a file per form. PHASE is the
//! honest axis instead — this file is the pass that actually WRITES into the `SymbolTable`:
//! `register_*` (the 13 fns the brief names) plus five helpers placed here by measuring their
//! callers, not by guessing (`meta_has_doc_axis_key`, `record_binding_metadata`,
//! `parametric_decl_type`, `restrictions_to_binding_metadata_ast`, `build_delegate_body` — the
//! last of these the DESIGN doc could not place; its only two callers are both `register_defalias`,
//! in this same file). Moved verbatim out of `src/runtime.rs` (arc 109 Stone 2). Behaviour is
//! unchanged; only the location moved.
//!
//! Siblings: `parse.rs` (read a declaration form's shape), `preregister.rs` (the earlier
//! stub-before-bodies pass), `typevar.rs` (free/bound type-variable walking).

use std::sync::Arc;

use wat_macros::wat_special_form_impl;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    EnumValue, Environment, EvalBreak, Function, FunctionBody, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value,
};

// The following are genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-1) and stay there: `ClauseRegPhase` (register_defclause's own
// phase enum), `eval_inner` (the evaluator's inner entry point; 2 of the design's measured 3
// touches land in this file), `no_field_names` (a shared empty-Arc<Vec<String>> helper).
use crate::runtime::{eval_inner, no_field_names, ClauseRegPhase};

// Arc 109 Stone — the defclause-into-function-home stone moved `parse_defclause_form` /
// `parse_extend_type_form` (the defclause/extend-type FORM parsers) out of `runtime.rs`'s
// `defclause_dispatch` region into `src/function/`'s existing parse.rs — this dissolves two of
// this file's `declare`-to-`runtime` cycle edges as a side effect (docs/arc/2026/04/109-kill-std/).
use crate::function::{parse_defclause_form, parse_extend_type_form};

use crate::declare::parse::{
    is_runtime_declaration_head, parse_defalias_form, try_parse_fn_shape_def,
    try_parse_metadata_map, try_parse_user_variadic_def_fn_form, try_parse_variadic_def_fn_form,
};
use crate::declare::preregister::{preregister_fn_defs_in_do, preregister_fn_defs_in_let};

/// Arc 170 #13 — the ONE door for defclause registration.
///
/// `parse_defclause_form` used to be called from four sites in this file,
/// each re-deriving its own subset of THREE effects:
///   1. the stub `Function` in `sym.functions` (so the checker resolves
///      recursive/forward calls to the defclause name before the real
///      ClauseSet exists);
///   2. the real `ClauseSet` (as `Value::wat__core__clauses`) into
///      `sym.runtime_def_values`, removing the stub;
///   3. the `binding_metadata` insert for the defclause's optional
///      `{...}` metadata-map (e.g. `{:restricted-to […]}`), stored exactly
///      as `def`/`defn` do so the EXISTING `walk_for_restricted_call`
///      walker (`check.rs`, reading `SymbolTable.binding_metadata` via
///      `CheckEnv::from_symbols`) enforces it with NO change to the
///      enforcement mechanism itself.
///
/// Two of the four original sites dropped effect 3 — a stdlib defclause's
/// metadata never reached `binding_metadata`, and neither did a defclause
/// registered purely at eval time (e.g. a future REPL). This function is the
/// one place that answers "what does registering a defclause mean?"; the
/// four original call sites now just call it with the right `privilege` +
/// `phase` and their locations do not move (freeze-ordering is unchanged).
///
/// `privilege` selects ONLY the reserved-prefix behaviour — already a
/// parameter of `parse_defclause_form`, and the `is_reserved_prefix` guard
/// on stub creation below that stdlib bypasses (`allow_reserved`). It does
/// not get its own copy of the registration logic.
///
/// `phase` selects which of effects 1/2 land (see [`ClauseRegPhase`]).
/// Effect 3 (the metadata insert) is UNCONDITIONAL on both `privilege` AND
/// `phase` — it lands whichever phase actually calls this function. This is
/// deliberate, not an oversight: the User Stub phase (`register_defines`)
/// runs before `check_program` for file-loaded user source, so metadata
/// landing there is sufficient for that path; but stdlib's Runtime phase
/// (`register_stdlib_runtime_defs`) and a future eval-time/REPL defclause
/// (`register_runtime_defs_form`) each reach this function ONLY via the
/// Runtime phase, with no preceding Stub-phase call for that same form — if
/// the metadata insert lived in the Stub arm only, both of those paths would
/// drop the metadata-map exactly as they did before this collapse. Landing
/// it in both phases is idempotent (same key, same map — a re-insert is a
/// harmless no-op) and is what closes the REPL-time gap the brief calls out
/// alongside the stdlib one ("Same class, next door").
pub fn register_defclause(
    form: &WatAST,
    privilege: crate::resolve::Privilege,
    phase: ClauseRegPhase,
    sym: &mut SymbolTable,
) -> Result<String, RuntimeError> {
    let (name, cs) = parse_defclause_form(form, privilege)?;

    // Effect 3 — the metadata-map binding. Unconditional on privilege and on
    // phase: a defclause is a defclause, and where its form was loaded from is
    // not a property its metadata knows about. Stored exactly as `def`/`defn`
    // do, so the EXISTING `walk_for_restricted_call` enforces `{:restricted-to
    // […]}` with no change to the enforcement mechanism.
    //
    // This insert is load-bearing and PROVEN so: deleting it turns
    // `defclause_metadata_gap_stdlib_registered_restricted_to_enforced`
    // (check.rs) RED. Verified by hand, 2026-07-28, in both directions.
    if let Some(meta) = &cs.metadata {
        if !meta.is_empty() {
            record_binding_metadata(sym, name.clone(), meta.clone(), form.span())?;
        }
    }

    match phase {
        ClauseRegPhase::Stub => {
            // Effect 1 — the stub Function. The reserved-prefix guard is
            // user-path behaviour; stdlib deliberately bypasses it
            // (allow_reserved), since stdlib defclauses live under the
            // reserved `:wat::core::` namespace by construction.
            let reserved_ok = matches!(privilege, crate::resolve::Privilege::Stdlib)
                || !crate::resolve::is_reserved_prefix(&name);
            if reserved_ok && !sym.has_function(&name) {
                // Arc 244 — use NilLit (canonical nil value literal) not Keyword.
                let stub_body = WatAST::NilLit(form.span().clone());
                let stub_fn = Arc::new(Function {
                    name: Some(name.clone()),
                    params: vec![],
                    type_params: vec![],
                    param_types: vec![],
                    ret_type: crate::types::TypeExpr::Tuple(vec![]),
                    rest_param: None,
                    rest_param_type: None,
                    body: FunctionBody::Wat(Arc::new(stub_body)),
                    closed_env: None,
                    rete: None,
                    synthesized_for: None,
                });
                sym.register_function(name.clone(), stub_fn);
            }
        }
        ClauseRegPhase::Runtime => {
            // Effect 2 — the real ClauseSet into runtime_def_values,
            // replacing any stub registered by a prior Stub-phase call.
            let value = Value::wat__core__clauses(cs.clone());
            sym.remove_function(&name); // remove stub if pre-registered
            sym.register_def_value(name.clone(), value);
        }
    }

    Ok(name)
}

/// Walk `forms`, register every `(:wat::core::define ...)` into `sym`,
/// and return the remaining (non-define) forms in order. Dupe
/// registration halts with [`RuntimeError::DuplicateDefine`].
///
/// Arc 255 Stone 1a-β-ii — this is `:wat::core::def`'s declare-time processor (the fn-shape
/// `try_parse_fn_shape_def` arm below pre-registers a `def` whose RHS is `(:wat::core::fn
/// …)`); `:wat::core::defalias` is ALSO processed from inside this same fn (the
/// `parse_defalias_form` arm), but that name's own `role = declare` pointer lives on
/// `parse_defalias_form` itself (`src/declare/parse.rs`), not here — see that fn's doc.
#[wat_special_form_impl(":wat::core::def", role = declare)]
pub fn register_defines(
    forms: Vec<WatAST>,
    sym: &mut SymbolTable,
) -> Result<Vec<WatAST>, RuntimeError> {
    let mut rest = Vec::new();
    for form in forms {
        // Stone 241.12 — `:wat::core::defalias` native registration.
        // Parsed + registered in Rust; the form is then consumed (does NOT go to `rest`
        // so it does not reach check_program's expression-level inference; defalias is
        // a declaration form, not a value-producing expression).
        if let Some((alias, target)) = parse_defalias_form(&form) {
            let form_span = form.span().clone();
            register_defalias(
                &alias,
                &target,
                sym,
                form_span,
                crate::resolve::Privilege::User,
            )?;
            // Consumed — defalias does not participate in expression-level inference.
            // The alias name is now in sym.functions; the checker resolves call sites.
            rest.push(form);
            continue;
        }
        // Stone 241.11 — `:wat::core::define` is HARD CUT. The is_define_form branch
        // that pre-registered define forms is DELETED. Define forms now pass through
        // to the checker (step 8), which rejects them via the MalformedForm arm and
        // surfaces the retirement remedy pointing at :wat::core::defn.
        if let Some((path, func, metadata_opt)) = try_parse_fn_shape_def(&form)? {
            // Arc 166 — `(:wat::core::def :name (:wat::core::fn sig body))`
            // pre-registers into `sym.functions` so the type checker resolves
            // recursive self-references inside the fn body. Form stays in
            // `rest` so `register_runtime_defs` still evaluates the def at
            // freeze time and populates `runtime_def_values` (call dispatch's
            // precedence ladder picks `sym.functions` first; the runtime
            // entry is vestigial-but-correct).
            //
            // Collision policy: pre-register ONLY if the name is new.
            // A collision (same name already in `sym.functions`) is NOT an
            // error here — `def`'s redef discipline (arc 157 slice 1a-ii)
            // owns that decision in `infer_def` (`DefRedefForbidden` by
            // default; opt-in with type-stability when `redef_allowed`).
            // Pre-registering a divergent second def would emit
            // `DuplicateDefine` from the runtime side, masking the
            // type-check-side `DefRedefForbidden` the user expects from
            // `def`. Silent skip keeps the def-redef path authoritative.
            //
            // Stone 241.6 — if a metadata-map was present, store it now.
            // Storage is pre-registration: binding_metadata is populated
            // at `register_defines` time alongside the fn pre-registration.
            let form_span = form.span().clone();
            // Phase-1 migration to the ONE gate (resolve::registration). register_defines
            // is the USER path (stdlib runtime defs go through register_stdlib_runtime_defs);
            // presence maps to Equivalent → NoOp (skip the pre-register; `def`'s redef
            // discipline in infer_def owns real divergence — a Duplicate never arises here).
            let existing = if sym.has_function(&path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &path,
                crate::resolve::Privilege::User,
                existing,
                &form_span,
                || -> Result<(), RuntimeError> {
                    sym.register_function(path.clone(), func);
                    Ok(())
                },
            )?;
            if let Some(meta) = metadata_opt {
                record_binding_metadata(sym, path, meta, &form_span)?;
            }
            rest.push(form);
        // Stone 241.14 — def-restricted fn-shape pre-registration arm DELETED.
        // def-restricted is HARD CUT; forms reaching this path are rejected
        // at check.rs before register_defines runs for them.
        //
        // Arc 150 — user-source variadic `defn` forms expand to
        // `(:wat::core::def :name (:wat::core::fn [... & rest <- :T] -> :Ret body))`.
        // `try_parse_fn_shape_def` returns None for these (allow_rest_binder=false).
        // `try_parse_user_variadic_def_fn_form` handles them: parses with
        // allow_rest_binder=true and PROPAGATES argspec errors as RuntimeError
        // so malformed forms (double `&`, `&` without binder, non-Vector rest type)
        // surface as StartupError::Runtime rather than silently skipping registration
        // and later hitting the resolver with UnresolvedReference.
        } else if let Some((path, func)) = try_parse_user_variadic_def_fn_form(&form)? {
            let form_span = form.span().clone();
            // Phase-1 migration to the ONE gate (user variadic def arm; see the fn-shape arm above).
            let existing = if sym.has_function(&path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &path,
                crate::resolve::Privilege::User,
                existing,
                &form_span,
                || -> Result<(), RuntimeError> {
                    sym.register_function(path.clone(), func);
                    Ok(())
                },
            )?;
            rest.push(form);
        } else if let WatAST::List(ref do_items, _) = form {
            // Arc 170 Gap C — top-level `(:wat::core::do ...)` splice.
            // Peek into the do body and pre-register any fn-shape defs so
            // `resolve_references` (step 7) can validate call heads that
            // reference those names. The do form itself stays in `rest`
            // so `register_runtime_defs` can evaluate it later.
            if matches!(
                do_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::do"
            ) {
                preregister_fn_defs_in_do(do_items, sym, crate::resolve::Privilege::User)?;
            // Arc 170 Gap D — top-level `(:wat::core::let bindings body...)` splice.
            // Mirror of Gap C for `let`. The body forms live at items[2..] (per
            // arc 168 multi-form body). Peek into the body and pre-register any
            // fn-shape defs so `resolve_references` can validate call heads.
            // The let form itself stays in `rest` so `register_runtime_defs`
            // can evaluate it later.
            } else if matches!(
                do_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::let"
            ) {
                preregister_fn_defs_in_let(do_items, sym, crate::resolve::Privilege::User)?;
            } else if matches!(
                do_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::defclause"
            ) {
                // Stone 237.3 — defclause pre-registration into sym.functions.
                //
                // The resolver (step 7) runs BEFORE register_runtime_defs (step 9)
                // and validates every call head via sym.get(). Defclause names are
                // normally registered into sym.runtime_def_values at step 9, but
                // recursive clause bodies (e.g. factorial) call the defclause name
                // inside the clause body — those call heads fail the resolver
                // because the name isn't in sym.functions yet.
                //
                // Pre-register a minimal stub Function so the resolver can find the
                // name. The real Value::wat__core__clauses lands at step 9 via
                // register_runtime_defs_form; the stub is vestigial-but-harmless
                // (eval_dispatch_call's runtime_def_values precedence picks up the
                // clauses value before checking sym.functions).
                //
                // Mirror pattern from try_parse_fn_shape_def (arc 166) + the
                // preregister_fn_defs_in_do (arc 170 Gap C) stubs.
                //
                // The head keyword already matched `:wat::core::defclause` above,
                // so this form is UNAMBIGUOUSLY committed to being a defclause
                // declaration — there is no other parser downstream that would
                // give a malformed form a second chance. A parse failure here
                // (e.g. a malformed metadata-map, or any unexpected extra form
                // in the name/metadata position) MUST propagate as a located
                // RuntimeError at THIS definition site via `?`. Previously this
                // was `if let Ok(...) = ...` — a parse failure silently skipped
                // registration (no stub, no binding_metadata), so the name never
                // existed anywhere and the resolver (step 7, which runs before
                // check_program) reported an unrelated "unresolved reference" at
                // every CALL site instead — the wrong form never ruined itself
                // at the place it was written. See defclause metadata-map probes.
                //
                // Arc 170 #13 — the ONE door (`register_defclause`) owns the
                // stub-Function + binding_metadata effects for the Stub phase;
                // see its doc comment for the full three-effect/two-phase shape.
                register_defclause(
                    &form,
                    crate::resolve::Privilege::User,
                    crate::runtime::ClauseRegPhase::Stub,
                    sym,
                )?;
            }
            rest.push(form);
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Arc 278 BRIEF-STONE-extend-user-checked — the surface-inheriting `extend-type`
/// registration, extracted from the stdlib arm so BOTH the stdlib path (build_env
/// step 7.6, `register_stdlib_runtime_defs` below) and the new user pre-check path
/// (build_env step 7.7, `env.rs`) register surface impls with their REAL inherited
/// sig — BEFORE `check_program`'s body-check sweep (check.rs:826) ever runs — rather
/// than a second, independently-drifting copy of "inherit sigs from
/// `SurfaceMember::Method`".
///
/// For each `(method_name, clause)` in the extend-type's impl clauses: `self`
/// (`fixed_params[0]`) is typed as the CONCRETE satisfier (`ed.type_name`, never the
/// surface's own self-type — impl bodies access concrete fields the structural
/// surface type does not carry); other params + the return type are inherited from
/// the matching `SurfaceMember::Method { args, ret }`.
///
/// If `ed.protocol_name` does not resolve to a `TypeDef::Surface` (i.e. this is a
/// PROTOCOL-target extend-type), this is a no-op — the protocol path
/// (`runtime_def_values`) is handled entirely by the caller, unchanged.
///
/// `skip_if_present`: when `true`, an already-registered `:<T>/<method>` key is
/// silently skipped rather than erroring — used at freeze step 9
/// (`register_runtime_defs_form`), which re-walks the SAME residue forms that
/// build_env step 7.7 already registered (the re-walk is expected, not a collision).
/// When `false` (the first-registration call sites: stdlib step 7.6, user step 7.7),
/// an already-present key is a genuine `DuplicateDefine` — two distinct extend-type
/// forms racing for the same `:<T>/<method>`.
pub(crate) fn register_extend_type_surface_impls(
    form: &WatAST,
    sym: &mut SymbolTable,
    skip_if_present: bool,
) -> Result<(), RuntimeError> {
    let (_canonical_key, ed) = parse_extend_type_form(form)?;
    let surface_def = sym
        .types_deref()
        .and_then(|t| t.get(&ed.protocol_name))
        .and_then(|td| match td {
            crate::types::TypeDef::Surface(s) => Some(s.clone()),
            _ => None,
        });
    let surf = match surface_def {
        Some(s) => s,
        None => return Ok(()), // protocol target — caller handles it
    };
    let members = surf.members;
    // Arc 170 C2 — bind the surface's own `<T>` type params to the concrete args parsed
    // from this extend-type's `:Protocol<ConcreteArgs>` target (positional zip). Empty for
    // a monomorphic surface (`surf.type_params` empty) → empty mapping → `check::rename`
    // is the identity function on every type → pure no-op for existing surfaces.
    let surface_type_subst: std::collections::HashMap<String, crate::types::TypeExpr> = surf
        .type_params
        .iter()
        .cloned()
        .zip(ed.protocol_type_args.iter().cloned())
        .collect();
    // Arc 109 ③ — the RUNTIME DISPATCH key (`concrete_type_fqdn/method`, built from a live
    // Value's type-erased class name — `runtime.rs`'s `eval_call`/surface-method dispatch
    // arm) is ALWAYS the BARE base name: a `Value::Aggregate`'s `class` field never carries
    // instantiation args. `ed.type_name` stays the FULL identity string (base + args) for
    // `is_subtype`/`transport_edge_keys` (see the comment on its field, `src/value/value.rs`)
    // — those key on the exact declared spelling on purpose — but a method REGISTRATION key
    // built from that full string would never match the bare dispatch lookup for a
    // genuinely parametric target (was previously masked only because a MONOMORPHIC
    // service's `handle-bare-name`-style target rendered bare too, with no args to carry;
    // Arc 109 ③ made every Handle target carry at least the transport-marker arg, which
    // surfaced this for monomorphic services too). Derive the base STRUCTURALLY off
    // `ed.type_te` (never re-parse `ed.type_name`'s string — for a parametric target that
    // string is exactly the now-illegal `Head<args>` shape this stone's wall refuses).
    let dispatch_type_base: String = match &ed.type_te {
        Some(crate::types::TypeExpr::Parametric { head, .. }) => format!(":{head}"),
        Some(crate::types::TypeExpr::Path(p)) => p.clone(),
        // STONE reap-the-angle-machinery (arc 109) — this fallback used to strip a
        // turbofish suffix off `ed.type_name` via `canonical_callable_name`. Angle-bracket
        // syntax is unexpressible now, so a non-Parametric/non-Path `type_te` never leaves
        // a suffix on `type_name` to strip; use it directly.
        _ => ed.type_name.clone(),
    };
    for (method_name, clause) in &ed.impl_clauses {
        let method_key = format!("{}/{}", dispatch_type_base, method_name);
        if sym.has_function(&method_key) {
            if skip_if_present {
                continue;
            }
            return Err(RuntimeError::new(
                form.span().clone(),
                RuntimeErrorKind::DuplicateDefine(method_key),
            ));
        }
        let member = members.iter().find(|m| match m {
            crate::types::SurfaceMember::Method { name, .. } => name == method_name,
            crate::types::SurfaceMember::Field { name, .. } => name == method_name,
        });
        let (param_types, ret_type) = match member {
            Some(crate::types::SurfaceMember::Method {
                args: member_args,
                ret,
                ..
            }) => {
                let pts: Vec<crate::types::TypeExpr> = clause
                    .args
                    .fixed_params
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        if i == 0 {
                            // self — already the concrete satisfier type; no substitution
                            // needed. Arc 109 ③ — use the STRUCTURED `type_te` computed once
                            // in `parse_extend_type_form`, not a re-parse of `type_name`'s
                            // string: for a parametric target that string carries the `<…>`
                            // arg suffix `format_type` renders it with, and re-parsing THAT
                            // via `parse_type_expr` is exactly the angle-bracket parse this
                            // stone's wall refuses — the old `unwrap_or_else` fallback would
                            // silently collapse to an opaque `Path("Handle<K,V,T>")` that
                            // never unifies with the properly-structured type elsewhere.
                            ed.type_te.clone().unwrap_or_else(|| {
                                crate::types::TypeExpr::Path(ed.type_name.clone())
                            })
                        } else {
                            // Arc 170 C2 — resolve the surface's own `<T>` (if any) to this
                            // satisfier's concrete binding. No-op when `surface_type_subst` is
                            // empty (monomorphic surface).
                            let raw = member_args
                                .fixed_params
                                .get(i)
                                .map(|(_, t)| t.clone())
                                .unwrap_or_else(|| {
                                    crate::types::TypeExpr::Path(":wat::core::nil".into())
                                });
                            crate::check::rename(&raw, &surface_type_subst)
                        }
                    })
                    .collect();
                (pts, crate::check::rename(ret, &surface_type_subst))
            }
            Some(crate::types::SurfaceMember::Field { ty, .. }) => (
                vec![ed
                    .type_te
                    .clone()
                    .unwrap_or_else(|| crate::types::TypeExpr::Path(ed.type_name.clone()))],
                crate::check::rename(ty, &surface_type_subst),
            ),
            // No matching surface member — shouldn't happen for a valid
            // extend-type, but fall back to the prior (nil placeholder)
            // behavior rather than fabricate a signature.
            None => (
                clause
                    .args
                    .fixed_params
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect(),
                clause.return_type.clone(),
            ),
        };
        let func = Arc::new(Function {
            name: Some(method_key.clone()),
            params: clause
                .args
                .fixed_params
                .iter()
                .map(|(n, _)| n.clone())
                .collect(),
            type_params: vec![],
            param_types,
            ret_type,
            rest_param: clause
                .args
                .rest_param
                .as_ref()
                .map(|(n, _)| crate::scope::env_key(n).into_owned()),
            rest_param_type: clause.args.rest_param.as_ref().map(|(_, t)| t.clone()),
            body: FunctionBody::Wat(clause.body.clone()),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        });
        sym.register_function(method_key, func);
    }
    Ok(())
}

/// Stone 237.8b — register stdlib defclause forms into runtime_def_values.
/// Passes allow_reserved=true to permit the `:wat::core::*` prefix on stdlib names.
/// For use ONLY with stdlib forms that live under :wat::core::* (reserved).
pub fn register_stdlib_runtime_defs(
    forms: &[WatAST],
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    for form in forms {
        let head = match form {
            crate::ast::WatAST::List(items, _) => match items.first() {
                Some(crate::ast::WatAST::Keyword(k, _)) => k.as_str(),
                _ => continue,
            },
            _ => continue,
        };
        match head {
            ":wat::core::defclause" => {
                // Arc 170 #13 — the ONE door. A defclause's metadata-map binds
                // the SAME way whether the form came from the stdlib or from
                // user source — where a form was loaded from is not a property
                // its metadata knows about; `register_defclause` stores it
                // unconditionally on privilege AND phase (see its doc comment).
                register_defclause(
                    form,
                    crate::resolve::Privilege::Stdlib,
                    ClauseRegPhase::Runtime,
                    sym,
                )?;
            }
            // Arc 293.4c — stdlib extend-type: branch on surface vs. protocol (mirrors user path).
            // Arc 278 BRIEF-STONE-extend-user-checked — the surface-inheriting registration
            // itself is now the SHARED routine (`register_extend_type_surface_impls`), also
            // called from build_env step 7.7 for USER surface impls; this is the FIRST
            // registration for stdlib forms, so a colliding key is a genuine DuplicateDefine
            // (skip_if_present=false).
            ":wat::core::extend-type" => {
                register_extend_type_surface_impls(form, sym, /*skip_if_present=*/ false)?;
                let (canonical_key, ed) = parse_extend_type_form(form)?;
                let is_surface = sym
                    .types_deref()
                    .and_then(|t| t.get(&ed.protocol_name))
                    .map(|td| matches!(td, crate::types::TypeDef::Surface(_)))
                    .unwrap_or(false);
                if !is_surface {
                    sym.register_def_value(canonical_key, Value::wat__core__extend_def(ed));
                }
            }
            // Arc 255 escape-hatch — scalar stdlib `def` forms (e.g. MAX-READLN-BYTES).
            // Fn-shape defs are pre-registered in sym.functions by register_stdlib_defines;
            // SCALAR defs (non-fn values like i64 constants) are NOT registered there.
            // Without this arm, evaluating the keyword at runtime falls through to a bare
            // keyword value (runtime_def_values miss → no sym.get hit → keyword literal).
            // Mirror of the `:wat::core::def` arm in register_runtime_defs_form with a
            // fresh Environment (no let-bindings at stdlib top level).
            ":wat::core::def" => {
                let def_items = match form {
                    crate::ast::WatAST::List(it, _) => it,
                    _ => continue,
                };
                if def_items.len() != 3 && def_items.len() != 4 {
                    continue; // malformed; check already caught it
                }
                let name = match &def_items[1] {
                    WatAST::Keyword(k, _) => k.clone(),
                    _ => continue,
                };
                // If 4 items, def_items[2] is metadata-map and def_items[3] is expr.
                // If 3 items, def_items[2] is the expr directly.
                let expr = if def_items.len() == 4 {
                    &def_items[3]
                } else {
                    &def_items[2]
                };
                // Skip fn-shape defs — they were already registered in sym.functions
                // by register_stdlib_defines (try_parse_fn_shape_def). Only register
                // non-fn (scalar) defs here to avoid duplicate/clobber.
                let sym_ref: &SymbolTable = sym;
                let env = Environment::new();
                match eval_inner(expr, &env, sym_ref) {
                    Ok(tv) => {
                        let value = tv.value_owned();
                        if !matches!(value, Value::wat__core__fn(_)) {
                            sym.register_def_value(name, value);
                        }
                    }
                    Err(_) => continue, // stdlib def eval failed; check pass caught it
                }
            }
            _ => continue,
        }
    }
    Ok(())
}

/// Arc 255 Stone "metadata-of answers in one shape" — the ONE predicate that decides whether a
/// stored `{...}` metadata map is a doc declaration (subject to `wat_doc::from_metadata`) or a
/// capability-only map (e.g. `{:restricted-to […]}`, read raw, untouched). Shared by the
/// registration gate below (`register_stdlib_defines`) and the read side
/// (`eval_metadata_of`'s wat branch) so the two cannot answer "is this a doc declaration?"
/// differently — the exact drift class this stone exists to close.
const AXIS_DECLARATION_KEYS: &[&str] = &[
    ":purity",
    ":determinism",
    ":totality",
    ":expand-time",
    ":category",
];

/// Does `meta` claim to declare substrate axis properties? See [`AXIS_DECLARATION_KEYS`].
///
/// ⚠ NARROWED 2026-08-31, by a RED. This predicate first read `:doc` alone (a silent skip:
/// `{:purity …}` with no `:doc` was never validated), then ALL doc directives — which broke
/// `probe_arc241_stone7_metadata_of_reflection`, whose fixtures carry `(def :my::x {:doc "the x
/// value"} 42)`. That is ARBITRARY USER METADATA, and `:doc`/`:added`/`:deprecated`/`:see` are
/// ordinary human documentation vocabulary a user's map legitimately holds.
///
/// The five axis keys are not: their values are `:wat::runtime::*` enum symbols, they are the
/// substrate's own closed-domain vocabulary, and nobody writes them as a casual note. **Using one
/// is an unambiguous claim to be declaring a substrate property** — which is the only honest
/// discriminator between a declaration and a comment. Both earlier predicates were wrong in
/// opposite directions; this one asks what the map CLAIMS, not what shape it happens to have.
///
/// `pub(crate)`, not module-private — BRIEF-STONE-see-can-cross-the-boundary (arc 255) makes
/// `intrinsic::reflect::check_see_refs` a THIRD consumer (storage door here + the `metadata-of`
/// reflection surface below it + now the `@see` gate), and it must call this SAME fn rather than
/// restate the five keys — a second key list is exactly how the three drift apart. No other
/// change to this fn or to `AXIS_DECLARATION_KEYS` itself.
pub(crate) fn meta_has_doc_axis_key(meta: &std::collections::HashMap<String, WatAST>) -> bool {
    AXIS_DECLARATION_KEYS.iter().any(|k| meta.contains_key(*k))
}

/// Arc 255 Stone "a declaration cannot be STORED unvalidated" — the ONE and ONLY door into
/// `sym.binding_metadata`. Storing and validating a binding's `{...}` metadata map become a
/// single operation, so a map that claims substrate axis properties (any key in
/// [`AXIS_DECLARATION_KEYS`]) has no way to reach the symbol table without first passing
/// `wat_doc::from_metadata` — the same gate `register_stdlib_defines` alone used to run before
/// this stone, now absorbed here so all six former direct-insert sites share it.
///
/// `span` MUST be the DECLARATION's own span — the `def`/`defn`/`defclause` form the author
/// wrote — never a later reader's call site. That substitution is the exact defect this stone
/// removes: before, only one of six sites validated, so a bad map written at one line surfaced
/// its error at whatever line first called `metadata-of`, arbitrarily far away. Every caller
/// below hands in the span of the form it just parsed, so the diagnostic lands on the author's
/// own line.
///
/// A capability-only map (no key in `AXIS_DECLARATION_KEYS`, e.g. `{:restricted-to […]}`) is not
/// a doc declaration and is stored exactly as before, unvalidated — `meta_has_doc_axis_key` is
/// the SAME predicate `eval_metadata_of`'s read side uses, so the two can never disagree on what
/// counts as one.
pub(crate) fn record_binding_metadata(
    sym: &mut SymbolTable,
    name: String,
    meta: std::collections::HashMap<String, WatAST>,
    span: &Span,
) -> Result<(), RuntimeError> {
    if meta_has_doc_axis_key(&meta) {
        let map_ast = WatAST::Map(
            meta.iter()
                .map(|(k, v)| (WatAST::Keyword(k.clone(), v.span().clone()), v.clone()))
                .collect(),
            span.clone(),
        );
        if let Err(e) = wat_doc::from_metadata(&map_ast) {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: name,
                    reason: format!(
                        "metadata-map doc contract violation (wat_doc::from_metadata): {e:?}"
                    ),
                },
            ));
        }
    }
    sym.binding_metadata.insert(name, meta);
    Ok(())
}

/// Stdlib-registration variant of [`register_defines`] that bypasses
/// the reserved-prefix check. Called by the startup pipeline on the
/// baked stdlib sources; user source still goes through
/// [`register_defines`] where the prefix check blocks mis-namespaced
/// user defines.
pub fn register_stdlib_defines(
    forms: Vec<WatAST>,
    sym: &mut SymbolTable,
) -> Result<Vec<WatAST>, RuntimeError> {
    let mut rest = Vec::new();
    for form in forms {
        // Stone 241.11 — stdlib `(:wat::core::defn ...)` macro-expands to
        // `(:wat::core::def :name (:wat::core::fn sig body))` before this
        // function runs (macro expansion is step 4; registration is step 6).
        // Pre-register the fn-shape def into `sym` so the checker (step 8)
        // resolves recursive self-references. Bypasses the reserved-prefix
        // gate (stdlib is privileged — all names are under :wat::* by design).
        if let Some((path, func, metadata_opt)) = try_parse_fn_shape_def(&form)? {
            if !sym.has_function(&path) {
                sym.register_function(path.clone(), func);
            }
            if let Some(meta) = metadata_opt {
                // Arc 255 Stone "wire the wat side to wat-doc" — a stdlib `defn`'s
                // `{...}` metadata-map is the wat-side entry point into the SAME
                // shared-contract crate an intrinsic's `///` block goes through
                // (`wat_doc::parse`): `wat_doc::from_metadata` reads the map
                // directly (no docstring exists to feed the text grammar — see
                // the DESIGN doc's finding) and enforces the SAME required set
                // with the SAME `DocError`s.
                //
                // ⛔ GATED on an axis key's presence, NOT unconditional — a corpus
                // check (2026-08-30) found THREE pre-existing stdlib `defn`s
                // (`wat/kernel/services/stdio.wat`: `write-fd-raw`,
                // `flood-stdout-raw`, `str-double`) whose metadata-map carries
                // ONLY `{:restricted-to […]}` — a capability restriction,
                // unrelated to and pre-dating this stone, enforced entirely by
                // `check.rs`'s restricted-call walker, never by `wat_doc`. Made
                // unconditional, `from_metadata` would raise `MissingProse` on
                // ALL THREE and fail stdlib startup — exactly the "migrate the
                // 409" breadth this stone's own DESIGN rejects, done by
                // accident to verbs nobody asked to move.
                //
                // Arc 255 Stone "a declaration cannot be STORED unvalidated" — the
                // validate-then-gate logic that used to live inline here (and only
                // here, of six insert sites) is now `record_binding_metadata`, the
                // ONE door every insert site routes through. The span passed is
                // this form's own — the stdlib `defn`'s declaration — so a bad map
                // is blamed on the line that wrote it, not on a later reader.
                record_binding_metadata(sym, path, meta, form.span())?;
            }
            rest.push(form);
        } else if let Some((path, func)) = try_parse_variadic_def_fn_form(&form) {
            // Stone 241.11 — stdlib variadic `defn` forms (e.g. `defn :i64::+ [_a <- & xs <- :T] -> ...`)
            // expand to `def :name (fn [_a <- & xs <- :T] -> ...)`. `try_parse_fn_shape_def` returns
            // None for variadic forms (allow_rest_binder=false). This branch handles them:
            // parse with allow_rest_binder=true, set rest_param + rest_param_type on the Function.
            // Stdlib is PRIVILEGED — reserved-prefix gate bypassed.
            sym.function_entry(path).or_insert(func);
            rest.push(form);
        } else if let Some((alias, target)) = parse_defalias_form(&form) {
            // Stone 241.12 — stdlib defalias native registration.
            // Stdlib is PRIVILEGED — reserved-prefix gate bypassed (check_reserved=false).
            let form_span = form.span().clone();
            register_defalias(
                &alias,
                &target,
                sym,
                form_span,
                crate::resolve::Privilege::Stdlib,
            )?;
            // Consumed — defalias declaration form does not reach check_program.
            // The stdlib residue is DISCARDED after step 6 anyway, but being explicit
            // about NOT pushing to rest avoids any check-time exposure.
        } else if let WatAST::List(ref do_items, _) = form {
            // Arc 170 Gap C — top-level `(:wat::core::do ...)` splice.
            // Mirror of the arm in `register_defines`; bypasses the
            // reserved-prefix check since stdlib source is privileged.
            if matches!(
                do_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::do"
            ) {
                preregister_fn_defs_in_do(do_items, sym, crate::resolve::Privilege::Stdlib)?;
            // Arc 170 Gap D — top-level `(:wat::core::let ...)` splice.
            // Mirror of Gap C for `let`; bypasses the reserved-prefix check
            // since stdlib source is privileged.
            } else if matches!(
                do_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::let"
            ) {
                preregister_fn_defs_in_let(do_items, sym, crate::resolve::Privilege::Stdlib)?;
            }
            rest.push(form);
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Walk every `:wat::core::struct` declaration in `types` and
/// synthesize its auto-generated constructor + per-field accessors
/// into `sym`. Runs after both stdlib and user defines have been
/// registered so any name collision with a user-supplied path raises
/// `DuplicateDefine` at a sensible point in the pipeline.
///
/// **What's synthesized, per struct `:my::ns::T` with fields
/// `(f1 :T1) (f2 :T2) ... (fn :Tn)`:**
///
/// - One constructor at the bare keyword path `:my::ns::T` (arc 293.R2.3 — `/new` annihilated):
///   ```text
///   :fn(T1, T2, ..., Tn) -> :my::ns::T
///   body: (:wat::core::struct-new :my::ns::T p1 p2 ... pn)
///   ```
/// - One accessor per field at `:my::ns::T/<field-name>`:
///   ```text
///   :fn(:my::ns::T) -> Ti
///   body: (:wat::core::struct-field self i)
///   ```
///
/// Users never write these; they invoke them by full keyword path.
/// The checker picks them up through [`crate::check::CheckEnv::from_symbols`]
/// as ordinary [`Function`] entries — no new scheme-registration path.
///
/// **Self-trust bypass.** Struct-method paths under `:wat::holon::*`
/// (the built-in `:wat::holon::CapacityExceeded/…`) would otherwise
/// hit the reserved-prefix check. This function skips the check: the
/// paths it emits are derived mechanically from struct declarations
/// the user / builtins authored legitimately, so emitting them under
/// the same prefix is legitimate too.
/// Arc 071 — build the type expression that names a declared
/// struct/enum/newtype. For monomorphic decls (`type_params` empty),
/// returns `:Foo` as a `Path`. For parametric decls (`type_params =
/// ["A","B"]`), returns `(:Foo :- [A B])` as a `Parametric` whose head
/// strips the leading `:` (matching how the type parser stores
/// Parametric heads — see arc 058's `Result`/`Option`/`Vec` registrations).
///
/// Without this, `register_struct_methods` / `register_enum_methods`
/// synthesized constructors with bare-path return types — fine for
/// monomorphic decls but broken for parametric ones, since the type
/// checker saw the body produce `:Foo` and rejected it against a
/// `(:Foo :- [i64])` signature. Surfaced by arc 070's `(WalkStep :- [A])`
/// (the first parametric built-in enum) when the lab harness
/// type-checked a real consumer.
fn parametric_decl_type(name: &str, type_params: &[String]) -> crate::types::TypeExpr {
    if type_params.is_empty() {
        crate::types::TypeExpr::Path(name.into())
    } else {
        crate::types::TypeExpr::Parametric {
            head: name.trim_start_matches(':').into(),
            args: type_params
                .iter()
                .map(|p| crate::types::TypeExpr::Path(p.clone()))
                .collect(),
        }
    }
}

/// Stone 241.14 — build a `WatAST::List` representing a `:restricted-to`
/// Encodes a restriction whitelist as a `WatAST::List` whose first item is
/// the `:wat::core::Vector` head keyword and whose remaining items are the
/// prefix keyword strings. This is the "internal path" encoding (distinct
/// from user-written `{:restricted-to [...]}` brace-forms which parse to
/// `WatAST::Vector`). `extract_prefix_list_from_metadata` in `check.rs`
/// handles both encodings.
fn restrictions_to_binding_metadata_ast(prefixes: &[String]) -> WatAST {
    let mut items = vec![WatAST::Keyword(
        ":wat::core::Vector".into(),
        crate::rust_caller_span!(),
    )];
    for p in prefixes {
        items.push(WatAST::Keyword(p.clone(), crate::rust_caller_span!()));
    }
    WatAST::List(items, crate::rust_caller_span!())
}

pub fn register_struct_methods(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::types::TypeDef;

    for (_name, def) in types.iter() {
        // Arc 293.2b — only Aggregate with kind==Struct gets struct methods.
        let struct_def = match def {
            TypeDef::Aggregate(a) if a.nature == crate::types::Nature::Struct => a,
            _ => continue,
        };

        // Arc 293 surface-splice — the struct CONSTRUCTOR mint moved to
        // `register_aggregate_methods` (THE ONE ctor source for every nature). This loop
        // now handles ONLY the struct-only restriction metadata below. Minting the ctor
        // here too would DuplicateDefine against the unified mint.

        // Arc 203 / Stone 241.14 — if the struct carries restriction metadata
        // (from `struct-restricted`, now HARD CUT per Stone 241.8), write the
        // ctor + per-field whitelists into `binding_metadata` so the arc 198
        // walker (`walk_for_restricted_call`) enforces them at type-check time.
        // Public fields (absent from `field_restrictions`) get no entry —
        // no `:restricted-to` entry = no restriction = any caller allowed.
        //
        // Stone 241.14 — populate-target changed from the deleted
        // `defined_value_restrictions` to `binding_metadata`.
        if let Some(restrictions) = &struct_def.restrictions {
            let ctor_path = struct_def.name.clone();
            let ctor_ast = restrictions_to_binding_metadata_ast(&restrictions.ctor_whitelist);
            sym.binding_metadata
                .entry(ctor_path)
                .or_default()
                .insert(":restricted-to".to_string(), ctor_ast);
            for (field_name, field_wlist) in &restrictions.field_restrictions {
                let accessor_path = format!("{}/{}", struct_def.name, field_name);
                let field_ast = restrictions_to_binding_metadata_ast(field_wlist);
                sym.binding_metadata
                    .entry(accessor_path)
                    .or_default()
                    .insert(":restricted-to".to_string(), field_ast);
            }
        }
    }
    Ok(())
}

/// Arc 293.R2.2 — ONE unified accessor codegen for every `TypeDef::Aggregate`
/// (Struct + Record + HolonRecord). The `register_struct_methods` ctor loop
/// and the deleted `register_record_methods` accessor loop are collapsed here:
/// same `parametric_decl_type` / `type_params` / `struct-field` body for ALL
/// natures, bare name guaranteed by the `parse_declared_name` fix in
/// `parse_recordtype`.
///
/// Arc 293 inheritance annihilation: all types are flat (nature + own fields only).
/// Inherited fields are always 0; field index == own enumeration index.
///
/// **DuplicateDefine is an error** — after the macro's accessor emission was
/// removed, no other path registers these accessor paths.
pub fn register_aggregate_methods(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::types::TypeDef;

    for (_name, def) in types.iter() {
        let agg = match def {
            TypeDef::Aggregate(a) => a,
            _ => continue,
        };

        // The parametric self-type (e.g. `(:t::R :- [T])` for generic, `:myapp::Pt`
        // for monomorphic) — used as the accessor's single param type so the
        // type checker binds type params at each call site.
        //
        // Accessor param type strategy (Arc 293.R2.2):
        //   Struct: always use the specific parametric type (was the existing behaviour
        //     and must stay so for struct type-safety).
        //   Record/HolonRecord, non-generic (type_params.is_empty()): use `:wat::core::Record`
        //     (backward compatible — existing stdlib code in rete.wat etc. passes
        //     `:wat::core::Record`-typed values to these accessors; changing to the specific
        //     type would break them. The old macro accessor used `:wat::core::Record` too.)
        //   Record/HolonRecord, generic (!type_params.is_empty()): use the specific
        //     parametric type so the type checker can bind the type variable at each
        //     call site and infer the correct return type (the probe requires this).
        let aggregate_type = parametric_decl_type(&agg.name, &agg.type_params);
        let accessor_param_type = match agg.nature {
            crate::types::Nature::Struct => aggregate_type.clone(),
            _ => {
                if agg.type_params.is_empty() {
                    crate::types::TypeExpr::Path(":wat::core::Record".into())
                } else {
                    aggregate_type.clone()
                }
            }
        };

        // Arc 293 inheritance annihilation: all types are flat; field index == own enumeration index.

        // Arc 293 surface-splice — THE ONE aggregate constructor mint (all natures).
        //
        // Before this arc there were TWO ways to construct an aggregate: `register_struct_methods`
        // minted the struct ctor in Rust (from registered fields → splice-aware), while
        // `defrecord`/`holon::defrecord` hand-built a ctor `defn` in the wat macro at expand-time
        // (Record.wat's `raw-ch/nf/syms` groups-of-3 walk → registry-BLIND, so `~@:Surface`
        // splices choked there). 293.R2.2 already unified the ACCESSORS here for every nature;
        // this unifies the CTOR the same way. Now that the macro no longer emits a ctor `defn`
        // and `register_struct_methods` no longer mints one, THIS is the sole ctor source for
        // Struct + Record + HolonRecord alike.
        //
        // Body is `(:wat::core::aggregate-new :T field-syms…)` — `eval_aggregate_new` is already
        // nature-blind (it dispatches Struct/Record/HolonRecord internally, incl. the holon
        // hologram), so the body is identical for every nature. Because it reads the REGISTERED
        // fields (splice already expanded by `parse_aggregate_fields_with_splices` at
        // registration), surface-splice works for records for free.
        //
        // ret_type = the parametric self-type (specific type, not the root `:wat::core::Record`)
        // so a constructed value flows where the specific type is required — matching both the old
        // struct ctor and the old record `defn`'s `-> ~fqdn`.
        {
            // Arc 294 item 9a — the ctor codegen flip: the bare type name (`agg.name`)
            // becomes a KWARGS macro (emitted from wat, see the aggregate macros in
            // Record.wat/core.wat); the POSITIONAL ctor this loop mints moves to the
            // type-name PRIME `:ns::T'`. Both the `name:` field and the `sym.functions`
            // registration key move together — this is THE ONE ctor source, now at the prime.
            let ctor_name = format!("{}'", agg.name);
            let all_param_names: Vec<crate::scope::Identifier> = agg
                .fields
                .iter()
                .map(|(n, _)| crate::scope::Identifier::bare(n.clone()))
                .collect();
            let all_param_types: Vec<crate::types::TypeExpr> =
                agg.fields.iter().map(|(_, t)| t.clone()).collect();
            let mut new_body_items = Vec::with_capacity(2 + agg.fields.len());
            new_body_items.push(WatAST::Keyword(
                ":wat::core::aggregate-new".into(),
                crate::rust_caller_span!(),
            ));
            new_body_items.push(WatAST::Keyword(
                agg.name.clone(),
                crate::rust_caller_span!(),
            ));
            for param_name in &all_param_names {
                // Arc 170 — REUSE the binder node.
                new_body_items.push(WatAST::Symbol(
                    param_name.clone(),
                    crate::rust_caller_span!(),
                ));
            }
            let ctor_func = Function {
                name: Some(ctor_name.clone()),
                params: all_param_names,
                type_params: agg.type_params.clone(),
                param_types: all_param_types,
                ret_type: aggregate_type.clone(),
                rest_param: None,
                rest_param_type: None,
                body: FunctionBody::Wat(Arc::new(WatAST::List(
                    new_body_items,
                    crate::rust_caller_span!(),
                ))),
                closed_env: None,
                rete: None,
                // Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — B2. `T'`'s body
                // NAMES `agg.name` (arg to `aggregate-new` above) by construction — that
                // mention is this fn's whole reason to exist, not a caller reaching for T.
                // Owner-scoped so `walk_for_restricted_call` exempts ONLY a mention of this
                // exact FQDN; a mention of any other restricted binding inside a future
                // companion is still walked and still refused.
                synthesized_for: Some(agg.name.clone()),
            };
            // Arc 294 item 9a follow-up — conform to `register_defines`' collision
            // policy (lines 530-549 above): silent-skip on re-walk. A forked bracket
            // worker legitimately re-registers a surface it was shipped (the
            // `surface-forms` carrier exists precisely so the child re-registers the
            // protocol), so a name collision here is an expected same-aggregate
            // re-walk, not an error. The TypeEnv is the authoritative collision
            // check for aggregate *types*; this loop only mints derived functions
            // from already-registered types, so re-minting the identical ctor is safe.
            sym.function_entry(ctor_name)
                .or_insert_with(|| Arc::new(ctor_func));

            // Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — A1: `T'` inherits
            // T's own `:restricted-to` whitelist. `(:T' v1 v2 …)` is a directly-callable
            // constructor in its own right (the kwargs macro `(:T :field v …)` is the OTHER
            // ctor route, and it names `T` itself so the mention rule already enforces it) —
            // without this, a `:user::` caller reaches a restricted type's constructor
            // through the prime name with NO gate at all. Registered unconditionally
            // whenever `agg.restrictions` is `Some` — `contract_03_defstruct_with_field_metadata`
            // proved this must not be gated on "the whitelist is non-empty": `[]` means
            // "nobody", and `T'` must be exactly as unconstructable as `T` itself.
            if let Some(restrictions) = &agg.restrictions {
                let ctor_ast = restrictions_to_binding_metadata_ast(&restrictions.ctor_whitelist);
                sym.binding_metadata
                    .entry(format!("{}'", agg.name))
                    .or_default()
                    .insert(":restricted-to".to_string(), ctor_ast);
            }
        }

        // Emit ONE accessor per OWN field with the correct absolute index.
        //
        // Arc 293.R2.2 — class-safety strategy:
        //   Struct: `struct-field self idx` — type system already enforces the specific type;
        //     no runtime class check needed (accessor param type IS the struct type).
        //   Generic Record/HolonRecord: `struct-field self idx` — type system enforces it via
        //     the parametric type at each call site (accessor param type = specific generic type).
        //   Non-generic Record/HolonRecord: class-checked `Record/field-at` body — the accessor
        //     param type is `:wat::core::Record` (backward compat), so the type checker allows ANY
        //     record. The runtime check is the only guard; it mirrors the old macro accessor body
        //     that was removed from wat/Record.wat.
        let use_class_check = matches!(
            agg.nature,
            crate::types::Nature::Record | crate::types::Nature::HolonRecord
        ) && agg.type_params.is_empty();

        for (own_idx, (field_name, field_type)) in agg.fields.iter().enumerate() {
            let accessor_path = format!("{}/{}", agg.name, field_name);

            // Class-no-colon: what `(type self)` returns for this aggregate at runtime.
            // `declared_type_name()` for Aggregate returns `a.class.clone()` which is
            // colon-free (e.g., `"myapp::Voltage"`). `agg.name` = `":myapp::Voltage"`.
            let class_no_colon = agg.name.trim_start_matches(':').to_string();

            let accessor_body = if use_class_check {
                // Build:
                //   (:wat::core::Record/field-at
                //     (:wat::core::Option/expect
                //       (:wat::core::if
                //         (:wat::core::= (:wat::core::type self) "<class-no-colon>")
                //         -> (:wat::core::Option :- [:wat::core::Record])
                //         (:wat::core::Some self)
                //         :wat::core::None)
                //       (:wat::string::concat "<msg-prefix>" (:wat::core::type self)))
                //     <own_idx>)
                let msg_prefix = format!(
                    "{}/{}: expected receiver of class {}, got class :",
                    agg.name, field_name, agg.name
                );
                WatAST::List(
                    vec![
                        WatAST::Keyword(
                            ":wat::core::Record/field-at".into(),
                            crate::rust_caller_span!(),
                        ),
                        WatAST::List(
                            vec![
                                WatAST::Keyword(
                                    ":wat::core::Option/expect".into(),
                                    crate::rust_caller_span!(),
                                ),
                                // if-form: (if cond -> :Type then else)
                                WatAST::List(
                                    vec![
                                        WatAST::Keyword(
                                            ":wat::core::if".into(),
                                            crate::rust_caller_span!(),
                                        ),
                                        // condition: (= (type self) "class-no-colon")
                                        WatAST::List(
                                            vec![
                                                WatAST::Keyword(
                                                    ":wat::core::=".into(),
                                                    crate::rust_caller_span!(),
                                                ),
                                                WatAST::List(
                                                    vec![
                                                        WatAST::Keyword(
                                                            ":wat::core::type".into(),
                                                            crate::rust_caller_span!(),
                                                        ),
                                                        WatAST::Symbol(
                                                            crate::scope::Identifier::bare("self"),
                                                            crate::rust_caller_span!(),
                                                        ),
                                                    ],
                                                    crate::rust_caller_span!(),
                                                ),
                                                WatAST::StringLit(
                                                    class_no_colon,
                                                    crate::rust_caller_span!(),
                                                ),
                                            ],
                                            crate::rust_caller_span!(),
                                        ),
                                        // Arc 258.4 — bare if: (if cond then else); type inferred from the branches.
                                        // then: (Some self)
                                        WatAST::List(
                                            vec![
                                                WatAST::Keyword(
                                                    ":wat::core::Some".into(),
                                                    crate::rust_caller_span!(),
                                                ),
                                                WatAST::Symbol(
                                                    crate::scope::Identifier::bare("self"),
                                                    crate::rust_caller_span!(),
                                                ),
                                            ],
                                            crate::rust_caller_span!(),
                                        ),
                                        // else: :wat::core::None
                                        WatAST::Keyword(
                                            ":wat::core::None".into(),
                                            crate::rust_caller_span!(),
                                        ),
                                    ],
                                    crate::rust_caller_span!(),
                                ),
                                // message: (string::concat msg_prefix (type self))
                                WatAST::List(
                                    vec![
                                        WatAST::Keyword(
                                            ":wat::string::concat".into(),
                                            crate::rust_caller_span!(),
                                        ),
                                        WatAST::StringLit(msg_prefix, crate::rust_caller_span!()),
                                        WatAST::List(
                                            vec![
                                                WatAST::Keyword(
                                                    ":wat::core::type".into(),
                                                    crate::rust_caller_span!(),
                                                ),
                                                WatAST::Symbol(
                                                    crate::scope::Identifier::bare("self"),
                                                    crate::rust_caller_span!(),
                                                ),
                                            ],
                                            crate::rust_caller_span!(),
                                        ),
                                    ],
                                    crate::rust_caller_span!(),
                                ),
                            ],
                            crate::rust_caller_span!(),
                        ),
                        WatAST::IntLit(own_idx as i64, crate::rust_caller_span!()),
                    ],
                    crate::rust_caller_span!(),
                )
            } else {
                // Struct or generic Record/HolonRecord: bare struct-field (type system enforces).
                WatAST::List(
                    vec![
                        WatAST::Keyword(
                            ":wat::core::struct-field".into(),
                            crate::rust_caller_span!(),
                        ),
                        WatAST::Symbol(
                            crate::scope::Identifier::bare("self"),
                            crate::rust_caller_span!(),
                        ),
                        WatAST::IntLit(own_idx as i64, crate::rust_caller_span!()),
                    ],
                    crate::rust_caller_span!(),
                )
            };
            let accessor_func = Function {
                name: Some(accessor_path.clone()),
                params: vec![crate::scope::Identifier::bare("self")],
                type_params: agg.type_params.clone(),
                param_types: vec![accessor_param_type.clone()],
                ret_type: field_type.clone(),
                rest_param: None,
                rest_param_type: None,
                body: FunctionBody::Wat(Arc::new(accessor_body)),
                closed_env: None,
                rete: None,
                synthesized_for: None,
            };
            // Arc 296 stone H-1c — route through THE ONE gate before minting.
            // This loop is the actual accessor registration site for every
            // Aggregate nature (Struct + Record + HolonRecord); the field name
            // came straight off `agg.fields` with no dot check anywhere upstream
            // of here. `preregister_struct_accessors_from_form` (the ONE other
            // accessor gate) only fires for `defstruct`/`structtype` forms
            // nested in a `do`/`let` body — a `defrecord`'s expansion
            // (`recordtype`) never reaches it, so this was the hole a dotted
            // record field slipped through. Privilege::Stdlib mirrors
            // `CheckEnv::from_symbols`'s identical precedent (env.rs ~151-158):
            // `types` was already vetted for Reserved/Unnamespaced at type
            // registration, so only the DottedName wall can still fire here —
            // and DottedName is held even against Stdlib, so the choice of
            // privilege changes nothing for that arm.
            let acc_existing = if sym.has_function(&accessor_path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &accessor_path,
                crate::resolve::Privilege::Stdlib,
                acc_existing,
                &crate::rust_caller_span!(),
                || -> Result<(), RuntimeError> {
                    sym.function_entry(accessor_path.clone())
                        .or_insert_with(|| Arc::new(accessor_func));
                    Ok(())
                },
            )?;
        }
    }
    Ok(())
}

/// Walk every `:wat::core::enum` declaration in `types` and synthesize
/// per-variant constructors into `sym`. Arc 048. Mirrors
/// [`register_struct_methods`]'s structure.
///
/// **What's synthesized, per enum `:my::ns::E` with variants:**
///
/// - **Unit variant `Variant`**: insert a pre-built [`EnumValue`]
///   into `sym.unit_variants` at keyword path `:my::ns::E::Variant`.
///   Eval's keyword arm checks this map before the function lookup,
///   so a bare keyword reference produces the variant value
///   directly (mirrors the `:None` shortcut for Option).
///
/// - **Tagged variant `Variant(f1: T1, ..., fn: Tn)`**: synthesize
///   a [`Function`] entry at keyword path `:my::ns::E::Variant` with:
///   - Params `f1, f2, ..., fn` (typed per declaration)
///   - Return type `:my::ns::E`
///   - Body `(:wat::core::enum-new :my::ns::E :Variant f1 f2 ... fn)`
///
///   Invocation `(:my::ns::E::Variant arg1 arg2)` dispatches to the
///   synthesized function, which evaluates the args and emits
///   `Value::Enum`.
///
/// Users never write either form — they invoke via the keyword path.
/// The checker picks up the synthesized functions through
/// [`crate::check::CheckEnv::from_symbols`] just like struct
/// constructors. Unit-variant typing is handled separately by the
/// checker's variant-keyword registry.
pub fn register_enum_methods(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::types::{EnumVariant, TypeDef};

    for (_name, def) in types.iter() {
        let enum_def = match def {
            TypeDef::Enum(e) => e,
            _ => continue,
        };

        // Arc 071 — parametric enums (e.g., `(WalkStep :- [A])`) need
        // their constructor return types to read `(:Enum :- [A B])`, not
        // bare `:Enum`. Without this the type checker sees the body
        // produce `:Enum` and rejects against a `(:Enum :- [i64])` signature.
        // The lab harness probe at experiment/099-walkstep-probe is
        // the regression case.
        let enum_type = parametric_decl_type(&enum_def.name, &enum_def.type_params);

        for variant in &enum_def.variants {
            match variant {
                EnumVariant::Unit(variant_name) => {
                    let key = format!("{}::{}", enum_def.name, variant_name);
                    if sym.has_unit_variant(&key) {
                        // arc 138: no span — synthesized enum unit-variant.
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::DuplicateDefine(key),
                        ));
                    }
                    if sym.has_function(&key) {
                        // arc 138: no span — synthesized enum unit-variant.
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::DuplicateDefine(key),
                        ));
                    }
                    sym.register_unit_variant(
                        key,
                        EnumValue {
                            type_path: enum_def.name.clone(),
                            variant_name: variant_name.clone(),
                            names: no_field_names(),
                            fields: Vec::new(),
                        },
                    );
                }
                EnumVariant::Tagged {
                    name: variant_name,
                    fields,
                } => {
                    let constructor_path = format!("{}::{}", enum_def.name, variant_name);
                    let param_names: Vec<crate::scope::Identifier> = fields
                        .iter()
                        .map(|(n, _)| crate::scope::Identifier::bare(n.clone()))
                        .collect();
                    let param_types: Vec<crate::types::TypeExpr> =
                        fields.iter().map(|(_, t)| t.clone()).collect();

                    // Body: (:wat::core::variant :enum-path :Variant p1 p2 ... pn)
                    let mut body_items = Vec::with_capacity(2 + fields.len());
                    body_items.push(WatAST::Keyword(
                        ":wat::core::variant".into(),
                        crate::rust_caller_span!(),
                    ));
                    body_items.push(WatAST::Keyword(
                        enum_def.name.clone(),
                        crate::rust_caller_span!(),
                    ));
                    body_items.push(WatAST::Keyword(
                        format!(":{}", variant_name),
                        crate::rust_caller_span!(),
                    ));
                    for param_name in &param_names {
                        // Arc 170 — REUSE the binder node.
                        body_items.push(WatAST::Symbol(
                            param_name.clone(),
                            crate::rust_caller_span!(),
                        ));
                    }

                    let func = Function {
                        name: Some(constructor_path.clone()),
                        params: param_names,
                        type_params: enum_def.type_params.clone(),
                        param_types,
                        ret_type: enum_type.clone(),
                        rest_param: None,
                        rest_param_type: None,
                        body: FunctionBody::Wat(Arc::new(WatAST::List(
                            body_items,
                            crate::rust_caller_span!(),
                        ))),
                        closed_env: None,
                        rete: None,
                        synthesized_for: None,
                    };
                    if sym.has_function(&constructor_path)
                        || sym.has_unit_variant(&constructor_path)
                    {
                        // arc 138: no span — synthesized enum tagged-variant.
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::DuplicateDefine(constructor_path),
                        ));
                    }
                    sym.register_function(constructor_path, Arc::new(func));
                }
            }
        }
    }
    Ok(())
}

/// Walk every `:wat::core::newtype` declaration in `types` and synthesize
/// a positional constructor + accessor into `sym`. Arc 049. Mirrors
/// [`register_struct_methods`] for arity-1 tuple structs — newtype's
/// Rust compilation per 058-030 line 538 IS `struct A(B);`, so the
/// natural representation is `Value::Aggregate(nature=Struct)` of arity 1 with the
/// inner value at index 0.
///
/// Per newtype `:my::ns::Price` with inner `:f64`:
///
/// - Constructor `:my::ns::Price` (bare — arc 293.R2.3, `/new` annihilated) —
///   Function `(:fn(:f64) -> :Price)`, body invokes `(:wat::core::struct-new :Price value)`.
/// - Accessor `:my::ns::Price/0` — Function `(:fn(:Price) -> :f64)`,
///   body invokes `(:wat::core::struct-field self 0)`. The `/0` name
///   mirrors Rust's `.0` tuple-struct positional access — embodying
///   the host language. No invented field name.
///
/// Atom hashing of newtype values gets nominal distinction for free
/// because `Value::Aggregate(Struct)` carries the class FQDN in its EDN encoding —
/// `(Atom (:Price 100.0))` and `(Atom 100.0)` produce different
/// vectors.
pub fn register_newtype_methods(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::scope::Identifier;
    use crate::types::TypeDef;

    for (_name, def) in types.iter() {
        let nt_def = match def {
            TypeDef::Newtype(n) => n,
            _ => continue,
        };

        let nt_type = crate::types::TypeExpr::Path(nt_def.name.clone());

        // Constructor — bare `<newtype>` (parity with records; arc 293.R2.3:
        // every type-name is its own constructor, `/new` annihilated).
        // Single param `value` of inner type. Body invokes
        // `:wat::core::struct-new` with the type-name keyword and the param.
        // Same shape as a struct of arity 1.
        let constructor_path = nt_def.name.clone();
        let new_body = WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::struct-new".into(), crate::rust_caller_span!()),
                WatAST::Keyword(nt_def.name.clone(), crate::rust_caller_span!()),
                WatAST::Symbol(Identifier::bare("value"), crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        );
        let new_func = Function {
            name: Some(constructor_path.clone()),
            params: vec![crate::scope::Identifier::bare("value")],
            type_params: nt_def.type_params.clone(),
            param_types: vec![nt_def.inner.clone()],
            ret_type: nt_type.clone(),
            rest_param: None,
            rest_param_type: None,
            body: FunctionBody::Wat(Arc::new(new_body)),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        };
        if sym.has_function(&constructor_path) {
            // arc 138: no span — synthesized newtype constructor.
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::DuplicateDefine(constructor_path),
            ));
        }
        sym.register_function(constructor_path, Arc::new(new_func));

        // Accessor — `<newtype>/0`. Single param `self` of newtype.
        // Body invokes `:wat::core::struct-field self 0`. The `/0`
        // accessor mirrors Rust's `.0` for tuple structs.
        let accessor_path = format!("{}/0", nt_def.name);
        let accessor_body = WatAST::List(
            vec![
                WatAST::Keyword(
                    ":wat::core::struct-field".into(),
                    crate::rust_caller_span!(),
                ),
                WatAST::Symbol(
                    crate::scope::Identifier::bare("self"),
                    crate::rust_caller_span!(),
                ),
                WatAST::IntLit(0, crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        );
        let accessor_func = Function {
            name: Some(accessor_path.clone()),
            params: vec![crate::scope::Identifier::bare("self")],
            type_params: nt_def.type_params.clone(),
            param_types: vec![nt_type.clone()],
            ret_type: nt_def.inner.clone(),
            rest_param: None,
            rest_param_type: None,
            body: FunctionBody::Wat(Arc::new(accessor_body)),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        };
        if sym.has_function(&accessor_path) {
            // arc 138: no span — synthesized newtype accessor.
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::DuplicateDefine(accessor_path),
            ));
        }
        sym.register_function(accessor_path, Arc::new(accessor_func));
    }
    Ok(())
}

/// Arc 237 Stone 237.6 — auto-mint `is-<Name>?` membership predicates for every
/// non-Alias TypeDef registered in the TypeEnv (Struct / Enum / Newtype / Union).
///
/// Each synthesized predicate is a named convenience over the one mechanism
/// (`conforms?`):
///
/// - **name**: `:<ns>::is-<LastSegment>?` — derived from the FQDN by splitting
///   on `::`, taking the last segment, prepending `is-`, appending `?`, and
///   rejoining with the namespace prefix. E.g. `:my::Shape` → `:my::is-Shape?`.
/// - **params**: `[v]`; **type_params**: `["T"]`; **param_types**: `[TypeExpr::Path("T")]`
///   — the type variable `T` makes the predicate accept any value (∀T); the checker
///   instantiates a fresh Var at each call site.
/// - **body**: `(:wat::core::conforms? v :<FQDN>)` — a `WatAST::List` composing
///   the one mechanism rather than re-computing conformance independently.
/// - **ret**: `:wat::core::bool`.
///
/// Mirrors `register_struct_methods` (runtime.rs:2852). Called from `src/freeze.rs`
/// alongside `register_{struct,enum,newtype}_methods`. Skips `TypeDef::Alias` — typealias
/// names a type, it does not introduce one; `(conforms? v :Alias)` works directly.
pub fn register_type_predicates(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::scope::Identifier;
    use crate::types::{TypeDef, TypeExpr};

    for (_name, def) in types.iter() {
        // Skip Alias — it names a type, not introduces one; no predicate.
        // `agg_restrictions` is `Some` only for the Aggregate arm (arc 203's
        // `StructRestrictions` is Aggregate-specific) — carried alongside `fqdn` for A1's
        // predicate-whitelist propagation below.
        let (fqdn, agg_restrictions) = match def {
            // Arc 293.2b — Struct + Record collapsed into Aggregate.
            TypeDef::Aggregate(a) => (&a.name, a.restrictions.as_ref()),
            TypeDef::Enum(e) => (&e.name, None),
            TypeDef::Newtype(n) => (&n.name, None),
            TypeDef::Union(u) => (&u.name, None),
            // Arc 293.3-core — surface gets an is-<Name>? predicate (structural conformance).
            TypeDef::Surface(s) => (&s.name, None),
            TypeDef::Alias(_) => continue,
        };

        // Derive predicate name: split FQDN on "::", take last segment,
        // prepend "is-", append "?", rejoin namespace prefix with "::".
        // E.g. "my::Shape" → "my::is-Shape?", "ns::sub::Foo" → "ns::sub::is-Foo?".
        let stripped = fqdn.trim_start_matches(':');
        let predicate_name: String = if !stripped.contains("::") {
            // No namespace prefix — bare name (unusual but handled).
            format!(":is-{}?", stripped)
        } else {
            let base = wat_reader::identifier::leaf(stripped);
            let prefix = wat_reader::identifier::path(stripped);
            format!(":{}::is-{}?", prefix, base)
        };

        // Full FQDN keyword for the conforms? call body (with leading colon).
        let fqdn_kw = if fqdn.starts_with(':') {
            fqdn.clone()
        } else {
            format!(":{}", fqdn)
        };

        // Body: (:wat::core::conforms? v :<FQDN>)
        let body = WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::conforms?".into(), crate::rust_caller_span!()),
                WatAST::Symbol(Identifier::bare("v"), crate::rust_caller_span!()),
                WatAST::Keyword(fqdn_kw.clone(), crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        );

        let pred_func = Function {
            name: Some(predicate_name.clone()),
            params: vec![crate::scope::Identifier::bare("v")],
            // Fresh type param T — accepts any value (∀T); the checker
            // instantiates a fresh Var at each call site.
            type_params: vec!["T".into()],
            param_types: vec![TypeExpr::Path("T".into())],
            ret_type: TypeExpr::Path(":wat::core::bool".into()),
            rest_param: None,
            rest_param_type: None,
            body: FunctionBody::Wat(Arc::new(body)),
            closed_env: None,
            rete: None,
            // Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — B2. `is-T?`'s body
            // NAMES `fqdn_kw` (arg to `conforms?` above) by construction — that mention is
            // this fn's whole reason to exist, not a caller reaching for T. Owner-scoped so
            // `walk_for_restricted_call` exempts ONLY a mention of this exact FQDN; a mention
            // of any other restricted binding inside a future companion is still walked.
            synthesized_for: Some(fqdn_kw),
        };

        if sym.has_function(&predicate_name) {
            // Collision: a user-defined function already occupies this name.
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::DuplicateDefine(predicate_name),
            ));
        }

        // Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — A1: `is-T?` inherits
        // T's own `:restricted-to` whitelist, same reasoning as the `T'` ctor above.
        // Unconditional whenever the type carries restrictions (an empty whitelist
        // propagates too).
        if let Some(restrictions) = agg_restrictions {
            let pred_ast = restrictions_to_binding_metadata_ast(&restrictions.ctor_whitelist);
            sym.binding_metadata
                .entry(predicate_name.clone())
                .or_default()
                .insert(":restricted-to".to_string(), pred_ast);
        }

        sym.register_function(predicate_name, Arc::new(pred_func));
    }
    Ok(())
}

/// Arc 157 slice 1a-ii — evaluate top-level `def` forms in the program
/// residue and populate `sym.runtime_def_values` with the resulting values.
///
/// Called from `FrozenWorld::freeze` after all capability carriers are
/// installed on `symbols`. At that point `symbols` is still `mut`, so
/// this function can write to `runtime_def_values` directly.
///
/// Walks only splice-eligible positions (the same predicate enforced by
/// the type checker's position predicate):
/// - Direct `(:wat::core::def :name expr)` at top-level → evaluates `expr`
///   in `env`, inserts `name → value` into `sym.runtime_def_values`.
/// - `(:wat::core::do ...)` at top-level → recurse on each child.
/// - `(:wat::core::let (bindings) body)` at top-level → evaluate the
///   bindings and build a child environment, then recurse into the body
///   with that env. This enables the closure-capture case:
///   `(let [config 42] (def :get-config (fn [] config)))` — the fn
///   closure captures `config` from the let-env at freeze time.
/// - Everything else → evaluate for side effects (non-def top-level forms)
///   but don't write to `runtime_def_values`.
///
/// The `env` parameter carries any enclosing let-bindings. At the
/// outermost call (from `FrozenWorld::freeze`) this is `Environment::new()`.
///
/// Arc 278 #88 v2 — also THE ONE DOOR for the rete-defn definition-site check
/// (`crate::rete::purity::apply_rete_defn_contracts`). It moved here from `build_env`
/// (`freeze/env.rs`) because `register_runtime_defs` re-registers every `defn`-turned-`def`
/// — rebuilding a fresh `Function` with `rete: None` (see the `:wat::core::def` arm below) —
/// so a check that ran BEFORE this pass had its stamp silently dropped the moment this pass
/// touched the same name (DESIGN-STONE-the-rete-defn.md, "WHAT THE FIRST STRIKE LEARNED" §3).
///
/// ⚠ THE STAMP RUNS AFTER **EACH** TOP-LEVEL FORM, NOT ONCE AFTER THE WHOLE LOOP — this is
/// itself a correction, caught live on the SESSION path (`eval_form_against_defs`), not the
/// boot path, which is exactly why it hides at boot. A top-level `(:wat::core::let […] body)`
/// is registration-BEARING (`RUNTIME_DECLARATION_HEADS`) precisely so a closure-capture def
/// nested in its body can see the let's bindings — and that means the `:wat::core::let` arm
/// below EAGERLY EVALUATES its bindings as *registration*, not as the later "run the
/// contributed expression" phase. A live-session line like
/// `(let [rules (:wat::rete::collect-rules :usr) session (:wat::rete::compile rules)] …)`
/// therefore calls `:wat::rete::compile` DURING this very loop. Stamping only once at the
/// end meant every rete-defn registered EARLIER in `program` (by a PRIOR turn, re-registered
/// fresh with `rete: None` by THIS pass's own `:wat::core::def` arm) was still unstamped at
/// the moment that eager compile ran — a real, reproduced refusal
/// (`:wat::rete::compile-rule` naming a just-declared, law-A-clean helper as "not a rete
/// primitive") that never reproduces at boot, because a boot-time `let` almost always lives
/// inside a `defn` body (evaluated on invocation, long after freeze) rather than bare at
/// top level. Stamping per-form is safe to call repeatedly: a name not yet in `sym.functions`
/// is skipped (`apply_rete_defn_contracts`'s own guard), an already-stamped name is
/// re-verified identically (idempotent), and the whole-group `seen` seeding (stone §2) is
/// unaffected — it seeds from `declared_rete_defns` itself, not from what happens to be
/// registered yet.
///
/// Both callers of this fn — the boot path (`FrozenWorld::freeze`) and the live-session path
/// (`eval_form_against_defs`) — get it for free, identically, with no second implementation
/// (STOP-3).
///
/// `declared_rete_defns` names which of `program`'s registrations to check + stamp — see
/// `freeze::env::extract_rete_defn_names` / `FrozenWorld::declared_rete_defns`.
pub fn register_runtime_defs(
    program: &[WatAST],
    env: &Environment,
    sym: &mut SymbolTable,
    declared_rete_defns: &std::collections::HashSet<String>,
) -> Result<(), EvalBreak> {
    for form in program {
        register_runtime_defs_form(form, env, sym)?;
        match crate::rete::purity::apply_rete_defn_contracts(sym, declared_rete_defns) {
            crate::rete::purity::ReteDefnCheckOutcome::Ok => {}
            crate::rete::purity::ReteDefnCheckOutcome::Err(err) => match err.kind {
                crate::rete::purity::ReteDefnCheckErrorKind::AxisViolation { name, axis, head } => {
                    return Err(RuntimeError::new(
                        err.span,
                        RuntimeErrorKind::ReteDefnAxisViolation { name, axis, head },
                    )
                    .into());
                }
                crate::rete::purity::ReteDefnCheckErrorKind::Recursive { name, head } => {
                    return Err(RuntimeError::new(
                        err.span,
                        RuntimeErrorKind::ReteDefnRecursive { name, head },
                    )
                    .into());
                }
            },
        }
    }
    Ok(())
}

/// Recursive helper for [`register_runtime_defs`]. Processes a single form.
///
/// Arc 255 Stone 1a-ε — the OTHER of TWO `role = declare` pointers for
/// `:wat::config::set-redef!` AND `:wat::config::set-eval-redef!` (STOP-3: measured, not one
/// honest primary — see `collect_entry_file_inner`'s own pointer, `src/config.rs`, for the
/// leading-position half). This fn is the LATER-position processor: both setter heads are
/// listed in `RUNTIME_DECLARATION_HEADS` (`src/declare/parse.rs:136`), so a setter that is NOT
/// the entry file's leading form — legal, if unusual — falls through
/// `collect_entry_file_inner`'s leading-setter scan untouched and is processed here instead,
/// mutating `sym.redef_allowed`/`sym.eval_redef_allowed` directly. Measured with a probe
/// (`wat-scripts/scratch-pad/1a-epsilon-probe/probe-nonleading-setredef.wat`): a `set-redef!`
/// placed after an earlier top-level `defn` is accepted and the program runs to completion.
#[wat_special_form_impl(":wat::config::set-redef!", role = declare)]
#[wat_special_form_impl(":wat::config::set-eval-redef!", role = declare)]
fn register_runtime_defs_form(
    form: &WatAST,
    env: &Environment,
    sym: &mut SymbolTable,
) -> Result<(), EvalBreak> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(()), // non-list top-level forms are not def-splice positions
    };
    if items.is_empty() {
        return Ok(());
    }
    let head = match &items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => return Ok(()),
    };

    // Arc 170 — the gate. Every head the match below handles must be listed in
    // RUNTIME_DECLARATION_HEADS, so `is_runtime_declaration_head` is never a second
    // opinion about what a declaration is: it is the SAME question, asked earlier.
    if !is_runtime_declaration_head(head) {
        return Ok(());
    }

    match head {
        // Arc 157 slice 1a-ii — config setters. Update the SymbolTable
        // carrier flags so subsequent def-processing in this freeze pass
        // sees the correct redef_allowed / eval_redef_allowed state.
        // Shape: (:wat::config::set-redef! <bool-literal>)
        ":wat::config::set-redef!" => {
            if items.len() == 2 {
                if let WatAST::BoolLit(b, _) = &items[1] {
                    sym.redef_allowed = *b;
                } // malformed; check already caught it
            }
        }
        // Shape: (:wat::config::set-eval-redef! <bool-literal>)
        ":wat::config::set-eval-redef!" => {
            if items.len() == 2 {
                if let WatAST::BoolLit(b, _) = &items[1] {
                    sym.eval_redef_allowed = *b;
                } // malformed; check already caught it
            }
        }
        // Stone 241.14 — `:wat::core::def-restricted` runtime arm DELETED.
        // def-restricted is HARD CUT at check.rs; no form reaches runtime eval.
        ":wat::core::def" => {
            // Shape: (:wat::core::def :name expr)               — 3 items, no metadata
            //        (:wat::core::def :name {metadata} expr)    — 4 items, Stone 241.6
            // The type checker already validated shape + position; here we
            // trust the form is well-formed (same guarantee as the runtime
            // eval arm).
            if items.len() != 3 && items.len() != 4 {
                return Ok(()); // malformed; type checker already caught it
            }
            let name = match &items[1] {
                WatAST::Keyword(k, _) => k.clone(),
                _ => return Ok(()), // malformed; type checker already caught it
            };
            // Stone 241.6 — discriminate: if 4 items, items[2] is the
            // metadata-map and items[3] is the expr. If 3 items, items[2] is
            // the expr directly.
            let (expr, metadata_opt) = if items.len() == 4 {
                // Stone 241.7 — for non-fn defs, try_parse_fn_shape_def returns None
                // (value is not a fn-form), so register_defines never stores the
                // metadata for these bindings. Store it here, at runtime-def time,
                // so metadata-of can read it regardless of whether the value is a fn.
                let meta = try_parse_metadata_map(&items[2]);
                (&items[3], meta)
            } else {
                (&items[2], None)
            };
            // Arc 157 slice 1a-ii — redef gating at freeze time.
            // If the name is already in runtime_def_values and redef_allowed
            // is false, this is a redef violation. The type checker already
            // emitted DefRedefForbidden; here we simply skip to avoid
            // overwriting the prior value (the program may still partially
            // execute to surface other errors).
            if sym.has_def_value(&name) && !sym.redef_allowed {
                return Ok(()); // redef rejected; type checker already caught it
            }
            // Evaluate the expr in the current env (which carries any
            // enclosing let-bindings from a splice-eligible let wrapper).
            // sym must be passed as immutable here for eval; the write
            // to runtime_def_values happens after.
            let sym_ref: &SymbolTable = sym;
            let value = eval_inner(expr, env, sym_ref)?.value_owned();
            // Arc 170 Gap D — if the evaluated value is a fn (possibly with
            // a `closed_env` captured from enclosing let-bindings), also
            // update `sym.functions` with the properly-evaluated fn. This
            // overwrites any pre-registered stub (from `preregister_fn_defs_in_let`)
            // that carried `closed_env: None`. Without this update, `eval_tail`
            // dispatches through `sym.functions` (the stub) and loses the closure.
            //
            // Stone 241.11 — preserve the `name` field from the def's name keyword.
            // `eval_fn` creates Functions with `name: None`; the `def` form's name
            // keyword is the authoritative name. Set `func.name = Some(name)` so that
            // closure extraction (`function_to_define_form`) can reconstruct the
            // correct defn form with the canonical name instead of `__anon`.
            if let Value::wat__core__fn(ref func) = value {
                let named_func = if func.name.is_none() {
                    Arc::new(Function {
                        name: Some(name.clone()),
                        params: func.params.clone(),
                        type_params: func.type_params.clone(),
                        param_types: func.param_types.clone(),
                        ret_type: func.ret_type.clone(),
                        rest_param: func.rest_param.clone(),
                        rest_param_type: func.rest_param_type.clone(),
                        body: func.body.clone(),
                        closed_env: func.closed_env.clone(),
                        rete: None,
                        synthesized_for: None,
                    })
                } else {
                    func.clone()
                };
                sym.register_function(name.clone(), named_func);
            }
            // Stone 241.7 — store metadata for non-fn defs. fn-shape defs are
            // handled earlier by try_parse_fn_shape_def → register_defines. Non-fn
            // defs (value is a literal, struct, etc.) reach only this arm; store
            // their metadata here so metadata-of can read binding_metadata uniformly.
            if let Some(meta) = metadata_opt {
                if !meta.is_empty() {
                    record_binding_metadata(sym, name.clone(), meta, form.span())?;
                }
            }
            sym.register_def_value(name, value);
        }
        ":wat::core::do" => {
            // Splice: each child is a potential def position.
            for child in &items[1..] {
                register_runtime_defs_form(child, env, sym)?;
            }
        }
        ":wat::core::let" => {
            // Splice: evaluate the bindings in order, build a child env,
            // then recurse into each body form with the richer env.
            // Shape: (:wat::core::let [binder expr binder expr ...] body-1 body-2 ...)
            //
            // Arc 168 slice 1 — body becomes 1+ trailing forms (implicit-do).
            // Arc 168 slice 1 — bindings are WatAST::Vector flat-shape only;
            // eval_let emits MalformedForm on non-Vector. This registrar
            // mirrors that discipline — Vector-only. (Slice 3 retired the
            // walker + legacy outer-List parser arms.)
            if items.len() < 2 {
                return Ok(()); // malformed; type checker already caught it
            }
            let bindings_form = &items[1];

            // Build the child env from the bindings. Mirror eval_let's
            // sequential-binding logic: each binding is evaluated in the
            // env accumulated so far, then extends it.
            let mut scope = env.clone();

            // Vector outer with alternating (binder, expr) chunks. Binder
            // is a bare Symbol (canonical post-arc-159). Destructure
            // binders (Vector of Symbols) skip splice-time env extension —
            // destructure binding for closure capture would require
            // tuple-eval; the def-splice-into-let-body case load-bearing
            // here always uses single-name Symbol binders.
            let vector_items = match bindings_form {
                WatAST::Vector(items_v, _) => items_v,
                _ => return Ok(()), // legacy List outer rejected; walker fires
            };

            let mut i = 0;
            while i + 1 < vector_items.len() {
                let binder = &vector_items[i];
                let rhs = &vector_items[i + 1];
                if let WatAST::Symbol(ident, _) = binder {
                    let binding_name = crate::scope::env_key(ident);
                    let sym_ref: &SymbolTable = sym;
                    let tv = eval_inner(rhs, &scope, sym_ref)?;
                    scope = scope.child().bind_unknown_span(binding_name, tv).build();
                }
                // Non-Symbol binder (Vector destructure) — skip env extension;
                // not load-bearing for def-splice-into-let-body.
                i += 2;
            }

            // Recurse into each body form. Arc 168 multi-form body: any
            // body form may be a def position; iterate all of them.
            for body_form in &items[2..] {
                register_runtime_defs_form(body_form, &scope, sym)?;
            }
        }
        // Stone 237.2 — `:wat::core::defclause` at top-level.
        // Shape: (:wat::core::defclause :name [-> :T] (clause...) ...)
        // Parse + produce Value::wat__core__clauses; register in runtime_def_values.
        // Stone 237.3: also remove the resolver-stub from sym.functions (if present)
        // so dispatch falls through to runtime_def_values and picks up the real
        // ClauseSet rather than the 0-param stub.
        //
        // Arc 170 #13 — the ONE door (`register_defclause`, Runtime phase). This
        // is also the EVAL-TIME path (a defclause typed at the REPL reaches
        // registration only here, with no prior Stub-phase call), so the door's
        // metadata insert being unconditional on phase is what makes a
        // REPL-defined `{:restricted-to […]}` defclause bind its metadata at
        // all — previously this site stored only `runtime_def_values` and
        // dropped the metadata-map.
        ":wat::core::defclause" => {
            register_defclause(
                form,
                crate::resolve::Privilege::User,
                ClauseRegPhase::Runtime,
                sym,
            )?;
        }
        // Arc 232 Stone 232.1 — `:wat::core::extend-type` at top-level.
        // Shape: (:wat::core::extend-type :T :P (method-impl ...) ...)
        // Arc 293.4c — branch on surface vs. protocol target:
        //   Surface: register each impl as a `:<T>/<method>` Function in sym.functions
        //            (collision = DuplicateDefine).
        //   Protocol: keep existing behavior (store extend_def in runtime_def_values).
        // Arc 278 BRIEF-STONE-extend-user-checked — the surface path's REAL registration
        // (with the sig inherited from the surface, not nil placeholders) already ran at
        // build_env step 7.7 (env.rs), BEFORE `check_program`'s body-check sweep
        // (check.rs:826), via the shared `register_extend_type_surface_impls`. This is
        // freeze step 9's re-walk of the SAME residue forms — call the same shared
        // routine idempotently (skip_if_present=true): an already-present key here is
        // step 7.7's own prior registration of this exact form, not a fresh collision (a
        // genuine duplicate extend-type was already rejected at step 7.7). The protocol
        // arm is unchanged.
        ":wat::core::extend-type" => {
            let (canonical_key, ed) = parse_extend_type_form(form)?;
            let is_surface = sym
                .types_deref()
                .and_then(|t| t.get(&ed.protocol_name))
                .map(|td| matches!(td, crate::types::TypeDef::Surface(_)))
                .unwrap_or(false);
            if is_surface {
                register_extend_type_surface_impls(form, sym, /*skip_if_present=*/ true)?;
            } else {
                // Protocol path: keep existing behavior.
                let value = Value::wat__core__extend_def(ed);
                sym.register_def_value(canonical_key, value);
            }
        }
        // Type-relationship. The edge is registered at check/freeze; here
        // it is only a declaration so a `do` of companions (defservice)
        // classifies as Declared rather than Evaluated-nil.
        ":wat::core::derive" => {}
        _ => {
            // Non-splice top-level form (define, struct, enum, etc.) —
            // not a def-eligible position. No action needed.
        }
    }
    Ok(())
}

/// Stone 241.12 — register a `(:wat::core::defalias :alias :target)` form into `sym`.
///
/// Alias registration strategy:
///
/// 1. **User-defined target** (`sym.functions` contains the target):
///    Create a new Arc<Function> with the alias name but the same params, types, and body
///    as the target. The body delegates to the target by already containing the right call.
///    More precisely: for a user-defined fn, the body IS the fn body — but we want the alias
///    to CALL the target, not duplicate it. We use a body that calls `(target args...)`.
///
/// 2. **Builtin target** (in `CheckEnv::with_builtins()` but not in `sym.functions`):
///    Look up the TypeScheme. Synthesize synthetic param names (`_p0`, `_p1`, ...).
///    Create a body `(target _p0 _p1 ...)`. This mirrors what the old define-alias macro did.
///
/// 3. **Unknown target** (not found anywhere): Register a stub with an empty body.
///    The checker will surface an UnresolvedReference error at call time. This is the
///    honest failure path.
///
/// For the reserved-prefix gate: caller passes `check_reserved` = true for user code,
/// false for stdlib (which is privileged).
fn register_defalias(
    alias: &str,
    target: &str,
    sym: &mut SymbolTable,
    span: Span,
    privilege: crate::resolve::Privilege,
) -> Result<(), RuntimeError> {
    // Phase-1 migration to the ONE gate (resolve::registration). check_reserved maps to
    // Privilege; present -> NoOp (idempotent skip). A Duplicate can't arise (Existing is
    // only Absent|Equivalent — this path doesn't compare definitions).
    let existing = if sym.has_function(alias) {
        crate::resolve::Existing::Equivalent
    } else {
        crate::resolve::Existing::Absent
    };
    crate::resolve::register(
        alias,
        privilege,
        existing,
        &span,
        || -> Result<(), RuntimeError> {
            // Case 1: target is a user-defined function already in sym.functions.
            if let Some(target_fn) = sym.get(target) {
                // Create a delegating Function whose body calls `(target params...)`.
                // This mirrors what the old define-alias macro produced: a new define whose
                // signature copies the target's params and return type, and whose body calls
                // `(target p0 p1 ...)`.
                let target_fn = Arc::clone(target_fn);
                let body = build_delegate_body(
                    target,
                    &target_fn.params,
                    target_fn.rest_param.as_deref(),
                    span.clone(),
                );
                let alias_fn = Arc::new(Function {
                    name: Some(alias.to_string()),
                    params: target_fn.params.clone(),
                    type_params: target_fn.type_params.clone(),
                    param_types: target_fn.param_types.clone(),
                    ret_type: target_fn.ret_type.clone(),
                    rest_param: target_fn.rest_param.clone(),
                    rest_param_type: target_fn.rest_param_type.clone(),
                    body: FunctionBody::Wat(Arc::new(body)),
                    closed_env: None,
                    rete: None,
                    synthesized_for: None,
                });
                sym.register_function(alias.to_string(), alias_fn);
                return Ok(());
            }

            // Case 2: target is a substrate primitive (in CheckEnv::with_builtins_and_types).
            // Stone 243.3.1 — with_builtins() removed; caller binds TypeEnv first.
            let _builtin_types = crate::types::TypeEnv::with_builtins();
            let builtin_env = crate::check::CheckEnv::with_builtins_and_types(&_builtin_types);
            if let Some(scheme) = builtin_env.get(target) {
                // Synthesize param names _p0, _p1, ... from scheme.params.
                let param_names: Vec<crate::scope::Identifier> = scheme
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, _)| crate::scope::Identifier::bare(format!("_p{}", i)))
                    .collect();
                let rest_param = scheme.rest_param_type.as_ref().map(|_| "_rest".to_string());
                let body =
                    build_delegate_body(target, &param_names, rest_param.as_deref(), span.clone());
                let alias_fn = Arc::new(Function {
                    name: Some(alias.to_string()),
                    params: param_names,
                    type_params: scheme.type_params.clone(),
                    param_types: scheme.params.clone(),
                    ret_type: scheme.ret.clone(),
                    rest_param,
                    rest_param_type: scheme.rest_param_type.clone(),
                    body: FunctionBody::Wat(Arc::new(body)),
                    closed_env: None,
                    rete: None,
                    synthesized_for: None,
                });
                sym.register_function(alias.to_string(), alias_fn);
                return Ok(());
            }

            // Case 3: unknown target — register a minimal stub so the alias name is
            // "known" at check time, but the UnresolvedReference will surface at the
            // first actual call-site. The target itself will also surface as an error.
            // Arc 244 — use NilLit (canonical nil value literal) not Keyword.
            let stub_body = WatAST::NilLit(span.clone());
            let stub_fn = Arc::new(Function {
                name: Some(alias.to_string()),
                params: vec![],
                type_params: vec![],
                param_types: vec![],
                ret_type: crate::types::TypeExpr::Tuple(vec![]),
                rest_param: None,
                rest_param_type: None,
                body: FunctionBody::Wat(Arc::new(stub_body)),
                closed_env: None,
                rete: None,
                synthesized_for: None,
            });
            sym.register_function(alias.to_string(), stub_fn);
            Ok(())
        },
    )?;
    Ok(())
}

/// Stone 241.12 — build a delegate call body `(target p0 p1 ... & rest)`.
///
/// Used by `register_defalias` to synthesize the body of the alias Function.
fn build_delegate_body(
    target: &str,
    params: &[crate::scope::Identifier],
    rest_param: Option<&str>,
    span: Span,
) -> WatAST {
    let mut items: Vec<WatAST> =
        Vec::with_capacity(1 + params.len() + rest_param.is_some() as usize);
    items.push(WatAST::Keyword(target.to_string(), span.clone()));
    for p in params {
        // Arc 170 — REUSE the binder node, scopes included.
        items.push(WatAST::Symbol(p.clone(), span.clone()));
    }
    if let Some(rest) = rest_param {
        // Splice rest param into the call — the runtime spreads rest args via
        // the variadic call path when the last positional is missing but rest is present.
        // Emit as a plain symbol reference; the eval loop handles rest-arg forwarding.
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare(rest),
            span.clone(),
        ));
    }
    WatAST::List(items, span)
}

