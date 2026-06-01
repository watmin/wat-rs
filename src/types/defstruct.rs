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

    // Validate it IS a HashMap list.
    let meta_items = match &meta_node {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError {
                span: meta_node.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "expected a metadata-map `{...}` as second arg".into(),
                },
            });
        }
    };
    // Head must be :wat::core::HashMap.
    match meta_items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap" => {}
        _ => {
            return Err(TypeError {
                span: meta_node.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "second arg must be a metadata-map `{...}` (HashMap form)".into(),
                },
            });
        }
    }
    // Structure: [head, K-type, V-type, k0, v0, k1, v1, ...]
    // Minimum: 3 items (head + K + V). Pairs start at index 3.
    if meta_items.len() < 3 {
        return Err(TypeError {
            span: meta_node.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "malformed metadata-map (internal structure corrupt)".into(),
            },
        });
    }
    let pairs = &meta_items[3..];
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
    let mut meta_pair_idx = 0;
    while meta_pair_idx + 1 < pairs.len() {
        let key_str = match &pairs[meta_pair_idx] {
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
        let val = &pairs[meta_pair_idx + 1];
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
        meta_pair_idx += 2;
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
    // Value must be a HashMap list mapping field-symbols to metadata-maps.
    let fm_items = match val {
        WatAST::List(items, _) => items,
        _ => {
            return Err(TypeError {
                span: val.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: ":field-metadata value must be a map `{field {meta} ...}`".into(),
                },
            });
        }
    };
    // Head must be :wat::core::HashMap.
    match fm_items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap" => {}
        _ => {
            return Err(TypeError {
                span: val.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: ":field-metadata value must be a HashMap map form `{...}`".into(),
                },
            });
        }
    }
    // Structure: [head, K-type, V-type, field0, meta0, field1, meta1, ...]
    if fm_items.len() < 3 {
        return Err(TypeError {
            span: val.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "malformed :field-metadata map (internal structure corrupt)".into(),
            },
        });
    }
    let fm_pairs = &fm_items[3..];
    let mut field_pair_idx = 0;
    while field_pair_idx + 1 < fm_pairs.len() {
        // field identifier — Keyword with optional leading colon stripped to get bare name.
        // In the HashMap literal form {witness {meta}}, `witness` must be written as
        // `:witness` (keyword) because the parser routes `{sym {map}}` to
        // struct-destructure (parse error). Keyword `:witness` → strip colon → "witness".
        let field_sym = match &fm_pairs[field_pair_idx] {
            WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
            WatAST::Symbol(ident, _) => ident.name.clone(),
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
        // field metadata-map — a HashMap list.
        let fmeta = &fm_pairs[field_pair_idx + 1];
        let fmeta_items = match fmeta {
            WatAST::List(items, _) => items,
            _ => {
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
        };
        match fmeta_items.first() {
            Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap" => {}
            _ => {
                return Err(TypeError {
                    span: fmeta.span().clone(),
                    kind: TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: format!(
                            ":field-metadata value for field '{}' must be a HashMap map form",
                            field_sym
                        ),
                    },
                });
            }
        }
        if fmeta_items.len() < 3 {
            return Err(TypeError {
                span: fmeta.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "malformed :field-metadata for field '{}' (corrupt structure)",
                        field_sym
                    ),
                },
            });
        }
        let fpairs = &fmeta_items[3..];
        // Parse inner keys: recognize :restricted-to.
        let mut field_wlist: Vec<String> = Vec::new();
        let mut inner_key_idx = 0;
        while inner_key_idx + 1 < fpairs.len() {
            let fkey = match &fpairs[inner_key_idx] {
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
            let fval = &fpairs[inner_key_idx + 1];
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
            inner_key_idx += 2;
        }
        if !field_wlist.is_empty() {
            field_restrictions.insert(field_sym, field_wlist);
        }
        field_pair_idx += 2;
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
    Ok(argspec.fixed_params)
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
