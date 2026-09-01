//! Arc 109 Stone 2 — the declare home's PREREGISTER phase.
//!
//! Split by PHASE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-declare-home.md`). `preregister_*` runs
//! BEFORE `register.rs`'s pass — a stub-before-bodies step so the resolver (which runs between
//! the two) can see forward/recursive references to a name whose real body hasn't registered yet.
//! Moved verbatim out of `src/runtime.rs` (arc 109 Stone 2). Behaviour is unchanged; only the
//! location moved.
//!
//! `register_defclause` / `preregister_stdlib_defclause_stub` are a named practitioner's-call in
//! the DESIGN doc (lifecycle vs. feature grouping); they ship split across `register.rs` /
//! `preregister.rs` here, by lifecycle, matching where they already sat.
//!
//! Siblings: `register.rs` (populate the SymbolTable), `parse.rs` (read a declaration form's
//! shape), `typevar.rs` (free/bound type-variable walking).

use std::sync::Arc;

use crate::ast::WatAST;
use crate::value::{EvalBreak, Function, FunctionBody, RuntimeError, SymbolTable};

// `ClauseRegPhase` is genuinely defined in `crate::runtime`, not a facade re-export of a
// `crate::value` type — see STOP-1.
use crate::runtime::ClauseRegPhase;

use crate::declare::parse::{
    is_enum_form, is_struct_form, parse_declare_acronyms_form, try_parse_fn_shape_def,
};
use crate::declare::register::{record_binding_metadata, register_defclause};

/// Stone 237.8b — pre-register a stdlib defclause form as a minimal stub Function
/// into `sym.functions`. Required so the type checker (`CheckEnv::from_symbols`)
/// can find the defclause name and dispatch call sites through the defclause
/// inference path rather than erroring with UnknownCallee.
///
/// Stdlib defclauses live under `:wat::core::*` (reserved prefix) and are
/// excluded from the user-side `preregister_defclause_in_env` path which guards
/// against reserved-prefix pollution. Uses allow_reserved=true for stdlib-only.
///
/// The stub has 0 params and unit return type — same shape as the user-side stubs
/// in `register_defines`. The real ClauseSet lands in `runtime_def_values` via
/// `register_runtime_defs` at freeze time.
///
/// Arc 170 #13 — delegates to the ONE door (`register_defclause`, Stub phase).
/// This loop runs over EVERY stdlib residue form, not just defclauses (see the
/// call site in `freeze/env.rs`), so a parse failure here (non-defclause form,
/// or a genuinely malformed defclause) is swallowed exactly as it always was —
/// `register_stdlib_runtime_defs` (the Runtime phase, which runs later and IS
/// `?`-propagating) is where a malformed stdlib defclause actually surfaces.
pub fn preregister_stdlib_defclause_stub(form: &WatAST, sym: &mut SymbolTable) {
    let _ = register_defclause(
        form,
        crate::resolve::Privilege::Stdlib,
        ClauseRegPhase::Stub,
        sym,
    );
}

/// Arc 265 — pre-register `declare-acronyms` forms into `sym.acronym_registry`
/// BEFORE the macro-expansion pass so a `defservice` macro expanding later can
/// consult the registry via `pascal->kebab-in` at expand time.
///
/// Mirrors `preregister_protocol_names` (the 232.3 pattern) — a single
/// pre-pass that covers ONLY `declare-acronyms` forms. The full (no-op)
/// runtime eval of these forms happens in `register_runtime_defs`; this
/// pre-pass is the ORDERING guarantee.
pub fn preregister_acronyms(residue: &[WatAST], sym: &mut SymbolTable) -> Result<(), EvalBreak> {
    for form in residue {
        let items = match form {
            WatAST::List(items, _) => items,
            _ => continue,
        };
        if items.is_empty() {
            continue;
        }
        match &items[0] {
            WatAST::Keyword(k, _) if k.as_str() == ":wat::string::declare-acronyms" => {
                if let Ok((ns, acronyms)) = parse_declare_acronyms_form(form) {
                    sym.acronym_registry.entry(ns).or_default().extend(acronyms);
                }
            }
            _ => continue,
        }
    }
    Ok(())
}

/// Arc 170 slice 3 Gap F-1 — pre-register accessor stubs for a struct form.
///
/// Called by `preregister_fn_defs_in_do` / `_in_let` when a
/// `(:wat::core::struct :Name (field1 :T1) ...)` form is found in a `do`/`let`
/// body. Extracts the type name and field names from the form and inserts
/// minimal stub `Function` entries into `sym.functions` for:
///   - `{name}` — the constructor (bare; arc 293.R2.3, `/new` annihilated)
///   - `{name}/{field}` for each field — the field accessors
///
/// The stubs have `closed_env: None` and unit return type — they exist only
/// so `resolve_references` (step 7) can validate call heads referencing them.
/// `register_struct_methods` (step 6a, after `register_defines` returns) will
/// insert the fully-typed, real `Function` entries, overwriting these stubs.
///
/// Shape of struct form:
///   items[0] = `:wat::core::struct` keyword
///   items[1] = `:TypeName` keyword (the struct's type name)
///   items[2..] = field declarations: `(field-name :FieldType)` lists
///
/// Malformed forms (missing name, non-keyword name) are silently skipped —
/// the type checker will diagnose them later. No error is returned.
fn preregister_struct_accessors_from_form(
    form: &WatAST,
    sym: &mut SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<(), RuntimeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(()),
    };
    // items[1] is the type name keyword. `<K,V>` is unexpressible (arc 109 ③'s wall,
    // `src/types.rs:4688`) — no keyword the reader can hand back ever carries a `<...>`
    // suffix, so `type_name` is already the base name; used directly, never stripped
    // (arc 109 "reap the twelve" — measured 0 calls carrying a type-head).
    let type_name = match items.get(1) {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(()), // malformed; type checker will catch it
    };
    // Stub Function — zero params, unit return type, unit body.
    // The resolver only checks presence in `sym.functions`; the body/types
    // are irrelevant at pre-registration time.
    let stub_body = Arc::new(WatAST::List(vec![], crate::rust_caller_span!()));
    let unit_type = crate::types::TypeExpr::Path(":()".into());

    // Constructor: bare `{type}` (arc 293.R2.3 — parity with records; `/new` annihilated)
    let constructor_path = type_name.to_string();
    // Phase-1 migration to the ONE gate (struct constructor). present -> NoOp (skip).
    let cons_existing = if sym.has_function(&constructor_path) {
        crate::resolve::Existing::Equivalent
    } else {
        crate::resolve::Existing::Absent
    };
    crate::resolve::register(
        &constructor_path,
        privilege,
        cons_existing,
        &form.span().clone(),
        || -> Result<(), RuntimeError> {
            sym.register_function(
                constructor_path.clone(),
                Arc::new(Function {
                    name: None,
                    params: Vec::new(),
                    type_params: Vec::new(),
                    param_types: Vec::new(),
                    ret_type: unit_type.clone(),
                    rest_param: None,
                    rest_param_type: None,
                    body: FunctionBody::Wat(stub_body.clone()),
                    closed_env: None,
                    rete: None,
                    synthesized_for: None,
                }),
            );
            Ok(())
        },
    )?;

    // Stone 241.8 — defstruct field-vector shape:
    //   items[0] = `:wat::core::defstruct`
    //   items[1] = `:TypeName`
    //   items[2] = either a metadata-map List (head :wat::core::HashMap) OR the field-vector
    //   items[3] = field-vector (only if items[2] is metadata; optional)
    //
    // Field-vector is WatAST::Vector with flat triples: field <- :T field <- :T ...
    // Field names are at positions 0, 3, 6, ... in the Vector's items.
    let field_vec_items = {
        let candidate = items.get(2);
        let field_vec = match candidate {
            Some(WatAST::Vector(fv, _)) => Some(fv.as_slice()),
            Some(WatAST::List(_, _)) => {
                // items[2] is metadata-map; field-vector is at items[3].
                match items.get(3) {
                    Some(WatAST::Vector(fv, _)) => Some(fv.as_slice()),
                    _ => None,
                }
            }
            _ => None,
        };
        field_vec
    };

    if let Some(fv) = field_vec_items {
        // Walk triples: field(0) <-(1) :T(2)  field(3) <-(4) :T(5) ...
        let mut idx = 0;
        while idx + 2 < fv.len() {
            let field_name = match &fv[idx] {
                WatAST::Symbol(ident, _) => ident.as_str(),
                _ => {
                    idx += 3;
                    continue;
                }
            };
            let accessor_path = format!("{}/{}", type_name, field_name);
            let acc_existing = if sym.has_function(&accessor_path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &accessor_path,
                privilege,
                acc_existing,
                &form.span().clone(),
                || -> Result<(), RuntimeError> {
                    sym.register_function(
                        accessor_path.clone(),
                        Arc::new(Function {
                            name: None,
                            params: Vec::new(),
                            type_params: Vec::new(),
                            param_types: Vec::new(),
                            ret_type: unit_type.clone(),
                            rest_param: None,
                            rest_param_type: None,
                            body: FunctionBody::Wat(stub_body.clone()),
                            closed_env: None,
                            rete: None,
                            synthesized_for: None,
                        }),
                    );
                    Ok(())
                },
            )?;
            idx += 3;
        }
    }
    Ok(())
}

/// Arc 170 slice 3 Gap F-1 — pre-register tagged-variant constructor stubs for a defenum form.
/// Stone 241.9 — updated from :wat::core::enum to :wat::core::defenum (HARD CUT).
///
/// Called by `preregister_fn_defs_in_do` / `_in_let` when a
/// `(:wat::core::defenum :Name :V1 :V2 [f <- :T] ...)` form is found in
/// a `do`/`let` body. Extracts the type name and variant names and inserts
/// minimal stub `Function` entries into `sym.functions` for every variant
/// (both unit AND tagged) at path `{name}::{VariantName}`.
///
/// Unit variants are normally registered in `sym.unit_variants` by
/// `register_enum_methods` (step 6.5). For the resolver's call-head check
/// (`is_resolvable_call_head` in resolve.rs) they must appear in `sym.functions`
/// when used as call heads — e.g. `(:my::E::None)`. Pre-registering all
/// variants as stubs in `sym.functions` satisfies the resolver. The real
/// registration (unit → `unit_variants`; tagged → `functions`) happens at
/// step 6.5 and overwrites/complements these stubs.
///
/// Shape of defenum form (positional + one-token look-ahead per FORM-COLLAPSE verdict D):
///   items[0] = `:wat::core::defenum` keyword
///   items[1] = `:TypeName` keyword
///   items[2] = OPTIONAL metadata-map (WatAST::List with :wat::core::HashMap head)
///   items[2..] or items[3..] = positional variant specs:
///     - bare keyword: `:NoOp` → unit variant (no following Vector)
///     - bare keyword + Vector: `:Push [value <- :T]` → tagged variant
///
/// Malformed forms are silently skipped — the type checker will catch them.
fn preregister_enum_constructors_from_form(
    form: &WatAST,
    sym: &mut SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<(), RuntimeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(()),
    };
    // items[1] is the type name keyword. `<K,V>` is unexpressible (arc 109 ③'s wall,
    // `src/types.rs:4688`) — no keyword the reader can hand back ever carries a `<...>`
    // suffix, so `type_name` is already the base name; used directly, never stripped
    // (arc 109 "reap the twelve" — measured 0 calls carrying a type-head).
    let type_name = match items.get(1) {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(()), // malformed; type checker will catch it
    };

    // Determine start index for variant items: skip optional metadata-map at items[2].
    // Arc 257 slice 1: is_metadata_map() accepts Map literal and legacy HashMap List.
    let variant_start = if items.get(2).map(|n| n.is_metadata_map()).unwrap_or(false) {
        3 // metadata-map present; variants start at items[3]
    } else {
        2 // no metadata-map; variants start at items[2]
    };

    let stub_body = Arc::new(WatAST::List(vec![], crate::rust_caller_span!()));
    let unit_type = crate::types::TypeExpr::Path(":()".into());

    let variant_items = items.get(variant_start..).unwrap_or(&[]);
    let mut vi = 0;
    while vi < variant_items.len() {
        // defenum grammar: positional keyword with one-token look-ahead.
        // Keyword `:VariantName` → variant name; peek next for unit vs tagged.
        let variant_name: &str = match &variant_items[vi] {
            WatAST::Keyword(k, _) => match k.strip_prefix(':') {
                Some(name) => name,
                None => {
                    vi += 1;
                    continue; // malformed; skip
                }
            },
            _ => {
                vi += 1;
                continue; // unexpected item; skip
            }
        };
        // Look-ahead: is the next item a Vector (tagged variant)?
        let is_tagged = matches!(variant_items.get(vi + 1), Some(WatAST::Vector(_, _)));

        let constructor_path = format!("{}::{}", type_name, variant_name);
        let cons_existing = if sym.has_function(&constructor_path) {
            crate::resolve::Existing::Equivalent
        } else {
            crate::resolve::Existing::Absent
        };
        crate::resolve::register(
            &constructor_path,
            privilege,
            cons_existing,
            &form.span().clone(),
            || -> Result<(), RuntimeError> {
                sym.register_function(
                    constructor_path.clone(),
                    Arc::new(Function {
                        name: None,
                        params: Vec::new(),
                        type_params: Vec::new(),
                        param_types: Vec::new(),
                        ret_type: unit_type.clone(),
                        rest_param: None,
                        rest_param_type: None,
                        body: FunctionBody::Wat(stub_body.clone()),
                        closed_env: None,
                        rete: None,
                        synthesized_for: None,
                    }),
                );
                Ok(())
            },
        )?;

        // Advance: consume keyword + optional Vector.
        vi += if is_tagged { 2 } else { 1 };
    }
    Ok(())
}

/// Arc 170 Gap C — pre-register fn-shape defs found inside a top-level
/// `(:wat::core::do ...)` into `sym.functions`.
///
/// `register_defines` calls this when it encounters a `do` form at top
/// level. The `do` form itself remains in `rest` (so `register_runtime_defs`
/// can evaluate it later); this helper only *peeks* inside to pre-register
/// any `(:wat::core::def :name (:wat::core::fn ...))` children into
/// `sym.functions` so `resolve_references` (which runs after
/// `register_defines`) can validate call heads that reference those names.
///
/// Recursion: nested `do` forms inside the outer `do` are also scanned
/// (e.g., a macro that emits `(do (do defn-a) defn-b)`).
///
/// `check_reserved_prefix`: pass `true` for user source (blocks `:wat::*`
/// and `:rust::*` names); pass `false` for stdlib source which is permitted
/// under those prefixes.
pub(crate) fn preregister_fn_defs_in_do(
    items: &[WatAST],
    sym: &mut SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<(), RuntimeError> {
    // items is the children of a do form — i.e. items[0] is the :wat::core::do
    // keyword; items[1..] are the body children.
    for child in &items[1..] {
        if let Some((path, func, metadata_opt)) = try_parse_fn_shape_def(child)? {
            let existing = if sym.has_function(&path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &path,
                privilege,
                existing,
                &child.span().clone(),
                || -> Result<(), RuntimeError> {
                    sym.register_function(path.clone(), func);
                    Ok(())
                },
            )?;
            if let Some(meta) = metadata_opt {
                record_binding_metadata(sym, path, meta, child.span())?;
            }
        // Stone 241.14 — def-restricted fn-shape arm DELETED from do-preregister.
        // def-restricted is HARD CUT; forms reaching this path are rejected
        // at check.rs before this pre-registration runs.
        // Stone 241.11 — is_define_form branch DELETED (define HARD CUT).
        } else if is_struct_form(child) {
            // Arc 170 slice 3 Gap F-1 — pre-register struct accessor stubs.
            // The form stays in `rest`; `register_struct_methods` (step 6a) will
            // overwrite these stubs with fully-typed Function entries after
            // `register_defines` returns.
            preregister_struct_accessors_from_form(child, sym, privilege)?;
        } else if is_enum_form(child) {
            // Arc 170 slice 3 Gap F-1 — pre-register enum variant constructor stubs.
            // The form stays in `rest`; `register_enum_methods` (step 6.5) will
            // insert the real unit_variants and tagged Function entries after
            // `register_defines` returns.
            preregister_enum_constructors_from_form(child, sym, privilege)?;
        } else if let WatAST::List(nested_items, _) = child {
            // Recurse into nested do forms.
            if matches!(
                nested_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::do"
            ) {
                preregister_fn_defs_in_do(nested_items, sym, privilege)?;
            }
        }
    }
    Ok(())
}

/// Pre-registers fn-shape `def` forms found in the body of a top-level
/// `(:wat::core::let bindings body...)` into `sym.functions`.
///
/// `register_defines` calls this when it encounters a `let` form at top
/// level. The `let` form itself remains in `rest` (so `register_runtime_defs`
/// can evaluate it later); this helper only *peeks* into the body (items[2..],
/// per arc 168 multi-form body) to pre-register any
/// `(:wat::core::def :name (:wat::core::fn ...))` children into `sym.functions`
/// so `resolve_references` (which runs after `register_defines`) can validate
/// call heads that reference those names.
///
/// Separate from `preregister_fn_defs_in_do` because `let` body starts at
/// `items[2..]` (after the keyword and the bindings vector) whereas `do` body
/// starts at `items[1..]` (after the keyword only).
///
/// `check_reserved_prefix`: pass `true` for user source (blocks `:wat::*`
/// and `:rust::*` names); pass `false` for stdlib source which is permitted
/// under those prefixes.
pub(crate) fn preregister_fn_defs_in_let(
    items: &[WatAST],
    sym: &mut SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<(), RuntimeError> {
    // items[0] = :wat::core::let keyword
    // items[1] = bindings vector
    // items[2..] = body forms (arc 168 multi-form body)
    for child in items.get(2..).unwrap_or(&[]) {
        if let Some((path, func, metadata_opt)) = try_parse_fn_shape_def(child)? {
            let existing = if sym.has_function(&path) {
                crate::resolve::Existing::Equivalent
            } else {
                crate::resolve::Existing::Absent
            };
            crate::resolve::register(
                &path,
                privilege,
                existing,
                &child.span().clone(),
                || -> Result<(), RuntimeError> {
                    sym.register_function(path.clone(), func);
                    Ok(())
                },
            )?;
            if let Some(meta) = metadata_opt {
                record_binding_metadata(sym, path, meta, child.span())?;
            }
        // Stone 241.14 — def-restricted fn-shape arm DELETED from let-preregister.
        // def-restricted is HARD CUT; forms reaching this path are rejected
        // at check.rs before this pre-registration runs.
        // Stone 241.11 — is_define_form branch DELETED (define HARD CUT).
        } else if is_struct_form(child) {
            // Arc 170 slice 3 Gap F-1 — pre-register struct accessor stubs.
            // Mirror of the `do` arm: the form stays in `rest`; `register_struct_methods`
            // (step 6a) overwrites these stubs with fully-typed Function entries.
            preregister_struct_accessors_from_form(child, sym, privilege)?;
        } else if is_enum_form(child) {
            // Arc 170 slice 3 Gap F-1 — pre-register enum variant constructor stubs.
            // Mirror of the `do` arm: the form stays in `rest`; `register_enum_methods`
            // (step 6.5) inserts real unit_variants and tagged Function entries.
            preregister_enum_constructors_from_form(child, sym, privilege)?;
        } else if let WatAST::List(nested_items, _) = child {
            // Recurse into nested let forms in the body.
            if matches!(
                nested_items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::let"
            ) {
                preregister_fn_defs_in_let(nested_items, sym, privilege)?;
            }
        }
    }
    Ok(())
}

