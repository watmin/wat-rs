//! Arc 293.3-core — `parse_defsurface`.
//!
//! A surface declares a structural interface: a set of required named members
//! with types. Structs satisfy a surface by having (at least) those members
//! with assignable types (width subtyping). No `:satisfies`, no `:parent`,
//! no declaration at the use site.
//!
//! `parse_defsurface` mirrors `parse_defstruct` but is simpler: name + fields
//! only (no metadata-map), v1 monomorphic (no `<T>` type params shipped here).
//!
//! Arc 293.4a — member list now carries both Field members (`name <- :T` triples)
//! and Method members (`(name [args...] -> :R)` lists). `struct_satisfies_surface`
//! takes a `resolve_method` closure (supplied by the `check` layer at the call site
//! in `assignable`) so that Method satisfaction can consult `defn :T/<name>` sigs.

use crate::ast::WatAST;
use crate::span::Span;

use super::{Holder, SurfaceDef, SurfaceMember, TypeDef, TypeExpr, TypeError, TypeErrorKind};

const HEAD: &str = ":wat::core::defsurface";

/// True iff `struct_fields` satisfies every member of `surface` (width-open: extras OK).
///
/// Row-polymorphic width subtyping for Field members: for each `Field { name, ty }` in
/// `surface.members`, `struct_fields` must contain `(fname, fty)` where `fname == name`
/// and `is_assignable(fty, ty)` holds. Extra fields in `struct_fields` are fine.
///
/// Arc 293.4a — Method members: a `Method { name, ret, .. }` is satisfied iff
/// `resolve_method(name)` returns a `(arg_types, defn_ret)` whose `defn_ret` is
/// assignable to the member's `ret`. The resolver is supplied by the caller (typically
/// built in `check::assignable` from `env.schemes`) so this module does not depend on
/// the check layer.
///
/// `is_assignable` is a caller-supplied check (typically `check::assignable`),
/// passed as a closure so this module does not depend on `check`.
pub fn struct_satisfies_surface<F, R>(
    struct_fields: &[(String, TypeExpr)],
    surface: &SurfaceDef,
    mut is_assignable: F,
    mut resolve_method: R,
) -> bool
where
    F: FnMut(&TypeExpr, &TypeExpr) -> bool,
    R: FnMut(&str) -> Option<(Vec<TypeExpr>, TypeExpr)>,
{
    surface.members.iter().all(|member| match member {
        SurfaceMember::Field { name: mname, ty: mty } => {
            // Arc 293.4d — Field member: satisfied by a struct field with an assignable type
            // OR by a `:<T>/<name>` accessor (method / extend-type) that returns an assignable
            // type. This lets a foreign type back a Field member with an extend-type method.
            let has_struct_field = struct_fields
                .iter()
                .any(|(fname, fty)| fname == mname && is_assignable(fty, mty));
            if has_struct_field {
                return true;
            }
            // Fall through to the method resolver (for foreign types with extend-type).
            if let Some((_, defn_ret)) = resolve_method(mname) {
                return is_assignable(&defn_ret, mty);
            }
            false
        }
        SurfaceMember::Method { name: mname, args: margs, ret: mret, .. } => {
            // Method member: a `defn :T/<name>` must exist with an assignable sig.
            // The resolver forms the key `":<T>/<name>"` from the candidate type context
            // and returns (defn_arg_types, defn_ret) from env.schemes.
            if let Some((defn_arg_types, defn_ret)) = resolve_method(mname) {
                // Return type must be assignable.
                if !is_assignable(&defn_ret, mret) {
                    return false;
                }
                // If the surface member declared explicit arg-type constraints (non-empty
                // ArgSpec.fixed_params), check that the defn's arg types are assignable
                // per-position. An empty fixed_params means the surface only constrains
                // the return type (e.g. bare `[self]` with no annotation).
                if !margs.fixed_params.is_empty() {
                    let member_arg_types: Vec<&TypeExpr> =
                        margs.fixed_params.iter().map(|(_, ty)| ty).collect();
                    if defn_arg_types.len() < member_arg_types.len() {
                        return false;
                    }
                    for (defn_ty, member_ty) in defn_arg_types.iter().zip(member_arg_types.iter()) {
                        if !is_assignable(defn_ty, member_ty) {
                            return false;
                        }
                    }
                }
                true
            } else {
                false // no matching defn → not satisfied
            }
        }
    })
}

/// Parse a method-member list `(name [args...] -> :RetType)` into a `SurfaceMember::Method`.
///
/// Arc 293.4a — copies the shape of `parse_defprotocol_form`'s per-sig logic but adapted
/// to `TypeError` (STOP-3 resolution: no shared helper because `defprotocol` returns
/// `RuntimeError` while `defsurface` returns `TypeError`).
///
/// The argspec vector `[args...]` is parsed via `parse_argspec_triples`, keeping the full
/// `ArgSpec` (not flattened to `Vec<TypeExpr>`). If the argvec contains only bare symbols
/// without `<-` type annotations (e.g. `[self]`), `parse_argspec_triples` would fail with
/// `IncompleteTriple`; in that case we fall back to an empty `ArgSpec` — the surface member
/// constrains only the return type. Typed args (e.g. `[self <- :Shape  n <- :i64]`) produce
/// a populated `ArgSpec` whose `fixed_params` are checked per-position at satisfaction time.
fn parse_method_member_sig(
    sig_items: &[WatAST],
    sig_span: &Span,
) -> Result<SurfaceMember, TypeError> {
    // (name [args...] -> :RetType)
    // sig_items[0] = method name (Symbol)
    // sig_items[1] = argspec Vector [...]
    // sig_items[2] = -> (Symbol)
    // sig_items[3] = :RetType (Keyword)
    if sig_items.len() < 4 {
        return Err(TypeError {
            span: sig_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "method member sig must have at least 4 elements \
                     `(name [args] -> :R)`; got {}",
                    sig_items.len()
                ),
            },
        });
    }

    // Item 0: method name (bare Symbol).
    let method_name = match &sig_items[0] {
        WatAST::Symbol(s, _) => s.as_str().to_owned(),
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member first element must be a Symbol name; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };

    // Item 1: argspec Vector — parse via parse_argspec_triples, keeping the full ArgSpec.
    // If the argvec has only bare symbols (e.g. `[self]` without `<-`), fall back to an
    // empty ArgSpec (satisfaction will only check the return type).
    let args = match &sig_items[1] {
        WatAST::Vector(items, vec_span) => {
            match crate::argspec::parse_argspec_triples(
                items,
                HEAD,
                vec_span,
                crate::argspec::ParseOptions { allow_rest_binder: false },
            ) {
                Ok(spec) => spec,
                Err(_) => {
                    // Argvec has bare symbols without type annotations (e.g. `[self]`).
                    // Produce an empty ArgSpec — the surface constrains only the return type.
                    crate::argspec::ArgSpec {
                        fixed_params: vec![],
                        rest_param: None,
                    }
                }
            }
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member second element must be an argspec Vector `[...]`; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };

    // Item 2: `->` arrow Symbol.
    match &sig_items[2] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "expected `->` symbol after argspec in method member `{}`; got {}",
                        method_name,
                        other.variant_name()
                    ),
                },
            })
        }
    }

    // Item 3: `:RetType` keyword.
    let ret = match &sig_items[3] {
        WatAST::Keyword(k, _) => {
            super::parse_type_expr(k).map_err(|e| TypeError {
                span: sig_items[3].span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "bad return type in method member `{}`: {}",
                        method_name, e
                    ),
                },
            })?
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member return type must be a keyword after `->` in `{}`; got {}",
                        method_name,
                        other.variant_name()
                    ),
                },
            })
        }
    };

    Ok(SurfaceMember::Method {
        name: method_name,
        args,
        ret,
        type_params: vec![],
    })
}

/// Parse a `(:wat::core::defsurface :Name [name <- :T ...])` declaration.
///
/// Positional form after the head keyword:
///   args[0]  — name keyword (e.g. `:geo::Shape`)
///   args[1]  — member-vector `[name <- :T ...]` (WatAST::Vector)
///
/// Empty member list is legal (zero-member surface — every struct satisfies it).
///
/// Arc 293.4a — the member vector may now mix field triples `name <- :T`
/// (sequences of Symbol, Symbol("->"), Keyword) and method lists
/// `(name [args...] -> :RetType)` (WatAST::List elements). The walker groups
/// consecutive non-List items as field-triple sub-runs and passes each to
/// `parse_argspec_triples`; List items are parsed as Method members.
pub(crate) fn parse_defsurface(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // Valid arities: 2 (name + members) or 4 (name + :holder + value + members).
    if args.len() != 2 && args.len() != 4 {
        return Err(TypeError {
            span: decl_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defsurface :Name [members]) or \
                     (:wat::core::defsurface :Name :holder :<kw> [members]); \
                     got {} args after head",
                    args.len()
                ),
            },
        });
    }

    let mut iter = args.into_iter();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, _type_params) = super::parse_declared_name(HEAD, &name_kw, &decl_span)?;

    // Slot 1 — either the member-vector (arity 2) or `:holder` keyword (arity 4).
    let next = iter.next().unwrap();
    let holder: Option<Holder> = match &next {
        WatAST::Keyword(k, _) if k == ":holder" => {
            // Slot 2 — holder-value keyword.
            let val_node = iter.next().unwrap();
            let holder_val = match &val_node {
                WatAST::Keyword(v, _) => match v.as_str() {
                    ":struct"       => Holder::Struct,
                    ":record"       => Holder::Record,
                    ":holon-record" => Holder::HolonRecord,
                    other => {
                        return Err(TypeError {
                            span: val_node.span().clone(),
                            kind: TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    ":holder value must be :struct, :record, or :holon-record; got {}",
                                    other
                                ),
                            },
                        });
                    }
                },
                other => {
                    return Err(TypeError {
                        span: other.span().clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: ":holder value must be a keyword (:struct, :record, or :holon-record)".into(),
                        },
                    });
                }
            };
            Some(holder_val)
        }
        // No :holder clause — next arg must be the member-vector (handled below).
        _ => None,
    };

    // The member-vector: either `next` (when no :holder) or the remaining arg (after :holder + value).
    let members_node = if holder.is_some() {
        iter.next().unwrap()
    } else {
        next
    };

    // Arc 293.4d-fix — STRUCTURAL invariant: the member vector is the LAST arg; nothing follows it.
    // The arity gate (`len == 2 || len == 4`) alone is too loose — a 4-arg form whose arg[1] is NOT
    // `:holder` (the stale `definterface` shape: method members written as separate top-level args)
    // passes the count, fails the holder probe, and is read as the 2-arg shape with args 2.. SILENTLY
    // DROPPED. Reject any leftover so a mismatch cannot be coerced into a valid-looking surface:
    // every member — a field `name <- :T` AND a method `(name [self] -> :ret)` — goes INSIDE the
    // single `[...]` vector.
    if let Some(extra) = iter.next() {
        return Err(TypeError {
            span: extra.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "unexpected form after the member vector — every surface member (a field \
                         `name <- :T` AND a method `(name [self] -> :ret)`) goes INSIDE the single \
                         `[...]` member vector; nothing follows it"
                    .into(),
            },
        });
    }

    let (member_items, member_span) = match members_node {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "member-vector must be a Vector `[name <- :T ...]`".into(),
                },
            });
        }
    };

    // Arc 293.4a — walk member_items: List elements are Method members; everything else
    // is collected as field-triple sub-runs and parsed by parse_argspec_triples.
    let mut members = Vec::<SurfaceMember>::new();
    let mut field_items: Vec<WatAST> = Vec::new();

    for item in member_items {
        match item {
            WatAST::List(sig_items, sig_span) => {
                // Flush any accumulated field-triple items first.
                if !field_items.is_empty() {
                    flush_field_items(&field_items, &member_span, &mut members)?;
                    field_items.clear();
                }
                // Parse the method member.
                members.push(parse_method_member_sig(&sig_items, &sig_span)?);
            }
            other => {
                // Accumulate field-triple items (Symbol / Symbol("<-") / Keyword).
                field_items.push(other);
            }
        }
    }

    // Final flush for any trailing field-triple items.
    if !field_items.is_empty() {
        flush_field_items(&field_items, &member_span, &mut members)?;
    }

    Ok(TypeDef::Surface(SurfaceDef {
        name,
        type_params: vec![],
        members,
        holder,
    }))
}

/// Pass a slice of field-triple items through `parse_argspec_triples` and append
/// the resulting `SurfaceMember::Field` entries to `members`.
fn flush_field_items(
    field_items: &[WatAST],
    member_span: &Span,
    members: &mut Vec<SurfaceMember>,
) -> Result<(), TypeError> {
    let argspec = crate::argspec::parse_argspec_triples(
        field_items,
        HEAD,
        member_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;

    for (id, ty) in argspec.fixed_params {
        members.push(SurfaceMember::Field {
            name: id.as_str().to_owned(),
            ty,
        });
    }
    Ok(())
}
