//! vigilatum: 2026-06-01T02:47:26Z — vigilia 9-spell L1+L2=0
//!
//! Field/metadata parsing shared by every aggregate declarator, decomposed by concern.
//!
//! Stone 243.5 home for the `(:wat::core::defstruct ...)` declaration parser.
//! Concerns separated: arity validation, name parsing, slot discrimination,
//! metadata-map parsing, field-vector parsing, restrictions assembly, struct
//! assembly. Stone 255.1a-β-i-b retired the orchestrator itself
//! (`parse_defstruct` — dead, since `:wat::core::defstruct` never survives
//! `expand_all`) and its arity gate; `parse_defstruct_metadata` and
//! `parse_aggregate_fields_with_splices` remain the live helpers, now reached
//! from `types.rs`'s `parse_aggregate` (the `structtype` path) instead.

use crate::ast::WatAST;
use crate::span::Span;
use std::collections::HashMap;

use super::{SurfaceMember, TypeDef, TypeEnv, TypeExpr, TypeError, TypeErrorKind};

const HEAD: &str = ":wat::core::defstruct";

// Stone 255.1a-β-i-b — `validate_defstruct_arity` REMOVED (its only caller, `parse_defstruct`,
// is removed below). `:wat::core::defstruct` is a stdlib `defmacro` (`wat/core.wat:2030`) that
// `expand_all` rewrites to `:wat::core::structtype`; `parse_type_decl`'s `"defstruct"` arm that
// called `parse_defstruct` could therefore never fire (it consumes post-expansion forms) and is
// removed too. `parse_structtype`'s own arity check (routed through `parse_aggregate`) is the
// live equivalent for the post-expansion `structtype` head.

/// Parsed struct metadata: ordered field-name list + per-field restriction map.
///
/// Returned by `parse_defstruct_metadata`. The `Vec<String>` is the ctor
/// whitelist; the `HashMap` maps field names to their restriction-prefix lists.
type ParsedStructMeta = (Vec<String>, HashMap<String, Vec<String>>);

/// Parse the optional metadata-map node into `(ctor_whitelist, field_restrictions)`.
///
/// The metadata node must be a `{...}` HashMap list. Recognized keys:
/// - `:restricted-to [kwlist]` — form-level ctor restriction prefix list.
/// - `:field-metadata {field → {meta}}` — per-field restriction maps.
///
/// Unknown keys are silently accepted (D5).
pub(super) fn parse_defstruct_metadata(
    meta_node: WatAST,
) -> Result<ParsedStructMeta, TypeError> {
    let mut ctor_whitelist: Vec<String> = Vec::new();
    let mut field_restrictions: HashMap<String, Vec<String>> = HashMap::new();

    // Arc 257 slice 1: use is_metadata_map() / metadata_map_pairs() to accept
    // both WatAST::Map and the legacy List-with-HashMap-head form.
    if !meta_node.is_metadata_map() {
        return Err(TypeError::new(
            meta_node.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "expected a metadata-map `{...}` as second arg".into(),
            },
        ));
    }
    let pairs = meta_node.metadata_map_pairs().ok_or_else(|| TypeError::new(
        meta_node.span().clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "malformed metadata-map (internal structure corrupt)".into(),
        },
    ))?;
    // Empty {} → pairs.len() == 0 → REJECTED per FORM-COLLAPSE-NOTES.
    if pairs.is_empty() {
        return Err(TypeError::new(
            meta_node.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "empty `{}` metadata-map is illegal (use no metadata-map arg for plain struct)".into(),
            },
        ));
    }
    // Walk key/value pairs.
    for (k_node, val) in &pairs {
        let key_str = match k_node {
            WatAST::Keyword(k, _) => k.clone(),
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: "metadata-map keys must be keywords".into(),
                    },
                ));
            }
        };
        match key_str.as_str() {
            ":restricted-to" => {
                // Value must be a Vector of keyword prefixes.
                match val {
                    WatAST::Vector(prefix_items, _) => {
                        for item in prefix_items {
                            match item {
                                WatAST::Keyword(k, _) => ctor_whitelist.push(k.clone()),
                                _ => {
                                    return Err(TypeError::new(
                                        item.span().clone(),
                                        TypeErrorKind::MalformedDecl {
                                            head: HEAD.into(),
                                            reason: ":restricted-to entries must be keyword prefixes".into(),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(TypeError::new(
                            val.span().clone(),
                            TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: ":restricted-to value must be a Vector of keyword prefixes `[...]`".into(),
                            },
                        ));
                    }
                }
            }
            ":field-metadata" => {
                parse_field_metadata_key(val, &mut field_restrictions)?;
            }
            _ => {
                // Unknown metadata keys silently accepted (D5).
            }
        }
    }

    Ok((ctor_whitelist, field_restrictions))
}

/// Parse the `:field-metadata` map value into `field_restrictions`.
///
/// Called from `parse_defstruct_metadata` for the `:field-metadata` key.
/// Separated into its own fn to keep the metadata-walk loop readable.
fn parse_field_metadata_key(
    val: &WatAST,
    field_restrictions: &mut HashMap<String, Vec<String>>,
) -> Result<(), TypeError> {
    // Arc 257 slice 1: use is_metadata_map() / metadata_map_pairs() to accept
    // both WatAST::Map and the legacy List-with-HashMap-head form.
    if !val.is_metadata_map() {
        return Err(TypeError::new(
            val.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: ":field-metadata value must be a map `{field {meta} ...}`".into(),
            },
        ));
    }
    let fm_pairs = val.metadata_map_pairs().ok_or_else(|| TypeError::new(
        val.span().clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "malformed :field-metadata map (internal structure corrupt)".into(),
        },
    ))?;
    for (fk_node, fmeta) in &fm_pairs {
        // field identifier — Keyword with optional leading colon stripped to get bare name.
        // In the Map literal form {witness {meta}}, `witness` must be written as
        // `:witness` (keyword) because the parser routes `{sym {map}}` to
        // struct-destructure (parse error). Keyword `:witness` → strip colon → "witness".
        let field_sym = match fk_node {
            WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
            WatAST::Symbol(ident, _) => ident.as_str().to_owned(),
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: ":field-metadata field keys must be keyword field names (e.g. `:witness`)".into(),
                    },
                ));
            }
        };
        // field metadata-map — must be a metadata-map itself.
        if !fmeta.is_metadata_map() {
            return Err(TypeError::new(
                fmeta.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        ":field-metadata value for field '{}' must be a map `{{...}}`",
                        field_sym
                    ),
                },
            ));
        }
        let fpairs = fmeta.metadata_map_pairs().ok_or_else(|| TypeError::new(
            fmeta.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "malformed :field-metadata for field '{}' (corrupt structure)",
                    field_sym
                ),
            },
        ))?;
        // Parse inner keys: recognize :restricted-to.
        let mut field_wlist: Vec<String> = Vec::new();
        for (fkey_node, fval) in &fpairs {
            let fkey = match fkey_node {
                WatAST::Keyword(k, _) => k.clone(),
                other => {
                    return Err(TypeError::new(
                        other.span().clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                ":field-metadata inner keys for '{}' must be keywords",
                                field_sym
                            ),
                        },
                    ));
                }
            };
            if fkey == ":restricted-to" {
                match fval {
                    WatAST::Vector(prefix_items, _) => {
                        for item in prefix_items {
                            match item {
                                WatAST::Keyword(k, _) => field_wlist.push(k.clone()),
                                _ => {
                                    return Err(TypeError::new(
                                        item.span().clone(),
                                        TypeErrorKind::MalformedDecl {
                                            head: HEAD.into(),
                                            reason: format!(
                                                ":field-metadata :restricted-to entries for '{}' must be keyword prefixes",
                                                field_sym
                                            ),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(TypeError::new(
                            fval.span().clone(),
                            TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    ":field-metadata :restricted-to for '{}' must be a Vector `[...]`",
                                    field_sym
                                ),
                            },
                        ));
                    }
                }
            }
            // Unknown inner keys silently accepted (D5).
        }
        if !field_wlist.is_empty() {
            field_restrictions.insert(field_sym, field_wlist);
        }
    }
    Ok(())
}

/// Parse the field-vector node into `Vec<(String, TypeExpr)>`.
///
/// Expects a `WatAST::Vector` containing `field <- :Type` triples.
///
/// Arc 293 decl-a — extracted as `pub(super)` so `parse_aggregate` in the parent
/// module (`types.rs`) can call this as the ONE canonical field parser.
///
/// STOP-FIELD resolved: chosen over `parse_recordtype`'s inline groups-of-3 parser.
/// Diff: (a) this parser accepts both `<-` and `:-` arrows (arc 251 superset);
///          the inline parser only accepts `<-`.
///       (b) this parser requires Symbol field names;
///          the inline parser also accepts Keyword names (strips leading `:`).
/// Reconciliation: `:-` is the forward direction (arc 251 sweep); Keyword field
/// names are not used in practice for records — Symbol is the established form.
pub(super) fn parse_aggregate_fields(
    fields_node: WatAST,
    head: &str,
) -> Result<Vec<(String, TypeExpr)>, TypeError> {
    let (field_items, field_span) = match fields_node {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: head.into(),
                    reason: "field-vector must be a Vector `[field <- :T ...]`".into(),
                },
            ));
        }
    };
    let argspec = crate::argspec::parse_argspec_triples(
        &field_items,
        head,
        &field_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;
    Ok(argspec.fixed_params.into_iter().map(|(id, ty)| (id.as_str().to_owned(), ty)).collect())
}

/// Arc 293 surface-splice — match `(:wat::core::unquote-splicing :Surface)`, the reader's
/// `~@:Surface` node (`crates/wat-reader/src/parser.rs:353`). Returns the surface keyword
/// when `item` is that exact shape; `None` otherwise (an ordinary field-triple element).
fn splice_target(item: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = item {
        if items.len() == 2 {
            if let (WatAST::Keyword(head_kw, _), WatAST::Keyword(surface_kw, _)) =
                (&items[0], &items[1])
            {
                if head_kw == ":wat::core::unquote-splicing" {
                    return Some(surface_kw.clone());
                }
            }
        }
    }
    None
}

/// Parse the field-vector node, expanding any `~@:Surface` splice elements against the
/// (partially built) type registry BEFORE running the plain triple parser.
///
/// Arc 293 surface-splice (`BRIEF-293-surface-splice-build.md`). THE CRUX: this function
/// is called from the type-registration pass (`register_types_impl` / `splice_type_decls`
/// in `types.rs`), the one layer where both the record decl form AND the registry-so-far
/// (`env`) are available together — `parse_aggregate_fields` itself stays registry-free
/// and UNCHANGED; callers that have no splice elements pay zero extra cost (this function
/// delegates straight to `parse_aggregate_fields` when no `~@:Surface` is present).
///
/// Semantics (pinned with the builder, 2026-07-04):
/// - each `~@:Surface` expands to that surface's `Field` members ONLY (Method members are
///   skipped — a record cannot hold a function; methods are `extend-surface`'s concern);
/// - the merge is a union in first-occurrence order (splices in written order, then own
///   fields, each contributing its members/fields in declared order);
/// - a field name repeated at an IDENTICAL type dedupes to one; at a CONFLICTING type it is
///   a compile-time `MalformedDecl` ("if A says int, B says string, it does not compile").
/// - an unresolved splice target (surface not yet registered — forward reference, or not a
///   surface at all) is a clean `MalformedDecl`, not a two-pass build (out of scope, brief
///   STOP-FORWARD-REF).
pub(super) fn parse_aggregate_fields_with_splices(
    fields_node: WatAST,
    head: &str,
    env: &TypeEnv,
) -> Result<Vec<(String, TypeExpr)>, TypeError> {
    let (field_items, field_span) = match &fields_node {
        WatAST::Vector(items, span) => (items, span.clone()),
        _ => {
            // Not a Vector at all — let the existing parser produce its own,
            // already-established error for this shape.
            return parse_aggregate_fields(fields_node, head);
        }
    };

    // Fast path: no splice elements at all — identical to pre-splice behavior.
    if !field_items.iter().any(|item| splice_target(item).is_some()) {
        return parse_aggregate_fields(fields_node, head);
    }

    // Walk the vector, accumulating ordered (name, TypeExpr) entries: either parsed from a
    // contiguous non-splice run (via the existing triple parser) or expanded from a spliced
    // surface's Field members (in the surface's declared order).
    let mut raw: Vec<(String, TypeExpr)> = Vec::new();
    let mut run: Vec<WatAST> = Vec::new();

    let field_items = match fields_node {
        WatAST::Vector(items, _) => items,
        _ => unreachable!("matched WatAST::Vector above"),
    };

    for item in field_items {
        match splice_target(&item) {
            Some(surface_kw) => {
                if !run.is_empty() {
                    flush_field_run(&run, head, &field_span, &mut raw)?;
                    run.clear();
                }
                let item_span = item.span().clone();
                match env.get(&surface_kw) {
                    Some(TypeDef::Surface(surf)) => {
                        for member in &surf.members {
                            if let SurfaceMember::Field { name, ty } = member {
                                raw.push((name.clone(), ty.clone()));
                            }
                            // Method members are skipped (brief STOP-METHOD-SPLICE resolved:
                            // a record cannot hold a function; extend-surface installs methods).
                        }
                    }
                    Some(_) => {
                        return Err(TypeError::new(
                            item_span,
                            TypeErrorKind::MalformedDecl {
                                head: head.into(),
                                reason: format!(
                                    "surface-splice `~@{}` target is registered but is not a \
                                     `defsurface`",
                                    surface_kw
                                ),
                            },
                        ));
                    }
                    None => {
                        return Err(TypeError::new(
                            item_span,
                            TypeErrorKind::MalformedDecl {
                                head: head.into(),
                                reason: format!(
                                    "surface-splice `~@{}` refers to an unknown surface — it must \
                                     be `defsurface`-declared BEFORE the splicing record (forward \
                                     references are out of scope)",
                                    surface_kw
                                ),
                            },
                        ));
                    }
                }
            }
            None => run.push(item),
        }
    }
    if !run.is_empty() {
        flush_field_run(&run, head, &field_span, &mut raw)?;
    }

    // Merge = union, first-occurrence order. Dedup by identical type; conflicting type → error.
    let mut merged: Vec<(String, TypeExpr)> = Vec::new();
    for (name, ty) in raw {
        match merged.iter().find(|(n, _)| *n == name) {
            Some((_, existing_ty)) if existing_ty == &ty => {
                // Same name, identical type — dedupes to the first occurrence; drop this one.
            }
            Some((_, existing_ty)) => {
                return Err(TypeError::new(
                    field_span.clone(),
                    TypeErrorKind::MalformedDecl {
                        head: head.into(),
                        reason: format!(
                            "surface-splice conflict: field `{}` is installed at conflicting \
                             types ({:?} vs {:?}) by two splices (or a splice and an own field) \
                             — a field repeated across splices must carry an identical type",
                            name, existing_ty, ty
                        ),
                    },
                ));
            }
            None => merged.push((name, ty)),
        }
    }
    Ok(merged)
}

/// Run a contiguous non-splice sub-slice through the existing triple parser and append its
/// `(name, TypeExpr)` pairs to `raw`, in order.
fn flush_field_run(
    run: &[WatAST],
    head: &str,
    field_span: &Span,
    raw: &mut Vec<(String, TypeExpr)>,
) -> Result<(), TypeError> {
    let argspec = crate::argspec::parse_argspec_triples(
        run,
        head,
        field_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;
    raw.extend(
        argspec
            .fixed_params
            .into_iter()
            .map(|(id, ty)| (id.as_str().to_owned(), ty)),
    );
    Ok(())
}

// Stone 255.1a-β-i-b — `parse_defstruct` REMOVED. It parsed a
// `(:wat::core::defstruct :Name [...fields...])` / `(:wat::core::defstruct :Name {metadata}
// [...fields...])` declaration directly, but `:wat::core::defstruct` is a stdlib `defmacro`
// (`wat/core.wat:2030`) that `expand_all` rewrites to `:wat::core::structtype` — its ONE caller,
// `parse_type_decl`'s `"defstruct"` arm (`types.rs`), consumed post-expansion forms and so could
// never see a raw `defstruct` head. Retirement: the field/metadata parsing this function did by
// hand is not gone — `parse_structtype` reaches the SAME `parse_defstruct_metadata` and
// `parse_aggregate_fields_with_splices` helpers below via `parse_aggregate` (`types.rs:4678`/
// `4684`), which is why this file is not dead, only these two functions and the two dispatch arms
// that reached them.
