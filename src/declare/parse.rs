//! Arc 109 Stone 2 — the declare home's PARSE phase.
//!
//! Split by PHASE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-declare-home.md`). This file reads a
//! declaration form's SHAPE and answers "is this a declaration, and what does it say" — the
//! `is_*`/`parse_*`/`try_parse_*` fns the brief names, plus `resolve_type_slot_args`, placed here
//! because its only callers (both inside `parse_type_slot`, this file) are parse-phase, not
//! registration. `parse_*`/`is_*` SERVES both `register.rs` and `preregister.rs`, which is why it
//! is its own file rather than folded into either. Moved verbatim out of `src/runtime.rs` (arc 109
//! Stone 2). Behaviour is unchanged; only the location moved.
//!
//! Two small const tables move here with their sole consumers, kept adjacent as they were in
//! `runtime.rs`: `RUNTIME_DECLARATION_HEADS` (read only by [`is_runtime_declaration_head`]) and
//! `DECLARATION_HEADS` (read only by [`is_declaration_head`]). Two private helper types move with
//! their sole consumer [`try_parse_fn_shape_def`]: `FnShapeMetadata`, `ParsedFnShapeDef`.
//!
//! Siblings: `register.rs` (populate the SymbolTable), `preregister.rs` (the earlier
//! stub-before-bodies pass), `typevar.rs` (free/bound type-variable walking).

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::value::{EvalBreak, Function, FunctionBody, RuntimeError, RuntimeErrorKind};

// `synthesize_fn_body` is genuinely defined in `crate::runtime` (the `defclause_dispatch`
// region), not a facade re-export of a `crate::value` type — see STOP-1.
use crate::runtime::synthesize_fn_body;

use crate::declare::typevar::collect_free_type_vars;

/// Arc 265 — parse a `declare-acronyms` form and return `(namespace, acronyms)`.
///
/// Shape: `(:wat::string::declare-acronyms :ns ["ACL" "HTTP"])`.
/// The namespace is the keyword string (with leading colon, e.g. `":my::aws"`).
/// The acronym list is a `WatAST::Vector` of `WatAST::String` nodes.
/// Returns `Err` on malformed input; callers silently skip errors (pre-pass).
pub(crate) fn parse_declare_acronyms_form(
    form: &WatAST,
) -> Result<(String, Vec<String>), RuntimeError> {
    const HEAD: &str = ":wat::string::declare-acronyms";
    let form_span = form.span().clone();
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(RuntimeError::new(
                form_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: "expected list".into(),
                },
            ))
        }
    };
    // items[0] = :wat::string::declare-acronyms keyword
    // items[1] = :ns namespace keyword
    // items[2] = ["ACL" ...] vector of string literals
    if items.len() != 3 {
        return Err(RuntimeError::new(form_span, RuntimeErrorKind::MalformedForm {
            head: HEAD.into(),
            reason: format!(
                "expected (:wat::string::declare-acronyms :ns [\"ACL\" ...]); got {} elements",
                items.len()
            )
        }));
    }
    let ns = match &items[1] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "first arg must be a keyword namespace; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    let acronyms = match &items[2] {
        WatAST::Vector(children, _) => {
            let mut acc = Vec::with_capacity(children.len());
            for child in children {
                match child {
                    WatAST::StringLit(s, _) => acc.push(s.clone()),
                    other => {
                        return Err(RuntimeError::new(
                            other.span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: HEAD.into(),
                                reason: format!(
                                    "acronym list entries must be String literals; got {}",
                                    other.variant_name()
                                ),
                            },
                        ))
                    }
                }
            }
            acc
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "second arg must be a Vector of String literals; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    Ok((ns, acronyms))
}

/// Arc 170 — the heads [`register_runtime_defs_form`] consumes: the RUNTIME
/// declaration forms (as opposed to the TYPE declarations, which the freeze
/// consumes before residue is ever produced).
///
/// This is a GATE the match below sits behind, not a copy of it — see
/// [`is_runtime_declaration_head`]. A head absent from this list can never reach
/// the match, so the two cannot silently disagree; an un-listed arm goes dead
/// (fails closed) rather than diverging.
///
/// `do` / `let` are here because they SPLICE — either may contain a def in a
/// nested position, so both are declaration-BEARING even though neither is a
/// declaration itself. That distinction is load-bearing: see
/// [`is_declaration_head`], which is the other question and must NOT be answered
/// from this list.
pub(crate) const RUNTIME_DECLARATION_HEADS: &[&str] = &[
    ":wat::config::set-redef!",
    ":wat::config::set-eval-redef!",
    ":wat::core::def",
    ":wat::core::do",
    ":wat::core::let",
    ":wat::core::defclause",
    ":wat::core::extend-type",
    ":wat::core::derive",
];

/// The heads that ARE a declaration, as opposed to merely CARRYING one.
///
/// `do` / `let` are deliberately ABSENT. They are expressions: a top-level
/// `(let [x 1] x)` has the value `1`, and `(do 1 2 3)` has the value `3`.
///
/// ⚠ THIS SET EXISTS BECAUSE ONE LIST WAS ANSWERING TWO QUESTIONS, and the wrong
/// answer was shipped. [`RUNTIME_DECLARATION_HEADS`] answers *"will
/// `register_runtime_defs` walk into this looking for a def?"* — yes for `do`/`let`,
/// because they splice. `:wat::eval-with-defs!` was asking that same list *"is this
/// an expression whose value I should return?"* and getting the splice answer, so a
/// top-level `let` or `do` was classified `FormOutcome::Declared` and its value
/// DISCARDED — `wat --repl` printed nothing, `wat --mcp` answered `nil`. Reported
/// live from a zero-prior model driving the MCP; reproduced by hand in both modes.
///
/// The old doc comment named its own defect two lines above the list it was wrong
/// about (*"neither is a declaration itself"*) while the predicate returned true for
/// both — the arc's recurring shape: live code plus a confident comment.
const DECLARATION_HEADS: &[&str] = &[
    ":wat::config::set-redef!",
    ":wat::config::set-eval-redef!",
    ":wat::core::def",
    ":wat::core::defclause",
    ":wat::core::extend-type",
    ":wat::core::derive",
];

/// Does this head name a form that [`register_runtime_defs`] will consume?
///
/// A GATE on the walk — "might this form carry a def?" — NOT the answer to
/// "is this a declaration": for that, ask [`is_declaration_head`]. It must be asked
/// of a POST-MACRO-EXPANSION form: `defn` is a macro that expands to `def`
/// (`wat/core.wat:1175`), so the raw surface head a user typed is not what lands in
/// residue — which is exactly why an eval-time error cannot classify (`defn` fails
/// eval as `unknown-function`, byte-identical to a typo; measured in
/// `wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat`).
pub(crate) fn is_runtime_declaration_head(head: &str) -> bool {
    RUNTIME_DECLARATION_HEADS.contains(&head)
}

/// Is this residue form a declaration, rather than an expression whose value answers?
///
/// The authority `:wat::eval-with-defs!` asks to choose `FormOutcome::Declared` over
/// `::Evaluated`. Same post-macro-expansion caveat as
/// [`is_runtime_declaration_head`].
pub(crate) fn is_declaration_head(head: &str) -> bool {
    DECLARATION_HEADS.contains(&head)
}

/// A form that grows the session: a declaration head, or a `do` whose
/// every child is itself a declaration. `defservice` expands to the
/// latter; classifying the `do` as an expression left companions in a
/// world the next turn never saw.
pub(crate) fn is_declaration_form(form: &WatAST) -> bool {
    let items = match form {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => return false,
    };
    let head = match &items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => return false,
    };
    if head == ":wat::core::do" {
        // Type-decl stripping leaves `(do nil)` in the residue. That is
        // not a value; it is an empty splice.
        return items[1..]
            .iter()
            .all(|c| matches!(c, WatAST::NilLit(_)) || is_declaration_form(c));
    }
    is_declaration_head(head)
}

/// Stone 241.12 — detect `(:wat::core::defalias :alias-name :target-name)` shape.
///
/// Returns `(alias_name, target_name)` strings if the form matches.
pub(crate) fn parse_defalias_form(form: &WatAST) -> Option<(String, String)> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return None,
    };
    if items.len() != 3 {
        return None;
    }
    match &items[0] {
        WatAST::Keyword(k, _) if k == ":wat::core::defalias" => {}
        _ => return None,
    }
    let alias = match &items[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,
    };
    let target = match &items[2] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,
    };
    Some((alias, target))
}

/// Stone 241.8 — detect `(:wat::core::defstruct :Name ...)` shape.
/// Arc 293.2-parity — also matches `:wat::core::structtype` (the primitive defstruct expands to).
/// Replaces legacy struct / struct-restricted detection (HARD CUT).
pub(crate) fn is_struct_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(
                items.first(),
                Some(WatAST::Keyword(k, _))
                    if k == ":wat::core::defstruct" || k == ":wat::core::structtype"
            )
    )
}

/// Arc 170 slice 3 Gap F-1 — detect `(:wat::core::defenum :Name ...)` shape.
/// Stone 241.9 — updated from :wat::core::enum to :wat::core::defenum (HARD CUT).
pub(crate) fn is_enum_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::defenum")
    )
}

/// Arc 166 — detect `(:wat::core::def :name (:wat::core::fn sig body))` shape.
/// Returns the parsed `(name, Function)` pair if the form is a def whose
/// RHS is a fn-form, else `None`.
///
/// The defn macro expands to this exact shape. Pre-registering the name
/// in `sym.functions` (via `register_defines`) gives the type checker
/// visibility for recursive self-reference inside the fn body — same
/// pre-registration contract `define` enjoys. Without this, a recursive
/// `(:wat::core::defn :fact (sig) (... (:fact ...) ...))` fails with
/// `UnresolvedReferences` because `infer_def` infers the RHS BEFORE
/// writing to `defined_values`.
///
/// The form is ALSO kept in `rest` (returned by `register_defines`), so
/// `register_runtime_defs` evaluates the def expression at runtime and
/// populates `sym.runtime_def_values`. The double-registration is
/// benign: call dispatch checks `sym.functions` first (per
/// `lookup_form`'s precedence ladder), so the pre-registered Function
/// wins; the `runtime_def_values` entry is vestigial-but-correct.
/// Stone 241.6 — detect if a WatAST is a metadata-map (the `{...}` clause
/// between binding name and value-expr on `def` / `defn`). Arc 257 slice 1
/// changed the parser to emit a native `WatAST::Map(pairs, span)` node for
/// `{k v ...}` directly — no `(:wat::core::HashMap ...)` constructor call is
/// synthesized any more (`is_metadata_map` / `metadata_map_pairs` also accept
/// the legacy List-with-HashMap-head shape for backward compat).
/// An empty `{}` renders as `WatAST::Map([], span)` (0 pairs) and is ILLEGAL
/// per FORM-COLLAPSE-NOTES (divide-by-zero).
///
/// Returns `Some(inner_map)` where inner_map maps keyword-string → WatAST value.
/// Returns `None` if the node is not a metadata-map at all.
/// Caller is responsible for emitting an error on empty `{}` when `Some(empty)` is
/// returned — this fn returns `Some(empty_map)` in that case so the caller can
/// distinguish "not a map" from "empty map".
// Arc 257 slice 1 — updated to use `is_metadata_map()` / `metadata_map_pairs()`
// so both WatAST::Map and legacy List-with-HashMap-head are accepted.
pub(crate) fn try_parse_metadata_map(node: &WatAST) -> Option<HashMap<String, WatAST>> {
    // Use the authoritative predicate from ast.rs.
    let pairs = node.metadata_map_pairs()?;
    let mut meta: HashMap<String, WatAST> = HashMap::new();
    for (k_node, v_node) in pairs {
        let key_str = match &k_node {
            WatAST::Keyword(k, _) => k.clone(),
            _ => return None, // non-keyword key — malformed
        };
        meta.insert(key_str, v_node);
    }
    Some(meta)
}

/// A `def`'s `{...}` metadata-map clause, keyed by the metadata key's bare name.
type FnShapeMetadata = HashMap<String, WatAST>;

/// What a well-formed fn-shape `def` parses into: the bound name, the function it
/// binds, and the metadata clause if the 4-item form carried one.
///
/// Named because the tuple is the noun — all four call sites of
/// [`try_parse_fn_shape_def`] destructure it identically as
/// `(path, func, metadata_opt)`, so the shape is a contract, not an incidental
/// return. `None` for the metadata means the 3-item form was used, NOT that a
/// metadata clause was present and empty.
type ParsedFnShapeDef = (String, Arc<Function>, Option<FnShapeMetadata>);

/// Returns `Ok(Some((name, function, metadata_opt)))` where `metadata_opt` is `Some(map)`
/// if a `{...}` metadata-map clause was present, `None` if the 3-item form was used.
/// `Ok(None)` means "not this shape" — load-bearing for the recognizer chain; callers
/// fall through to the next parser. `Err` is reserved for a shape this fn DOES recognize
/// as an fn-shape-def but that is internally contradictory (arc 109 row 3: BOTH a
/// name-embedded `<T>` spelling and a `:- [...]` binder) — a case no other parser in the
/// chain would recognize either, so returning `Ok(None)` here would silently drop the
/// form instead of reporting the contradiction.
pub(crate) fn try_parse_fn_shape_def(form: &WatAST) -> Result<Option<ParsedFnShapeDef>, RuntimeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(None),
    };
    // Accept 3-item (no metadata) or 4-item (metadata at items[2]) forms.
    if items.len() != 3 && items.len() != 4 {
        return Ok(None);
    }
    // Head must be :wat::core::def.
    match &items[0] {
        WatAST::Keyword(k, _) if k == ":wat::core::def" => {}
        _ => return Ok(None),
    }
    // First arg is the name keyword.
    // STONE reap-the-angle-machinery (arc 109) — this used to split off a `<T,...>` name
    // suffix via `split_name_and_type_params` (arc 139). That suffix is unexpressible now
    // (verified directly: `(def :foo<T> ...)` is a LEXER error, before this parser ever
    // runs), so `raw_type_params` is always empty here; type params for a generic `defn`
    // come from the `:- [T ...]` binder peeled below instead.
    let (name, raw_type_params): (String, Vec<String>) = match &items[1] {
        WatAST::Keyword(k, _) => (k.clone(), Vec::new()),
        // Arc 300.1 — faithful-Clojure dual surface: a namespaced Symbol in
        // def-name position (`user/main`) is the keyword FQDN's twin. Convert
        // to `:user::main` so the def registers under the SAME key the harness
        // and resolver look up. Additive — the Keyword arm above is unchanged.
        WatAST::Symbol(s, _) if s.is_reference() => {
            (crate::edn::render::ns_to_wat_path(s.receiver(), s.method()), Vec::new())
        }
        _ => return Ok(None),
    };
    // Stone 241.6 — if 4 items, items[2] must be a non-empty metadata-map;
    // items[3] is the fn-form. If 3 items, items[2] is the fn-form directly.
    let (fn_slot_idx, metadata_opt) = if items.len() == 4 {
        // items[2] must be a metadata-map (head :wat::core::HashMap).
        // If it's not a HashMap list, this is not the fn-shape-def path.
        match try_parse_metadata_map(&items[2]) {
            Some(meta) => (3usize, Some(meta)), // metadata present; fn-form is at index 3
            None => return Ok(None), // items[2] is not a metadata-map; let other parsers handle
        }
    } else {
        (2usize, None) // 3-item form; fn-form is at index 2; no metadata
    };
    // Second arg must be `(:wat::core::fn ARGS-VECTOR -> :RET body...)`
    // — the canonical flat shape per arc 167. Arc 168 — body slot
    // accepts 1+ trailing forms (implicit-do); the structural arity
    // grows from "exactly 5 elements" to "5+ elements". An empty
    // body (4-element form: head + sig-trio) is also legal —
    // synthesizes `:wat::core::nil`.
    //
    // Stone 241.6 — the fn-form may also carry a leading metadata-map
    // from defn macro expansion: `(fn {meta} [args] -> :ret body)`.
    // When fn_items[1] is a HashMap list, peel it as binding-level metadata
    // and treat fn_items[2..] as the actual signature. This allows the
    // defn macro to remain unchanged while `defn :name {meta} [args] -> ...`
    // expands to `def :name (fn {meta} [args] -> ...)` with the metadata
    // stored at the binding level (not at the fn level).
    let fn_items = match &items[fn_slot_idx] {
        WatAST::List(fn_items, _) => fn_items,
        _ => return Ok(None),
    };
    match fn_items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn" => {}
        _ => return Ok(None),
    }
    if fn_items.len() < 4 {
        return Ok(None);
    }
    // Stone 241.6 — detect fn-embedded metadata: fn_items[1] is a HashMap list
    // produced by defn macro expansion threading `{meta}` into the fn-form.
    // Peel it off: fn signature starts at fn_items[2..].
    let (sig_start, metadata_opt) = if fn_slot_idx == 2 {
        // Only peel from fn-embedded metadata when the 3-item def path was taken
        // (fn_slot_idx == 2 means no explicit metadata-map on def itself).
        match try_parse_metadata_map(&fn_items[1]) {
            Some(meta) if !meta.is_empty() => {
                // fn has embedded metadata; sig starts at fn_items[2]
                if fn_items.len() < 5 {
                    return Ok(None); // not enough items after peeling
                }
                (2usize, Some(meta))
            }
            _ => (1usize, metadata_opt), // no fn-embedded metadata; sig starts at fn_items[1]
        }
    } else {
        // 4-item def: explicit metadata-map already captured; fn has no embedded metadata.
        (1usize, metadata_opt)
    };
    // Arc 109 gamma-i — peel an optional `:- [T U ...]` type-param binder,
    // riding into the emitted `fn` (via defn's rest-binder splicing)
    // immediately after the (already-peeled) fn-embedded metadata and
    // before the args-vector.
    let (binder, sig_slice) = crate::function::peel_type_binder(&fn_items[sig_start..]);
    if sig_slice.len() < 3 {
        return Ok(None);
    }
    // Arc 109 gamma-i row 3 — a declaration carrying BOTH a name-embedded
    // `<T>` spelling (raw_type_params, non-empty) AND a `:- [...]` binder
    // is a contradiction, never something to silently resolve. Mirrors
    // `types.rs`'s `take_declared_binder`, which raises the identical
    // message for the type-declarator heads (defrecord/defenum/etc). This
    // fn recognizes the shape as an fn-shape-def by this point (head, name,
    // fn-form all matched) — Ok(None) would silently drop a form no other
    // parser in the chain recognizes either, masking the contradiction
    // instead of reporting it.
    if binder.is_some() && !raw_type_params.is_empty() {
        return Err(RuntimeError::new(
            items[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::defn".into(),
                reason: format!(
                    "declaration carries BOTH a name-embedded `<...>` type-param spelling \
                     ({:?}) and a `:- [...]` binder — pick one; a declaration with both is a \
                     contradiction, never something to silently resolve",
                    raw_type_params
                ),
            },
        ));
    }
    // Synthesize body via implicit-do over the trailing forms after the
    // 3-element signature prefix. Empty trailing slice → :wat::core::nil
    // keyword; single → pass-through; multiple → (:wat::core::do f1 f2 ... fN).
    let body = synthesize_fn_body(&sig_slice[3..]);
    // parse_fn_signature consumes the 3-element signature prefix:
    // ARGS-VECTOR / `->` / :RET-TYPE. Body is synthesized independently.
    let sig_args = [
        sig_slice[0].clone(),
        sig_slice[1].clone(),
        sig_slice[2].clone(),
    ];
    let (params, param_types, ret_type) = match crate::function::parse_fn_signature(&sig_args) {
        Ok(triple) => triple,
        Err(_) => return Ok(None),
    };
    // Stone 251.7 — union raw_type_params (from `<T,U>` name suffix) with any
    // free bare type-vars in the signature so bare-var forms auto-generalize.
    // Arc 109 gamma-i — also union the fn's own `:- [T ...]` binder, if any.
    // (The both-spellings contradiction is rejected above; by construction
    // at most one of raw_type_params/binder is non-empty here.)
    let mut raw_type_params = raw_type_params;
    if let Some(binder_names) = binder {
        for tp in binder_names {
            if !raw_type_params.contains(&tp) {
                raw_type_params.push(tp);
            }
        }
    }
    for fv in collect_free_type_vars(&param_types, &ret_type) {
        if !raw_type_params.contains(&fv) {
            raw_type_params.push(fv);
        }
    }
    Ok(Some((
        name.clone(),
        Arc::new(Function {
            name: Some(name),
            params,
            // Arc 139 — preserve type_params from the name keyword (e.g. `<T>`)
            // so the type checker can instantiate generic functions correctly.
            // Stone 251.7 — extended with free signature vars (bare-Uppercase Paths).
            type_params: raw_type_params,
            param_types,
            ret_type,
            rest_param: None,
            rest_param_type: None,
            body: FunctionBody::Wat(Arc::new(body)),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        }),
        metadata_opt,
    )))
}

/// Stone 241.11 — detect variadic `(:wat::core::def :name (:wat::core::fn [...& xs...] -> :T body))`
/// shapes that `try_parse_fn_shape_def` cannot handle because `parse_fn_signature` uses
/// `allow_rest_binder: false`.
///
/// Returns `Some((name, Arc<Function>))` only when the form is a `def` with a `fn` body that
/// contains a rest binder (`& xs <- :T`).  Non-variadic `def/fn` forms return `None` so
/// `try_parse_fn_shape_def` (which runs first in `register_stdlib_defines`) handles them.
///
/// Stdlib is PRIVILEGED — reserved-prefix gate bypassed.  Called exclusively from
/// `register_stdlib_defines`.
pub(crate) fn try_parse_variadic_def_fn_form(form: &WatAST) -> Option<(String, Arc<Function>)> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return None,
    };
    // Must be exactly `(:wat::core::def :name (:wat::core::fn ...))` — 3 items.
    if items.len() != 3 {
        return None;
    }
    // Head must be :wat::core::def.
    match &items[0] {
        WatAST::Keyword(k, _) if k == ":wat::core::def" => {}
        _ => return None,
    }
    // items[1] is the name keyword.
    // STONE reap-the-angle-machinery (arc 109) — a `<T,...>` name suffix is unexpressible
    // now, so the `split_name_and_type_params` strip this used to do is a no-op; use the
    // keyword directly (empty type params, same as before every call this stdlib path saw).
    let (name, raw_type_params): (String, Vec<String>) = match &items[1] {
        WatAST::Keyword(k, _) => (k.clone(), Vec::new()),
        _ => return None,
    };
    // items[2] must be a `(:wat::core::fn ARGS-VECTOR -> :RET body...)` list.
    let fn_items = match &items[2] {
        WatAST::List(fn_items, _) => fn_items,
        _ => return None,
    };
    match fn_items.first()? {
        WatAST::Keyword(k, _) if k == ":wat::core::fn" => {}
        _ => return None,
    }
    if fn_items.len() < 4 {
        return None;
    }
    // fn_items layout (no fn-embedded metadata — stdlib variadic forms are substrate-authored):
    //   [0] :wat::core::fn
    //   [1] ARGS-VECTOR   (must be WatAST::Vector)
    //   [2] ->
    //   [3] :RET-TYPE
    //   [4..] body forms (zero or more)
    let (args_vec, args_vec_span) = match &fn_items[1] {
        WatAST::Vector(items, span) => (items, span),
        _ => return None,
    };
    // Arrow check.
    match &fn_items[2] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => return None,
    }
    // Return type.
    let ret_type = match &fn_items[3] {
        WatAST::Keyword(k, _) => parse_type_keyword(k).ok()?,
        _ => return None,
    };
    // Parse args with rest-binder allowed.
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        crate::argspec::ParseOptions {
            allow_rest_binder: true,
        },
    )
    .ok()?;
    // Only handle variadic forms here; non-variadic should have been caught by
    // try_parse_fn_shape_def (which runs first). Guard defensively.
    spec.rest_param.as_ref()?;
    let (fixed_idents, fixed_param_types): (
        Vec<crate::scope::Identifier>,
        Vec<crate::types::TypeExpr>,
    ) = spec.fixed_params.into_iter().unzip();
    let fixed_params: Vec<crate::scope::Identifier> = fixed_idents.clone();
    let (rest_ident, rest_ty) = spec.rest_param?;
    let rest_name = crate::scope::env_key(&rest_ident).into_owned();
    // Stone 251.7 — union raw_type_params with free bare type-vars in the signature.
    let mut raw_type_params = raw_type_params;
    for fv in collect_free_type_vars(&fixed_param_types, &ret_type) {
        if !raw_type_params.contains(&fv) {
            raw_type_params.push(fv);
        }
    }
    // Synthesize body from trailing fn_items.
    let body = synthesize_fn_body(&fn_items[4..]);
    Some((
        name.clone(),
        Arc::new(Function {
            name: Some(name),
            params: fixed_params,
            type_params: raw_type_params,
            param_types: fixed_param_types,
            ret_type,
            rest_param: Some(rest_name),
            rest_param_type: Some(rest_ty),
            body: FunctionBody::Wat(Arc::new(body)),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        }),
    ))
}

/// Arc 150 — user-source variadic `(:wat::core::def :name (:wat::core::fn [...& rest...] -> :T body))`
/// parser that PROPAGATES argspec errors as `RuntimeError`.
///
/// Distinct from `try_parse_variadic_def_fn_form` (stdlib path, which uses `.ok()?` to
/// swallow errors so non-matching forms fall through silently). The user-source path
/// must surface malformed argspecs (double `&`, `&` without binder, fixed-after-rest)
/// as `StartupError::Runtime` rather than silently leaving the name unregistered and
/// hitting the resolver with `UnresolvedReference`.
///
/// Returns:
/// - `Ok(None)` — form is not a 3-item `def + fn` shape at all (let other handlers try).
/// - `Ok(Some((name, func)))` — valid variadic form; `func` carries rest_param + rest_param_type.
/// - `Err(RuntimeError)` — form IS a `def + fn` shape with `&` in the argspec but the
///   argspec is malformed, OR the rest-binder type is not `(Vector :- [T])` / `(Vec :- [T])`.
///
/// Called exclusively from `register_defines` (user-source path); reserved-prefix check
/// is the caller's responsibility.
pub(crate) fn try_parse_user_variadic_def_fn_form(
    form: &WatAST,
) -> Result<Option<(String, Arc<Function>)>, RuntimeError> {
    let items = match form {
        WatAST::List(items, _) => items,
        _ => return Ok(None),
    };
    // Must be exactly `(:wat::core::def :name (:wat::core::fn ...))` — 3 items.
    if items.len() != 3 {
        return Ok(None);
    }
    // Head must be :wat::core::def.
    match &items[0] {
        WatAST::Keyword(k, _) if k == ":wat::core::def" => {}
        _ => return Ok(None),
    }
    // items[1] is the name keyword.
    // STONE reap-the-angle-machinery (arc 109) — this used to strip a `<T,...>` name
    // suffix via `split_name_and_type_params`; unexpressible now, so it's a no-op — use
    // the keyword directly (empty type params).
    let (name, raw_type_params): (String, Vec<String>) = match &items[1] {
        WatAST::Keyword(k, _) => (k.clone(), Vec::new()),
        // Arc 300.1 — faithful-Clojure dual surface (variadic twin of Site B):
        // a namespaced Symbol def-name (`my/fold`) → keyword FQDN so faithful
        // VARIADIC defs register too. Additive — Keyword arm above unchanged.
        WatAST::Symbol(s, _) if s.is_reference() => {
            (crate::edn::render::ns_to_wat_path(s.receiver(), s.method()), Vec::new())
        }
        _ => return Ok(None),
    };
    // items[2] must be a `(:wat::core::fn ARGS-VECTOR -> :RET body...)` list.
    let fn_items = match &items[2] {
        WatAST::List(fn_items, _) => fn_items,
        _ => return Ok(None),
    };
    match fn_items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn" => {}
        _ => return Ok(None),
    }
    if fn_items.len() < 4 {
        return Ok(None);
    }
    // fn_items layout (no fn-embedded metadata for user variadic forms):
    //   [0] :wat::core::fn
    //   [1] ARGS-VECTOR   (must be WatAST::Vector containing `&`)
    //   [2] ->
    //   [3] :RET-TYPE
    //   [4..] body forms (zero or more)
    let (args_vec, args_vec_span) = match &fn_items[1] {
        WatAST::Vector(items, span) => (items, span),
        _ => return Ok(None),
    };
    // Quick check: does the args vector contain `&`? If not, this is a
    // non-variadic form that try_parse_fn_shape_def already handles.
    let has_rest_marker = args_vec.iter().any(|a| a.is_bare_symbol("&"));
    if !has_rest_marker {
        return Ok(None);
    }
    // Arrow check.
    match &fn_items[2] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => return Ok(None),
    }
    // Return type.
    let ret_type = match &fn_items[3] {
        WatAST::Keyword(k, _) => parse_type_keyword(k)?,
        _ => return Ok(None),
    };
    // Parse args with rest-binder allowed. Errors (double `&`, incomplete
    // triple after `&`, trailing items after rest) surface as RuntimeError.
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        crate::argspec::ParseOptions {
            allow_rest_binder: true,
        },
    )
    .map_err(RuntimeError::from)?;
    // Guard: rest_param must be present (we checked for `&` above; an argspec
    // with `&` that produces no rest_param would be a parser inconsistency).
    let (rest_ident, rest_ty) = match spec.rest_param {
        Some(pair) => pair,
        None => return Ok(None),
    };
    // Validate: rest-binder type must be (Vector :- [T]) (or (Vec :- [T])). A bare scalar
    // type like `:wat::core::i64` is rejected here so the error surfaces as a
    // startup registration error rather than a silent type-check failure.
    let is_vector = matches!(
        &rest_ty,
        crate::types::TypeExpr::Parametric { head, .. }
            if head == "wat::core::Vector" || head == "wat::core::Vec"
    );
    if !is_vector {
        let span = args_vec
            .iter()
            .find(|a| a.is_bare_symbol("&"))
            .map(|a| a.span().clone())
            .unwrap_or_else(|| args_vec_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                head: ":wat::core::fn".into(),
                reason: format!(
                    "rest-binder type must be Vector<T> (e.g. `:wat::core::Vector<:wat::core::i64>`); \
                     got `{}`",
                    crate::check::format_type(&rest_ty)
                ),
            }));
    }
    let (fixed_idents, fixed_param_types): (
        Vec<crate::scope::Identifier>,
        Vec<crate::types::TypeExpr>,
    ) = spec.fixed_params.into_iter().unzip();
    let fixed_params: Vec<crate::scope::Identifier> = fixed_idents.clone();
    let rest_name = crate::scope::env_key(&rest_ident).into_owned();
    // Stone 251.7 — union raw_type_params with free bare type-vars in the signature.
    let mut raw_type_params = raw_type_params;
    for fv in collect_free_type_vars(&fixed_param_types, &ret_type) {
        if !raw_type_params.contains(&fv) {
            raw_type_params.push(fv);
        }
    }
    // Synthesize body from trailing fn_items.
    let body = synthesize_fn_body(&fn_items[4..]);
    Ok(Some((
        name.clone(),
        Arc::new(Function {
            name: Some(name),
            params: fixed_params,
            type_params: raw_type_params,
            param_types: fixed_param_types,
            ret_type,
            rest_param: Some(rest_name),
            rest_param_type: Some(rest_ty),
            body: FunctionBody::Wat(Arc::new(body)),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        }),
    )))
}

pub(crate) fn parse_type_keyword(kw: &str) -> Result<crate::types::TypeExpr, RuntimeError> {
    // arc 138: no span — kw is a `&str` lifted from the keyword's payload;
    // the keyword's own span isn't carried through the parse helper.
    // Stone 241.16 — error head updated from `:wat::core::define` to `:wat::core::defn`.
    crate::types::parse_type_expr(kw).map_err(|e| {
        RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::defn".into(),
                reason: e.to_string(),
            },
        )
    })
}

/// Arc 201 slice 1 — accept EITHER a keyword (legacy source form) OR a
/// structured-AST list (the shape `type_expr_to_ast` emits) as the
/// type slot of a type annotation. The structured form arises
/// when a reflection consumer takes a signature head from `signature-of-defn`
/// and splices it back into a fresh `defn`: the splice carries `WatAST::List` for every
/// Parametric / Tuple / Fn type, where the original source wrote a
/// `WatAST::Keyword`.
/// (Stone 241.16 — doc updated; `:wat::core::define` reference removed.)
///
/// Both routes converge on the same `TypeExpr`. Source-keyword inputs
/// go through `crate::types::parse_type_expr` (which understands the
/// `:(T,U)` / `:Fn(T)->U` surface spelling — a bare atomic keyword or one of
/// those two compound forms; `:Head<args>` is refused, arc 109 ③). Structured-
/// AST inputs are walked directly:
///
/// - `WatAST::List [Keyword ":Tuple", ...args]` → `TypeExpr::Tuple`
/// - `WatAST::List [Keyword ":Fn", ...args, Symbol "->", ret]` → `TypeExpr::Fn`
/// - `WatAST::List [Keyword ":Head", ...args]` (any other head) →
///   `TypeExpr::Parametric { head: Head sans-colon, args: recurse }`
///
/// Atomic positions (`WatAST::Keyword`) recurse via `parse_type_keyword`
/// so the existing surface spelling stays the source of truth for
/// Path / Var shapes.
/// Arc 109 ③ — `parse_type_slot`'s arg-list resolver. `rest` is a structured type List's
/// items AFTER the head (`items[1..]`). Three accepted shapes, checked in this order:
///   1. `:- [args…] ` (Arc 109 the binder marker) — the args are the vector's own items. A
///      trailing item after the vector is a LITERAL, not a type (mirrors `parse_type_form`'s
///      identical refusal in `src/types.rs`) — refused, not silently accepted.
///   2. `[args…]` alone (arc 109 step ① bracket sugar, no `:-`) — same reading, no marker.
///   3. Anything else — the ORIGINAL flat positional reading (`rest` itself, each item its
///      own arg) — `parse_type_slot`'s pre-existing behavior, unchanged for backward
///      compatibility with any caller still spelling it that way.
///
/// Without this, a List shaped `(Head :- [args])` — the ONLY reference-role spelling Arc 109
/// ③ leaves legal — hit case 3 here: `:-` (a Keyword) and the args Vector both got read as
/// if they were themselves TYPE ARGS, and the Vector arm doesn't even parse (this fn's `other`
/// arm has no Vector case) — a hard failure on every structured type slot this stone's
/// codemod produced.
fn resolve_type_slot_args(rest: &[WatAST]) -> Result<Vec<crate::types::TypeExpr>, EvalBreak> {
    let (peeled, extra) = crate::types::peel_param_spec(rest);
    if let Some(inner) = peeled {
        if !extra.is_empty() {
            // `peel_param_spec` only returns `Some` when `rest[1]` was the `Vector`
            // it peeled — its span is the right anchor for this diagnostic.
            return Err(RuntimeError::new(
                rest[1].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::defn".into(),
                    reason: "a type declaration cannot carry initial values — \
                              `(Head :- [types] v…)` is a LITERAL, and a literal is not a \
                              type"
                        .into(),
                },
            )
            .into());
        }
        return inner.iter().map(parse_type_slot).collect();
    }
    match rest {
        [WatAST::Vector(inner, _)] => inner.iter().map(parse_type_slot).collect(),
        positional => positional.iter().map(parse_type_slot).collect(),
    }
}

pub(crate) fn parse_type_slot(ast: &WatAST) -> Result<crate::types::TypeExpr, EvalBreak> {
    match ast {
        WatAST::Keyword(k, _) => parse_type_keyword(k).map_err(Into::into),
        WatAST::List(items, span) => {
            if items.is_empty() {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::defn".into(),
                        reason: "structured type slot must be a non-empty list".into(),
                    },
                )
                .into());
            }
            let head_kw = match &items[0] {
                WatAST::Keyword(k, _) => k.as_str(),
                other => {
                    return Err(RuntimeError::new(
                        other.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::defn".into(),
                            reason: format!(
                                "structured type slot head must be a keyword; got {}",
                                other.variant_name()
                            ),
                        },
                    )
                    .into());
                }
            };
            // :Fn — args*, Symbol("->"), ret. Split at the arrow.
            if head_kw == ":Fn" {
                let mut arrow_idx: Option<usize> = None;
                for (i, child) in items.iter().enumerate().skip(1) {
                    if let WatAST::Symbol(ident, _) = child {
                        if ident.as_str() == "->" {
                            arrow_idx = Some(i);
                            break;
                        }
                    }
                }
                let arrow = arrow_idx.ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::defn".into(),
                            reason: "structured :Fn type missing '->' arrow".into(),
                        },
                    )
                })?;
                if arrow + 1 >= items.len() {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::defn".into(),
                            reason: "structured :Fn type missing return-type slot after '->'"
                                .into(),
                        },
                    )
                    .into());
                }
                if arrow + 2 != items.len() {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::defn".into(),
                            reason: "structured :Fn type has extra slots after return type".into(),
                        },
                    )
                    .into());
                }
                let mut fn_args: Vec<crate::types::TypeExpr> =
                    Vec::with_capacity(arrow.saturating_sub(1));
                for child in items.iter().skip(1).take(arrow.saturating_sub(1)) {
                    fn_args.push(parse_type_slot(child)?);
                }
                let ret = parse_type_slot(&items[arrow + 1])?;
                return Ok(crate::types::TypeExpr::Fn {
                    args: fn_args,
                    ret: Box::new(ret),
                });
            }
            // :Tuple — all remaining children are element types. Arc 109 ③ — also the FQDN
            // spelling `:wat::core::Tuple` (what this stone's codemod emits, matching
            // `parse_type_form`'s own `raw_head == "wat::core::Tuple"` special-case in
            // `src/types.rs` — the canonical checker-side parser tests the FQDN, not the bare
            // short name, so this runtime-side twin now tests both). Args resolve through
            // `resolve_type_slot_args` so a `:- [args]` binder (or the bare `[args]` bracket)
            // reads correctly here too, not just the flat positional legacy shape.
            if head_kw == ":Tuple" || head_kw == ":wat::core::Tuple" {
                let elems = resolve_type_slot_args(&items[1..])?;
                return Ok(crate::types::TypeExpr::Tuple(elems));
            }
            // Any other head — Parametric. Strip the leading ':' to
            // recover the head spelling used by `TypeExpr::Parametric`
            // (which stores the FQDN sans-colon, e.g. `wat::core::Option`).
            let head_no_colon = head_kw.strip_prefix(':').unwrap_or(head_kw).to_string();
            let p_args = resolve_type_slot_args(&items[1..])?;
            Ok(crate::types::TypeExpr::Parametric {
                head: head_no_colon,
                args: p_args,
            })
        }
        other => Err(RuntimeError::new(
            other.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::defn".into(),
                reason: format!(
                    "parameter type must be a type keyword or structured type list; got {}",
                    other.variant_name()
                ),
            },
        )
        .into()),
    }
}

/// Arc 109 ③ — shape test for a runtime type-keyword ARG whose content this call site never
/// actually consumes (`self-peer`, `listener'`'s socket-pair args): historically `WatAST::
/// Keyword(_, _)` only, since a parametric arg always arrived as one angle-bracket keyword.
/// Angle brackets are illegal now — the SAME parametric arg arrives as the reference FORM
/// `(Head :- [args])`, a `WatAST::List` — so this widens the shape test to accept both,
/// additive only (a bare Keyword still passes exactly as before).
pub(crate) fn is_type_arg_shaped(a: &WatAST) -> bool {
    matches!(a, WatAST::Keyword(_, _) | WatAST::List(_, _))
}

/// Stone 251.7 — THE VAR TEST, extracted so every consumer asks it the same way.
///
/// A `TypeExpr::Path` names a lexically-scoped type VARIABLE (`:T`, `:K`, `:V`) iff, after
/// stripping the leading `:`, it is bare (contains neither `"::"` nor `'.'`) and its first
/// alphabetic character is uppercase. This excludes the lowercase legacy bare primitives
/// (`:i64`, `:bool`) and every FQDN type (`:wat::core::i64`, `:user::Foo`).
pub(crate) fn is_type_var_path(p: &str) -> bool {
    let s = p.strip_prefix(':').unwrap_or(p);
    if s.contains("::") || s.contains('.') {
        return false;
    }
    s.chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(|c| c.is_uppercase())
}

