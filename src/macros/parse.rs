use crate::ast::WatAST;

use super::error::{MacroError, MacroErrorKind};
use super::registry::{MacroDef, MacroRegistry};

/// Walk `forms`, register every `(:wat::core::defmacro ...)` into
/// `registry`, and return the remaining forms in order.
pub fn register_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> super::ExpandBatch {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register(def, crate::resolve::Privilege::User)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant of [`register_defmacros`] that
/// bypasses the `:wat::std::*` reserved-prefix gate. Called by the
/// startup pipeline on the baked stdlib sources; user source still
/// goes through [`register_defmacros`] so mis-namespaced user
/// defmacros halt at startup.
pub fn register_stdlib_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> super::ExpandBatch {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register(def, crate::resolve::Privilege::Stdlib)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

pub(super) fn is_defmacro_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmacro")
    )
}

/// Parse `(:wat::core::defmacro :name::path [p <- :T ...] -> :Ret body)`.
///
/// Stone 241.17 — canonical Vector-of-triples shape mirroring defn (arc 166).
///
/// New shape (6 items):
///   items[0] = `:wat::core::defmacro` keyword (head)
///   items[1] = macro name keyword
///   items[2] = argspec Vector (`[name <- :T ...]`)
///   items[3] = `->` symbol
///   items[4] = return-type keyword
///   items[5] = body
///
/// Optional metadata-map shape (7 items):
///   items[0] = `:wat::core::defmacro` keyword (head)
///   items[1] = macro name keyword
///   items[2] = metadata map (`{...}`) — accepted in the form shape but NOT stored by
///              macro parse (the `_meta` binding is intentionally dropped; `MacroDef` carries
///              no metadata field). Metadata handling for defmacro forms, if any, lives
///              downstream of macro registration.
///   items[3] = argspec Vector
///   items[4] = `->` symbol
///   items[5] = return-type keyword
///   items[6] = body
///
/// HARD-CUT rejection (Stone 241.17): old 3-item paren-pair-with-type form emits
/// `MalformedDefmacro` with structured reason pointing at the canonical shape.
/// Per `feedback_hard_cut_admits_no_bypasses` — no compatibility shim.
///
/// `parse_defmacro_signature` DELETED (Stone 241.17). The canonical argspec parser
/// (`parse_argspec_triples`) is the sole argspec parser across fn/defn/defclause/defmacro.
pub(super) fn parse_defmacro_form(form: WatAST) -> Result<MacroDef, MacroError> {
    // Arc 278 — retain the declaration VERBATIM before destructuring. The
    // parts cannot rebuild it (params carry no types; no return type is
    // kept), and closure extraction must ship macros to forked children.
    let source_form = form.clone();
    let (items, list_span) = match form {
        WatAST::List(items, span) => (items, span),
        // All four call sites guard with `is_defmacro_form`, which requires WatAST::List.
        // If this arm fires, the caller violated the contract.
        // rune:coverage(unreachable) — `is_defmacro_form` requires `WatAST::List`; all four
        // callers (`register_defmacros`, `register_stdlib_defmacros` × 2) gate on it before
        // dispatch. A non-List reaching here means the caller violated the contract — the
        // panic IS the proof of the invariant.
        _ => unreachable!("parse_defmacro_form: all call sites guard with is_defmacro_form (List required)"),
    };

    // HARD-CUT: 3-item old paren-pair form is REJECTED (Stone 241.17).
    // Old form: (:wat::core::defmacro (:name (param :T) ... -> :Ret) body)
    // Per `feedback_hard_cut_admits_no_bypasses` — no shim; no backward compat path.
    if items.len() == 3 && matches!(items.get(1), Some(WatAST::List(_, _))) {
        return Err(MacroError {
            span: list_span,
            kind: MacroErrorKind::MalformedDefmacro {
                reason: "old defmacro signature shape (paren-pair-with-type) is retired (Stone 241.17); use canonical Vector-of-triples form: (:wat::core::defmacro :name [param <- :Type ...] -> :Ret body)".into(),
            },
        });
    }

    // Determine if metadata-map is present: 7 items vs 6 items.
    // 6-item canonical: head name argvec -> rettype body
    // 7-item with-metadata: head name meta argvec -> rettype body
    let (name_item, argvec_item, arrow_item, rettype_item, body_item) =
        match items.as_slice() {
            [_, name, argvec, arrow, rettype, body] => {
                // 6-item canonical shape: arity enforced by the pattern.
                (name.clone(), argvec.clone(), arrow.clone(), rettype.clone(), body.clone())
            }
            [_, name, _meta, argvec, arrow, rettype, body] => {
                // 7-item with-metadata: metadata-map stored by binding_metadata discipline; ignored in macro parse.
                (name.clone(), argvec.clone(), arrow.clone(), rettype.clone(), body.clone())
            }
            _ => {
                return Err(MacroError {
                    span: list_span,
                    kind: MacroErrorKind::MalformedDefmacro { reason: format!(
                        "expected (:wat::core::defmacro :name [arg <- :T ...] -> :Ret body) — 6 items (or 7 with metadata-map); got {} elements",
                        items.len()
                    ) },
                });
            }
        };

    // items[1] must be the macro name keyword.
    let name = match name_item {
        WatAST::Keyword(k, _) => k,
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "macro name (item 1) must be a keyword-path (e.g. `:my::macro`)".into() } });
        }
    };

    // items[2] (or items[3] with metadata) must be the argspec Vector.
    let (argvec_items, argvec_span) = match argvec_item {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "argspec must be a Vector `[name <- :T ...]`".into() } });
        }
    };

    // Arrow symbol `->` must follow argspec.
    if !arrow_item.is_bare_symbol("->") {
        return Err(MacroError { span: arrow_item.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "expected `->` symbol after argspec Vector".into() } });
    }

    // Return-type keyword.
    match &rettype_item {
        WatAST::Keyword(_, _) => {}
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "expected return-type keyword after `->`".into() } });
        }
    }

    // Route argspec through canonical parser — third major consumer after fn + defclause.
    // `allow_rest_binder: true` mirrors defclause (arc 174 / Stone 241.3/241.4).
    let spec = crate::argspec::parse_argspec_triples(
        &argvec_items,
        ":wat::core::defmacro",
        &argvec_span,
        crate::argspec::ParseOptions { allow_rest_binder: true },
    ).map_err(MacroError::from)?;

    // ENFORCE (arc 251.5 / 209) — a macro param binds unevaluated SYNTAX, so its declared
    // type is not free: a fixed param always binds a form (`:wat::WatAST`), a rest param a
    // sequence of forms (`(:wat::core::Vector :- [:wat::WatAST])`). The annotation used to be
    // mandatory-then-discarded — a lie like `[x <- :wat::core::i64]` was silently accepted.
    // Validate the SOLE argspec output here so the lie is a `MalformedDefmacro` at definition
    // time, not a confusing failure (or silent wrong behaviour) at first expansion.
    use crate::types::TypeExpr;
    fn is_watast(ty: &TypeExpr) -> bool {
        matches!(ty, TypeExpr::Path(p) if p == ":wat::WatAST")
    }
    fn is_watast_vec(ty: &TypeExpr) -> bool {
        matches!(ty, TypeExpr::Parametric { head, args }
            if head == "wat::core::Vector" && args.len() == 1 && is_watast(&args[0]))
    }
    for (ident, ty) in &spec.fixed_params {
        if !is_watast(ty) {
            return Err(MacroError {
                span: argvec_span.clone(),
                kind: MacroErrorKind::MalformedDefmacro {
                    reason: format!(
                        "macro param `{}` is declared `{ty:?}`, but a macro param always binds a \
                         form — its type must be `:wat::WatAST`",
                        ident.as_str()
                    ),
                },
            });
        }
    }
    if let Some((ident, ty)) = &spec.rest_param {
        if !is_watast_vec(ty) {
            return Err(MacroError {
                span: argvec_span.clone(),
                kind: MacroErrorKind::MalformedDefmacro {
                    reason: format!(
                        "macro rest-param `{}` is declared `{ty:?}`, but a rest param binds a \
                         sequence of forms — its type must be `(:wat::core::Vector :- [:wat::WatAST])`",
                        ident.as_str()
                    ),
                },
            });
        }
    }
    if let WatAST::Keyword(ret_kw, ret_span) = &rettype_item {
        if ret_kw != ":wat::WatAST" {
            return Err(MacroError {
                span: ret_span.clone(),
                kind: MacroErrorKind::MalformedDefmacro {
                    reason: format!(
                        "macro return type is declared `{ret_kw}`, but a macro always expands to a \
                         form — its return type must be `:wat::WatAST`"
                    ),
                },
            });
        }
    }

    // Extract param names only — MacroDef carries names, not types.
    // Bare derivation: macro substitution keys are bare (expansion-time pattern match).
    let params: Vec<String> = spec.fixed_params.into_iter().map(|(ident, _ty)| ident.as_str().to_owned()).collect();
    let rest_param: Option<String> = spec.rest_param.map(|(ident, _ty)| ident.as_str().to_owned());

    // Hoist: definition-time validation runs ONCE here (not per expansion call) — arc 249 stone O.
    // `validate_macro_definition` checks hygiene (Gate E) and purity. Both are pure predicates
    // of the immutable body; running once at definition means a bad program-body macro fails
    // at definition, not silently at first invocation. Only applies to program-body templates
    // (non-quasiquote bodies); quasiquote bodies use the walk_template path instead.
    // `is_quasiquote_form` is the single shared head-only discriminant — see expand.rs.
    if !super::expand::is_quasiquote_form(&body_item) {
        super::expand::validate_macro_definition(&body_item, &list_span, &name)?;
    }

    Ok(MacroDef {
        name,
        params,
        rest_param,
        body: body_item,
        span: list_span,
        source_form,
    })
}

// Stone 241.17 — parse_defmacro_signature DELETED (~80 lines of arc 010/150 paren-pair parser).
// `:wat::core::defmacro` signature shape migrated from paren-pair-with-type form to canonical
// Vector-of-triples form mirroring arc 166 defn shape.
// The HARD-CUT-rejection arm in parse_defmacro_form fires for any old 3-item paren-pair form.
// `parse_argspec_triples` (Stone 241.1's canonical parser) is now the third major consumer
// after fn (Stones 241.2) and defclause (Stone 241.3/241.4).
// Per `feedback_hard_cut_admits_no_bypasses` — no compatibility shim.

// ─── Arc 294 item 9a — Rust-registered-aggregate kwargs companion class closure ──
//
// A wat `defstruct`/`defrecord` invocation mints its own kwargs companion macro
// at ITS OWN macro-expansion time (`wat/core.wat:1694` — the `(:wat::core::do
// (:wat::core::structtype ~@args) (:wat::core::defmacro ~fqdn-bare-kw ...))`
// shape). An aggregate registered directly in Rust (`TypeEnv::with_builtins()`
// via `register_builtin_types`, including the `inventory`-driven `EdnSchema`
// drain) never goes through that macro, so it never gets a companion — bare
// kwargs construction (`(:wat::holon::CapacityExceeded :cost 200 :budget 100)`)
// fails `UnknownFunction`. This closes the class structurally: every
// `TypeDef::Aggregate` the baked `TypeEnv` knows about gets a companion, full
// stop — no aggregate can lack one.
//
// The synthesized companion is byte-for-byte the same THIN FORWARDER shape
// `defstruct`'s macro emits (proven by hand in a scratch probe before this
// code existed — see arc 294 item 9a strike notes): a `defmacro` whose body
// bakes in the prime ctor keyword, the field-name vector, and the
// pascal->kebab-in namespace, then forwards the call args to
// `:wat::core::kwargs-lower`. It does NOT mint the ctor — `register_aggregate_methods`
// (runtime.rs) already mints the prime `:T'` for every aggregate unconditionally.
//
// Rendered as wat SOURCE TEXT and routed through the real parser +
// `parse_defmacro_form` (rather than hand-rolled `WatAST` construction) so the
// synthesized macro is provably identical in shape to what a human — or
// `defstruct` — would author, and so it passes the same definition-time
// hygiene/purity validation (`validate_macro_definition`) every other macro does.
pub fn register_aggregate_kwargs_companions(
    types: &crate::types::TypeEnv,
    registry: &mut MacroRegistry,
) -> Result<(), MacroError> {
    use crate::types::TypeDef;

    for (_key, def) in types.iter() {
        let agg = match def {
            TypeDef::Aggregate(a) => a,
            _ => continue,
        };
        // Skip-if-present — never clobber a companion a wat `defstruct`/`defrecord`
        // already registered under this bare name. Determined the only way the
        // registry exposes: `MacroRegistry::contains`.
        if registry.contains(&agg.name) {
            continue;
        }
        let source = aggregate_kwargs_companion_source(&agg.name, agg.field_names());
        let form = crate::parse_one!(&source).map_err(|e| MacroError {
            span: crate::rust_caller_span!(),
            kind: MacroErrorKind::MalformedDefmacro {
                reason: format!(
                    "internal: synthesized kwargs companion for {} failed to parse: {:?}",
                    agg.name, e
                ),
            },
        })?;
        let macro_def = parse_defmacro_form(form)?;
        // `register_stdlib` — every candidate here comes from `TypeEnv::with_builtins()`,
        // which seeds substrate (`:wat::*`-prefixed) types exclusively, so the companion
        // needs the same reserved-prefix bypass the literal stdlib defmacro path gets.
        registry.register(macro_def, crate::resolve::Privilege::Stdlib)?;
    }
    Ok(())
}

/// Render the companion `defmacro` source for aggregate `bare_name` with
/// `field_names` in declaration order — the exact shape `defstruct`'s macro
/// (`wat/core.wat:1694`) generates for the `(:wat::core::defmacro ~fqdn-bare-kw
/// ...)` half (the `structtype` half is skipped: the aggregate is already
/// registered).
fn aggregate_kwargs_companion_source<'a>(
    bare_name: &str,
    _field_names: impl Iterator<Item = &'a str>,
) -> String {
    // Arc 294 item (C) — emit the LIVE `kwargs-construct` form over the bare `:T`
    // keyword; check/eval resolve `:T`'s (splice-merged, post-register) field order off
    // the registry and reorder the kwargs there. Replaces the expand-time `kwargs-lower`
    // forward (baked field-vector), whose hole is the SPLICED-record bug — so the
    // field-name vector + prime + ns constants this used to bake are no longer needed.
    format!(
        "(:wat::core::defmacro {bare_name} \
           [& call-args <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::WatAST \
           (:wat::core::let \
             [_kc-type (:wat::core::keyword-node \"{bare_name}\")] \
             `(:wat::core::kwargs-construct ~_kc-type ~@call-args)))",
        bare_name = bare_name,
    )
}
