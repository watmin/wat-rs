//! vigilatum: 2026-06-01T02:47:26Z — vigilia 9-spell L1+L2=0
//!
//! `parse_defstruct` decomposed by concern.
//!
//! Stone 243.5 home for the `(:wat::core::defstruct ...)` declaration parser.
//! Concerns separated: arity validation, name parsing, slot discrimination,
//! metadata-map parsing, field-vector parsing, restrictions assembly, struct
//! assembly. Each concern is a named helper; `parse_defstruct` orchestrates.

use crate::ast::WatAST;
use crate::span::Span;
use std::collections::HashMap;

use super::{StructDef, StructRestrictions, TypeDef, TypeExpr, TypeError, TypeErrorKind};

const HEAD: &str = ":wat::core::defstruct";

/// Validate that `parse_defstruct` received a legal number of args.
///
/// Legal: 2 (name + fields) or 3 (name + metadata + fields). Returns `Ok(())`
/// if the count is in range; returns a `MalformedDecl` error otherwise.
fn validate_defstruct_arity(args_len: usize, decl_span: &Span) -> Result<(), TypeError> {
    if args_len < 2 {
        return Err(TypeError {
            span: decl_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defstruct :Name [fields]) or with optional metadata-map; got {} args after head",
                    args_len
                ),
            },
        });
    }
    if args_len > 3 {
        return Err(TypeError {
            span: decl_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "too many args: expected 2 (name + fields) or 3 (name + metadata + fields); got {}",
                    args_len
                ),
            },
        });
    }
    Ok(())
}

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
fn parse_defstruct_metadata(
    meta_node: WatAST,
) -> Result<ParsedStructMeta, TypeError> {
    let mut ctor_whitelist: Vec<String> = Vec::new();
    let mut field_restrictions: HashMap<String, Vec<String>> = HashMap::new();

    // Arc 257 slice 1: use is_metadata_map() / metadata_map_pairs() to accept
    // both WatAST::Map and the legacy List-with-HashMap-head form.
    if !meta_node.is_metadata_map() {
        return Err(TypeError {
            span: meta_node.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "expected a metadata-map `{...}` as second arg".into(),
            },
        });
    }
    let pairs = meta_node.metadata_map_pairs().ok_or_else(|| TypeError {
        span: meta_node.span().clone(),
        kind: TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "malformed metadata-map (internal structure corrupt)".into(),
        },
    })?;
    // Empty {} → pairs.len() == 0 → REJECTED per FORM-COLLAPSE-NOTES.
    if pairs.is_empty() {
        return Err(TypeError {
            span: meta_node.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "empty `{}` metadata-map is illegal (use no metadata-map arg for plain struct)".into(),
            },
        });
    }
    // Walk key/value pairs.
    for (k_node, val) in &pairs {
        let key_str = match k_node {
            WatAST::Keyword(k, _) => k.clone(),
            other => {
                return Err(TypeError {
                    span: other.span().clone(),
                    kind: TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: "metadata-map keys must be keywords".into(),
                    },
                });
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
                                    return Err(TypeError {
                                        span: item.span().clone(),
                                        kind: TypeErrorKind::MalformedDecl {
                                            head: HEAD.into(),
                                            reason: ":restricted-to entries must be keyword prefixes".into(),
                                        },
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(TypeError {
                            span: val.span().clone(),
                            kind: TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: ":restricted-to value must be a Vector of keyword prefixes `[...]`".into(),
                            },
                        });
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
        return Err(TypeError {
            span: val.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: ":field-metadata value must be a map `{field {meta} ...}`".into(),
            },
        });
    }
    let fm_pairs = val.metadata_map_pairs().ok_or_else(|| TypeError {
        span: val.span().clone(),
        kind: TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "malformed :field-metadata map (internal structure corrupt)".into(),
        },
    })?;
    for (fk_node, fmeta) in &fm_pairs {
        // field identifier — Keyword with optional leading colon stripped to get bare name.
        // In the Map literal form {witness {meta}}, `witness` must be written as
        // `:witness` (keyword) because the parser routes `{sym {map}}` to
        // struct-destructure (parse error). Keyword `:witness` → strip colon → "witness".
        let field_sym = match fk_node {
            WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
            WatAST::Symbol(ident, _) => ident.as_str().to_owned(),
            other => {
                return Err(TypeError {
                    span: other.span().clone(),
                    kind: TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: ":field-metadata field keys must be keyword field names (e.g. `:witness`)".into(),
                    },
                });
            }
        };
        // field metadata-map — must be a metadata-map itself.
        if !fmeta.is_metadata_map() {
            return Err(TypeError {
                span: fmeta.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        ":field-metadata value for field '{}' must be a map `{{...}}`",
                        field_sym
                    ),
                },
            });
        }
        let fpairs = fmeta.metadata_map_pairs().ok_or_else(|| TypeError {
            span: fmeta.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "malformed :field-metadata for field '{}' (corrupt structure)",
                    field_sym
                ),
            },
        })?;
        // Parse inner keys: recognize :restricted-to.
        let mut field_wlist: Vec<String> = Vec::new();
        for (fkey_node, fval) in &fpairs {
            let fkey = match fkey_node {
                WatAST::Keyword(k, _) => k.clone(),
                other => {
                    return Err(TypeError {
                        span: other.span().clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                ":field-metadata inner keys for '{}' must be keywords",
                                field_sym
                            ),
                        },
                    });
                }
            };
            if fkey == ":restricted-to" {
                match fval {
                    WatAST::Vector(prefix_items, _) => {
                        for item in prefix_items {
                            match item {
                                WatAST::Keyword(k, _) => field_wlist.push(k.clone()),
                                _ => {
                                    return Err(TypeError {
                                        span: item.span().clone(),
                                        kind: TypeErrorKind::MalformedDecl {
                                            head: HEAD.into(),
                                            reason: format!(
                                                ":field-metadata :restricted-to entries for '{}' must be keyword prefixes",
                                                field_sym
                                            ),
                                        },
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(TypeError {
                            span: fval.span().clone(),
                            kind: TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    ":field-metadata :restricted-to for '{}' must be a Vector `[...]`",
                                    field_sym
                                ),
                            },
                        });
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
fn parse_defstruct_fields(
    fields_node: WatAST,
) -> Result<Vec<(String, TypeExpr)>, TypeError> {
    let (field_items, field_span) = match fields_node {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "field-vector must be a Vector `[field <- :T ...]`".into(),
                },
            });
        }
    };
    let argspec = crate::argspec::parse_argspec_triples(
        &field_items,
        HEAD,
        &field_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;
    Ok(argspec.fixed_params.into_iter().map(|(id, ty)| (id.as_str().to_owned(), ty)).collect())
}

/// Stone 241.8 — parse a `(:wat::core::defstruct :Name [...fields...])` or
/// `(:wat::core::defstruct :Name {metadata} [...fields...])` declaration.
///
/// Three positional forms after the head keyword (consumed by `parse_type_decl`):
///   args[0]       — name keyword (e.g. `:my::ns::MyType`)
///   args[1..N-1]  — optional metadata-map `{...}` (WatAST::List with head
///                   `:wat::core::HashMap`); absent in the 2-arg form.
///   args[last]    — field-vector `[field <- :T ...]` (WatAST::Vector)
///
/// Metadata keys recognized:
///   `:restricted-to [kwlist]`          — form-level ctor restriction
///   `:field-metadata {sym → meta}`     — per-field metadata map
///   (unknown keys are silently stored; D5)
///
/// Empty `{}` is REJECTED per FORM-COLLAPSE-NOTES (divide-by-zero principle).
/// Field-vector is parsed by `parse_argspec_triples` with
/// `ParseOptions { allow_rest_binder: false }`.
pub(crate) fn parse_defstruct(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    validate_defstruct_arity(args.len(), &decl_span)?;

    let mut iter = args.into_iter();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, type_params) = super::parse_declared_name(HEAD, &name_kw, &decl_span)?;

    // Discriminate: 2-arg form vs 3-arg form.
    let (metadata_node_opt, fields_node) = if iter.len() == 1 {
        // 2-arg form: name + fields (no metadata).
        (None, iter.next().unwrap())
    } else {
        // 3-arg form: name + metadata + fields.
        let meta_node = iter.next().unwrap();
        let fields_node = iter.next().unwrap();
        (Some(meta_node), fields_node)
    };

    // Parse optional metadata-map.
    let (ctor_whitelist, field_restrictions) = if let Some(meta_node) = metadata_node_opt {
        parse_defstruct_metadata(meta_node)?
    } else {
        (Vec::new(), HashMap::new())
    };

    // Parse field-vector.
    let fields = parse_defstruct_fields(fields_node)?;

    // Build restrictions: None if no whitelist + no field restrictions; Some(_) otherwise.
    let restrictions = if ctor_whitelist.is_empty() && field_restrictions.is_empty() {
        None
    } else {
        Some(StructRestrictions {
            ctor_whitelist,
            field_restrictions,
        })
    };

    Ok(TypeDef::Struct(StructDef {
        name,
        type_params,
        fields,
        restrictions,
    }))
}
